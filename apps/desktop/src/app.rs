//! Main MatStudioApp application implementing eframe::App.

use crate::command_window::CommandWindow;
use crate::current_folder::CurrentFolderPanel;
use crate::dock::{DockTree, PanelId};
use crate::editor::EditorPanel;
use crate::execution::{GraphicsState, VariableRow, WorkerClient, WorkerResponse};
use crate::figures::FiguresPanel;
use crate::preferences::{LayoutPreset, Preferences};
use crate::statusbar::{EngineStatus, StatusBar};
use crate::theme::{Theme, ThemeMode};
use crate::workspace_panel::WorkspacePanel;
use egui::{Key, RichText, ScrollArea, Ui, Window};
use std::fs;
use std::time::Instant;

pub struct MatStudioApp {
    pub theme: Theme,
    pub prefs: Preferences,
    pub statusbar: StatusBar,
    pub dock_tree: DockTree,

    pub editor: EditorPanel,
    pub command_window: CommandWindow,
    pub current_folder: CurrentFolderPanel,
    pub workspace_panel: WorkspacePanel,
    pub figures_panel: FiguresPanel,

    pub worker: WorkerClient,
    pub graphics: GraphicsState,
    pub variables: Vec<VariableRow>,

    // Import Data Dialog state
    pub import_dialog_open: bool,
    pub import_path_input: String,
    pub import_var_name: String,
    pub import_delimiter: u8,
    pub import_has_header: bool,
    pub import_preview_text: String,
    pub import_error: Option<String>,

    // About Dialog
    pub about_dialog_open: bool,
}

impl Default for MatStudioApp {
    fn default() -> Self {
        let mut app = Self {
            theme: Theme::dark(),
            prefs: Preferences::default(),
            statusbar: StatusBar::default(),
            dock_tree: DockTree::standard(),

            editor: EditorPanel::default(),
            command_window: CommandWindow::default(),
            current_folder: CurrentFolderPanel::default(),
            workspace_panel: WorkspacePanel::default(),
            figures_panel: FiguresPanel::default(),

            worker: WorkerClient::new(),
            graphics: GraphicsState::default(),
            variables: Vec::new(),

            import_dialog_open: false,
            import_path_input: String::new(),
            import_var_name: "data".to_string(),
            import_delimiter: b',',
            import_has_header: true,
            import_preview_text: String::new(),
            import_error: None,

            about_dialog_open: false,
        };
        app.try_load_dock_layout();
        app
    }
}

impl eframe::App for MatStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for asynchronous worker responses
        self.poll_worker();

        // Apply theme visuals
        ctx.style_mut(|s| self.theme.apply_to_visuals(&mut s.visuals));

        // Global Keyboard Shortcuts
        self.handle_shortcuts(ctx);

        // Top Menu Bar & Icon Toolbar
        egui::TopBottomPanel::top("top_menu_and_toolbar").show(ctx, |ui| {
            self.render_menu_bar(ui);
            ui.separator();
            self.render_toolbar(ui);
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_status_bar").show(ctx, |ui| {
            self.statusbar.active_file = self.editor.active_doc().and_then(|d| d.path.clone());
            self.statusbar.working_dir = self.current_folder.current_dir.clone();
            self.statusbar.show(ui, &self.theme);
        });

        // Central Multi-Panel Docking Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut open_file = None;
            let mut on_run_script = None;
            let mut on_run_selection = None;
            let mut on_clear_workspace = false;
            let mut on_refresh_workspace = false;
            let mut exec_cmd = None;

            let theme = self.theme.clone();
            let variables = self.variables.clone();

            let current_folder = &mut self.current_folder;
            let editor = &mut self.editor;
            let workspace_panel = &mut self.workspace_panel;
            let command_window = &mut self.command_window;
            let figures_panel = &mut self.figures_panel;
            let graphics = &mut self.graphics;

            self.dock_tree
                .render(ui, &theme, |body_ui, panel_id| match panel_id {
                    PanelId::CurrentFolder => {
                        current_folder.show(body_ui, &theme, &mut open_file);
                    }
                    PanelId::Editor => {
                        editor.show(
                            body_ui,
                            &theme,
                            &variables,
                            &mut on_run_script,
                            &mut on_run_selection,
                        );
                    }
                    PanelId::Workspace => {
                        workspace_panel.show(
                            body_ui,
                            &theme,
                            &variables,
                            &mut on_clear_workspace,
                            &mut on_refresh_workspace,
                        );
                    }
                    PanelId::CommandWindow => {
                        command_window.show(body_ui, &theme, &mut exec_cmd);
                    }
                    PanelId::Figures => {
                        figures_panel.show(body_ui, &theme, graphics);
                    }
                    PanelId::History => {
                        command_window.show_history_panel(body_ui, &theme, &mut exec_cmd);
                    }
                });

            if let Some(path) = open_file {
                let _ = self.editor.open_file(path);
            }
            if let Some(code) = on_run_script {
                self.execute_code(code);
            }
            if let Some(code) = on_run_selection {
                self.execute_code(code);
            }
            if on_clear_workspace {
                self.worker.clear();
            }
            if on_refresh_workspace {
                self.worker.refresh_workspace();
            }
            if let Some(cmd) = exec_cmd {
                self.execute_code(cmd);
            }

            // Synchronize active document breakpoints to worker engine
            if let Some(doc) = self.editor.active_doc() {
                self.worker.set_breakpoints(doc.breakpoints.clone());
            }
        });

        // Variable detailed inspection modal
        self.workspace_panel.show_inspector(ctx, &self.theme);

        // Import Data Dialog modal
        self.render_import_dialog(ctx);

        // About Dialog modal
        self.render_about_dialog(ctx);
    }
}

impl MatStudioApp {
    fn poll_worker(&mut self) {
        while let Ok(resp) = self.worker.rx.try_recv() {
            match resp {
                WorkerResponse::ExecutionFinished { result, .. } => {
                    self.statusbar.status = EngineStatus::Ready;
                    if let Some(doc) = self.editor.active_doc_mut() {
                        doc.error_line = None;
                    }
                    if !result.output.is_empty() {
                        self.command_window.push_output(result.output);
                    }
                    if let Some(err) = result.error {
                        self.statusbar.status = EngineStatus::Error(err.clone());
                        self.command_window.push_error(err);
                    }
                    self.variables = result.variables;
                    self.graphics = result.graphics;
                }
                WorkerResponse::BreakpointHit {
                    line, variables, ..
                } => {
                    self.statusbar.status = EngineStatus::Paused(line);
                    self.command_window
                        .push_output(format!(">> Paused at breakpoint on line {line}\n"));
                    self.variables = variables;
                    if let Some(doc) = self.editor.active_doc_mut() {
                        doc.error_line = Some(line);
                    }
                }
                WorkerResponse::WorkspaceUpdated(vars) => {
                    self.variables = vars;
                }
                WorkerResponse::ImportFinished { result, .. } => match result {
                    Ok(var) => {
                        self.command_window.push_output(format!(
                            "Imported variable '{}' ({} double matrix).\n",
                            var.name, var.size_str
                        ));
                        self.import_dialog_open = false;
                        self.worker.refresh_workspace();
                    }
                    Err(e) => {
                        self.import_error = Some(e);
                    }
                },
                WorkerResponse::WorkerPanic { message, .. } => {
                    self.statusbar.status = EngineStatus::Error("Worker Panic".to_string());
                    self.command_window.push_error(format!(
                        "Session Panic Intercepted: {message}\nWorker engine recovered. Workspace variables reset for safety.\n"
                    ));
                    self.worker.refresh_workspace();
                }
            }
        }
    }

    fn execute_code(&mut self, code: String) {
        self.statusbar.status = EngineStatus::Running(Instant::now());
        self.worker.execute(code);
    }

    fn execute_debug_code(&mut self, code: String) {
        self.statusbar.status = EngineStatus::Running(Instant::now());
        self.worker.execute_debug(code);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // F5 = Run Script / Continue if paused
        if ctx.input(|i| i.key_pressed(Key::F5)) {
            if matches!(self.statusbar.status, EngineStatus::Paused(_)) {
                self.worker.debug_continue();
                self.statusbar.status = EngineStatus::Running(Instant::now());
                if let Some(doc) = self.editor.active_doc_mut() {
                    doc.error_line = None;
                }
            } else if let Some(doc) = self.editor.active_doc() {
                self.execute_debug_code(doc.content.clone());
            }
        }
        // F9 = Run Selection / Active Line
        if ctx.input(|i| i.key_pressed(Key::F9)) {
            if let Some(doc) = self.editor.active_doc() {
                self.execute_code(doc.content.clone());
            }
        }
        // Ctrl+C = Stop execution
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::C)) {
            if matches!(self.statusbar.status, EngineStatus::Paused(_)) {
                self.worker.debug_stop();
            } else {
                self.worker.stop();
            }
            self.statusbar.status = EngineStatus::Ready;
        }
        // Ctrl+N = New Script
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::N)) {
            self.editor.new_script();
        }
        // Ctrl+S = Save
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            if let Some(doc) = self.editor.active_doc_mut() {
                let _ = doc.save();
            }
        }
    }

    fn render_menu_bar(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                if ui.button("📄 New Script (Ctrl+N)").clicked() {
                    self.editor.new_script();
                    ui.close_menu();
                }
                if ui.button("📂 Open File... (Ctrl+O)").clicked() {
                    ui.close_menu();
                }
                if ui.button("💾 Save (Ctrl+S)").clicked() {
                    if let Some(doc) = self.editor.active_doc_mut() {
                        let _ = doc.save();
                    }
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("📑 Import Data (CSV)...").clicked() {
                    self.open_import_dialog();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("✕ Close Tab (Ctrl+W)").clicked() {
                    self.editor.close_active_tab();
                    ui.close_menu();
                }
                if ui.button("🚪 Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Edit Menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    ui.close_menu();
                }
                if ui.button("Redo (Ctrl+Y)").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Clear Console (clc)").clicked() {
                    self.command_window.clear();
                    ui.close_menu();
                }
                if ui.button("Clear Workspace (clear)").clicked() {
                    self.worker.clear();
                    ui.close_menu();
                }
            });

            // View Menu
            ui.menu_button("View", |ui| {
                for panel in [
                    PanelId::CurrentFolder,
                    PanelId::Editor,
                    PanelId::Workspace,
                    PanelId::CommandWindow,
                    PanelId::Figures,
                    PanelId::History,
                ] {
                    let mut is_open = self.dock_tree.is_open(panel);
                    if ui
                        .checkbox(&mut is_open, format!("{} {}", panel.icon(), panel.title()))
                        .clicked()
                    {
                        if is_open {
                            self.dock_tree.open_panel(panel);
                        } else {
                            self.dock_tree.root.remove_panel(panel);
                            self.dock_tree.root.prune_empty();
                        }
                    }
                }
                ui.separator();

                ui.menu_button("Layout Presets", |ui| {
                    if ui.button("Standard Layout").clicked() {
                        self.dock_tree = DockTree::standard();
                        self.save_dock_layout();
                        ui.close_menu();
                    }
                    if ui.button("Editor Focus").clicked() {
                        self.dock_tree = DockTree::editor_focus();
                        self.save_dock_layout();
                        ui.close_menu();
                    }
                    if ui.button("Data Analysis").clicked() {
                        self.dock_tree = DockTree::data_analysis();
                        self.save_dock_layout();
                        ui.close_menu();
                    }
                    if ui.button("Plotting").clicked() {
                        self.dock_tree = DockTree::plotting();
                        self.save_dock_layout();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        self.dock_tree = DockTree::standard();
                        self.save_dock_layout();
                        ui.close_menu();
                    }
                });

                ui.separator();
                if ui
                    .button(match self.theme.mode {
                        ThemeMode::Dark => "Switch to Light Theme",
                        ThemeMode::Light => "Switch to Dark Theme",
                    })
                    .clicked()
                {
                    self.theme = match self.theme.mode {
                        ThemeMode::Dark => Theme::light(),
                        ThemeMode::Light => Theme::dark(),
                    };
                    self.prefs.theme_mode = self.theme.mode;
                    ui.close_menu();
                }
            });

            // Run Menu
            ui.menu_button("Run", |ui| {
                if ui.button("▶ Run Script (F5)").clicked() {
                    if let Some(doc) = self.editor.active_doc() {
                        self.execute_code(doc.content.clone());
                    }
                    ui.close_menu();
                }
                if ui.button("▶ Run Selection (F9)").clicked() {
                    if let Some(doc) = self.editor.active_doc() {
                        self.execute_code(doc.content.clone());
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("⏹ Stop Execution (Ctrl+C)").clicked() {
                    self.worker.stop();
                    ui.close_menu();
                }
            });

            // Help Menu
            ui.menu_button("Help", |ui| {
                if ui.button("Built-in Help").clicked() {
                    self.execute_code("help".to_string());
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("About MatRust Studio").clicked() {
                    self.about_dialog_open = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            if ui
                .button("📄 New")
                .on_hover_text("New Script (Ctrl+N)")
                .clicked()
            {
                self.editor.new_script();
            }
            if ui
                .button("💾 Save")
                .on_hover_text("Save Script (Ctrl+S)")
                .clicked()
            {
                if let Some(doc) = self.editor.active_doc_mut() {
                    let _ = doc.save();
                }
            }
            if ui
                .button("📑 Import Data")
                .on_hover_text("Import CSV file into workspace")
                .clicked()
            {
                self.open_import_dialog();
            }

            ui.separator();

            let is_paused = matches!(self.statusbar.status, EngineStatus::Paused(_));
            if is_paused {
                if ui
                    .button(
                        RichText::new("▶ Continue")
                            .color(self.theme.accent)
                            .strong(),
                    )
                    .on_hover_text("Resume execution past breakpoint (F5)")
                    .clicked()
                {
                    self.worker.debug_continue();
                    self.statusbar.status = EngineStatus::Running(Instant::now());
                    if let Some(doc) = self.editor.active_doc_mut() {
                        doc.error_line = None;
                    }
                }
                if ui
                    .button(RichText::new("⏹ Stop").color(self.theme.error).strong())
                    .on_hover_text("Stop execution at breakpoint (Ctrl+C)")
                    .clicked()
                {
                    self.worker.debug_stop();
                    self.statusbar.status = EngineStatus::Ready;
                    if let Some(doc) = self.editor.active_doc_mut() {
                        doc.error_line = None;
                    }
                }
            } else {
                if ui
                    .button(RichText::new("▶ Run").color(self.theme.success))
                    .on_hover_text("Execute active script with breakpoints (F5)")
                    .clicked()
                {
                    if let Some(doc) = self.editor.active_doc() {
                        self.execute_debug_code(doc.content.clone());
                    }
                }
                if ui
                    .button(RichText::new("⏹ Stop").color(self.theme.error))
                    .on_hover_text("Interrupt computation (Ctrl+C)")
                    .clicked()
                {
                    self.worker.stop();
                }
            }
            if ui
                .button("🗑 Clear")
                .on_hover_text("Clear console buffer")
                .clicked()
            {
                self.command_window.clear();
            }

            ui.separator();

            // Preset selector
            egui::ComboBox::from_label("Preset")
                .selected_text(match self.prefs.current_preset {
                    LayoutPreset::Standard => "Standard",
                    LayoutPreset::EditorFocus => "Editor Focus",
                    LayoutPreset::DataAnalysis => "Data Analysis",
                    LayoutPreset::Plotting => "Plotting",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut self.prefs.current_preset,
                            LayoutPreset::Standard,
                            "Standard",
                        )
                        .clicked()
                    {
                        self.dock_tree = DockTree::standard();
                        self.save_dock_layout();
                    }
                    if ui
                        .selectable_value(
                            &mut self.prefs.current_preset,
                            LayoutPreset::EditorFocus,
                            "Editor Focus",
                        )
                        .clicked()
                    {
                        self.dock_tree = DockTree::editor_focus();
                        self.save_dock_layout();
                    }
                    if ui
                        .selectable_value(
                            &mut self.prefs.current_preset,
                            LayoutPreset::DataAnalysis,
                            "Data Analysis",
                        )
                        .clicked()
                    {
                        self.dock_tree = DockTree::data_analysis();
                        self.save_dock_layout();
                    }
                    if ui
                        .selectable_value(
                            &mut self.prefs.current_preset,
                            LayoutPreset::Plotting,
                            "Plotting",
                        )
                        .clicked()
                    {
                        self.dock_tree = DockTree::plotting();
                        self.save_dock_layout();
                    }
                });
        });
    }

    fn open_import_dialog(&mut self) {
        self.import_dialog_open = true;
        self.import_error = None;
        if self.import_path_input.is_empty() {
            // Default to an example or current directory csv
            self.import_path_input = "data.csv".to_string();
        }
    }

    fn render_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.import_dialog_open {
            return;
        }

        let mut open = self.import_dialog_open;
        Window::new("Import Data (CSV)")
            .open(&mut open)
            .fixed_size([450.0, 320.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        "Select and import numerical CSV matrix into the persistent workspace.",
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("File Path:");
                        let text_resp = ui.text_edit_singleline(&mut self.import_path_input);
                        if text_resp.changed() {
                            self.update_import_preview();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Variable Name:");
                        ui.text_edit_singleline(&mut self.import_var_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Delimiter:");
                        let delim_str = match self.import_delimiter {
                            b',' => "Comma (,)".to_string(),
                            b';' => "Semicolon (;".to_string(),
                            b'\t' => "Tab (\\t)".to_string(),
                            _ => "Space".to_string(),
                        };
                        egui::ComboBox::from_id_salt("delim_combo")
                            .selected_text(&delim_str)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(self.import_delimiter == b',', "Comma (,)")
                                    .clicked()
                                {
                                    self.import_delimiter = b',';
                                    self.update_import_preview();
                                }
                                if ui
                                    .selectable_label(
                                        self.import_delimiter == b';',
                                        "Semicolon (;)",
                                    )
                                    .clicked()
                                {
                                    self.import_delimiter = b';';
                                    self.update_import_preview();
                                }
                                if ui
                                    .selectable_label(self.import_delimiter == b'\t', "Tab (\\t)")
                                    .clicked()
                                {
                                    self.import_delimiter = b'\t';
                                    self.update_import_preview();
                                }
                            });

                        ui.checkbox(&mut self.import_has_header, "First row is header");
                    });

                    ui.separator();
                    ui.label(RichText::new("Data Preview:").strong());

                    if let Some(ref err) = self.import_error {
                        ui.colored_label(self.theme.error, format!("Error: {err}"));
                    }

                    ScrollArea::both().max_height(120.0).show(ui, |ui| {
                        if self.import_preview_text.is_empty() {
                            ui.label(
                                RichText::new("Enter a valid file path to preview data").italics(),
                            );
                        } else {
                            ui.monospace(&self.import_preview_text);
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Import Variable").clicked() {
                            let var_name = self.import_var_name.trim().to_string();
                            let path = self.import_path_input.trim().to_string();
                            if var_name.is_empty() {
                                self.import_error =
                                    Some("Variable name cannot be empty".to_string());
                            } else {
                                self.worker.import_csv(
                                    var_name,
                                    path,
                                    self.import_delimiter,
                                    self.import_has_header,
                                );
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.import_dialog_open = false;
                        }
                    });
                });
            });

        self.import_dialog_open = open;
    }

    fn update_import_preview(&mut self) {
        let path = self.import_path_input.trim();
        match fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<_> = content.lines().take(6).collect();
                self.import_preview_text = lines.join("\n");
                self.import_error = None;
            }
            Err(e) => {
                self.import_preview_text.clear();
                self.import_error = Some(format!("Cannot open file: {e}"));
            }
        }
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        if !self.about_dialog_open {
            return;
        }

        let mut open = self.about_dialog_open;
        Window::new("About MatRust Studio")
            .open(&mut open)
            .fixed_size([380.0, 220.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("MatRust Studio");
                    ui.label("Clean-Room MATLAB-Compatible Scientific IDE in Rust");
                    ui.label("Version 0.1.0");
                    ui.separator();
                    ui.label("Architecture: eframe + egui | IEEE-754 Double Precision");
                    ui.label("Platform: Windows ARM64 / Linux x86_64");
                    ui.add_space(8.0);
                    ui.label(RichText::new("Independent clean-room implementation.").italics());
                });
            });

        self.about_dialog_open = open;
    }

    pub fn save_dock_layout(&self) {
        if let Ok(json) = self.dock_tree.to_json() {
            let temp_dir = std::env::temp_dir();
            let path = temp_dir.join("matrust_dock_layout.json");
            let _ = std::fs::write(path, json);
        }
    }

    pub fn try_load_dock_layout(&mut self) {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("matrust_dock_layout.json");
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(tree) = DockTree::from_json(&json) {
                self.dock_tree = tree;
            }
        }
    }
}
