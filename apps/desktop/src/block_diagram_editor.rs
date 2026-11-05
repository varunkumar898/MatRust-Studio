//! Block Diagram Canvas & Interactive Solver (open, license-free reimplementation).
//!
//! Visual node-and-wire diagram editor supporting standard blocks:
//! Constant, SineWave, Step, Gain, Sum, Product, Integrator, UnitDelay,
//! Saturation, Scope, and ToWorkspace. Runs real simulations via `mlapp-sim`.

use crate::theme::Theme;
use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Key, Pos2, Rect, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Ui, Vec2,
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
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasWire {
    pub src_block: String,
    pub src_port: usize,
    pub dst_block: String,
    pub dst_port: usize,
    pub bend_offset: Vec2,
}

#[derive(Clone)]
struct BlockLibraryItem {
    label: &'static str,
    block_type: BlockType,
}

struct BlockDiagramLayout {
    #[allow(dead_code)]
    tab_body: Rect,
    toolbar: Rect,
    library: Rect,
    canvas: Rect,
    library_search: Rect,
    toolbar_widgets: Vec<Rect>,
    library_categories: Vec<Rect>,
    library_category_names: Vec<&'static str>,
    library_scroll_offset: f32,
    library_scroll_content_height: f32,
    library_scroll_viewport_height: f32,
}

/// Interactive block diagram editor state.
pub struct BlockDiagramEditor {
    pub blocks: Vec<CanvasBlock>,
    pub wires: Vec<CanvasWire>,
    pub selected_block_sid: Option<String>,
    pub selected_wire: Option<usize>,
    pub connecting_from: Option<(String, usize)>, // (block_sid, port_idx)
    pub pan_offset: Pos2,
    pub zoom: f32,
    library_search: String,
    fit_view_requested: bool,

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
            selected_wire: None,
            connecting_from: None,
            pan_offset: Pos2::new(50.0, 50.0),
            zoom: 1.0,
            library_search: String::new(),
            fit_view_requested: false,

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
            bend_offset: Vec2::ZERO,
        });
        // Sum (sid 2, port 0) -> Gain (sid 3, port 0)
        self.wires.push(CanvasWire {
            src_block: "2".to_string(),
            src_port: 0,
            dst_block: "3".to_string(),
            dst_port: 0,
            bend_offset: Vec2::ZERO,
        });
        // Gain (sid 3, port 0) -> Integrator (sid 4, port 0)
        self.wires.push(CanvasWire {
            src_block: "3".to_string(),
            src_port: 0,
            dst_block: "4".to_string(),
            dst_port: 0,
            bend_offset: Vec2::ZERO,
        });
        // Integrator (sid 4, port 0) -> Scope (sid 5, port 0)
        self.wires.push(CanvasWire {
            src_block: "4".to_string(),
            src_port: 0,
            dst_block: "5".to_string(),
            dst_port: 0,
            bend_offset: Vec2::ZERO,
        });
        // Feedback: Integrator (sid 4, port 0) -> Sum (sid 2, port 1)
        self.wires.push(CanvasWire {
            src_block: "4".to_string(),
            src_port: 0,
            dst_block: "2".to_string(),
            dst_port: 1,
            bend_offset: Vec2::ZERO,
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
}

impl BlockDiagramEditor {
    /// Renders the complete Block Diagram Editor panel.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        self.show_layout(ui, theme);
    }

    fn show_layout(&mut self, ui: &mut Ui, theme: &Theme) -> BlockDiagramLayout {
        let body_rect = ui.available_rect_before_wrap();
        let layout = if body_rect.width() < 460.0 || body_rect.height() < 300.0 {
            ScrollArea::both()
                .id_salt("block_diagram_body_scroll")
                .auto_shrink([false, false])
                .min_scrolled_width(460.0)
                .min_scrolled_height(300.0)
                .show(ui, |ui| self.render_panels(ui, theme, body_rect))
                .inner
        } else {
            self.render_panels(ui, theme, body_rect)
        };

        // Scope Window if opened
        if self.show_scope_window {
            self.render_scope_modal(ui.ctx(), theme);
        }

        // Parameter Edit Modal if opened
        if self.editing_block_sid.is_some() {
            self.render_param_modal(ui.ctx(), theme);
        }
        layout
    }

    fn render_panels(&mut self, ui: &mut Ui, theme: &Theme, tab_body: Rect) -> BlockDiagramLayout {
        let mut layout = BlockDiagramLayout {
            tab_body,
            toolbar: Rect::NOTHING,
            library: Rect::NOTHING,
            canvas: Rect::NOTHING,
            library_search: Rect::NOTHING,
            toolbar_widgets: Vec::new(),
            library_categories: Vec::new(),
            library_category_names: Vec::new(),
            library_scroll_offset: 0.0,
            library_scroll_content_height: 0.0,
            library_scroll_viewport_height: 0.0,
        };
        let mut toolbar_widgets = Vec::new();
        let toolbar = egui::TopBottomPanel::top("block_diagram_toolbar")
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.render_toolbar(ui, theme, &mut toolbar_widgets)
            });
        layout.toolbar = toolbar.response.rect;
        layout.toolbar_widgets = toolbar_widgets;

        let max_library_width = ui.available_width() * 0.4;
        let library = egui::SidePanel::left("block_diagram_library")
            .resizable(true)
            .default_width(220.0)
            .width_range(160.0..=max_library_width.max(160.0))
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| self.render_library_browser(ui, theme, &mut layout));
        layout.library = library.response.rect;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| layout.canvas = self.render_canvas(ui, theme));
        layout
    }

    fn render_toolbar(&mut self, ui: &mut Ui, theme: &Theme, widgets: &mut Vec<Rect>) {
        ui.horizontal_wrapped(|ui| {
            widgets.push(
                ui.add(egui::Label::new(
                    RichText::new("Block Diagram Editor").color(theme.text_primary),
                ))
                .rect,
            );
            ui.separator();

            let run = ui
                .button(RichText::new("Run Simulation").color(theme.accent).strong())
                .on_hover_text("Execute numerical integration of block diagram (Fixed-Step Euler)");
            widgets.push(run.rect);
            if run.clicked() {
                self.run_simulation();
            }

            let stop = ui
                .add_enabled(false, egui::Button::new("Stop"))
                .on_hover_text("The current fixed-step simulation runs synchronously");
            widgets.push(stop.rect);

            let scope = ui
                .button(RichText::new("Open Scope").color(theme.text_primary))
                .on_hover_text("Open oscilloscope signal waveform viewer");
            widgets.push(scope.rect);
            if scope.clicked() {
                self.show_scope_window = true;
            }

            ui.separator();

            widgets.push(ui.label(RichText::new("Stop Time:").size(12.0)).rect);
            widgets.push(
                ui.add(
                    egui::DragValue::new(&mut self.config.stop_time)
                        .speed(0.1)
                        .range(0.1..=100.0),
                )
                .rect,
            );

            widgets.push(ui.label(RichText::new("dt:").size(12.0)).rect);
            widgets.push(
                ui.add(
                    egui::DragValue::new(&mut self.config.step_size)
                        .speed(0.001)
                        .range(0.001..=0.5),
                )
                .rect,
            );

            widgets.push(ui.label("Solver:").rect);
            let solver = egui::ComboBox::from_id_salt("block_diagram_solver")
                .selected_text("Fixed Step Euler")
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.config.solver,
                        "FixedStepEuler".to_string(),
                        "Fixed Step Euler",
                    );
                });
            widgets.push(solver.response.rect);

            ui.separator();

            let load_example = ui.button("Load Feedback Example");
            widgets.push(load_example.rect);
            if load_example.clicked() {
                self.load_default_feedback_example();
            }

            let clear = ui.button("Clear Canvas");
            widgets.push(clear.rect);
            if clear.clicked() {
                self.blocks.clear();
                self.wires.clear();
                self.sim_output = None;
                self.sim_status = "Canvas cleared".to_string();
            }

            let fit = ui.button("Fit View");
            widgets.push(fit.rect);
            if fit.clicked() {
                self.fit_view_requested = true;
            }

            let zoom = ui.label(format!("{:.0}%", self.zoom * 100.0));
            widgets.push(zoom.rect);

            let status = ui.label(RichText::new(&self.sim_status).size(11.0).color(
                if self.sim_status.contains("Error") {
                    theme.error
                } else {
                    theme.text_secondary
                },
            ));
            widgets.push(status.rect);
        });
    }

    fn render_library_browser(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        layout: &mut BlockDiagramLayout,
    ) {
        ui.label(
            RichText::new("Block Library")
                .strong()
                .color(theme.text_secondary),
        );
        let search =
            ui.add(egui::TextEdit::singleline(&mut self.library_search).hint_text("Search blocks"));
        layout.library_search = search.rect;

        let search_query = self.library_search.to_lowercase();
        let libraries = Self::block_libraries();
        let scroll = ScrollArea::vertical()
            .id_salt("sim_library_scroll")
            .auto_shrink([false, false])
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                for (category, items) in libraries {
                    let matching: Vec<_> = items
                        .into_iter()
                        .filter(|item| {
                            search_query.is_empty()
                                || category.to_lowercase().contains(&search_query)
                                || item.label.to_lowercase().contains(&search_query)
                        })
                        .collect();
                    if !search_query.is_empty()
                        && matching.is_empty()
                        && !category.to_lowercase().contains(&search_query)
                    {
                        continue;
                    }

                    let response = egui::CollapsingHeader::new(category)
                        .default_open(true)
                        .show(ui, |ui| {
                            if matching.is_empty() {
                                ui.label("No blocks in this category");
                            }
                            for item in &matching {
                                let drag = ui.dnd_drag_source(
                                    Id::new(format!("library-{}-{}", category, item.label)),
                                    item.clone(),
                                    |ui| ui.button(format!("+ {}", item.label)),
                                );
                                if drag.response.clicked() {
                                    self.add_block(item.block_type.clone(), item.label);
                                }
                            }
                        })
                        .header_response;
                    layout.library_categories.push(response.rect);
                    layout.library_category_names.push(category);
                }
            });
        layout.library_scroll_offset = scroll.state.offset.y;
        layout.library_scroll_content_height = scroll.content_size.y;
        layout.library_scroll_viewport_height = scroll.inner_rect.height();
    }

    fn block_libraries() -> Vec<(&'static str, Vec<BlockLibraryItem>)> {
        let items = |values: &[(&'static str, BlockType)]| {
            values
                .iter()
                .map(|(label, block_type)| BlockLibraryItem {
                    label,
                    block_type: block_type.clone(),
                })
                .collect()
        };
        vec![
            (
                "Sources",
                items(&[
                    ("Constant", BlockType::Constant),
                    ("Sine Wave", BlockType::SineWave),
                    ("Step", BlockType::Step),
                    ("Inport", BlockType::Inport),
                ]),
            ),
            (
                "Sinks",
                items(&[
                    ("Scope", BlockType::Scope),
                    ("To Workspace", BlockType::ToWorkspace),
                    ("Outport", BlockType::Outport),
                ]),
            ),
            (
                "Math Operations",
                items(&[
                    ("Gain", BlockType::Gain),
                    ("Sum (+-)", BlockType::Sum),
                    ("Product", BlockType::Product),
                    ("Saturation", BlockType::Saturation),
                ]),
            ),
            (
                "Continuous",
                items(&[("Integrator", BlockType::Integrator)]),
            ),
            ("Discrete", items(&[("Unit Delay", BlockType::UnitDelay)])),
            (
                "Signal Routing",
                items(&[
                    ("Inport", BlockType::Inport),
                    ("Outport", BlockType::Outport),
                ]),
            ),
            ("Logic", Vec::new()),
            ("Subsystems", items(&[("Subsystem", BlockType::Subsystem)])),
        ]
    }

    fn add_block(&mut self, block_type: BlockType, name: &str) {
        let pos = Pos2::new(
            100.0 + (self.blocks.len() as f32 * 20.0),
            120.0 + (self.blocks.len() as f32 * 15.0),
        );
        self.add_block_at(block_type, name, pos);
    }

    fn add_block_at(&mut self, block_type: BlockType, name: &str, pos: Pos2) {
        let new_id = (self.blocks.len() + 1).to_string();
        let mut block = CanvasBlock {
            sid: new_id,
            name: name.to_string(),
            block_type,
            pos,
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

    fn render_canvas(&mut self, ui: &mut Ui, _theme: &Theme) -> Rect {
        let canvas_size = ui.available_size().max(vec2(300.0, 200.0));
        let (rect, response) = ui.allocate_exact_size(canvas_size, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(24, 26, 32));
        for x in
            (0..=((rect.width() / 30.0).ceil() as usize)).map(|i| rect.left() + i as f32 * 30.0)
        {
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 44)),
            );
        }
        for y in
            (0..=((rect.height() / 30.0).ceil() as usize)).map(|i| rect.top() + i as f32 * 30.0)
        {
            painter.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 44)),
            );
        }

        if self.fit_view_requested {
            self.fit_canvas(rect);
        }
        let pointer = ui.input(|input| input.pointer.hover_pos());
        if let Some(pointer) = pointer.filter(|pointer| rect.contains(*pointer)) {
            let (scroll_y, pan_active, pointer_delta) = ui.input(|input| {
                (
                    input.smooth_scroll_delta.y,
                    input.pointer.button_down(egui::PointerButton::Middle)
                        || (input.key_down(Key::Space)
                            && input.pointer.button_down(egui::PointerButton::Primary)),
                    input.pointer.delta(),
                )
            });
            if pan_active {
                self.pan_offset += pointer_delta;
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            } else if scroll_y != 0.0 {
                let world_at_pointer = (pointer - rect.min - self.pan_offset.to_vec2()) / self.zoom;
                self.zoom = (self.zoom * (scroll_y / 240.0).exp()).clamp(0.1, 4.0);
                let offset = pointer - rect.min - world_at_pointer * self.zoom;
                self.pan_offset = Pos2::new(offset.x, offset.y);
            }
        }

        if let Some(payload) = response.dnd_release_payload::<BlockLibraryItem>() {
            if let Some(pointer) = response.hover_pos() {
                let pos = (pointer - rect.min - self.pan_offset.to_vec2()) / self.zoom;
                let block_size = vec2(90.0, 50.0);
                self.add_block_at(
                    payload.block_type.clone(),
                    payload.label,
                    Pos2::new(pos.x - block_size.x / 2.0, pos.y - block_size.y / 2.0),
                );
            }
        }

        let mut clicked_block = None;
        let mut double_clicked_block = None;
        let mut clicked_wire = None;
        for (wire_index, wire) in self.wires.iter_mut().enumerate() {
            let Some(source) = self.blocks.iter().find(|block| block.sid == wire.src_block) else {
                continue;
            };
            let Some(target) = self.blocks.iter().find(|block| block.sid == wire.dst_block) else {
                continue;
            };
            let start = Self::world_to_screen(
                rect,
                self.pan_offset,
                self.zoom,
                source.pos + vec2(source.size.x, source.size.y / 2.0),
            );
            let end = Self::world_to_screen(
                rect,
                self.pan_offset,
                self.zoom,
                target.pos + vec2(0.0, target.size.y * (wire.dst_port as f32 + 1.0) / 3.0),
            );
            let bend_x = (start.x + end.x) * 0.5 + wire.bend_offset.x * self.zoom;
            let bend_y = (start.y + end.y) * 0.5 + wire.bend_offset.y * self.zoom;
            let points = [
                start,
                pos2(bend_x, start.y),
                pos2(bend_x, bend_y),
                pos2(end.x, bend_y),
                end,
            ];
            let selected = self.selected_wire == Some(wire_index);
            let stroke = Stroke::new(
                if selected { 3.0_f32 } else { 2.0_f32 },
                if selected {
                    Color32::from_rgb(255, 180, 50)
                } else {
                    Color32::from_rgb(90, 180, 255)
                },
            );
            for segment in 0..points.len() - 1 {
                painter.line_segment([points[segment], points[segment + 1]], stroke);
                let hit_rect = Rect::from_two_pos(points[segment], points[segment + 1]).expand(7.0);
                let hit = ui.interact(
                    hit_rect,
                    Id::new(("sim_wire", wire_index, segment)),
                    Sense::click_and_drag(),
                );
                if hit.clicked() {
                    clicked_wire = Some(wire_index);
                }
                if hit.dragged() {
                    wire.bend_offset += hit.drag_delta() / self.zoom;
                    clicked_wire = Some(wire_index);
                }
            }
            painter.circle_filled(end, 3.5, Color32::from_rgb(90, 180, 255));
        }

        if let Some((source_sid, _)) = &self.connecting_from {
            if let Some(source) = self.blocks.iter().find(|block| block.sid == *source_sid) {
                let start = Self::world_to_screen(
                    rect,
                    self.pan_offset,
                    self.zoom,
                    source.pos + vec2(source.size.x, source.size.y / 2.0),
                );
                if let Some(pointer) = response.hover_pos() {
                    painter.line_segment(
                        [start, pointer],
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 200, 80)),
                    );
                }
            }
        }

        let space_pan = ui.input(|input| input.key_down(Key::Space));
        for block in &mut self.blocks {
            let block_rect = Rect::from_min_size(
                rect.min + self.pan_offset.to_vec2() + block.pos.to_vec2() * self.zoom,
                block.size * self.zoom,
            );
            let block_resp = ui.interact(
                block_rect,
                Id::new(format!("sim_block_{}", block.sid)),
                Sense::click_and_drag(),
            );
            if block_resp.dragged()
                && !space_pan
                && !ui.input(|input| input.pointer.button_down(egui::PointerButton::Middle))
            {
                block.pos += block_resp.drag_delta() / self.zoom;
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            if block_resp.clicked() {
                clicked_block = Some(block.sid.clone());
            }
            if block_resp.double_clicked() {
                double_clicked_block = Some(block.sid.clone());
            }

            let selected = self.selected_block_sid.as_deref() == Some(&block.sid);
            painter.rect(
                block_rect,
                CornerRadius::same(4),
                Color32::from_rgb(40, 44, 55),
                Stroke::new(
                    if selected { 2.0_f32 } else { 1.0_f32 },
                    if selected {
                        Color32::from_rgb(255, 180, 50)
                    } else {
                        Color32::from_rgb(80, 90, 110)
                    },
                ),
                StrokeKind::Outside,
            );
            let icon = match block.block_type {
                BlockType::Constant => "1",
                BlockType::SineWave => "~",
                BlockType::Step => "[step]",
                BlockType::Gain => "K",
                BlockType::Sum => "[sum]",
                BlockType::Integrator => "1/s",
                BlockType::UnitDelay => "[z-1]",
                BlockType::Scope => "[scope]",
                _ => "[ ]",
            };
            painter.text(
                pos2(block_rect.center().x, block_rect.top() + 16.0 * self.zoom),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(16.0 * self.zoom),
                Color32::from_rgb(200, 220, 255),
            );
            painter.text(
                pos2(
                    block_rect.center().x,
                    block_rect.bottom() - 10.0 * self.zoom,
                ),
                egui::Align2::CENTER_CENTER,
                &block.name,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::WHITE,
            );

            let output = pos2(block_rect.right(), block_rect.center().y);
            painter.circle_filled(output, 4.0 * self.zoom, Color32::from_rgb(90, 180, 255));
            if ui
                .interact(
                    Rect::from_center_size(output, vec2(12.0, 12.0) * self.zoom),
                    Id::new(format!("out_port_{}", block.sid)),
                    Sense::click(),
                )
                .clicked()
            {
                self.connecting_from = Some((block.sid.clone(), 0));
            }
            let input = pos2(block_rect.left(), block_rect.center().y);
            painter.circle_filled(input, 4.0 * self.zoom, Color32::from_rgb(180, 220, 120));
            if ui
                .interact(
                    Rect::from_center_size(input, vec2(12.0, 12.0) * self.zoom),
                    Id::new(format!("in_port_{}", block.sid)),
                    Sense::click(),
                )
                .clicked()
            {
                if let Some((source_sid, source_port)) = self.connecting_from.take() {
                    if source_sid != block.sid {
                        self.wires.push(CanvasWire {
                            src_block: source_sid,
                            src_port: source_port,
                            dst_block: block.sid.clone(),
                            dst_port: 0,
                            bend_offset: Vec2::ZERO,
                        });
                    }
                }
            }
        }

        if let Some(sid) = clicked_block {
            self.selected_block_sid = Some(sid);
            self.selected_wire = None;
        } else if let Some(index) = clicked_wire {
            self.selected_wire = Some(index);
            self.selected_block_sid = None;
        }
        if let Some(sid) = double_clicked_block {
            self.editing_block_sid = Some(sid);
        }
        if response.clicked() {
            self.connecting_from = None;
            self.selected_block_sid = None;
            self.selected_wire = None;
        }

        let pointer_inside_canvas = ui
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pointer| rect.contains(pointer));
        let delete = pointer_inside_canvas
            && ui
                .input(|input| input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace));
        if delete {
            if let Some(sid) = self.selected_block_sid.take() {
                self.blocks.retain(|block| block.sid != sid);
                self.wires
                    .retain(|wire| wire.src_block != sid && wire.dst_block != sid);
            } else if let Some(index) = self.selected_wire.take() {
                if index < self.wires.len() {
                    self.wires.remove(index);
                }
            }
        }
        rect
    }

    fn world_to_screen(canvas: Rect, pan_offset: Pos2, zoom: f32, point: Pos2) -> Pos2 {
        canvas.min + pan_offset.to_vec2() + point.to_vec2() * zoom
    }

    fn fit_canvas(&mut self, canvas: Rect) {
        self.fit_view_requested = false;
        if self.blocks.is_empty() {
            self.zoom = 1.0;
            self.pan_offset = Pos2::new(canvas.width() * 0.5, canvas.height() * 0.5);
            return;
        }
        let bounds = self.blocks.iter().fold(Rect::NOTHING, |bounds, block| {
            bounds.union(Rect::from_min_size(block.pos, block.size))
        });
        let padding = 30.0;
        self.zoom = ((canvas.width() - 2.0 * padding) / bounds.width())
            .min((canvas.height() - 2.0 * padding) / bounds.height())
            .clamp(0.1, 4.0);
        let offset =
            (canvas.size() - bounds.size() * self.zoom) * 0.5 - bounds.min.to_vec2() * self.zoom;
        self.pan_offset = Pos2::new(offset.x, offset.y);
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

        egui::Window::new("Scope Waveform Viewer")
            .default_size([650.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(output) = &self.sim_output {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Time points: {}", output.tout.len())).size(11.0),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "Duration: 0.0s to {:.2}s",
                                output.tout.last().copied().unwrap_or(0.0)
                            ))
                            .size(11.0),
                        );
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
                        let y_min = output
                            .yout
                            .iter()
                            .copied()
                            .fold(f64::INFINITY, f64::min)
                            .min(-0.2);
                        let y_max = output
                            .yout
                            .iter()
                            .copied()
                            .fold(f64::NEG_INFINITY, f64::max)
                            .max(1.2);
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
                            painter.line_segment(
                                [p1, p2],
                                Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
                            );
                        }

                        painter.text(
                            pos2(rect.left() + 15.0, rect.top() + 15.0),
                            egui::Align2::LEFT_TOP,
                            format!(
                                "Scope Output Signal (Final: {:.4})",
                                output.yout.last().copied().unwrap_or(0.0)
                            ),
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(255, 215, 0),
                        );
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(
                            RichText::new(
                                "No simulation run yet. Click 'Run Simulation' to view waveforms.",
                            )
                            .italics(),
                        );
                    });
                }
            });

        if close_scope {
            self.show_scope_window = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::{DockNode, DockTree, PanelId};
    use egui::{Event, Modifiers, PointerButton, RawInput};

    fn raw_input(size: Vec2, events: Vec<Event>) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        }
    }

    fn run_frame(
        ctx: &egui::Context,
        editor: &mut BlockDiagramEditor,
        size: Vec2,
        events: Vec<Event>,
    ) -> BlockDiagramLayout {
        let mut layout = None;
        let _ = ctx.run(raw_input(size, events), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    layout = Some(editor.show_layout(ui, &Theme::dark()))
                });
        });
        layout.expect("block diagram layout should be rendered")
    }

    fn pointer_button(pos: Pos2, button: PointerButton, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    fn run_dock_frame(
        ctx: &egui::Context,
        tree: &mut DockTree,
        editor: &mut BlockDiagramEditor,
        size: Vec2,
        events: Vec<Event>,
    ) -> (Option<BlockDiagramLayout>, Option<Rect>) {
        let mut layout = None;
        let mut root_id = None;
        let _ = ctx.run(raw_input(size, events), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    root_id = Some(ui.id());
                    tree.render(ui, &Theme::dark(), |panel_ui, panel| {
                        if panel == PanelId::BlockDiagram {
                            layout = Some(editor.show_layout(panel_ui, &Theme::dark()));
                        } else {
                            panel_ui.label("Other tab");
                        }
                    });
                });
        });
        let divider = root_id.and_then(|id| {
            ctx.read_response(id.with("div_h_0"))
                .map(|response| response.rect)
        });
        (layout, divider)
    }

    #[test]
    fn block_diagram_canvas_fills_tab_body_at_common_window_sizes() {
        for size in [
            vec2(1280.0, 800.0),
            vec2(1920.0, 1080.0),
            vec2(900.0, 600.0),
        ] {
            let ctx = egui::Context::default();
            let mut editor = BlockDiagramEditor::default();
            let layout = run_frame(&ctx, &mut editor, size, Vec::new());
            let remaining_height = layout.tab_body.height() - layout.toolbar.height();
            let remaining_width = layout.tab_body.width() - layout.library.width();

            assert!(
                layout.canvas.height() >= remaining_height * 0.9,
                "canvas {:?} should fill the tab height {:?} at {:?}",
                layout.canvas,
                layout.tab_body,
                size
            );
            assert!(
                layout.canvas.width() >= remaining_width * 0.9,
                "canvas {:?} should fill the tab width {:?} at {:?}",
                layout.canvas,
                layout.tab_body,
                size
            );
            assert!(
                (layout.canvas.max.y - layout.tab_body.max.y).abs() <= 2.0,
                "canvas must reach the bottom of the tab body at {:?}",
                size
            );
            assert!(
                layout
                    .toolbar_widgets
                    .iter()
                    .all(|widget| widget.max.x <= layout.toolbar.max.x + 2.0),
                "toolbar widget extends beyond its right edge at {:?}",
                size
            );
            assert!(layout.library_search.intersects(layout.library));
            assert_eq!(
                layout.library_category_names,
                [
                    "Sources",
                    "Sinks",
                    "Math Operations",
                    "Continuous",
                    "Discrete",
                    "Signal Routing",
                    "Logic",
                    "Subsystems"
                ]
            );
        }
    }

    #[test]
    fn block_diagram_canvas_allocation_is_not_fixed_height() {
        let mut heights = Vec::new();
        for size in [
            vec2(1280.0, 800.0),
            vec2(1920.0, 1080.0),
            vec2(900.0, 600.0),
        ] {
            let ctx = egui::Context::default();
            let mut editor = BlockDiagramEditor::default();
            heights.push(
                run_frame(&ctx, &mut editor, size, Vec::new())
                    .canvas
                    .height(),
            );
        }
        assert!(heights[0] > heights[2]);
        assert!(heights[1] > heights[0]);
    }

    #[test]
    fn block_diagram_library_scrolls_to_subsystems() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        let size = vec2(460.0, 320.0);
        let mut layout = run_frame(&ctx, &mut editor, size, Vec::new());
        assert!(
            layout.library_scroll_content_height > layout.library_scroll_viewport_height,
            "expanded library categories should require vertical scrolling"
        );
        for _ in 0..5 {
            let pointer = layout.library.center();
            layout = run_frame(
                &ctx,
                &mut editor,
                size,
                vec![
                    Event::PointerMoved(pointer),
                    Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: vec2(0.0, -1200.0),
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        let last_category = layout.library_categories.last().copied().unwrap();
        assert!(layout.library_scroll_offset > 0.0);
        assert!(
            layout.library.intersects(last_category),
            "the last category should be visible after scrolling to the end"
        );
    }

    #[test]
    fn block_diagram_canvas_tracks_dock_splitter_resize() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        let mut tree = DockTree {
            root: DockNode::split_h(
                0.5,
                DockNode::leaf(PanelId::BlockDiagram),
                DockNode::leaf(PanelId::Editor),
            ),
            ..DockTree::default()
        };
        let size = vec2(1280.0, 800.0);
        let (before, divider) = run_dock_frame(&ctx, &mut tree, &mut editor, size, Vec::new());
        let before = before.unwrap();
        let divider = divider.unwrap();
        let start = divider.center();
        run_dock_frame(
            &ctx,
            &mut tree,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(start),
                pointer_button(start, PointerButton::Primary, true),
            ],
        );
        let end = start + vec2(80.0, 0.0);
        run_dock_frame(
            &ctx,
            &mut tree,
            &mut editor,
            size,
            vec![Event::PointerMoved(end)],
        );
        run_dock_frame(
            &ctx,
            &mut tree,
            &mut editor,
            size,
            vec![pointer_button(end, PointerButton::Primary, false)],
        );
        let (after, _) = run_dock_frame(&ctx, &mut tree, &mut editor, size, Vec::new());
        let after = after.unwrap();
        let body_delta = after.tab_body.width() - before.tab_body.width();
        let canvas_delta = after.canvas.width() - before.canvas.width();
        assert!(body_delta > 50.0, "dock splitter should widen the tab body");
        assert!(
            (canvas_delta - body_delta).abs() <= 2.0,
            "canvas width should track the splitter delta: {canvas_delta} vs {body_delta}"
        );
    }

    #[test]
    fn block_diagram_library_drag_drops_a_block_at_the_cursor() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        let size = vec2(1280.0, 800.0);
        let layout = run_frame(&ctx, &mut editor, size, Vec::new());
        let source = ctx
            .read_response(Id::new("library-Sources-Constant"))
            .unwrap()
            .rect
            .center();
        assert!(layout.library.contains(source));
        let cursor = layout.canvas.center();

        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(source),
                pointer_button(source, PointerButton::Primary, true),
            ],
        );
        run_frame(&ctx, &mut editor, size, vec![Event::PointerMoved(cursor)]);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![pointer_button(cursor, PointerButton::Primary, false)],
        );

        assert_eq!(editor.blocks.len(), 1);
        let block = &editor.blocks[0];
        let block_center = layout.canvas.min
            + editor.pan_offset.to_vec2()
            + block.pos.to_vec2() * editor.zoom
            + block.size * editor.zoom * 0.5;
        assert!((block_center - cursor).length() < 2.0);
    }

    #[test]
    fn block_diagram_canvas_pans_and_zooms_at_pointer() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        let size = vec2(900.0, 600.0);
        let layout = run_frame(&ctx, &mut editor, size, Vec::new());
        let pointer = layout.canvas.center();
        let original_pan = editor.pan_offset;
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(pointer),
                pointer_button(pointer, PointerButton::Middle, true),
            ],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![Event::PointerMoved(pointer + vec2(40.0, 20.0))],
        );
        assert_ne!(editor.pan_offset, original_pan);

        let pointer = pointer + vec2(40.0, 20.0);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![pointer_button(pointer, PointerButton::Middle, false)],
        );
        let zoom_before = editor.zoom;
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(pointer),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: vec2(0.0, 120.0),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        assert_ne!(editor.zoom, zoom_before);
    }

    #[test]
    fn block_diagram_canvas_selects_and_deletes_blocks_and_wires() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();
        let size = vec2(1280.0, 800.0);
        let layout = run_frame(&ctx, &mut editor, size, Vec::new());
        let source_pos = editor.blocks[0].pos;
        let source_size = editor.blocks[0].size;
        let target_pos = editor.blocks[1].pos;
        let target_size = editor.blocks[1].size;
        let start = BlockDiagramEditor::world_to_screen(
            layout.canvas,
            editor.pan_offset,
            editor.zoom,
            source_pos + vec2(source_size.x, source_size.y * 0.5),
        );
        let end = BlockDiagramEditor::world_to_screen(
            layout.canvas,
            editor.pan_offset,
            editor.zoom,
            target_pos + vec2(0.0, target_size.y / 3.0),
        );
        let wire_point = pos2((start.x + (start.x + end.x) * 0.5) * 0.5, start.y);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(wire_point),
                pointer_button(wire_point, PointerButton::Primary, true),
            ],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![Event::PointerMoved(wire_point + vec2(15.0, 10.0))],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![pointer_button(
                wire_point + vec2(15.0, 10.0),
                PointerButton::Primary,
                false,
            )],
        );
        assert_eq!(editor.selected_wire, Some(0));
        assert_ne!(editor.wires[0].bend_offset, Vec2::ZERO);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert_eq!(editor.wires.len(), 4);

        let block_center = BlockDiagramEditor::world_to_screen(
            layout.canvas,
            editor.pan_offset,
            editor.zoom,
            source_pos + source_size * 0.5,
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(block_center),
                pointer_button(block_center, PointerButton::Primary, true),
            ],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![pointer_button(block_center, PointerButton::Primary, false)],
        );
        assert_eq!(editor.selected_block_sid.as_deref(), Some("1"));
        let block_start = editor.blocks[0].pos;
        let block_end = block_center + vec2(25.0, 15.0);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![
                Event::PointerMoved(block_center),
                pointer_button(block_center, PointerButton::Primary, true),
            ],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![Event::PointerMoved(block_end)],
        );
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![pointer_button(block_end, PointerButton::Primary, false)],
        );
        assert_ne!(editor.blocks[0].pos, block_start);
        run_frame(
            &ctx,
            &mut editor,
            size,
            vec![Event::PointerMoved(block_end), Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert_eq!(editor.blocks.len(), 4);
        assert!(!editor
            .wires
            .iter()
            .any(|wire| wire.src_block == "1" || wire.dst_block == "1"));
    }
}
