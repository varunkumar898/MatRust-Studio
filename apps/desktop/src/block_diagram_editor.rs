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
    model::{
        param_specs_for, parse_float_vector, Block as SimBlock, BlockDiagram, BlockRotation,
        BlockType, Line as SimLine, ParamKind, PortKind, PortRef,
    },
    netlist::resolve_electrical_nets,
    solver::{simulate_diagram, SimError, SimulationConfig, SimulationOutput, SolverType},
    spec::pin_geometry,
};
use std::collections::HashMap;

const DEFAULT_LIBRARY_WIDTH: f32 = 180.0;
const MIN_LIBRARY_WIDTH: f32 = 140.0;
const DEFAULT_PROPERTIES_WIDTH: f32 = 200.0;
const MIN_PROPERTIES_WIDTH: f32 = 160.0;
const MIN_CANVAS_WIDTH: f32 = 200.0;
const MIN_CANVAS_HEIGHT: f32 = 200.0;
const MIN_TAB_BODY_HEIGHT: f32 = 300.0;

/// Canvas representation of a block node.
#[derive(Clone, Debug)]
pub struct CanvasBlock {
    pub sid: String,
    pub name: String,
    pub block_type: BlockType,
    pub pos: Pos2,
    pub size: Vec2,
    pub parameters: HashMap<String, String>,
    pub rotation: BlockRotation,
}

impl CanvasBlock {
    /// Returns the number of physical electrical terminals.
    pub fn terminal_count(&self) -> usize {
        self.block_type.terminal_count()
    }

    /// Returns the number of input ports based on block type and parameters.
    pub fn input_ports(&self) -> usize {
        self.block_type.input_ports(&self.parameters)
    }

    /// Returns the number of output ports based on block type.
    pub fn output_ports(&self) -> usize {
        self.block_type.output_ports()
    }
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

pub struct BlockDiagramLayout {
    #[allow(dead_code)]
    pub tab_body: Rect,
    pub toolbar: Rect,
    pub library: Rect,
    pub properties: Rect,
    pub canvas: Rect,
    pub library_search: Rect,
    pub toolbar_widgets: Vec<Rect>,
    pub library_categories: Vec<Rect>,
    pub library_category_names: Vec<&'static str>,
    pub library_scroll_offset: f32,
    pub library_scroll_content_height: f32,
    pub library_scroll_viewport_height: f32,
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
    pub library_search: String,
    pub fit_view_requested: bool,
    pub next_block_id: usize,

    // Parameter Edit Dialog (Modal)
    pub editing_block_sid: Option<String>,
    pub param_draft: HashMap<String, String>,
    pub param_errors: HashMap<String, String>,

    // Properties Panel (Right side)
    pub prop_draft_sid: Option<String>,
    pub prop_draft: HashMap<String, String>,
    pub prop_errors: HashMap<String, String>,

    // Simulation Config & Results
    pub config: SimulationConfig,
    pub auto_run: bool,
    pub sim_output: Option<SimulationOutput>,
    pub sim_status: String,
    pub show_scope_window: bool,
    pub scope_signal_name: String,
    pub scope_tab: usize,
    pub spectrum_signal_name: String,
    pub spectrum_window_size: usize,
    pub spectrum_window_fn: mlapp_sim::WindowFunction,
    pub spectrum_f0: f64,
    pub spectrum_db_scale: bool,

    // JSON Model Import/Export
    pub show_json_modal: bool,
    pub json_buffer: String,
    pub json_is_export: bool,
    pub json_error: Option<String>,

    // Diagnostics & Problems Panel
    pub show_problems_panel: bool,
    pub diagnostics: Vec<mlapp_sim::Diagnostic>,
    pub undo_diagram: Option<BlockDiagram>,
    pub undo_stack: Vec<BlockDiagram>,
    pub redo_stack: Vec<BlockDiagram>,
    pub is_simulating: bool,
    pub sim_cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl Default for BlockDiagramEditor {
    fn default() -> Self {
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
            next_block_id: 1,

            editing_block_sid: None,
            param_draft: HashMap::new(),
            param_errors: HashMap::new(),

            prop_draft_sid: None,
            prop_draft: HashMap::new(),
            prop_errors: HashMap::new(),

            config: SimulationConfig {
                start_time: 0.0,
                stop_time: 5.0,
                step_size: 0.01,
                solver: SolverType::FixedStepEuler,
                ..Default::default()
            },
            auto_run: false,
            sim_output: None,
            sim_status: "Ready".to_string(),
            show_scope_window: false,
            scope_signal_name: "All Signals".to_string(),
            scope_tab: 0,
            spectrum_signal_name: "yout".to_string(),
            spectrum_window_size: 1024,
            spectrum_window_fn: mlapp_sim::WindowFunction::Hann,
            spectrum_f0: 50.0,
            spectrum_db_scale: false,

            show_json_modal: false,
            json_buffer: String::new(),
            json_is_export: true,
            json_error: None,

            show_problems_panel: false,
            diagnostics: Vec::new(),
            undo_diagram: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_simulating: false,
            sim_cancel_flag: None,
        }
    }
}

impl BlockDiagramEditor {
    /// Generates the next monotonic unique block SID.
    pub fn generate_next_sid(&mut self) -> String {
        loop {
            let sid = self.next_block_id.to_string();
            self.next_block_id += 1;
            if !self.blocks.iter().any(|b| b.sid == sid) {
                return sid;
            }
        }
    }

    /// Duplicates the currently selected block, giving it a unique SID and an offset.
    pub fn duplicate_selected(&mut self) -> Option<String> {
        let sid = self.selected_block_sid.as_ref()?;
        let original = self.blocks.iter().find(|b| b.sid == *sid)?.clone();
        let new_sid = self.generate_next_sid();
        let new_name = format!("{} (Copy)", original.name);
        let new_pos = original.pos + vec2(30.0, 30.0);
        let duplicate = CanvasBlock {
            sid: new_sid.clone(),
            name: new_name,
            block_type: original.block_type,
            pos: new_pos,
            size: original.size,
            parameters: original.parameters,
            rotation: original.rotation,
        };
        self.blocks.push(duplicate);
        self.selected_block_sid = Some(new_sid.clone());
        self.selected_wire = None;
        self.sync_prop_draft();
        if self.auto_run {
            self.run_simulation();
        }
        Some(new_sid)
    }

    /// Deletes the currently selected block or wire.
    pub fn delete_selected(&mut self) {
        if let Some(sid) = self.selected_block_sid.take() {
            self.blocks.retain(|b| b.sid != sid);
            self.wires
                .retain(|w| w.src_block != sid && w.dst_block != sid);
            if self.editing_block_sid.as_deref() == Some(&sid) {
                self.editing_block_sid = None;
                self.param_draft.clear();
                self.param_errors.clear();
            }
            self.prop_draft_sid = None;
            self.prop_draft.clear();
            self.prop_errors.clear();
            if self.auto_run {
                self.run_simulation();
            }
        } else if let Some(index) = self.selected_wire.take() {
            if index < self.wires.len() {
                self.wires.remove(index);
                if self.auto_run {
                    self.run_simulation();
                }
            }
        }
    }

    /// Resets the selected block's parameters to default values.
    pub fn reset_selected_to_defaults(&mut self) {
        let Some(sid) = &self.selected_block_sid else {
            return;
        };
        let Some(block) = self.blocks.iter_mut().find(|b| b.sid == *sid) else {
            return;
        };
        block.parameters.clear();
        let specs = param_specs_for(&block.block_type);
        for spec in specs {
            block
                .parameters
                .insert(spec.key.to_string(), spec.default_val.to_string());
        }
        self.sync_prop_draft();
        if self.auto_run {
            self.run_simulation();
        }
    }

    /// Synchronizes the properties panel draft with the currently selected block.
    pub fn sync_prop_draft(&mut self) {
        if let Some(sid) = &self.selected_block_sid {
            if self.prop_draft_sid.as_ref() != Some(sid) {
                self.prop_draft_sid = Some(sid.clone());
                self.prop_draft.clear();
                self.prop_errors.clear();
                if let Some(block) = self.blocks.iter().find(|b| b.sid == *sid) {
                    let specs = param_specs_for(&block.block_type);
                    for spec in specs {
                        let val = block
                            .parameters
                            .get(spec.key)
                            .cloned()
                            .unwrap_or_else(|| spec.default_val.to_string());
                        self.prop_draft.insert(spec.key.to_string(), val);
                    }
                }
            }
        } else {
            self.prop_draft_sid = None;
            self.prop_draft.clear();
            self.prop_errors.clear();
        }
    }

    /// Opens the modal parameter editor dialog for a block.
    pub fn open_param_modal(&mut self, sid: &str) {
        if let Some(block) = self.blocks.iter().find(|b| b.sid == sid) {
            self.editing_block_sid = Some(sid.to_string());
            self.param_draft.clear();
            self.param_errors.clear();
            let specs = param_specs_for(&block.block_type);
            for spec in specs {
                let val = block
                    .parameters
                    .get(spec.key)
                    .cloned()
                    .unwrap_or_else(|| spec.default_val.to_string());
                self.param_draft.insert(spec.key.to_string(), val);
            }
        }
    }

    /// Serializes the current block diagram to a JSON string.
    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        let diagram = self.to_block_diagram();
        serde_json::to_string_pretty(&diagram)
    }

    /// Loads a block diagram from a JSON string.
    pub fn load_from_json(&mut self, json_str: &str) -> Result<(), String> {
        let diagram: BlockDiagram =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;
        self.load_from_block_diagram(&diagram);
        Ok(())
    }

    /// Loads the editor state from a BlockDiagram struct.
    pub fn load_from_block_diagram(&mut self, diagram: &BlockDiagram) {
        self.blocks.clear();
        self.wires.clear();
        self.selected_block_sid = None;
        self.selected_wire = None;
        self.connecting_from = None;
        self.sim_output = None;

        let mut max_id: usize = 0;
        for (i, b) in diagram.blocks.iter().enumerate() {
            let (pos, size) = if let Some(p) = &b.position {
                (
                    Pos2::new(p[0] as f32, p[1] as f32),
                    Vec2::new((p[2] - p[0]).abs() as f32, (p[3] - p[1]).abs() as f32),
                )
            } else {
                (
                    Pos2::new(100.0 + (i as f32 * 120.0), 150.0),
                    Vec2::new(90.0, 50.0),
                )
            };

            if let Ok(num) = b.sid.parse::<usize>() {
                if num > max_id {
                    max_id = num;
                }
            }

            self.blocks.push(CanvasBlock {
                sid: b.sid.clone(),
                name: b.name.clone(),
                block_type: b.block_type.clone(),
                pos,
                size,
                parameters: b.parameters.clone(),
                rotation: b.rotation,
            });
        }

        self.next_block_id = max_id + 1;

        for line in &diagram.lines {
            let src_port = if line.src.port_index > 0 {
                line.src.port_index - 1
            } else {
                0
            };
            let dst_port = if line.dst.port_index > 0 {
                line.dst.port_index - 1
            } else {
                0
            };
            self.wires.push(CanvasWire {
                src_block: line.src.block_sid.clone(),
                src_port,
                dst_block: line.dst.block_sid.clone(),
                dst_port,
                bend_offset: Vec2::ZERO,
            });
        }

        if let Some(cfg) = &diagram.simulation_config {
            self.config = cfg.clone();
        } else {
            // schema_version 1 migration: default to FixedStepEuler
            self.config.solver = SolverType::FixedStepEuler;
        }

        self.sim_status = format!(
            "Loaded diagram '{}' ({} blocks, {} wires)",
            diagram.name,
            self.blocks.len(),
            self.wires.len()
        );
        self.refresh_diagnostics();
    }

    /// Re-evaluates static diagnostics against the current diagram.
    pub fn refresh_diagnostics(&mut self) {
        let diagram = self.to_block_diagram();
        self.diagnostics = mlapp_sim::check_diagram(&diagram, Some(&self.config));
    }

    /// Loads a clean-room feedback control loop example:
    /// Step -> Sum (-) <- Gain (from Integrator) -> Integrator -> Scope.
    pub fn load_default_feedback_example(&mut self) {
        self.blocks.clear();
        self.wires.clear();
        self.selected_block_sid = None;
        self.selected_wire = None;
        self.connecting_from = None;

        // 1. Step block
        let mut step = CanvasBlock {
            sid: "1".to_string(),
            name: "Step Input".to_string(),
            block_type: BlockType::Step,
            pos: Pos2::new(80.0, 150.0),
            size: Vec2::new(90.0, 50.0),
            parameters: HashMap::new(),
            rotation: BlockRotation::R0,
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
            rotation: BlockRotation::R0,
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
            rotation: BlockRotation::R0,
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
            rotation: BlockRotation::R0,
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
            rotation: BlockRotation::R0,
        };

        self.blocks.push(step);
        self.blocks.push(sum);
        self.blocks.push(gain);
        self.blocks.push(intg);
        self.blocks.push(scope);

        self.next_block_id = 6;

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

        self.sync_prop_draft();
    }

    /// Converts canvas model to `mlapp_sim::BlockDiagram`.
    pub fn to_block_diagram(&self) -> BlockDiagram {
        let mut diagram = BlockDiagram::new("Untitled_Diagram");
        diagram.schema_version = 3;
        diagram.simulation_config = Some(self.config.clone());
        for b in &self.blocks {
            let mut sb = SimBlock::new(&b.sid, &b.name, b.block_type.clone());
            sb.rotation = b.rotation;
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
            let src_block_opt = self.blocks.iter().find(|b| b.sid == w.src_block);
            let dst_block_opt = self.blocks.iter().find(|b| b.sid == w.dst_block);

            let src_is_electrical = src_block_opt
                .map(|b| {
                    if matches!(
                        b.block_type,
                        BlockType::IdealSwitch
                            | BlockType::VoltageControlledVoltageSource
                            | BlockType::VoltageControlledCurrentSource
                            | BlockType::VoltageSensor
                            | BlockType::CurrentSensor
                    ) && w.src_port == 2
                    {
                        false
                    } else {
                        b.block_type.is_electrical()
                    }
                })
                .unwrap_or(false);

            let dst_is_electrical = dst_block_opt
                .map(|b| {
                    if matches!(
                        b.block_type,
                        BlockType::IdealSwitch
                            | BlockType::VoltageControlledVoltageSource
                            | BlockType::VoltageControlledCurrentSource
                            | BlockType::VoltageSensor
                            | BlockType::CurrentSensor
                    ) && w.dst_port == 2
                    {
                        false
                    } else {
                        b.block_type.is_electrical()
                    }
                })
                .unwrap_or(false);

            let src = if src_is_electrical {
                PortRef::electrical(&w.src_block, w.src_port)
            } else {
                PortRef::signal(&w.src_block, w.src_port + 1)
            };

            let dst = if dst_is_electrical {
                PortRef::electrical(&w.dst_block, w.dst_port)
            } else {
                PortRef::signal(&w.dst_block, w.dst_port + 1)
            };

            diagram.lines.push(SimLine {
                src,
                dst,
                name: None,
            });
        }

        diagram
    }

    /// Executes the simulation model using `mlapp-sim`.
    pub fn run_simulation(&mut self) {
        if let Err(SimError::Parameter { message, .. }) = self.config.validate() {
            self.sim_status = format!("Simulation Parameter Error: {message}");
            return;
        }
        let diagram = self.to_block_diagram();
        match simulate_diagram(&diagram, &self.config) {
            Ok(output) => {
                let warning_count = output
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == mlapp_sim::DiagnosticSeverity::Warning)
                    .count();
                if warning_count > 0 {
                    let first_warn = output
                        .diagnostics
                        .iter()
                        .find(|d| d.severity == mlapp_sim::DiagnosticSeverity::Warning)
                        .unwrap();
                    self.sim_status = format!(
                        "Simulation finished with {} warning(s): {} ({} time points)",
                        warning_count,
                        first_warn.message,
                        output.tout.len()
                    );
                } else {
                    self.sim_status = format!(
                        "Simulation finished: {} time points (0.0s to {:.2}s)",
                        output.tout.len(),
                        self.config.stop_time
                    );
                }
                self.sim_output = Some(output);
                self.show_scope_window = true;
            }
            Err(e) => {
                self.sim_status = format!("Simulation Error: {}", e.message());
            }
        }
    }

    /// Pushes the current diagram state to the undo history stack.
    pub fn push_undo_state(&mut self) {
        let current = self.to_block_diagram();
        self.undo_stack.push(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Reverts the diagram to the previous state on the undo stack.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            let current = self.to_block_diagram();
            self.redo_stack.push(current);
            self.load_from_block_diagram(&prev);
            self.refresh_diagnostics();
            true
        } else {
            false
        }
    }

    /// Re-applies the previously undone state from the redo stack.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            let current = self.to_block_diagram();
            self.undo_stack.push(current);
            self.load_from_block_diagram(&next);
            self.refresh_diagnostics();
            true
        } else {
            false
        }
    }

    /// Cancels a currently running background simulation.
    pub fn cancel_simulation(&mut self) {
        if let Some(flag) = &self.sim_cancel_flag {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.is_simulating = false;
        self.sim_status = "Simulation cancelled by user".to_string();
    }
}

impl BlockDiagramEditor {
    /// Renders the complete Block Diagram Editor panel.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        self.show_layout(ui, theme);
    }

    pub fn show_layout(&mut self, ui: &mut Ui, theme: &Theme) -> BlockDiagramLayout {
        let body_rect = ui.available_rect_before_wrap();
        let min_body_width = DEFAULT_LIBRARY_WIDTH
            + DEFAULT_PROPERTIES_WIDTH
            + MIN_CANVAS_WIDTH
            + ui.spacing().item_spacing.x * 2.0;
        let layout =
            if body_rect.width() < min_body_width || body_rect.height() < MIN_TAB_BODY_HEIGHT {
                ScrollArea::both()
                    .id_salt("block_diagram_body_scroll")
                    .auto_shrink([false, false])
                    .min_scrolled_width(min_body_width)
                    .min_scrolled_height(MIN_TAB_BODY_HEIGHT)
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

        // JSON Import/Export Modal if opened
        if self.show_json_modal {
            self.render_json_modal(ui.ctx(), theme);
        }

        // Problems Window if opened
        if self.show_problems_panel {
            self.render_problems_modal(ui.ctx(), theme);
        }

        layout
    }

    fn render_panels(&mut self, ui: &mut Ui, theme: &Theme, tab_body: Rect) -> BlockDiagramLayout {
        let mut layout = BlockDiagramLayout {
            tab_body,
            toolbar: Rect::NOTHING,
            library: Rect::NOTHING,
            properties: Rect::NOTHING,
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
                let min = ui.next_widget_position();
                let max_y = ui.available_rect_before_wrap().max.y;
                let max = egui::pos2(min.x + tab_body.width(), max_y);
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(Rect::from_min_max(min, max)),
                    |ui| {
                        ui.set_clip_rect(ui.clip_rect().intersect(tab_body));
                        self.render_toolbar(ui, theme, &mut toolbar_widgets, max.x);
                    },
                );
            });
        layout.toolbar = toolbar.response.rect;
        layout.toolbar_widgets = toolbar_widgets;

        let max_library_width = ui.available_width() * 0.35;
        let library = egui::SidePanel::left("block_diagram_library")
            .resizable(true)
            .default_width(DEFAULT_LIBRARY_WIDTH)
            .width_range(MIN_LIBRARY_WIDTH..=max_library_width.max(MIN_LIBRARY_WIDTH))
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(tab_body));
                self.render_library_browser(ui, theme, &mut layout)
            });
        layout.library = library.response.rect;

        let max_properties_width = ui.available_width() * 0.4;
        let properties = egui::SidePanel::right("block_diagram_properties")
            .resizable(true)
            .default_width(DEFAULT_PROPERTIES_WIDTH)
            .width_range(MIN_PROPERTIES_WIDTH..=max_properties_width.max(MIN_PROPERTIES_WIDTH))
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(tab_body));
                self.render_properties_panel(ui, theme, &mut layout);
            });
        layout.properties = properties.response.rect;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(tab_body));
                layout.canvas = self.render_canvas(ui, theme);
            });
        layout
    }

    fn render_toolbar(&mut self, ui: &mut Ui, theme: &Theme, widgets: &mut Vec<Rect>, max_x: f32) {
        let wrap = |ui: &mut Ui, width: f32| {
            if ui.next_widget_position().x + width > max_x + 1.0 {
                ui.end_row();
            }
        };
        ui.horizontal_wrapped(|ui| {
            wrap(ui, 120.0);
            widgets.push(
                ui.add(egui::Label::new(
                    RichText::new("Block Diagram Editor").color(theme.text_primary),
                ))
                .rect,
            );
            ui.separator();

            wrap(ui, 115.0);
            let run = ui
                .button(RichText::new("Run Simulation").color(theme.accent).strong())
                .on_hover_text("Execute numerical integration of block diagram (Fixed-Step Euler)");
            widgets.push(run.rect);
            if run.clicked() {
                self.run_simulation();
            }

            wrap(ui, 50.0);
            let stop = ui
                .add_enabled(false, egui::Button::new("Stop"))
                .on_hover_text("The current fixed-step simulation runs synchronously");
            widgets.push(stop.rect);

            wrap(ui, 90.0);
            let scope = ui
                .button(RichText::new("Open Scope").color(theme.text_primary))
                .on_hover_text("Open oscilloscope signal waveform viewer");
            widgets.push(scope.rect);
            if scope.clicked() {
                self.show_scope_window = true;
            }

            wrap(ui, 80.0);
            let autorun_chk = ui
                .checkbox(&mut self.auto_run, "Auto-run")
                .on_hover_text("Automatically re-run simulation when parameters or wiring change");
            widgets.push(autorun_chk.rect);

            ui.separator();

            wrap(ui, 120.0);
            widgets.push(ui.label(RichText::new("Stop Time:").size(12.0)).rect);
            let stop_res = ui.add(
                egui::DragValue::new(&mut self.config.stop_time)
                    .speed(0.1)
                    .range(0.1..=100.0),
            );
            widgets.push(stop_res.rect);
            if stop_res.changed() && self.auto_run {
                self.run_simulation();
            }

            if self.config.solver == SolverType::VariableStepDormandPrince {
                wrap(ui, 110.0);
                widgets.push(ui.label(RichText::new("AbsTol:").size(12.0)).rect);
                let abs_res = ui.add(
                    egui::DragValue::new(&mut self.config.abs_tol)
                        .speed(1e-7)
                        .range(1e-12..=1e-1),
                );
                widgets.push(abs_res.rect);
                if abs_res.changed() && self.auto_run {
                    self.run_simulation();
                }

                wrap(ui, 110.0);
                widgets.push(ui.label(RichText::new("RelTol:").size(12.0)).rect);
                let rel_res = ui.add(
                    egui::DragValue::new(&mut self.config.rel_tol)
                        .speed(1e-4)
                        .range(1e-12..=1e-1),
                );
                widgets.push(rel_res.rect);
                if rel_res.changed() && self.auto_run {
                    self.run_simulation();
                }
            } else {
                wrap(ui, 80.0);
                widgets.push(ui.label(RichText::new("dt:").size(12.0)).rect);
                let dt_res = ui.add(
                    egui::DragValue::new(&mut self.config.step_size)
                        .speed(0.001)
                        .range(0.001..=0.5),
                );
                widgets.push(dt_res.rect);
                if dt_res.changed() && self.auto_run {
                    self.run_simulation();
                }
            }

            wrap(ui, 180.0);
            widgets.push(ui.label("Solver:").rect);
            let solver_title = match self.config.solver {
                SolverType::FixedStepEuler => "Fixed-step Euler (ode1)",
                SolverType::FixedStepRk4 => "Fixed-step RK4 (ode4)",
                SolverType::VariableStepDormandPrince => "Variable-step Dormand-Prince (ode45)",
            };
            let mut solver_changed = false;
            let solver = egui::ComboBox::from_id_salt("block_diagram_solver")
                .selected_text(solver_title)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut self.config.solver,
                            SolverType::FixedStepEuler,
                            "Fixed-step Euler (ode1)",
                        )
                        .clicked()
                    {
                        solver_changed = true;
                    }
                    if ui
                        .selectable_value(
                            &mut self.config.solver,
                            SolverType::FixedStepRk4,
                            "Fixed-step RK4 (ode4)",
                        )
                        .clicked()
                    {
                        solver_changed = true;
                    }
                    if ui
                        .selectable_value(
                            &mut self.config.solver,
                            SolverType::VariableStepDormandPrince,
                            "Variable-step Dormand-Prince (ode45)",
                        )
                        .clicked()
                    {
                        solver_changed = true;
                    }
                });
            widgets.push(solver.response.rect);
            if solver_changed && self.auto_run {
                self.run_simulation();
            }

            if let Err(SimError::Parameter { message, .. }) = self.config.validate() {
                wrap(ui, 120.0);
                let val_lbl = ui
                    .label(
                        RichText::new("⚠️ Invalid Param")
                            .color(Color32::from_rgb(255, 80, 80))
                            .size(11.0),
                    )
                    .on_hover_text(&message);
                widgets.push(val_lbl.rect);
            }

            ui.separator();

            wrap(ui, 155.0);
            let load_example = ui.button("Load Feedback Example");
            widgets.push(load_example.rect);
            if load_example.clicked() {
                self.load_default_feedback_example();
            }

            wrap(ui, 100.0);
            let save_btn = ui.button("Save Diagram");
            widgets.push(save_btn.rect);
            if save_btn.clicked() {
                if let Ok(json) = self.save_to_json() {
                    self.json_buffer = json;
                    self.json_is_export = true;
                    self.json_error = None;
                    self.show_json_modal = true;
                }
            }

            wrap(ui, 100.0);
            let open_btn = ui.button("Open Diagram");
            widgets.push(open_btn.rect);
            if open_btn.clicked() {
                self.json_buffer = String::new();
                self.json_is_export = false;
                self.json_error = None;
                self.show_json_modal = true;
            }

            wrap(ui, 90.0);
            let clear = ui.button("Clear Canvas");
            widgets.push(clear.rect);
            if clear.clicked() {
                self.blocks.clear();
                self.wires.clear();
                self.selected_block_sid = None;
                self.selected_wire = None;
                self.connecting_from = None;
                self.sim_output = None;
                self.sim_status = "Canvas cleared".to_string();
                self.prop_draft_sid = None;
                self.prop_draft.clear();
                self.prop_errors.clear();
            }

            wrap(ui, 65.0);
            let fit = ui.button("Fit View");
            widgets.push(fit.rect);
            if fit.clicked() {
                self.fit_view_requested = true;
            }

            wrap(ui, 45.0);
            let zoom = ui.label(format!("{:.0}%", self.zoom * 100.0));
            widgets.push(zoom.rect);

            wrap(ui, 110.0);
            let prob_label = if self.diagnostics.is_empty() {
                "Problems (0)".to_string()
            } else {
                format!("Problems ({})", self.diagnostics.len())
            };
            let prob_btn = ui.toggle_value(&mut self.show_problems_panel, prob_label);
            widgets.push(prob_btn.rect);

            wrap(ui, 100.0);
            let status = ui
                .label(RichText::new(&self.sim_status).size(11.0).color(
                    if self.sim_status.contains("Error") {
                        theme.error
                    } else {
                        theme.text_secondary
                    },
                ))
                .on_hover_text(&self.sim_status);
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
                    ("Sawtooth", BlockType::Sawtooth),
                    ("Triangle", BlockType::Triangle),
                    ("Pulse Generator", BlockType::PulseGenerator),
                    ("Repeating Sequence", BlockType::RepeatingSequence),
                    (
                        "Repeating Sequence Stair",
                        BlockType::RepeatingSequenceStair,
                    ),
                    ("Clock", BlockType::Clock),
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
                    ("Abs", BlockType::Abs),
                    ("Math Function", BlockType::MathFunction),
                    ("Saturation", BlockType::Saturation),
                ]),
            ),
            (
                "Continuous",
                items(&[
                    ("Integrator", BlockType::Integrator),
                    ("Transfer Fcn", BlockType::TransferFunction),
                    ("PID Controller", BlockType::Pid),
                ]),
            ),
            (
                "Discrete",
                items(&[
                    ("Unit Delay", BlockType::UnitDelay),
                    ("Discrete Transfer Fcn", BlockType::DiscreteTransferFunction),
                    ("Zero-Order Hold", BlockType::ZeroOrderHold),
                ]),
            ),
            (
                "Signal Routing",
                items(&[
                    ("Inport", BlockType::Inport),
                    ("Outport", BlockType::Outport),
                ]),
            ),
            (
                "Logic",
                items(&[
                    ("Relational Operator", BlockType::RelationalOperator),
                    ("Logical Operator", BlockType::LogicalOperator),
                    ("Switch", BlockType::Switch),
                ]),
            ),
            (
                "Electrical",
                items(&[
                    ("Resistor", BlockType::Resistor),
                    ("Capacitor", BlockType::Capacitor),
                    ("Inductor", BlockType::Inductor),
                    ("DC Voltage Source", BlockType::DcVoltageSource),
                    ("AC Voltage Source", BlockType::AcVoltageSource),
                    ("DC Current Source", BlockType::DcCurrentSource),
                    ("Ground", BlockType::Ground),
                    ("Diode", BlockType::Diode),
                    ("Ideal Switch", BlockType::IdealSwitch),
                    ("Voltage Sensor", BlockType::VoltageSensor),
                    ("Current Sensor", BlockType::CurrentSensor),
                    (
                        "Controlled Voltage Source",
                        BlockType::VoltageControlledVoltageSource,
                    ),
                    (
                        "Controlled Current Source",
                        BlockType::VoltageControlledCurrentSource,
                    ),
                ]),
            ),
            (
                "Measurements",
                items(&[
                    ("RMS", BlockType::Rms),
                    ("Mean", BlockType::Mean),
                    ("Power Measurement", BlockType::PowerMeasurement),
                    ("THD", BlockType::Thd),
                ]),
            ),
            ("Subsystems", items(&[("Subsystem", BlockType::Subsystem)])),
        ]
    }

    pub fn add_block(&mut self, block_type: BlockType, name: &str) {
        let pos = Pos2::new(
            100.0 + (self.blocks.len() as f32 * 20.0),
            120.0 + (self.blocks.len() as f32 * 15.0),
        );
        self.add_block_at(block_type, name, pos);
    }

    pub fn add_block_at(&mut self, block_type: BlockType, name: &str, pos: Pos2) {
        let new_id = self.generate_next_sid();
        let mut block = CanvasBlock {
            sid: new_id,
            name: name.to_string(),
            block_type,
            pos,
            size: Vec2::new(90.0, 50.0),
            parameters: HashMap::new(),
            rotation: BlockRotation::R0,
        };

        // Initialize default parameters
        let specs = param_specs_for(&block.block_type);
        for spec in specs {
            block
                .parameters
                .insert(spec.key.to_string(), spec.default_val.to_string());
        }

        self.blocks.push(block);
    }

    fn render_properties_panel(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        _layout: &mut BlockDiagramLayout,
    ) {
        ui.set_min_width(MIN_PROPERTIES_WIDTH);
        ui.label(
            RichText::new("Block Properties")
                .strong()
                .color(theme.text_secondary),
        );
        ui.separator();

        self.sync_prop_draft();

        let mut param_changed = false;
        let mut delete_requested = false;
        let mut duplicate_requested = false;
        let mut reset_requested = false;
        let mut open_scope_requested = false;

        if let Some(sid) = self.selected_block_sid.clone() {
            if let Some(block_idx) = self.blocks.iter().position(|b| b.sid == sid) {
                let block_type = self.blocks[block_idx].block_type.clone();
                let mut block_name = self.blocks[block_idx].name.clone();

                ScrollArea::vertical()
                    .id_salt("sim_properties_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Name:").strong().size(12.0));
                            if ui.text_edit_singleline(&mut block_name).changed() {
                                self.blocks[block_idx].name = block_name;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Type:").size(11.0).color(theme.text_secondary));
                            ui.label(RichText::new(format!("{:?}", block_type)).size(11.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Rotation:").size(11.0).color(theme.text_secondary));
                            let rot_label = match self.blocks[block_idx].rotation {
                                BlockRotation::R0 => "0°",
                                BlockRotation::R90 => "90°",
                                BlockRotation::R180 => "180°",
                                BlockRotation::R270 => "270°",
                            };
                            if ui.button(format!("{rot_label} (Rotate 90°)")).on_hover_text("Rotate block 90° clockwise (Hotkey: R)").clicked() {
                                self.blocks[block_idx].rotation = self.blocks[block_idx].rotation.rotated_cw();
                            }
                        });

                        if block_type == BlockType::Scope {
                            ui.add_space(4.0);
                            if ui.button("Open Scope Waveforms").clicked() {
                                open_scope_requested = true;
                            }
                        }

                        ui.separator();
                        ui.label(RichText::new("Parameters").strong().color(theme.text_primary));

                        let specs = param_specs_for(&block_type);
                        if specs.is_empty() {
                            ui.label(
                                RichText::new("No configurable parameters.")
                                    .italics()
                                    .size(11.0),
                            );
                        } else {
                            for spec in specs {
                                ui.add_space(4.0);
                                ui.label(RichText::new(spec.label).strong().size(11.0))
                                    .on_hover_text(spec.doc);

                                let mut current_val = self
                                    .prop_draft
                                    .get(spec.key)
                                    .cloned()
                                    .unwrap_or_else(|| spec.default_val.to_string());

                                match spec.kind {
                                    ParamKind::Numeric => {
                                        ui.horizontal(|ui| {
                                            let mut parsed =
                                                current_val.trim().parse::<f64>().unwrap_or(0.0);
                                            if ui
                                                .add(egui::DragValue::new(&mut parsed).speed(0.1))
                                                .changed()
                                            {
                                                current_val = format!("{parsed}");
                                                self.prop_draft.insert(
                                                    spec.key.to_string(),
                                                    current_val.clone(),
                                                );
                                                self.prop_errors.remove(spec.key);
                                                self.blocks[block_idx].parameters.insert(
                                                    spec.key.to_string(),
                                                    current_val.clone(),
                                                );
                                                param_changed = true;
                                            }

                                            let edit_resp =
                                                ui.text_edit_singleline(&mut current_val);
                                            if edit_resp.changed() {
                                                self.prop_draft.insert(
                                                    spec.key.to_string(),
                                                    current_val.clone(),
                                                );
                                                let is_optional_empty = current_val.trim().is_empty()
                                                    && (spec.key == "UpperSaturationLimit"
                                                        || spec.key == "LowerSaturationLimit"
                                                        || spec.key == "SampleTime");
                                                if is_optional_empty {
                                                    self.prop_errors.remove(spec.key);
                                                    self.blocks[block_idx].parameters.insert(
                                                        spec.key.to_string(),
                                                        String::new(),
                                                    );
                                                    param_changed = true;
                                                } else {
                                                    match current_val.trim().parse::<f64>() {
                                                        Ok(_) => {
                                                            self.prop_errors.remove(spec.key);
                                                            self.blocks[block_idx].parameters.insert(
                                                                spec.key.to_string(),
                                                                current_val.trim().to_string(),
                                                            );
                                                            param_changed = true;
                                                        }
                                                        Err(_) => {
                                                            self.prop_errors.insert(
                                                                spec.key.to_string(),
                                                                "Must be a valid number (e.g. 1.0, -0.5, 1e-3)".to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        });

                                        // Cross-validation for Saturation: LowerLimit <= UpperLimit
                                        if block_type == BlockType::Saturation
                                            && !self.prop_errors.contains_key(spec.key)
                                        {
                                            let upper_str = self.blocks[block_idx]
                                                .parameters
                                                .get("UpperLimit")
                                                .map(|s| s.as_str())
                                                .unwrap_or("1.0");
                                            let lower_str = self.blocks[block_idx]
                                                .parameters
                                                .get("LowerLimit")
                                                .map(|s| s.as_str())
                                                .unwrap_or("-1.0");
                                            if let (Ok(u), Ok(l)) = (
                                                upper_str.trim().parse::<f64>(),
                                                lower_str.trim().parse::<f64>(),
                                            ) {
                                                if l > u {
                                                    self.prop_errors.insert(
                                                        "LowerLimit".to_string(),
                                                        "Lower limit must be <= Upper limit"
                                                            .to_string(),
                                                    );
                                                } else {
                                                    self.prop_errors.remove("LowerLimit");
                                                }
                                            }
                                        }
                                    }
                                    ParamKind::Choice(choices) => {
                                        let mut selected_choice = current_val.clone();
                                        egui::ComboBox::from_id_salt(format!("choice_{}_{}", sid, spec.key))
                                            .selected_text(&selected_choice)
                                            .show_ui(ui, |ui| {
                                                for &choice in choices {
                                                    if ui.selectable_value(&mut selected_choice, choice.to_string(), choice).changed() {
                                                        current_val = selected_choice.clone();
                                                        self.prop_draft.insert(spec.key.to_string(), current_val.clone());
                                                        self.blocks[block_idx].parameters.insert(spec.key.to_string(), current_val.clone());
                                                        param_changed = true;
                                                    }
                                                }
                                            });
                                    }
                                    ParamKind::Vector => {
                                        let edit_resp = ui.text_edit_singleline(&mut current_val);
                                        if edit_resp.changed() {
                                            self.prop_draft.insert(spec.key.to_string(), current_val.clone());
                                            match parse_float_vector(&current_val) {
                                                Ok(_) => {
                                                    self.prop_errors.remove(spec.key);
                                                    self.blocks[block_idx].parameters.insert(spec.key.to_string(), current_val.trim().to_string());
                                                    param_changed = true;
                                                }
                                                Err(e) => {
                                                    self.prop_errors.insert(spec.key.to_string(), e);
                                                }
                                            }
                                        }
                                    }
                                    ParamKind::Text => {
                                        let edit_resp = ui.text_edit_singleline(&mut current_val);
                                        if edit_resp.changed() {
                                            self.prop_draft.insert(spec.key.to_string(), current_val.clone());
                                            self.blocks[block_idx].parameters.insert(spec.key.to_string(), current_val.trim().to_string());
                                            param_changed = true;
                                        }
                                    }
                                    ParamKind::SumInputs => {
                                        let edit_resp = ui.text_edit_singleline(&mut current_val);
                                        if edit_resp.changed() {
                                            self.prop_draft
                                                .insert(spec.key.to_string(), current_val.clone());
                                            let is_valid = !current_val.is_empty()
                                                && current_val
                                                    .chars()
                                                    .all(|c| c == '+' || c == '-');
                                            if is_valid {
                                                self.prop_errors.remove(spec.key);
                                                self.blocks[block_idx]
                                                    .parameters
                                                    .insert(spec.key.to_string(), current_val);
                                                param_changed = true;
                                            } else {
                                                self.prop_errors.insert(
                                                    spec.key.to_string(),
                                                    "Inputs must contain only '+' and '-' signs (e.g. '+-' or '++-')".to_string(),
                                                );
                                            }
                                        }
                                    }
                                }

                                if let Some(err) = self.prop_errors.get(spec.key) {
                                    ui.colored_label(Color32::from_rgb(255, 90, 90), err);
                                }
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.label(RichText::new("Actions").strong().color(theme.text_secondary));

                        ui.horizontal(|ui| {
                            if ui.button("Reset Defaults").clicked() {
                                reset_requested = true;
                            }
                            if ui.button("Duplicate (Ctrl+D)").clicked() {
                                duplicate_requested = true;
                            }
                        });

                        ui.add_space(4.0);
                        if ui.button(RichText::new("Delete Block").color(theme.error)).clicked() {
                            delete_requested = true;
                        }
                    });
            }
        } else if let Some(wire_idx) = self.selected_wire {
            if wire_idx < self.wires.len() {
                let wire = &self.wires[wire_idx];
                ui.label(RichText::new("Selected Wire").strong());
                ui.add_space(4.0);
                ui.label(format!(
                    "From: Block {} (Port {})",
                    wire.src_block,
                    wire.src_port + 1
                ));
                ui.label(format!(
                    "To:   Block {} (Port {})",
                    wire.dst_block,
                    wire.dst_port + 1
                ));
                ui.add_space(8.0);
                if ui
                    .button(RichText::new("Delete Wire").color(theme.error))
                    .clicked()
                {
                    delete_requested = true;
                }
            }
        } else {
            // Nothing selected
            ui.label(
                RichText::new("No Selection")
                    .italics()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.label(format!("Blocks: {}", self.blocks.len()));
            ui.label(format!("Wires:  {}", self.wires.len()));
            ui.add_space(8.0);
            ui.label(RichText::new("Tips:").strong().size(11.0));
            ui.label(RichText::new("• Click block to select & edit properties").size(11.0));
            ui.label(RichText::new("• Double-click block to open modal").size(11.0));
            ui.label(RichText::new("• Drag from blue pin to green pin to connect").size(11.0));
            ui.label(RichText::new("• Press Delete or Backspace to remove").size(11.0));
            ui.label(RichText::new("• Press Ctrl+D to duplicate selected block").size(11.0));
        }

        if open_scope_requested {
            self.show_scope_window = true;
        }
        if reset_requested {
            self.reset_selected_to_defaults();
        }
        if duplicate_requested {
            self.duplicate_selected();
        }
        if delete_requested {
            self.delete_selected();
        }
        if param_changed && self.auto_run {
            self.run_simulation();
        }
    }

    fn render_canvas(&mut self, ui: &mut Ui, _theme: &Theme) -> Rect {
        let canvas_size = ui
            .available_size()
            .max(vec2(MIN_CANVAS_WIDTH, MIN_CANVAS_HEIGHT));
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

        let topology_opt = resolve_electrical_nets(&self.to_block_diagram()).topology;

        for (wire_index, wire) in self.wires.iter_mut().enumerate() {
            let Some(source) = self.blocks.iter().find(|block| block.sid == wire.src_block) else {
                continue;
            };
            let Some(target) = self.blocks.iter().find(|block| block.sid == wire.dst_block) else {
                continue;
            };
            let start =
                Self::get_block_port_pos(rect, self.pan_offset, self.zoom, source, wire.src_port);
            let end = Self::get_block_input_port_pos(
                rect,
                self.pan_offset,
                self.zoom,
                target,
                wire.dst_port,
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
            let is_electrical =
                source.block_type.is_electrical() && target.block_type.is_electrical();
            let stroke = Stroke::new(
                if selected { 3.0_f32 } else { 2.0_f32 },
                if selected {
                    Color32::from_rgb(255, 180, 50)
                } else if is_electrical {
                    Color32::from_rgb(100, 200, 255)
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
            painter.circle_filled(
                end,
                3.5,
                if is_electrical {
                    Color32::from_rgb(100, 200, 255)
                } else {
                    Color32::from_rgb(90, 180, 255)
                },
            );
        }

        if let Some((source_sid, source_port)) = &self.connecting_from {
            if let Some(source) = self.blocks.iter().find(|block| block.sid == *source_sid) {
                let start = Self::get_block_port_pos(
                    rect,
                    self.pan_offset,
                    self.zoom,
                    source,
                    *source_port,
                );
                if let Some(pointer) = response.hover_pos() {
                    painter.line_segment(
                        [start, pointer],
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 200, 80)),
                    );
                }
            }
        }

        let mut wire_to_connect = None;
        let space_pan = ui.input(|input| input.key_down(Key::Space));
        let block_domains: HashMap<String, (bool, bool)> = self
            .blocks
            .iter()
            .map(|b| {
                (
                    b.sid.clone(),
                    (
                        b.block_type.is_electrical(),
                        matches!(
                            b.block_type,
                            BlockType::IdealSwitch
                                | BlockType::VoltageControlledVoltageSource
                                | BlockType::VoltageControlledCurrentSource
                                | BlockType::VoltageSensor
                                | BlockType::CurrentSensor
                        ),
                    ),
                )
            })
            .collect();

        for block in &mut self.blocks {
            let block_rect = Rect::from_min_size(
                rect.min + self.pan_offset.to_vec2() + block.pos.to_vec2() * self.zoom,
                block.size * self.zoom,
            );
            let block_resp = ui
                .interact(
                    block_rect,
                    Id::new(format!("sim_block_{}", block.sid)),
                    Sense::click_and_drag(),
                )
                .on_hover_text(
                    "Double-click to edit parameters, or edit in Properties panel on the right. Press 'R' to rotate.",
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
                BlockType::Resistor => "-[R]-",
                BlockType::Capacitor => "-||-",
                BlockType::Inductor => "-mmm-",
                BlockType::DcVoltageSource => "(V)",
                BlockType::AcVoltageSource => "(~V)",
                BlockType::DcCurrentSource => "(I)",
                BlockType::Ground => "[GND]",
                BlockType::Diode => "->|-",
                BlockType::IdealSwitch => "-/ -",
                BlockType::VoltageSensor => "(V)",
                BlockType::CurrentSensor => "(A)",
                BlockType::VoltageControlledVoltageSource => "(K*u)",
                BlockType::VoltageControlledCurrentSource => "(Gm*u)",
                BlockType::Rms => "[RMS]",
                BlockType::Mean => "[AVG]",
                BlockType::PowerMeasurement => "[P-Q]",
                BlockType::Thd => "[THD]",
                _ => "[ ]",
            };
            painter.text(
                pos2(block_rect.center().x, block_rect.top() + 16.0 * self.zoom),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(14.0 * self.zoom),
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

            if block.block_type.is_electrical() {
                let term_count = block.terminal_count();
                for term_idx in 0..term_count {
                    let port_pos =
                        Self::get_block_port_pos(rect, self.pan_offset, self.zoom, block, term_idx);
                    let port_rect = Rect::from_center_size(port_pos, vec2(10.0, 10.0) * self.zoom);
                    // Open square for physical electrical terminal
                    painter.rect_stroke(
                        port_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.5 * self.zoom, Color32::from_rgb(100, 200, 255)),
                        StrokeKind::Middle,
                    );
                    painter.rect_filled(
                        port_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_premultiplied(30, 40, 60, 180),
                    );

                    let label = if term_idx == 0 { "+" } else { "-" };
                    let hover_text = if let Some(top) = &topology_opt {
                        if let Some(net_id) =
                            top.terminal_to_net.get(&(block.sid.clone(), term_idx))
                        {
                            format!(
                                "Electrical Terminal {} ({})\nNet: Node {}",
                                term_idx, label, net_id
                            )
                        } else {
                            format!("Electrical Terminal {} ({})", term_idx, label)
                        }
                    } else {
                        format!("Electrical Terminal {} ({})", term_idx, label)
                    };

                    let port_resp = ui
                        .interact(
                            Rect::from_center_size(port_pos, vec2(14.0, 14.0) * self.zoom),
                            Id::new(format!("elec_port_{}_{}", block.sid, term_idx)),
                            Sense::click(),
                        )
                        .on_hover_text(hover_text);

                    if port_resp.clicked() {
                        if let Some((source_sid, source_port)) = self.connecting_from.take() {
                            if source_sid != block.sid {
                                let (source_is_elec, source_is_switch) = block_domains
                                    .get(&source_sid)
                                    .copied()
                                    .unwrap_or((false, false));
                                let source_is_electrical = if source_is_switch && source_port == 2 {
                                    false
                                } else {
                                    source_is_elec
                                };

                                if !source_is_electrical {
                                    self.sim_status = format!(
                                        "Error [E-DOMAIN]: Direct wire between signal port and electrical terminal on blocks '{}' and '{}' rejected.",
                                        source_sid, block.sid
                                    );
                                } else {
                                    wire_to_connect = Some((
                                        source_sid,
                                        source_port,
                                        block.sid.clone(),
                                        term_idx,
                                    ));
                                }
                            }
                        } else {
                            self.connecting_from = Some((block.sid.clone(), term_idx));
                        }
                    }
                }

                if matches!(
                    block.block_type,
                    BlockType::IdealSwitch
                        | BlockType::VoltageControlledVoltageSource
                        | BlockType::VoltageControlledCurrentSource
                ) {
                    let ctrl_pos =
                        Self::get_block_input_port_pos(rect, self.pan_offset, self.zoom, block, 2);
                    painter.circle_filled(
                        ctrl_pos,
                        4.0 * self.zoom,
                        Color32::from_rgb(180, 220, 120),
                    );
                    let port_resp = ui
                        .interact(
                            Rect::from_center_size(ctrl_pos, vec2(12.0, 12.0) * self.zoom),
                            Id::new(format!("in_port_{}_ctrl", block.sid)),
                            Sense::click(),
                        )
                        .on_hover_text("Signal Control Input");

                    if port_resp.clicked() {
                        if let Some((source_sid, source_port)) = self.connecting_from.take() {
                            if source_sid != block.sid {
                                let (source_is_elec, source_is_hybrid) = block_domains
                                    .get(&source_sid)
                                    .copied()
                                    .unwrap_or((false, false));
                                let source_is_electrical = if source_is_hybrid && source_port == 2 {
                                    false
                                } else {
                                    source_is_elec
                                };

                                if source_is_electrical {
                                    self.sim_status = format!(
                                        "Error [E-DOMAIN]: Direct wire between electrical terminal and signal control input on blocks '{}' and '{}' rejected.",
                                        source_sid, block.sid
                                    );
                                } else {
                                    wire_to_connect =
                                        Some((source_sid, source_port, block.sid.clone(), 2));
                                }
                            }
                        }
                    }
                }

                if matches!(
                    block.block_type,
                    BlockType::VoltageSensor | BlockType::CurrentSensor
                ) {
                    let meas_pos =
                        Self::get_block_port_pos(rect, self.pan_offset, self.zoom, block, 2);
                    painter.circle_filled(
                        meas_pos,
                        4.0 * self.zoom,
                        Color32::from_rgb(90, 180, 255),
                    );
                    let port_resp = ui
                        .interact(
                            Rect::from_center_size(meas_pos, vec2(12.0, 12.0) * self.zoom),
                            Id::new(format!("out_port_{}_meas", block.sid)),
                            Sense::click(),
                        )
                        .on_hover_text("Signal Measurement Output");

                    if port_resp.clicked() {
                        if let Some((source_sid, source_port)) = self.connecting_from.take() {
                            if source_sid != block.sid {
                                let (source_is_elec, source_is_hybrid) = block_domains
                                    .get(&source_sid)
                                    .copied()
                                    .unwrap_or((false, false));
                                let source_is_electrical = if source_is_hybrid && source_port == 2 {
                                    false
                                } else {
                                    source_is_elec
                                };

                                if source_is_electrical {
                                    self.sim_status = format!(
                                        "Error [E-DOMAIN]: Direct wire between electrical terminal and signal measurement output on blocks '{}' and '{}' rejected.",
                                        source_sid, block.sid
                                    );
                                }
                            }
                        } else {
                            self.connecting_from = Some((block.sid.clone(), 2));
                        }
                    }
                }
            } else {
                // Output ports
                let num_outputs = block.output_ports();
                for port_idx in 0..num_outputs {
                    let output =
                        Self::get_block_port_pos(rect, self.pan_offset, self.zoom, block, port_idx);
                    painter.circle_filled(output, 4.0 * self.zoom, Color32::from_rgb(90, 180, 255));
                    if ui
                        .interact(
                            Rect::from_center_size(output, vec2(12.0, 12.0) * self.zoom),
                            Id::new(format!("out_port_{}_{}", block.sid, port_idx)),
                            Sense::click(),
                        )
                        .on_hover_text(format!("Signal Output {}", port_idx + 1))
                        .clicked()
                    {
                        self.connecting_from = Some((block.sid.clone(), port_idx));
                    }
                }

                // Input ports
                let num_inputs = block.input_ports();
                for port_idx in 0..num_inputs {
                    let input = Self::get_block_input_port_pos(
                        rect,
                        self.pan_offset,
                        self.zoom,
                        block,
                        port_idx,
                    );
                    painter.circle_filled(input, 4.0 * self.zoom, Color32::from_rgb(180, 220, 120));
                    if ui
                        .interact(
                            Rect::from_center_size(input, vec2(12.0, 12.0) * self.zoom),
                            Id::new(format!("in_port_{}_{}", block.sid, port_idx)),
                            Sense::click(),
                        )
                        .on_hover_text(format!("Signal Input {}", port_idx + 1))
                        .clicked()
                    {
                        if let Some((source_sid, source_port)) = self.connecting_from.take() {
                            if source_sid != block.sid {
                                let (source_is_elec, source_is_switch) = block_domains
                                    .get(&source_sid)
                                    .copied()
                                    .unwrap_or((false, false));
                                let source_is_electrical = if source_is_switch && source_port == 2 {
                                    false
                                } else {
                                    source_is_elec
                                };

                                if source_is_electrical {
                                    self.sim_status = format!(
                                        "Error [E-DOMAIN]: Direct wire between electrical terminal and signal input on blocks '{}' and '{}' rejected.",
                                        source_sid, block.sid
                                    );
                                } else {
                                    wire_to_connect = Some((
                                        source_sid,
                                        source_port,
                                        block.sid.clone(),
                                        port_idx,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((source_sid, source_port, dst_sid, dst_port)) = wire_to_connect {
            let dst_block = self.blocks.iter().find(|b| b.sid == dst_sid);
            let dst_is_signal = dst_block
                .map(|b| {
                    !b.block_type.is_electrical()
                        || (matches!(
                            b.block_type,
                            BlockType::IdealSwitch
                                | BlockType::VoltageControlledVoltageSource
                                | BlockType::VoltageControlledCurrentSource
                        ) && dst_port == 2)
                })
                .unwrap_or(true);
            if dst_is_signal {
                self.wires
                    .retain(|w| !(w.dst_block == dst_sid && w.dst_port == dst_port));
            }
            self.wires.push(CanvasWire {
                src_block: source_sid,
                src_port: source_port,
                dst_block: dst_sid,
                dst_port,
                bend_offset: Vec2::ZERO,
            });
            if self.auto_run {
                self.run_simulation();
            }
        }

        if let Some(sid) = clicked_block {
            self.selected_block_sid = Some(sid);
            self.selected_wire = None;
            self.sync_prop_draft();
        } else if let Some(index) = clicked_wire {
            self.selected_wire = Some(index);
            self.selected_block_sid = None;
            self.sync_prop_draft();
        }
        if let Some(sid) = double_clicked_block {
            self.open_param_modal(&sid);
        }
        if response.clicked() {
            self.connecting_from = None;
            self.selected_block_sid = None;
            self.selected_wire = None;
            self.sync_prop_draft();
        }

        let pointer_inside_canvas = ui
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pointer| rect.contains(pointer));

        let delete = pointer_inside_canvas
            && ui
                .input(|input| input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace));
        if delete {
            self.delete_selected();
        }

        let ctrl_d = pointer_inside_canvas
            && ui.input(|input| input.modifiers.command && input.key_pressed(Key::D));
        if ctrl_d {
            self.duplicate_selected();
        }

        let rotate_r = pointer_inside_canvas
            && !ui.memory(|mem| mem.focused().is_some())
            && ui.input(|input| input.key_pressed(Key::R));
        if rotate_r {
            if let Some(sid) = &self.selected_block_sid {
                if let Some(b) = self.blocks.iter_mut().find(|b| b.sid == *sid) {
                    b.rotation = b.rotation.rotated_cw();
                }
            }
        }

        rect
    }

    fn world_to_screen(canvas: Rect, pan_offset: Pos2, zoom: f32, point: Pos2) -> Pos2 {
        canvas.min + pan_offset.to_vec2() + point.to_vec2() * zoom
    }

    fn get_block_port_pos(
        canvas: Rect,
        pan_offset: Pos2,
        zoom: f32,
        block: &CanvasBlock,
        port_index: usize,
    ) -> Pos2 {
        let (nx, ny) = if block.block_type.is_electrical() {
            let is_hybrid = matches!(
                block.block_type,
                BlockType::IdealSwitch
                    | BlockType::VoltageControlledVoltageSource
                    | BlockType::VoltageControlledCurrentSource
                    | BlockType::VoltageSensor
                    | BlockType::CurrentSensor
            );
            if is_hybrid && port_index == 2 {
                pin_geometry(&block.block_type, PortKind::Signal, 0, block.rotation)
            } else {
                pin_geometry(
                    &block.block_type,
                    PortKind::Electrical,
                    port_index,
                    block.rotation,
                )
            }
        } else {
            let num_outputs = block.output_ports().max(1);
            let frac = (port_index as f32 + 1.0) / (num_outputs as f32 + 1.0);
            match block.rotation {
                BlockRotation::R0 => (1.0, frac),
                BlockRotation::R90 => (1.0 - frac, 1.0),
                BlockRotation::R180 => (0.0, 1.0 - frac),
                BlockRotation::R270 => (frac, 0.0),
            }
        };
        let block_topleft = Self::world_to_screen(canvas, pan_offset, zoom, block.pos);
        pos2(
            block_topleft.x + block.size.x * zoom * nx,
            block_topleft.y + block.size.y * zoom * ny,
        )
    }

    fn get_block_input_port_pos(
        canvas: Rect,
        pan_offset: Pos2,
        zoom: f32,
        block: &CanvasBlock,
        port_index: usize,
    ) -> Pos2 {
        let (nx, ny) = if block.block_type.is_electrical() {
            let is_hybrid = matches!(
                block.block_type,
                BlockType::IdealSwitch
                    | BlockType::VoltageControlledVoltageSource
                    | BlockType::VoltageControlledCurrentSource
                    | BlockType::VoltageSensor
                    | BlockType::CurrentSensor
            );
            if is_hybrid && port_index == 2 {
                pin_geometry(&block.block_type, PortKind::Signal, 0, block.rotation)
            } else {
                pin_geometry(
                    &block.block_type,
                    PortKind::Electrical,
                    port_index,
                    block.rotation,
                )
            }
        } else {
            let num_inputs = block.input_ports().max(1);
            let frac = (port_index as f32 + 1.0) / (num_inputs as f32 + 1.0);
            match block.rotation {
                BlockRotation::R0 => (0.0, frac),
                BlockRotation::R90 => (1.0 - frac, 0.0),
                BlockRotation::R180 => (1.0, 1.0 - frac),
                BlockRotation::R270 => (frac, 1.0),
            }
        };
        let block_topleft = Self::world_to_screen(canvas, pan_offset, zoom, block.pos);
        pos2(
            block_topleft.x + block.size.x * zoom * nx,
            block_topleft.y + block.size.y * zoom * ny,
        )
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
        let mut save_requested = false;
        let mut apply_requested = false;

        if let Some(sid) = self.editing_block_sid.clone() {
            if let Some(block) = self.blocks.iter().find(|b| b.sid == sid) {
                let block_name = block.name.clone();
                let block_type = block.block_type.clone();

                egui::Window::new(format!("Block Parameters: {}", block_name))
                    .id(egui::Id::new(format!("block-params-{sid}")))
                    .collapsible(false)
                    .resizable(false)
                    .default_width(360.0)
                    .show(ctx, |ui| {
                        ui.label(RichText::new(format!("Type: {:?}", block_type)).size(12.0));
                        ui.separator();

                        let specs = param_specs_for(&block_type);
                        if specs.is_empty() {
                            ui.label("No configurable parameters for this block.");
                        } else {
                            for spec in specs {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(spec.label)
                                        .strong()
                                        .color(theme.text_secondary),
                                )
                                .on_hover_text(spec.doc);

                                let current_val = self
                                    .param_draft
                                    .entry(spec.key.to_string())
                                    .or_insert_with(|| spec.default_val.to_string());

                                let mut edit_val = current_val.clone();
                                if ui.text_edit_singleline(&mut edit_val).changed() {
                                    *current_val = edit_val;
                                }

                                if let Some(err) = self.param_errors.get(spec.key) {
                                    ui.colored_label(Color32::from_rgb(255, 90, 90), err);
                                }
                            }
                        }

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                save_requested = true;
                            }
                            if ui.button("Apply").clicked() {
                                apply_requested = true;
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

        if save_requested || apply_requested {
            if let Some(sid) = &self.editing_block_sid {
                if let Some(block) = self.blocks.iter().find(|b| b.sid == *sid) {
                    let block_type = block.block_type.clone();
                    let specs = param_specs_for(&block_type);
                    self.param_errors.clear();

                    for spec in specs {
                        let val = self
                            .param_draft
                            .get(spec.key)
                            .map(|s| s.as_str())
                            .unwrap_or(spec.default_val);
                        match spec.kind {
                            ParamKind::Numeric => {
                                let is_optional_empty = val.trim().is_empty()
                                    && (spec.key == "UpperSaturationLimit"
                                        || spec.key == "LowerSaturationLimit"
                                        || spec.key == "SampleTime");
                                if !is_optional_empty && val.trim().parse::<f64>().is_err() {
                                    self.param_errors.insert(
                                        spec.key.to_string(),
                                        "Must be a valid number (e.g. 1.0, -0.5, 1e-3)".to_string(),
                                    );
                                }
                            }
                            ParamKind::Vector => {
                                if let Err(e) = parse_float_vector(val) {
                                    self.param_errors.insert(spec.key.to_string(), e);
                                }
                            }
                            ParamKind::Choice(choices) => {
                                if !choices.iter().any(|c| c.eq_ignore_ascii_case(val.trim())) {
                                    self.param_errors.insert(
                                        spec.key.to_string(),
                                        format!("Must be one of: {}", choices.join(", ")),
                                    );
                                }
                            }
                            ParamKind::Text => {}
                            ParamKind::SumInputs => {
                                let is_valid =
                                    !val.is_empty() && val.chars().all(|c| c == '+' || c == '-');
                                if !is_valid {
                                    self.param_errors.insert(
                                        spec.key.to_string(),
                                        "Inputs must contain only '+' and '-' signs (e.g. '+-' or '++-')"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }

                    // Saturation check LowerLimit <= UpperLimit
                    if block_type == BlockType::Saturation && self.param_errors.is_empty() {
                        let upper = self
                            .param_draft
                            .get("UpperLimit")
                            .and_then(|s| s.trim().parse::<f64>().ok())
                            .unwrap_or(1.0);
                        let lower = self
                            .param_draft
                            .get("LowerLimit")
                            .and_then(|s| s.trim().parse::<f64>().ok())
                            .unwrap_or(-1.0);
                        if lower > upper {
                            self.param_errors.insert(
                                "LowerLimit".to_string(),
                                "Lower limit cannot exceed Upper limit".to_string(),
                            );
                        }
                    }

                    if self.param_errors.is_empty() {
                        if let Some(block) = self.blocks.iter_mut().find(|b| b.sid == *sid) {
                            for (k, v) in &self.param_draft {
                                block.parameters.insert(k.clone(), v.clone());
                            }
                        }
                        self.sync_prop_draft();
                        if self.auto_run {
                            self.run_simulation();
                        }
                        if save_requested {
                            close_modal = true;
                        }
                    }
                }
            }
        }

        if close_modal {
            self.editing_block_sid = None;
            self.param_draft.clear();
            self.param_errors.clear();
        }
    }

    fn render_scope_modal(&mut self, ctx: &egui::Context, _theme: &Theme) {
        let mut close_scope = false;

        egui::Window::new("Scope Waveform Viewer")
            .default_size([700.0, 450.0])
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(output) = &self.sim_output {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.scope_tab, 0, "Time Domain");
                        ui.selectable_value(&mut self.scope_tab, 1, "Spectrum");
                        ui.separator();

                        if self.scope_tab == 0 {
                            ui.label(
                                RichText::new(format!("Points: {}", output.tout.len())).size(11.0),
                            );
                            ui.separator();
                            let mut signal_options = vec!["All Signals".to_string()];
                            if !output.signals.is_empty() {
                                let mut names: Vec<_> = output.signals.keys().cloned().collect();
                                names.sort();
                                signal_options.extend(names);
                            } else {
                                signal_options.push("yout".to_string());
                            }

                            egui::ComboBox::from_id_salt("scope_signal_selector")
                                .selected_text(&self.scope_signal_name)
                                .show_ui(ui, |ui| {
                                    for opt in &signal_options {
                                        ui.selectable_value(
                                            &mut self.scope_signal_name,
                                            opt.clone(),
                                            opt,
                                        );
                                    }
                                });
                        } else {
                            let mut signal_options = Vec::new();
                            if !output.signals.is_empty() {
                                let mut names: Vec<_> = output.signals.keys().cloned().collect();
                                names.sort();
                                signal_options.extend(names);
                            } else {
                                signal_options.push("yout".to_string());
                            }
                            if !signal_options.contains(&self.spectrum_signal_name) {
                                if let Some(first) = signal_options.first() {
                                    self.spectrum_signal_name = first.clone();
                                }
                            }

                            egui::ComboBox::from_id_salt("spectrum_sig_sel")
                                .selected_text(&self.spectrum_signal_name)
                                .show_ui(ui, |ui| {
                                    for opt in &signal_options {
                                        ui.selectable_value(
                                            &mut self.spectrum_signal_name,
                                            opt.clone(),
                                            opt,
                                        );
                                    }
                                });

                            ui.separator();
                            egui::ComboBox::from_id_salt("spectrum_win_size")
                                .selected_text(format!("N = {}", self.spectrum_window_size))
                                .show_ui(ui, |ui| {
                                    for &sz in &[128, 256, 512, 1024, 2048, 4096] {
                                        ui.selectable_value(
                                            &mut self.spectrum_window_size,
                                            sz,
                                            format!("{sz}"),
                                        );
                                    }
                                });

                            ui.separator();
                            let win_name = match self.spectrum_window_fn {
                                mlapp_sim::WindowFunction::Rectangular => "Rectangular",
                                mlapp_sim::WindowFunction::Hann => "Hann",
                                mlapp_sim::WindowFunction::FlatTop => "FlatTop",
                            };
                            egui::ComboBox::from_id_salt("spectrum_win_fn")
                                .selected_text(win_name)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.spectrum_window_fn,
                                        mlapp_sim::WindowFunction::Rectangular,
                                        "Rectangular",
                                    );
                                    ui.selectable_value(
                                        &mut self.spectrum_window_fn,
                                        mlapp_sim::WindowFunction::Hann,
                                        "Hann",
                                    );
                                    ui.selectable_value(
                                        &mut self.spectrum_window_fn,
                                        mlapp_sim::WindowFunction::FlatTop,
                                        "FlatTop",
                                    );
                                });

                            ui.separator();
                            ui.checkbox(&mut self.spectrum_db_scale, "dB");
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_scope = true;
                            }
                            if ui.button("Export CSV").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("CSV files", &["csv"])
                                    .set_file_name("simulation_results.csv")
                                    .save_file()
                                {
                                    if let Ok(mut f) = std::fs::File::create(path) {
                                        let _ = output.write_csv(&mut f);
                                    }
                                }
                            }
                        });
                    });
                    ui.separator();

                    let available_size = ui.available_size();
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(available_size.x, (available_size.y - 20.0).max(150.0)),
                        Sense::hover(),
                    );

                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(15, 18, 22));

                    if self.scope_tab == 0 {
                        let mut traces: Vec<(&str, &Vec<f64>)> = Vec::new();
                        if self.scope_signal_name == "All Signals" {
                            if !output.signals.is_empty() {
                                let mut sorted_keys: Vec<_> = output.signals.keys().collect();
                                sorted_keys.sort();
                                for k in sorted_keys {
                                    if let Some(v) = output.signals.get(k) {
                                        traces.push((k.as_str(), v));
                                    }
                                }
                            } else if !output.yout.is_empty() {
                                traces.push(("yout", &output.yout));
                            }
                        } else if let Some(sig) = output.signals.get(&self.scope_signal_name) {
                            traces.push((&self.scope_signal_name, sig));
                        } else if self.scope_signal_name == "yout" && !output.yout.is_empty() {
                            traces.push(("yout", &output.yout));
                        }

                        if output.tout.len() >= 2 && !traces.is_empty() {
                            let t_max = output.tout.last().copied().unwrap_or(1.0).max(0.001);

                            let mut y_min = f64::INFINITY;
                            let mut y_max = f64::NEG_INFINITY;
                            for (_, trace) in &traces {
                                for &y in *trace {
                                    if y.is_finite() {
                                        if y < y_min {
                                            y_min = y;
                                        }
                                        if y > y_max {
                                            y_max = y;
                                        }
                                    }
                                }
                            }
                            if !y_min.is_finite() || !y_max.is_finite() || y_max <= y_min {
                                y_min = -0.5;
                                y_max = 1.5;
                            } else {
                                let pad = ((y_max - y_min) * 0.1).max(0.05);
                                y_min -= pad;
                                y_max += pad;
                            }
                            let y_span = (y_max - y_min).max(0.001);

                            let map_pt = |t: f64, y: f64| -> Pos2 {
                                let norm_x = (t / t_max) as f32;
                                let norm_y = ((y - y_min) / y_span) as f32;
                                pos2(
                                    rect.left() + norm_x * (rect.width() - 30.0) + 15.0,
                                    rect.bottom() - norm_y * (rect.height() - 30.0) - 15.0,
                                )
                            };

                            if y_min <= 0.0 && y_max >= 0.0 {
                                let zero_y = map_pt(0.0, 0.0).y;
                                painter.line_segment(
                                    [pos2(rect.left(), zero_y), pos2(rect.right(), zero_y)],
                                    Stroke::new(1.0_f32, Color32::from_rgb(60, 70, 85)),
                                );
                            }

                            const SIGNAL_COLORS: [Color32; 6] = [
                                Color32::from_rgb(255, 215, 0),   // Gold
                                Color32::from_rgb(90, 180, 255),  // Sky Blue
                                Color32::from_rgb(255, 100, 100), // Coral Red
                                Color32::from_rgb(120, 220, 120), // Mint Green
                                Color32::from_rgb(220, 120, 220), // Orchid
                                Color32::from_rgb(255, 160, 60),  // Orange
                            ];

                            let mut legend_items: Vec<(String, Color32, f64)> = Vec::new();

                            for (idx, (sig_name, sig_data)) in traces.iter().enumerate() {
                                let color = SIGNAL_COLORS[idx % SIGNAL_COLORS.len()];
                                let n_points = sig_data.len().min(output.tout.len());
                                for i in 0..(n_points - 1) {
                                    let p1 = map_pt(output.tout[i], sig_data[i]);
                                    let p2 = map_pt(output.tout[i + 1], sig_data[i + 1]);
                                    painter.line_segment([p1, p2], Stroke::new(2.0_f32, color));
                                }
                                let final_val = sig_data.last().copied().unwrap_or(0.0);
                                legend_items.push((sig_name.to_string(), color, final_val));
                            }

                            let mut legend_y = rect.top() + 10.0;
                            for (sig_name, color, final_val) in legend_items {
                                painter.circle_filled(
                                    pos2(rect.left() + 20.0, legend_y + 6.0),
                                    4.0,
                                    color,
                                );
                                painter.text(
                                    pos2(rect.left() + 30.0, legend_y),
                                    egui::Align2::LEFT_TOP,
                                    format!("{sig_name}: {final_val:.4}"),
                                    egui::FontId::proportional(12.0),
                                    color,
                                );
                                legend_y += 18.0;
                            }
                        }
                    } else {
                        // Spectrum Tab
                        let signal_data: &[f64] = if let Some(sig) = output.signals.get(&self.spectrum_signal_name) {
                            sig.as_slice()
                        } else if !output.yout.is_empty() {
                            output.yout.as_slice()
                        } else {
                            &[]
                        };

                        if !signal_data.is_empty() && output.tout.len() == signal_data.len() {
                            match mlapp_sim::spectrum_with_window(
                                signal_data,
                                &output.tout,
                                Some(self.spectrum_window_size),
                                self.spectrum_f0,
                                self.spectrum_window_fn,
                            ) {
                                Ok(res) => {
                                    let num_bins = res.frequencies.len();
                                    let f_max = res.frequencies.last().copied().unwrap_or(1.0).max(1.0);

                                    let mut peak_freq = 0.0;
                                    let mut peak_mag = 0.0;
                                    for (&f, &m) in res.frequencies.iter().zip(&res.magnitudes).skip(1) {
                                        if m > peak_mag {
                                            peak_mag = m;
                                            peak_freq = f;
                                        }
                                    }

                                    let plotted_mags: Vec<f64> = if self.spectrum_db_scale {
                                        let ref_val = peak_mag.max(1e-12);
                                        res.magnitudes
                                            .iter()
                                            .map(|&m| (20.0 * (m / ref_val).max(1e-6).log10()).clamp(-100.0, 10.0))
                                            .collect()
                                    } else {
                                        res.magnitudes.clone()
                                    };

                                    let y_min = if self.spectrum_db_scale { -100.0 } else { 0.0 };
                                    let y_max = if self.spectrum_db_scale {
                                        10.0
                                    } else {
                                        peak_mag.max(1.0) * 1.1
                                    };
                                    let y_span = (y_max - y_min).max(0.001);

                                    let map_pt = |f: f64, y: f64| -> Pos2 {
                                        let norm_x = (f / f_max) as f32;
                                        let norm_y = ((y - y_min) / y_span) as f32;
                                        pos2(
                                            rect.left() + norm_x * (rect.width() - 30.0) + 15.0,
                                            rect.bottom() - norm_y * (rect.height() - 40.0) - 20.0,
                                        )
                                    };

                                    // Draw spectrum curve
                                    let color = Color32::from_rgb(90, 180, 255);
                                    for i in 0..(num_bins.saturating_sub(1)) {
                                        let p1 = map_pt(res.frequencies[i], plotted_mags[i]);
                                        let p2 = map_pt(res.frequencies[i + 1], plotted_mags[i + 1]);
                                        painter.line_segment([p1, p2], Stroke::new(1.5_f32, color));
                                    }

                                    // Display metrics
                                    let db_unit = if self.spectrum_db_scale { "dB" } else { "V/A" };
                                    painter.text(
                                        pos2(rect.left() + 20.0, rect.top() + 10.0),
                                        egui::Align2::LEFT_TOP,
                                        format!(
                                            "Fundamental: {:.2} Hz | Peak: {:.2} Hz ({:.3} {}) | THD: {:.3}%",
                                            res.fundamental_freq, peak_freq, peak_mag, db_unit, res.thd
                                        ),
                                        egui::FontId::proportional(13.0),
                                        Color32::from_rgb(255, 215, 0),
                                    );
                                }
                                Err(err) => {
                                    painter.text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        format!("Spectrum error ({}): {}", err.code, err.message),
                                        egui::FontId::proportional(12.0),
                                        Color32::from_rgb(255, 100, 100),
                                    );
                                }
                            }
                        } else {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "No signal data available for spectral analysis.",
                                egui::FontId::proportional(12.0),
                                Color32::GRAY,
                            );
                        }
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

    fn render_json_modal(&mut self, ctx: &egui::Context, theme: &Theme) {
        let mut close = false;
        let title = if self.json_is_export {
            "Export Block Diagram to JSON"
        } else {
            "Import Block Diagram from JSON"
        };

        egui::Window::new(title)
            .id(egui::Id::new("block_diagram_json_modal"))
            .default_size([550.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(err) = &self.json_error {
                    ui.colored_label(theme.error, format!("Error: {err}"));
                    ui.separator();
                }

                ui.label(if self.json_is_export {
                    "Copy this JSON representation of your diagram:"
                } else {
                    "Paste diagram JSON below and click 'Load':"
                });

                ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.json_buffer)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if self.json_is_export {
                        if ui.button("Copy to Clipboard").clicked() {
                            ui.ctx().copy_text(self.json_buffer.clone());
                        }
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    } else {
                        if ui.button("Load Diagram").clicked() {
                            match self.load_from_json(&self.json_buffer.clone()) {
                                Ok(()) => {
                                    self.json_error = None;
                                    close = true;
                                }
                                Err(e) => {
                                    self.json_error = Some(e);
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    }
                });
            });

        if close {
            self.show_json_modal = false;
            self.json_error = None;
        }
    }

    fn render_problems_modal(&mut self, ctx: &egui::Context, theme: &Theme) {
        let mut close_problems = false;
        egui::Window::new(format!("Problems ({})", self.diagnostics.len()))
            .id(egui::Id::new("block_diagram_problems_modal"))
            .default_size([700.0, 260.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Refresh Diagnostics").clicked() {
                        self.refresh_diagnostics();
                    }
                    if self.undo_diagram.is_some() && ui.button("Undo Fix").clicked() {
                        if let Some(prev) = self.undo_diagram.take() {
                            self.load_from_block_diagram(&prev);
                        }
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        close_problems = true;
                    }
                });
                ui.separator();

                if self.diagnostics.is_empty() {
                    ui.label(
                        RichText::new("No issues detected in current diagram.").color(theme.accent),
                    );
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut fix_to_apply = None;
                        for diag in &self.diagnostics {
                            ui.horizontal(|ui| {
                                let (badge, col) = match diag.severity {
                                    mlapp_sim::DiagnosticSeverity::Error => {
                                        ("[ERROR]", egui::Color32::from_rgb(220, 50, 50))
                                    }
                                    mlapp_sim::DiagnosticSeverity::Warning => {
                                        ("[WARN]", egui::Color32::from_rgb(220, 180, 50))
                                    }
                                    mlapp_sim::DiagnosticSeverity::Info => {
                                        ("[INFO]", egui::Color32::from_rgb(50, 150, 220))
                                    }
                                };
                                ui.colored_label(col, badge);
                                ui.colored_label(col, &diag.code);
                                if ui
                                    .button(format!("{}: {}", diag.title(), diag.message))
                                    .clicked()
                                {
                                    if let Some(sid) = diag.block_sids.first() {
                                        self.selected_block_sid = Some(sid.clone());
                                        if let Some(b) = self.blocks.iter().find(|b| b.sid == *sid)
                                        {
                                            self.pan_offset = Pos2::new(
                                                300.0 - b.pos.x * self.zoom,
                                                250.0 - b.pos.y * self.zoom,
                                            );
                                        }
                                    }
                                }
                                if let Some(fix) = &diag.fix {
                                    if ui.button("Fix").clicked() {
                                        fix_to_apply = Some(fix.as_ref().clone());
                                    }
                                }
                            });
                        }
                        if let Some(fix) = fix_to_apply {
                            self.undo_diagram = Some(self.to_block_diagram());
                            let mut d = self.to_block_diagram();
                            if mlapp_sim::apply_fix(&mut d, &fix).is_ok() {
                                self.load_from_block_diagram(&d);
                            }
                        }
                    });
                }
            });

        if close_problems {
            self.show_problems_panel = false;
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
            let remaining_width =
                layout.tab_body.width() - layout.library.width() - layout.properties.width();

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
                    "Electrical",
                    "Measurements",
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
    fn block_diagram_small_tab_scrolls_without_clipping_minimum_canvas() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        let layout = run_frame(&ctx, &mut editor, vec2(400.0, 220.0), Vec::new());
        let min_body_width = DEFAULT_LIBRARY_WIDTH
            + DEFAULT_PROPERTIES_WIDTH
            + MIN_CANVAS_WIDTH
            + ctx.style().spacing.item_spacing.x * 2.0;

        assert!(layout.toolbar.width() >= min_body_width - 2.0);
        assert!(layout.canvas.width() >= MIN_CANVAS_WIDTH);
        assert!(layout.canvas.height() >= MIN_CANVAS_HEIGHT);
        assert!(layout.canvas.min.x >= layout.library.max.x - 1.0);
        assert!(
            layout
                .toolbar_widgets
                .iter()
                .all(|widget| widget.max.x <= layout.tab_body.max.x + 2.0),
            "tab body: {:?}; toolbar: {:?}; widgets: {:?}",
            layout.tab_body,
            layout.toolbar,
            layout.toolbar_widgets
        );
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
            vec![
                Event::PointerMoved(block_end),
                Event::Key {
                    key: Key::Delete,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        assert_eq!(editor.blocks.len(), 4);
        assert!(!editor
            .wires
            .iter()
            .any(|wire| wire.src_block == "1" || wire.dst_block == "1"));
        editor.add_block(BlockType::Constant, "New Constant");
        let ids = editor
            .blocks
            .iter()
            .map(|block| block.sid.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), editor.blocks.len());
    }

    #[test]
    fn test_editing_numeric_param_persists_across_frames() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();

        // Open modal for Controller Gain (SID 3)
        editor.open_param_modal("3");
        assert_eq!(editor.editing_block_sid.as_deref(), Some("3"));
        assert_eq!(
            editor.param_draft.get("Gain").map(|s| s.as_str()),
            Some("2.0")
        );

        // Type "7.5" into param draft
        editor
            .param_draft
            .insert("Gain".to_string(), "7.5".to_string());

        // Run 3 frames; verify typed draft persists across frames without reverting
        let size = vec2(1280.0, 800.0);
        for _ in 0..3 {
            let _ = run_frame(&ctx, &mut editor, size, Vec::new());
            assert_eq!(
                editor.param_draft.get("Gain").map(|s| s.as_str()),
                Some("7.5"),
                "Draft must persist across frames without being overwritten"
            );
        }

        // Block parameter should still be original "2.0" before applying
        assert_eq!(
            editor
                .blocks
                .iter()
                .find(|b| b.sid == "3")
                .unwrap()
                .parameters
                .get("Gain")
                .map(|s| s.as_str()),
            Some("2.0")
        );

        // Simulate OK click / commit
        for (k, v) in &editor.param_draft {
            editor
                .blocks
                .iter_mut()
                .find(|b| b.sid == "3")
                .unwrap()
                .parameters
                .insert(k.clone(), v.clone());
        }
        editor.sync_prop_draft();
        assert_eq!(
            editor
                .blocks
                .iter()
                .find(|b| b.sid == "3")
                .unwrap()
                .parameters
                .get("Gain")
                .map(|s| s.as_str()),
            Some("7.5")
        );
    }

    #[test]
    fn test_invalid_numeric_param_rejected_old_value_kept() {
        let ctx = egui::Context::default();
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();

        // Select Controller Gain (SID 3)
        editor.selected_block_sid = Some("3".to_string());
        editor.sync_prop_draft();

        let initial_gain = editor
            .blocks
            .iter()
            .find(|b| b.sid == "3")
            .unwrap()
            .parameters
            .get("Gain")
            .cloned()
            .unwrap();
        assert_eq!(initial_gain, "2.0");

        // Attempt to insert invalid non-float string in modal
        editor.open_param_modal("3");
        editor
            .param_draft
            .insert("Gain".to_string(), "invalid_number_abc".to_string());

        // Run frame with modal open
        let _ = run_frame(&ctx, &mut editor, vec2(1280.0, 800.0), Vec::new());

        // Validation logic rejects invalid float
        let val = editor.param_draft.get("Gain").unwrap();
        let parsed = val.trim().parse::<f64>();
        assert!(parsed.is_err(), "Invalid numeric string should fail parse");

        // Block parameter must retain old value
        let current_block_gain = editor
            .blocks
            .iter()
            .find(|b| b.sid == "3")
            .unwrap()
            .parameters
            .get("Gain")
            .unwrap();
        assert_eq!(
            current_block_gain, &initial_gain,
            "Old parameter value must be kept on invalid input"
        );
    }

    #[test]
    fn test_sum_signs_creates_ports_and_wires_survive() {
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();

        // Initial Sum block (SID 2) has "+-" -> 2 input ports
        let sum_idx = editor.blocks.iter().position(|b| b.sid == "2").unwrap();
        assert_eq!(editor.blocks[sum_idx].input_ports(), 2);
        assert_eq!(editor.wires.len(), 5);

        // Verify incoming wires to Sum at port 0 and 1
        assert!(editor
            .wires
            .iter()
            .any(|w| w.dst_block == "2" && w.dst_port == 0));
        assert!(editor
            .wires
            .iter()
            .any(|w| w.dst_block == "2" && w.dst_port == 1));

        // Change Sum signs to "++-" (3 input ports)
        editor.blocks[sum_idx]
            .parameters
            .insert("Inputs".to_string(), "++-".to_string());
        assert_eq!(editor.blocks[sum_idx].input_ports(), 3);

        // Pre-existing wires to port 0 and 1 must survive
        assert_eq!(editor.wires.len(), 5);
        assert!(editor
            .wires
            .iter()
            .any(|w| w.dst_block == "2" && w.dst_port == 0));
        assert!(editor
            .wires
            .iter()
            .any(|w| w.dst_block == "2" && w.dst_port == 1));

        // Connect new wire to port 2 (index 2)
        editor.wires.push(CanvasWire {
            src_block: "1".to_string(),
            src_port: 0,
            dst_block: "2".to_string(),
            dst_port: 2,
            bend_offset: Vec2::ZERO,
        });
        assert_eq!(editor.wires.len(), 6);
        assert!(editor
            .wires
            .iter()
            .any(|w| w.dst_block == "2" && w.dst_port == 2));
    }

    #[test]
    fn test_deleting_block_removes_wires() {
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();
        assert_eq!(editor.blocks.len(), 5);
        assert_eq!(editor.wires.len(), 5);

        // Select Sum block (SID 2) which has incoming wires from sid 1 & 4, and outgoing to sid 3
        editor.selected_block_sid = Some("2".to_string());
        editor.delete_selected();

        assert_eq!(editor.blocks.len(), 4);
        assert!(!editor.blocks.iter().any(|b| b.sid == "2"));
        // All wires connected to SID 2 must be removed
        assert!(!editor
            .wires
            .iter()
            .any(|w| w.src_block == "2" || w.dst_block == "2"));
        // Remaining wires: sid 3->4 and sid 4->5
        assert_eq!(editor.wires.len(), 2);
    }

    #[test]
    fn test_block_diagram_editor_json_roundtrip() {
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();

        // Modify Gain parameter to 4.5
        editor
            .blocks
            .iter_mut()
            .find(|b| b.sid == "3")
            .unwrap()
            .parameters
            .insert("Gain".to_string(), "4.5".to_string());

        let json = editor.save_to_json().expect("Export JSON");
        let mut editor2 = BlockDiagramEditor::default();
        editor2.load_from_json(&json).expect("Import JSON");

        assert_eq!(editor2.blocks.len(), editor.blocks.len());
        assert_eq!(editor2.wires.len(), editor.wires.len());
        assert_eq!(
            editor2
                .blocks
                .iter()
                .find(|b| b.sid == "3")
                .unwrap()
                .parameters
                .get("Gain")
                .map(|s| s.as_str()),
            Some("4.5")
        );
        assert!(editor2.next_block_id > 5);
    }

    #[test]
    fn test_block_diagram_duplicate() {
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();
        assert_eq!(editor.blocks.len(), 5);

        editor.selected_block_sid = Some("3".to_string()); // Controller Gain
        let dup_sid = editor.duplicate_selected().expect("Duplicate block");

        assert_eq!(editor.blocks.len(), 6);
        assert_eq!(dup_sid, "6");
        let dup = editor.blocks.iter().find(|b| b.sid == "6").unwrap();
        assert_eq!(dup.parameters.get("Gain").map(|s| s.as_str()), Some("2.0"));
        assert_eq!(editor.selected_block_sid.as_deref(), Some("6"));
    }

    #[test]
    fn test_editor_solver_config_persistence() {
        let mut editor = BlockDiagramEditor::default();
        editor.load_default_feedback_example();

        // Configure variable-step Dormand-Prince with custom tolerances
        editor.config.solver = SolverType::VariableStepDormandPrince;
        editor.config.stop_time = 12.5;
        editor.config.abs_tol = 1e-8;
        editor.config.rel_tol = 1e-5;

        let json = editor.save_to_json().expect("Export JSON");
        assert!(json.contains("\"schema_version\": 3"));
        assert!(json.contains("VariableStepDormandPrince"));

        let mut editor2 = BlockDiagramEditor::default();
        editor2.load_from_json(&json).expect("Import JSON");

        assert_eq!(editor2.config.solver, SolverType::VariableStepDormandPrince);
        assert_eq!(editor2.config.stop_time, 12.5);
        assert_eq!(editor2.config.abs_tol, 1e-8);
        assert_eq!(editor2.config.rel_tol, 1e-5);

        // Test migration of legacy schema_version 1 diagram JSON lacking simulation_config
        let legacy_json = r#"{
            "name": "LegacyDiagram",
            "version": null,
            "release": null,
            "blocks": [],
            "lines": [],
            "toolboxes": [],
            "properties": {}
        }"#;
        let mut editor_legacy = BlockDiagramEditor::default();
        editor_legacy
            .load_from_json(legacy_json)
            .expect("Import legacy JSON");
        assert_eq!(
            editor_legacy.config.solver,
            SolverType::FixedStepEuler,
            "Legacy diagram must migrate to FixedStepEuler default"
        );
    }

    #[test]
    fn test_m4_electrical_library_and_palette_blocks() {
        let libraries = BlockDiagramEditor::block_libraries();
        let elec = libraries.iter().find(|(name, _)| *name == "Electrical");
        assert!(
            elec.is_some(),
            "Electrical category must exist in library browser"
        );
        let items = &elec.unwrap().1;
        assert_eq!(items.len(), 13);
        assert!(items
            .iter()
            .any(|i| i.label == "Resistor" && i.block_type == BlockType::Resistor));
        assert!(items
            .iter()
            .any(|i| i.label == "Capacitor" && i.block_type == BlockType::Capacitor));
        assert!(items
            .iter()
            .any(|i| i.label == "Inductor" && i.block_type == BlockType::Inductor));
        assert!(items
            .iter()
            .any(|i| i.label == "DC Voltage Source" && i.block_type == BlockType::DcVoltageSource));
        assert!(items
            .iter()
            .any(|i| i.label == "AC Voltage Source" && i.block_type == BlockType::AcVoltageSource));
        assert!(items
            .iter()
            .any(|i| i.label == "DC Current Source" && i.block_type == BlockType::DcCurrentSource));
        assert!(items
            .iter()
            .any(|i| i.label == "Ground" && i.block_type == BlockType::Ground));
        assert!(items
            .iter()
            .any(|i| i.label == "Diode" && i.block_type == BlockType::Diode));
        assert!(items
            .iter()
            .any(|i| i.label == "Ideal Switch" && i.block_type == BlockType::IdealSwitch));
        assert!(items
            .iter()
            .any(|i| i.label == "Voltage Sensor" && i.block_type == BlockType::VoltageSensor));
        assert!(items
            .iter()
            .any(|i| i.label == "Current Sensor" && i.block_type == BlockType::CurrentSensor));
        assert!(items.iter().any(|i| i.label == "Controlled Voltage Source"
            && i.block_type == BlockType::VoltageControlledVoltageSource));
        assert!(items.iter().any(|i| i.label == "Controlled Current Source"
            && i.block_type == BlockType::VoltageControlledCurrentSource));
    }

    #[test]
    fn test_m4_block_rotation_and_simulation_e_unsupported_in_editor() {
        let mut editor = BlockDiagramEditor::default();
        editor.add_block(BlockType::Resistor, "R1");
        assert_eq!(editor.blocks.len(), 1);
        assert_eq!(editor.blocks[0].rotation, BlockRotation::R0);

        // Rotate block cw
        editor.blocks[0].rotation = editor.blocks[0].rotation.rotated_cw();
        assert_eq!(editor.blocks[0].rotation, BlockRotation::R90);
        editor.blocks[0].rotation = editor.blocks[0].rotation.rotated_cw();
        assert_eq!(editor.blocks[0].rotation, BlockRotation::R180);
        editor.blocks[0].rotation = editor.blocks[0].rotation.rotated_cw();
        assert_eq!(editor.blocks[0].rotation, BlockRotation::R270);
        editor.blocks[0].rotation = editor.blocks[0].rotation.rotated_cw();
        assert_eq!(editor.blocks[0].rotation, BlockRotation::R0);

        // Convert to diagram and verify schema_version 3 and rotation
        let diag = editor.to_block_diagram();
        assert_eq!(diag.schema_version, 3);
        assert_eq!(diag.blocks[0].rotation, BlockRotation::R0);

        // An ungrounded electrical circuit produces a helpful E-SINGULAR error in sim_status
        editor.add_block(BlockType::Diode, "D1");
        editor.run_simulation();
        assert!(
            editor.sim_status.contains("E-SINGULAR"),
            "Editor sim_status must display full helpful error message: {}",
            editor.sim_status
        );
    }
}
