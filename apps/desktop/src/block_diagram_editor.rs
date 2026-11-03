//! Block Diagram Canvas & Interactive Solver (open, license-free reimplementation).
//!
//! Visual node-and-wire diagram editor supporting standard blocks:
//! Constant, SineWave, Step, Gain, Sum, Product, Integrator, UnitDelay,
//! Saturation, Scope, and ToWorkspace. Runs real simulations via `mlapp-sim`.

use crate::theme::Theme;
use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Pos2, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};
use mlapp_sim::{
    model::{Block as SimBlock, BlockDiagram, BlockType, Line as SimLine, PortRef},
    solver::{simulate_diagram, SimulationConfig, SimulationOutput},
};
use std::collections::HashMap;

/// Canvas representation of a block node.
#[derive(Clone, Debug)]
pub struct CanvasBlock {
    pub sid: String,
    pub name: String,
    pub block_type: BlockType,
    pub pos: Pos2,
    pub size: Vec2,
    pub parameters: HashMap<String, String>,
}

/// Canvas representation of a wire between ports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasWire {
    pub src_block: String,
    pub src_port: usize,
    pub dst_block: String,
    pub dst_port: usize,
}

/// Interactive block diagram editor state.
pub struct BlockDiagramEditor {
    pub blocks: Vec<CanvasBlock>,
    pub wires: Vec<CanvasWire>,
    pub selected_block_sid: Option<String>,
    pub connecting_from: Option<(String, usize)>, // (block_sid, port_idx)
    pub pan_offset: Pos2,
    pub zoom: f32,

    // Parameter Edit Dialog
    pub editing_block_sid: Option<String>,
    pub param_edit_key: String,
    pub param_edit_val: String,

    // Simulation Config & Results
    pub config: SimulationConfig,
    pub sim_output: Option<SimulationOutput>,
    pub sim_status: String,
    pub show_scope_window: bool,
    pub scope_signal_name: String,
}

impl Default for BlockDiagramEditor {
    fn default() -> Self {
        // G2: a fresh canvas is EMPTY; the feedback example is an explicit
        // user action ("Load Feedback Example" button).
        Self {
            blocks: Vec::new(),
            wires: Vec::new(),
            selected_block_sid: None,
            connecting_from: None,
            pan_offset: Pos2::new(50.0, 50.0),
            zoom: 1.0,

            editing_block_sid: None,
            param_edit_key: String::new(),
            param_edit_val: String::new(),

            config: SimulationConfig {
                start_time: 0.0,
                stop_time: 5.0,
                step_size: 0.01,
                solver: "FixedStepEuler".to_string(),
            },
            sim_output: None,
            sim_status: "Ready".to_string(),
            show_scope_window: false,
            scope_signal_name: "Scope".to_string(),
        }
    }
}

impl BlockDiagramEditor {
    /// Loads a clean-room feedback control loop example:
    /// Step -> Sum (-) <- Gain (from Integrator) -> Integrator -> Scope.
    pub fn load_default_feedback_example(&mut self) {
        self.blocks.clear();
        self.wires.clear();

        // 1. Step block
        let mut step = CanvasBlock {
            sid: "1".to_string(),
            name: "Step Input".to_string(),
            block_type: BlockType::Step,
            pos: Pos2::new(80.0, 150.0),
            size: Vec2::new(90.0, 50.0),
            parameters: HashMap::new(),
        };
        step.parameters
            .insert("Time".to_string(), "0.5".to_string());
        step.parameters
            .insert("Before".to_string(), "0.0".to_string());
        step.parameters
            .insert("After".to_string(), "1.0".to_string());

        // 2. Sum block (+, -)
        let mut sum = CanvasBlock {
            sid: "2".to_string(),
            name: "Sum".to_string(),
            block_type: BlockType::Sum,
            pos: Pos2::new(230.0, 155.0),
            size: Vec2::new(50.0, 50.0),
            parameters: HashMap::new(),
        };
        sum.parameters
            .insert("Inputs".to_string(), "+-".to_string());

        // 3. Gain block
        let mut gain = CanvasBlock {
            sid: "3".to_string(),
            name: "Controller Gain".to_string(),
            block_type: BlockType::Gain,
            pos: Pos2::new(330.0, 150.0),
            size: Vec2::new(80.0, 50.0),
            parameters: HashMap::new(),
        };
        gain.parameters
            .insert("Gain".to_string(), "2.0".to_string());

        // 4. Integrator block
        let mut intg = CanvasBlock {
            sid: "4".to_string(),
            name: "Plant Integrator".to_string(),
            block_type: BlockType::Integrator,
            pos: Pos2::new(460.0, 150.0),
            size: Vec2::new(90.0, 50.0),
            parameters: HashMap::new(),
        };
        intg.parameters
            .insert("InitialCondition".to_string(), "0.0".to_string());

        // 5. Scope block
        let scope = CanvasBlock {
            sid: "5".to_string(),
            name: "Scope".to_string(),
            block_type: BlockType::Scope,
            pos: Pos2::new(600.0, 150.0),
            size: Vec2::new(80.0, 50.0),
            parameters: HashMap::new(),
        };

        self.blocks.push(step);
        self.blocks.push(sum);
        self.blocks.push(gain);
        self.blocks.push(intg);
        self.blocks.push(scope);

        // Wires:
        // Step (sid 1, port 0) -> Sum (sid 2, port 0)
        self.wires.push(CanvasWire {
            src_block: "1".to_string(),
            src_port: 0,
            dst_block: "2".to_string(),
            dst_port: 0,
        });
        // Sum (sid 2, port 0) -> Gain (sid 3, port 0)
        self.wires.push(CanvasWire {
            src_block: "2".to_string(),
            src_port: 0,
            dst_block: "3".to_string(),
            dst_port: 0,
        });
        // Gain (sid 3, port 0) -> Integrator (sid 4, port 0)
        self.wires.push(CanvasWire {
            src_block: "3".to_string(),
            src_port: 0,
            dst_block: "4".to_string(),
            dst_port: 0,
        });
        // Integrator (sid 4, port 0) -> Scope (sid 5, port 0)
        self.wires.push(CanvasWire {
            src_block: "4".to_string(),
            src_port: 0,
            dst_block: "5".to_string(),
            dst_port: 0,
        });
        // Feedback: Integrator (sid 4, port 0) -> Sum (sid 2, port 1)
        self.wires.push(CanvasWire {
            src_block: "4".to_string(),
            src_port: 0,
            dst_block: "2".to_string(),
            dst_port: 1,
        });
    }

    /// Converts canvas model to `mlapp_sim::BlockDiagram`.
    pub fn to_block_diagram(&self) -> BlockDiagram {
        let mut diagram = BlockDiagram::new("Untitled_Diagram");
        for b in &self.blocks {
            let mut sb = SimBlock::new(&b.sid, &b.name, b.block_type.clone());
            sb.position = Some([
                b.pos.x as f64,
                b.pos.y as f64,
                (b.pos.x + b.size.x) as f64,
                (b.pos.y + b.size.y) as f64,
            ]);
            for (k, v) in &b.parameters {
                sb.parameters.insert(k.clone(), v.clone());
            }
            diagram.blocks.push(sb);
        }

        for w in &self.wires {
            diagram.lines.push(SimLine {
                src: PortRef {
                    block_sid: w.src_block.clone(),
                    port_index: w.src_port + 1, // 1-based port index for mlapp-sim
                },
                dst: PortRef {
                    block_sid: w.dst_block.clone(),
                    port_index: w.dst_port + 1,
                },
                name: None,
            });
        }

        diagram
    }

    /// Executes the simulation model using `mlapp-sim`.
    pub fn run_simulation(&mut self) {
        let diagram = self.to_block_diagram();
        match simulate_diagram(&diagram, &self.config) {
            Ok(output) => {
                self.sim_status = format!(
                    "Simulation finished: {} time points (0.0s to {:.2}s)",
                    output.tout.len(),
                    self.config.stop_time
                );
                self.sim_output = Some(output);
                self.show_scope_window = true;
            }
            Err(e) => {
                self.sim_status = format!("Simulation Error: {e}");
            }
        }
    }

    /// Renders the complete Block Diagram Editor panel.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.vertical(|ui| {
            // Toolbar at top
            self.render_toolbar(ui, theme);
            ui.separator();

            // Main working area: Left = Block Library, Right = Diagram Canvas
            ui.horizontal(|ui| {
                // Left: Library Browser
                self.render_library_browser(ui, theme);
                ui.separator();

                // Right: Diagram Canvas
                self.render_canvas(ui, theme);
            });
        });

        // Scope Window if opened
        if self.show_scope_window {
            self.render_scope_modal(ui.ctx(), theme);
        }

        // Parameter Edit Modal if opened
        if self.editing_block_sid.is_some() {
            self.render_param_modal(ui.ctx(), theme);
        }
    }

    fn render_toolbar(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🔄 Block Diagram Editor").color(theme.text_primary));
            ui.separator();

            if ui
                .button(
                    RichText::new("▶ Run Simulation")
                        .color(theme.accent)
                        .strong(),
                )
                .on_hover_text("Execute numerical integration of block diagram (Fixed-Step Euler)")
                .clicked()
            {
                self.run_simulation();
            }

            if ui
                .button(RichText::new("📈 Open Scope").color(theme.text_primary))
                .on_hover_text("Open oscilloscope signal waveform viewer")
                .clicked()
            {
                self.show_scope_window = true;
            }

            ui.separator();

            ui.label(RichText::new("Stop Time:").size(12.0));
            ui.add(
                egui::DragValue::new(&mut self.config.stop_time)
                    .speed(0.1)
                    .range(0.1..=100.0),
            );

            ui.label(RichText::new("dt:").size(12.0));
            ui.add(
                egui::DragValue::new(&mut self.config.step_size)
                    .speed(0.001)
                    .range(0.001..=0.5),
            );

            ui.separator();

            if ui.button("Load Feedback Example").clicked() {
                self.load_default_feedback_example();
            }

            if ui.button("Clear Canvas").clicked() {
                self.blocks.clear();
                self.wires.clear();
                self.sim_output = None;
                self.sim_status = "Canvas cleared".to_string();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&self.sim_status).size(11.0).color(
                    if self.sim_status.contains("Error") {
                        theme.error
                    } else {
                        theme.text_secondary
                    },
                ));
            });
        });
    }

    fn render_library_browser(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.vertical(|ui| {
            ui.set_width(170.0);
            ui.label(
                RichText::new("Block Library")
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(4.0);

            ScrollArea::vertical()
                .id_salt("sim_library_scroll")
                .show(ui, |ui| {
                    let libraries: &[(&str, &[(&str, BlockType)])] = &[
                        (
                            "Sources",
                            &[
                                ("Constant", BlockType::Constant),
                                ("Sine Wave", BlockType::SineWave),
                                ("Step", BlockType::Step),
                            ],
                        ),
                        (
                            "Math Operations",
                            &[
                                ("Gain", BlockType::Gain),
                                ("Sum (+-)", BlockType::Sum),
                                ("Product", BlockType::Product),
                                ("Saturation", BlockType::Saturation),
                            ],
                        ),
                        (
                            "Continuous / Discrete",
                            &[
                                ("Integrator", BlockType::Integrator),
                                ("Unit Delay", BlockType::UnitDelay),
                            ],
                        ),
                        (
                            "Sinks",
                            &[
                                ("Scope", BlockType::Scope),
                                ("To Workspace", BlockType::ToWorkspace),
                            ],
                        ),
                    ];

                    for (category, items) in libraries {
                        ui.collapsing(*category, |ui| {
                            for (label, b_type) in *items {
                                if ui
                                    .button(format!("+ {label}"))
                                    .on_hover_text(format!("Add {label} block to canvas"))
                                    .clicked()
                                {
                                    self.add_block(b_type.clone(), label);
                                }
                            }
                        });
                    }
                });
        });
    }

    fn add_block(&mut self, block_type: BlockType, name: &str) {
        let new_id = (self.blocks.len() + 1).to_string();
        let mut block = CanvasBlock {
            sid: new_id,
            name: name.to_string(),
            block_type,
            pos: Pos2::new(
                100.0 + (self.blocks.len() as f32 * 20.0),
                120.0 + (self.blocks.len() as f32 * 15.0),
            ),
            size: Vec2::new(90.0, 50.0),
            parameters: HashMap::new(),
        };

        // Initialize default parameters
        match block.block_type {
            BlockType::Constant => {
                block
                    .parameters
                    .insert("Value".to_string(), "1.0".to_string());
            }
            BlockType::SineWave => {
                block
                    .parameters
                    .insert("Amplitude".to_string(), "1.0".to_string());
                block
                    .parameters
                    .insert("Frequency".to_string(), "1.0".to_string());
            }
            BlockType::Step => {
                block
                    .parameters
                    .insert("Time".to_string(), "1.0".to_string());
                block
                    .parameters
                    .insert("Before".to_string(), "0.0".to_string());
                block
                    .parameters
                    .insert("After".to_string(), "1.0".to_string());
            }
            BlockType::Gain => {
                block
                    .parameters
                    .insert("Gain".to_string(), "2.0".to_string());
            }
            BlockType::Sum => {
                block
                    .parameters
                    .insert("Inputs".to_string(), "+-".to_string());
            }
            BlockType::Integrator => {
                block
                    .parameters
                    .insert("InitialCondition".to_string(), "0.0".to_string());
            }
            BlockType::Saturation => {
                block
                    .parameters
                    .insert("UpperLimit".to_string(), "1.0".to_string());
                block
                    .parameters
                    .insert("LowerLimit".to_string(), "-1.0".to_string());
            }
            _ => {}
        }

        self.blocks.push(block);
    }

    fn render_canvas(&mut self, ui: &mut Ui, _theme: &Theme) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        // Draw background grid
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(24, 26, 32));
        let grid_spacing = 30.0;
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 44)),
            );
            x += grid_spacing;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 44)),
            );
            y += grid_spacing;
        }

        // Draw existing wires
        for wire in &self.wires {
            let src_b = self.blocks.iter().find(|b| b.sid == wire.src_block);
            let dst_b = self.blocks.iter().find(|b| b.sid == wire.dst_block);
            if let (Some(sb), Some(db)) = (src_b, dst_b) {
                let start_pt = pos2(
                    rect.left() + sb.pos.x + sb.size.x,
                    rect.top() + sb.pos.y + (sb.size.y / 2.0),
                );
                let dst_y_offset = (sb.size.y / 3.0) * (wire.dst_port as f32 + 1.0);
                let end_pt = pos2(rect.left() + db.pos.x, rect.top() + db.pos.y + dst_y_offset);

                // Draw routed stepped line
                let mid_x = (start_pt.x + end_pt.x) / 2.0;
                let stroke = Stroke::new(2.0_f32, Color32::from_rgb(90, 180, 255));
                painter.line_segment([start_pt, pos2(mid_x, start_pt.y)], stroke);
                painter.line_segment([pos2(mid_x, start_pt.y), pos2(mid_x, end_pt.y)], stroke);
                painter.line_segment([pos2(mid_x, end_pt.y), end_pt], stroke);

                // Arrow head at target
                painter.circle_filled(end_pt, 3.5, Color32::from_rgb(90, 180, 255));
            }
        }

        // If currently dragging wire from a port
        if let Some((src_sid, _port)) = &self.connecting_from {
            if let Some(sb) = self.blocks.iter().find(|b| b.sid == *src_sid) {
                let start_pt = pos2(
                    rect.left() + sb.pos.x + sb.size.x,
                    rect.top() + sb.pos.y + (sb.size.y / 2.0),
                );
                if let Some(hover_pos) = response.hover_pos() {
                    painter.line_segment(
                        [start_pt, hover_pos],
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 200, 80)),
                    );
                }
            }
        }

        // Draw and interact with blocks
        let mut clicked_block = None;
        let mut double_clicked_block = None;

        for block in &mut self.blocks {
            let block_rect = Rect::from_min_size(
                pos2(rect.left() + block.pos.x, rect.top() + block.pos.y),
                block.size,
            );

            let block_id = Id::new(format!("sim_block_{}", block.sid));
            let block_resp = ui.interact(block_rect, block_id, Sense::click_and_drag());

            if block_resp.dragged() {
                block.pos += block_resp.drag_delta();
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }

            if block_resp.clicked() {
                clicked_block = Some(block.sid.clone());
            }

            if block_resp.double_clicked() {
                double_clicked_block = Some(block.sid.clone());
            }

            // Draw block body
            let is_selected = self.selected_block_sid.as_deref() == Some(&block.sid);
            let border_color = if is_selected {
                Color32::from_rgb(255, 180, 50)
            } else {
                Color32::from_rgb(80, 90, 110)
            };

            painter.rect(
                block_rect,
                CornerRadius::same(4),
                Color32::from_rgb(40, 44, 55),
                Stroke::new(if is_selected { 2.0_f32 } else { 1.0_f32 }, border_color),
                StrokeKind::Outside,
            );

            // Block Label & Type Icon
            let icon_str = match block.block_type {
                BlockType::Constant => "1",
                BlockType::SineWave => "~",
                BlockType::Step => "_|¯",
                BlockType::Gain => "K",
                BlockType::Sum => "∑",
                BlockType::Integrator => "1/s",
                BlockType::UnitDelay => "z⁻¹",
                BlockType::Scope => "📈",
                _ => "▪",
            };

            painter.text(
                pos2(block_rect.center().x, block_rect.top() + 16.0),
                egui::Align2::CENTER_CENTER,
                icon_str,
                egui::FontId::proportional(16.0),
                Color32::from_rgb(200, 220, 255),
            );

            painter.text(
                pos2(block_rect.center().x, block_rect.bottom() - 10.0),
                egui::Align2::CENTER_CENTER,
                &block.name,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );

            // Draw input and output port pins
            let out_port_pos = pos2(block_rect.right(), block_rect.center().y);
            painter.circle_filled(out_port_pos, 4.0, Color32::from_rgb(90, 180, 255));

            let out_port_rect = Rect::from_center_size(out_port_pos, vec2(12.0, 12.0));
            let out_port_id = Id::new(format!("out_port_{}", block.sid));
            let out_resp = ui.interact(out_port_rect, out_port_id, Sense::click());
            if out_resp.clicked() {
                self.connecting_from = Some((block.sid.clone(), 0));
            }

            let in_port_pos = pos2(block_rect.left(), block_rect.center().y);
            painter.circle_filled(in_port_pos, 4.0, Color32::from_rgb(180, 220, 120));

            let in_port_rect = Rect::from_center_size(in_port_pos, vec2(12.0, 12.0));
            let in_port_id = Id::new(format!("in_port_{}", block.sid));
            let in_resp = ui.interact(in_port_rect, in_port_id, Sense::click());
            if in_resp.clicked() {
                if let Some((src_sid, src_port)) = self.connecting_from.take() {
                    if src_sid != block.sid {
                        self.wires.push(CanvasWire {
                            src_block: src_sid,
                            src_port,
                            dst_block: block.sid.clone(),
                            dst_port: 0,
                        });
                    }
                }
            }
        }

        if let Some(sid) = clicked_block {
            self.selected_block_sid = Some(sid);
        }
        if let Some(sid) = double_clicked_block {
            self.editing_block_sid = Some(sid);
        }

        // Cancel pending connection if canvas background clicked
        if response.clicked() && !response.dragged() {
            self.connecting_from = None;
        }
    }

    fn render_param_modal(&mut self, ctx: &egui::Context, theme: &Theme) {
        let mut close_modal = false;
        let mut save_params = None;

        if let Some(sid) = &self.editing_block_sid {
            if let Some(block) = self.blocks.iter().find(|b| b.sid == *sid) {
                egui::Window::new(format!("Block Parameters: {}", block.name))
                    .id(egui::Id::new(format!("block-params-{sid}")))
                    .collapsible(false)
                    .resizable(false)
                    .default_width(320.0)
                    .show(ctx, |ui| {
                        ui.label(RichText::new(format!("Type: {:?}", block.block_type)).size(12.0));
                        ui.separator();

                        let mut params_clone = block.parameters.clone();
                        for (k, v) in &mut params_clone {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(k).strong().color(theme.text_secondary));
                                ui.text_edit_singleline(v);
                            });
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save Parameters").clicked() {
                                save_params = Some(params_clone);
                                close_modal = true;
                            }
                            if ui.button("Cancel").clicked() {
                                close_modal = true;
                            }
                        });
                    });
            } else {
                close_modal = true;
            }
        }

        if let Some(params) = save_params {
            if let Some(sid) = &self.editing_block_sid {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.sid == *sid) {
                    block.parameters = params;
                }
            }
        }

        if close_modal {
            self.editing_block_sid = None;
        }
    }

    fn render_scope_modal(&mut self, ctx: &egui::Context, _theme: &Theme) {
        let mut close_scope = false;

        egui::Window::new("📈 Scope Waveform Viewer")
            .default_size([650.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(output) = &self.sim_output {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Time points: {}", output.tout.len())).size(11.0));
                        ui.separator();
                        ui.label(RichText::new(format!("Duration: 0.0s to {:.2}s", output.tout.last().copied().unwrap_or(0.0))).size(11.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_scope = true;
                            }
                        });
                    });
                    ui.separator();

                    let (rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), ui.available_height() - 20.0),
                        Sense::hover(),
                    );

                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(15, 18, 22));

                    // Plot waveform: output.tout vs output.yout
                    if output.tout.len() >= 2 && output.yout.len() == output.tout.len() {
                        let t_max = output.tout.last().copied().unwrap_or(1.0).max(0.001);
                        let y_min = output.yout.iter().copied().fold(f64::INFINITY, f64::min).min(-0.2);
                        let y_max = output.yout.iter().copied().fold(f64::NEG_INFINITY, f64::max).max(1.2);
                        let y_span = (y_max - y_min).max(0.001);

                        let map_pt = |t: f64, y: f64| -> Pos2 {
                            let norm_x = (t / t_max) as f32;
                            let norm_y = ((y - y_min) / y_span) as f32;
                            pos2(
                                rect.left() + norm_x * (rect.width() - 20.0) + 10.0,
                                rect.bottom() - norm_y * (rect.height() - 20.0) - 10.0,
                            )
                        };

                        // Zero reference line
                        if y_min <= 0.0 && y_max >= 0.0 {
                            let zero_y = map_pt(0.0, 0.0).y;
                            painter.line_segment(
                                [pos2(rect.left(), zero_y), pos2(rect.right(), zero_y)],
                                Stroke::new(1.0_f32, Color32::from_rgb(50, 60, 75)),
                            );
                        }

                        // Curve points
                        for i in 0..(output.tout.len() - 1) {
                            let p1 = map_pt(output.tout[i], output.yout[i]);
                            let p2 = map_pt(output.tout[i + 1], output.yout[i + 1]);
                            painter.line_segment([p1, p2], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
                        }

                        painter.text(
                            pos2(rect.left() + 15.0, rect.top() + 15.0),
                            egui::Align2::LEFT_TOP,
                            format!("Scope Output Signal (Final: {:.4})", output.yout.last().copied().unwrap_or(0.0)),
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(255, 215, 0),
                        );
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("No simulation run yet. Click '▶ Run Simulation' to view waveforms.").italics());
                    });
                }
            });

        if close_scope {
            self.show_scope_window = false;
        }
    }
}
