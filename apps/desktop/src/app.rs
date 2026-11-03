//! Main MatStudioApp application implementing eframe::App.
//!
//! Provides the complete scientific computing IDE shell:
//! - Multi-tab toolstrip ribbon (HOME, PLOTS, APPS, contextual EDITOR, PUBLISH, VIEW)
//! - Global search box (Ctrl+Shift+Space)
//! - Address bar navigation with editable working directory
//! - Multi-panel dock (Current Folder, Workspace, Editor, Command Window, Figures, History, Help, Add-Ons, Block Diagram Editor)
//! - Status bar with engine status, active file, line/col, working dir, and zoom
//! - Offline clean-room Help & Documentation panel

use crate::addons_panel::AddOnsPanel;
use crate::block_diagram_editor::BlockDiagramEditor;
use crate::command_window::CommandWindow;
use crate::current_folder::CurrentFolderPanel;
use crate::dock::{DockTree, PanelId};
use crate::editor::{extract_active_section, EditorPanel};
use crate::execution::{GraphicsState, VariableRow, WorkerClient, WorkerResponse};
use crate::figures::FiguresPanel;
use crate::help_panel::HelpPanel;
use crate::preferences::Preferences;
use crate::statusbar::{EngineStatus, StatusBar};
use crate::theme::{Theme, ThemeMode};
use crate::workspace_panel::WorkspacePanel;
use egui::{Key, RichText, ScrollArea, TextEdit, Ui, Window};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Active tab in the top Toolstrip Ribbon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopTab {
    Home,
    Plots,
    Apps,
    Editor,
    Publish,
    View,
}

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
    pub help_panel: HelpPanel,
    pub addons_panel: AddOnsPanel,
    pub block_diagram_editor: BlockDiagramEditor,

    pub worker: WorkerClient,
    pub graphics: GraphicsState,
    pub variables: Vec<VariableRow>,

    // Top Toolstrip & Navigation State
    pub active_top_tab: TopTab,
    pub global_search_query: String,
    pub current_folder_input: String,
    pub folder_history: Vec<PathBuf>,
    pub folder_history_idx: usize,

    // Import Data Dialog state
    pub import_dialog_open: bool,
    pub import_path_input: String,
    pub import_var_name: String,
    pub import_delimiter: u8,
    pub import_has_header: bool,
    pub import_preview_text: String,
    pub import_error: Option<String>,

    // Dialogs
    pub set_path_dialog_open: bool,
    pub about_dialog_open: bool,
}

impl Default for MatStudioApp {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
            help_panel: HelpPanel::default(),
            addons_panel: AddOnsPanel::default(),
            block_diagram_editor: BlockDiagramEditor::default(),

            worker: WorkerClient::new(),
            graphics: GraphicsState::default(),
            variables: Vec::new(),

            active_top_tab: TopTab::Home,
            global_search_query: String::new(),
            current_folder_input: current_dir.to_string_lossy().to_string(),
            folder_history: vec![current_dir],
            folder_history_idx: 0,

            import_dialog_open: false,
            import_path_input: String::new(),
            import_var_name: "data".to_string(),
            import_delimiter: b',',
            import_has_header: true,
            import_preview_text: String::new(),
            import_error: None,

            set_path_dialog_open: false,
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

        // Top Header Panel: Menu Bar + Tab Strip + Toolstrip Ribbon + Address Bar
        egui::TopBottomPanel::top("top_shell_panel").show(ctx, |ui| {
            self.render_menu_bar(ui);
            ui.separator();
            self.render_tab_strip(ui);
            ui.separator();
            self.render_toolstrip(ui);
            ui.separator();
            self.render_address_bar(ui);
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
            let mut on_run_section = None;
            let mut on_clear_workspace = false;
            let mut on_refresh_workspace = false;
            let mut exec_cmd = None;
            let mut on_launch_app = None;

            let theme = self.theme.clone();
            let variables = self.variables.clone();

            let current_folder = &mut self.current_folder;
            let editor = &mut self.editor;
            let workspace_panel = &mut self.workspace_panel;
            let command_window = &mut self.command_window;
            let figures_panel = &mut self.figures_panel;
            let help_panel = &mut self.help_panel;
            let addons_panel = &mut self.addons_panel;
            let block_diagram_editor = &mut self.block_diagram_editor;
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
                            &mut on_run_section,
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
                    PanelId::Help => {
                        help_panel.show(body_ui, &theme);
                    }
                    PanelId::AddOns => {
                        addons_panel.show(body_ui, &theme, &mut on_launch_app);
                    }
                    PanelId::BlockDiagram => {
                        block_diagram_editor.show(body_ui, &theme);
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
            if let Some(code) = on_run_section {
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
            if let Some(app_id) = on_launch_app {
                self.handle_launch_app(app_id);
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

        // Set Path Dialog modal
        self.render_set_path_dialog(ctx);

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

    pub fn execute_code(&mut self, code: String) {
        self.statusbar.status = EngineStatus::Running(Instant::now());
        self.worker.execute(code);
    }

    pub fn execute_debug_code(&mut self, code: String) {
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

        // Ctrl+Enter = Run Section (between %% markers)
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Enter)) {
            if let Some(doc) = self.editor.active_doc() {
                let section = extract_active_section(&doc.content, doc.cursor_line);
                self.execute_code(section);
            }
        }

        // Ctrl+Shift+Space = Global Search Shortcut
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(Key::Space)) {
            self.dock_tree.open_panel(PanelId::Help);
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
                    PanelId::Help,
                    PanelId::AddOns,
                    PanelId::BlockDiagram,
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
                if ui.button("▶ Run Section (Ctrl+Enter)").clicked() {
                    if let Some(doc) = self.editor.active_doc() {
                        let sec = extract_active_section(&doc.content, doc.cursor_line);
                        self.execute_code(sec);
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
                if ui.button("Documentation").clicked() {
                    self.dock_tree.open_panel(PanelId::Help);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("About MATSTUDIO").clicked() {
                    self.about_dialog_open = true;
                    ui.close_menu();
                }
            });
        });
    }

    /// Renders the top Tab Strip (HOME, PLOTS, APPS, EDITOR, PUBLISH, VIEW) + Global Search.
    fn render_tab_strip(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            let tabs = [
                (TopTab::Home, "HOME"),
                (TopTab::Plots, "PLOTS"),
                (TopTab::Apps, "APPS"),
                (TopTab::Editor, "EDITOR"),
                (TopTab::Publish, "PUBLISH"),
                (TopTab::View, "VIEW"),
            ];

            for (tab, label) in tabs {
                let is_active = self.active_top_tab == tab;
                let text = if is_active {
                    RichText::new(label).strong().color(self.theme.accent)
                } else {
                    RichText::new(label).color(self.theme.text_secondary)
                };

                if ui.selectable_label(is_active, text).clicked() {
                    self.active_top_tab = tab;
                }
            }

            // Global Search Box on right side of tab strip
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Ctrl+Shift+Space")
                        .size(10.0)
                        .color(self.theme.text_secondary),
                );
                let search_box = TextEdit::singleline(&mut self.global_search_query)
                    .hint_text("🔍 Search Documentation and Commands...")
                    .desired_width(280.0);

                let resp = ui.add(search_box);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    let q = self.global_search_query.trim().to_string();
                    if !q.is_empty() {
                        self.dock_tree.open_panel(PanelId::Help);
                        self.help_panel.select_function(&q);
                    }
                }
            });
        });
    }

    /// Renders the active Toolstrip ribbon corresponding to selected TopTab.
    fn render_toolstrip(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            match self.active_top_tab {
                TopTab::Home => self.render_home_toolstrip(ui),
                TopTab::Plots => self.render_plots_toolstrip(ui),
                TopTab::Apps => self.render_apps_toolstrip(ui),
                TopTab::Editor => self.render_editor_toolstrip(ui),
                TopTab::Publish => self.render_publish_toolstrip(ui),
                TopTab::View => self.render_view_toolstrip(ui),
            }
        });
    }

    fn render_home_toolstrip(&mut self, ui: &mut Ui) {
        // Group: FILE
        ui.group(|ui| {
            ui.label(
                RichText::new("FILE")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("📄 New Script")
                .on_hover_text("Create a new script file (Ctrl+N)")
                .clicked()
            {
                self.editor.new_script();
            }
            let _ = ui
                .button("📄 New Live Script")
                .on_hover_text("Create interactive live script (.mlx) - Not implemented yet");
            let _ = ui
                .button("📂 Open")
                .on_hover_text("Open script or data file from disk");
            let _ = ui
                .button("🔍 Find Files")
                .on_hover_text("Search across project files - Not implemented yet");
        });

        // Group: VARIABLE
        ui.group(|ui| {
            ui.label(
                RichText::new("VARIABLE")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("📑 Import Data")
                .on_hover_text("Import tabular or numeric CSV data into workspace")
                .clicked()
            {
                self.open_import_dialog();
            }
            if ui
                .button("💾 Save Workspace")
                .on_hover_text("Save workspace variables to MAT-file")
                .clicked()
            {
                self.execute_code("save('workspace.mat');".to_string());
            }
            if ui
                .button("🗑 Clear Workspace")
                .on_hover_text("Remove all variables from active workspace memory (clear)")
                .clicked()
            {
                self.worker.clear();
            }
            let _ = ui
                .button("🧹 Clean Data")
                .on_hover_text("Interactive data cleaning tools - Not implemented yet");
        });

        // Group: CODE
        ui.group(|ui| {
            ui.label(
                RichText::new("CODE")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            let _ = ui
                .button("⭐ Favorites")
                .on_hover_text("Quick-access script favorites - Not implemented yet");
            if ui
                .button("🧹 Clear Commands")
                .on_hover_text("Clear command window output (clc)")
                .clicked()
            {
                self.command_window.clear();
            }
        });

        // Group: BLOCK DIAGRAM
        ui.group(|ui| {
            ui.label(
                RichText::new("BLOCK DIAGRAM")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("🔄 Block Diagram")
                .on_hover_text("Open the block diagram editor with solver")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::BlockDiagram);
            }
        });

        // Group: ENVIRONMENT
        ui.group(|ui| {
            ui.label(
                RichText::new("ENVIRONMENT")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            ui.menu_button("📐 Layout", |ui| {
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

            if ui
                .button("🛣 Set Path")
                .on_hover_text("View and configure script search paths")
                .clicked()
            {
                self.set_path_dialog_open = true;
            }

            if ui
                .button("🧩 Add-Ons")
                .on_hover_text("Open Add-On Explorer and package catalog")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::AddOns);
            }

            let _ = ui
                .button("⚙ Preferences")
                .on_hover_text("Configure IDE font, theme, and keybindings");
            let _ = ui
                .button("🔗 Connections")
                .on_hover_text("Hardware and board connections - Not implemented yet");
            let _ = ui
                .button("⚡ Parallel")
                .on_hover_text("Parallel worker cluster configuration - Not implemented yet");
        });

        // Group: RESOURCES
        ui.group(|ui| {
            ui.label(
                RichText::new("RESOURCES")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("❓ Help")
                .on_hover_text("Toggle Help & Documentation panel")
                .clicked()
            {
                if self.dock_tree.is_open(PanelId::Help) {
                    self.dock_tree.root.remove_panel(PanelId::Help);
                    self.dock_tree.root.prune_empty();
                } else {
                    self.dock_tree.open_panel(PanelId::Help);
                }
            }
            if ui
                .button("🎓 Learn")
                .on_hover_text("Open getting started documentation")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::Help);
                self.help_panel.select_function("log2");
            }
            let _ = ui
                .button("💬 Feedback")
                .on_hover_text("Submit feedback or issue report on GitHub repository");
        });
    }

    fn render_plots_toolstrip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("2-D PLOTS").size(10.0).color(self.theme.text_secondary));
            if ui.button("📈 Line Plot").on_hover_text("2-D line plot (plot)").clicked() {
                self.execute_code("t = 0:0.05:10; plot(t, sin(t)); title('Line Plot'); grid on;".to_string());
                self.dock_tree.open_panel(PanelId::Figures);
            }
            if ui.button("⁙ Scatter").on_hover_text("Scatter plot (scatter)").clicked() {
                self.execute_code("x = rand(1, 50); y = rand(1, 50); scatter(x, y); title('Scatter Plot'); grid on;".to_string());
                self.dock_tree.open_panel(PanelId::Figures);
            }
            if ui.button("📊 Bar").on_hover_text("Bar chart (bar)").clicked() {
                self.execute_code("bar([12 34 25 45 19]); title('Bar Chart');".to_string());
                self.dock_tree.open_panel(PanelId::Figures);
            }
            if ui.button("📶 Histogram").on_hover_text("Histogram distribution (hist)").clicked() {
                self.execute_code("hist(randn(1, 200), 15); title('Histogram');".to_string());
                self.dock_tree.open_panel(PanelId::Figures);
            }
            let _ = ui.button("▨ Area").on_hover_text("Filled area 2-D plot - Not implemented yet");
            let _ = ui.button("🥧 Pie").on_hover_text("Pie chart - Not implemented yet");
        });
    }

    fn render_apps_toolstrip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(
                RichText::new("BUILT-IN APPS")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("📑 Data Importer")
                .on_hover_text("Open CSV Data Importer & Cleaner")
                .clicked()
            {
                self.open_import_dialog();
            }
            if ui
                .button("📈 Plot Explorer")
                .on_hover_text("Open interactive curve and plot analyzer")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::Figures);
            }
            if ui
                .button("〰 Signal Viewer")
                .on_hover_text("Open signal waveform analyzer")
                .clicked()
            {
                self.execute_code(
                    "t = 0:0.01:1; plot(t, sin(2*pi*5*t)); title('Signal Waveform');".to_string(),
                );
                self.dock_tree.open_panel(PanelId::Figures);
            }
            if ui
                .button("🔄 Inspector")
                .on_hover_text("Open block diagram model inspector")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::BlockDiagram);
            }
            if ui
                .button("🧩 Add-On Explorer")
                .on_hover_text("Browse package and app catalog")
                .clicked()
            {
                self.dock_tree.open_panel(PanelId::AddOns);
            }
        });
    }

    fn render_editor_toolstrip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(
                RichText::new("EXECUTION")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button(RichText::new("▶ Run").color(self.theme.success).strong())
                .on_hover_text("Run active script (F5)")
                .clicked()
            {
                if let Some(doc) = self.editor.active_doc() {
                    self.execute_debug_code(doc.content.clone());
                }
            }
            if ui
                .button("▶ Run Section")
                .on_hover_text("Run section between %% markers (Ctrl+Enter)")
                .clicked()
            {
                if let Some(doc) = self.editor.active_doc() {
                    let sec = extract_active_section(&doc.content, doc.cursor_line);
                    self.execute_code(sec);
                }
            }
            if ui
                .button("▶ Run Selection")
                .on_hover_text("Run selected text (F9)")
                .clicked()
            {
                if let Some(doc) = self.editor.active_doc() {
                    self.execute_code(doc.content.clone());
                }
            }
            if ui
                .button(RichText::new("⏹ Stop").color(self.theme.error))
                .on_hover_text("Interrupt computation (Ctrl+C)")
                .clicked()
            {
                self.worker.stop();
            }
        });

        ui.group(|ui| {
            ui.label(
                RichText::new("FILE ACTIONS")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            if ui
                .button("💾 Save")
                .on_hover_text("Save file (Ctrl+S)")
                .clicked()
            {
                if let Some(doc) = self.editor.active_doc_mut() {
                    let _ = doc.save();
                }
            }
            let _ = ui
                .button("⚡ Format Code")
                .on_hover_text("Re-indent and format script code");
        });
    }

    fn render_publish_toolstrip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(
                RichText::new("PUBLISH")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            let _ = ui
                .button("📄 Publish to HTML")
                .on_hover_text("Export script report to HTML - Not implemented yet");
            let _ = ui
                .button("📑 Publish to PDF")
                .on_hover_text("Export script report to PDF - Not implemented yet");
        });
    }

    fn render_view_toolstrip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(
                RichText::new("PANELS")
                    .size(10.0)
                    .color(self.theme.text_secondary),
            );
            for panel in [
                PanelId::CurrentFolder,
                PanelId::Editor,
                PanelId::Workspace,
                PanelId::CommandWindow,
                PanelId::Figures,
                PanelId::Help,
                PanelId::BlockDiagram,
            ] {
                let mut is_open = self.dock_tree.is_open(panel);
                if ui.checkbox(&mut is_open, panel.title()).clicked() {
                    if is_open {
                        self.dock_tree.open_panel(panel);
                    } else {
                        self.dock_tree.root.remove_panel(panel);
                        self.dock_tree.root.prune_empty();
                    }
                }
            }
        });
    }

    /// Renders the Address Bar below the toolstrip ribbon.
    fn render_address_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Back button
            if ui
                .button("◀")
                .on_hover_text("Back to previous folder")
                .clicked()
                && self.folder_history_idx > 0
            {
                self.folder_history_idx -= 1;
                let p = self.folder_history[self.folder_history_idx].clone();
                self.current_folder.set_directory(p.clone());
                self.current_folder_input = p.to_string_lossy().to_string();
            }

            // Forward button
            if ui
                .button("▶")
                .on_hover_text("Forward to next folder")
                .clicked()
                && self.folder_history_idx + 1 < self.folder_history.len()
            {
                self.folder_history_idx += 1;
                let p = self.folder_history[self.folder_history_idx].clone();
                self.current_folder.set_directory(p.clone());
                self.current_folder_input = p.to_string_lossy().to_string();
            }

            // Up folder button
            if ui.button("▲").on_hover_text("Go up one folder").clicked() {
                if let Some(parent) = self.current_folder.current_dir.parent() {
                    let parent_buf = parent.to_path_buf();
                    self.current_folder.set_directory(parent_buf.clone());
                    self.current_folder_input = parent_buf.to_string_lossy().to_string();
                    self.folder_history.push(parent_buf);
                    self.folder_history_idx = self.folder_history.len() - 1;
                }
            }

            ui.label(RichText::new("📁").size(14.0));

            // Editable directory text input
            let text_edit = TextEdit::singleline(&mut self.current_folder_input)
                .desired_width(ui.available_width() - 40.0);
            let resp = ui.add(text_edit);
            if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                let target_path = PathBuf::from(self.current_folder_input.trim());
                if target_path.is_dir() {
                    self.current_folder.set_directory(target_path.clone());
                    self.folder_history.push(target_path);
                    self.folder_history_idx = self.folder_history.len() - 1;
                } else {
                    self.current_folder_input = self
                        .current_folder
                        .current_dir
                        .to_string_lossy()
                        .to_string();
                }
            }
        });
    }

    fn handle_launch_app(&mut self, app_id: &'static str) {
        match app_id {
            "app-data-importer" => self.open_import_dialog(),
            "app-plot-explorer" => self.dock_tree.open_panel(PanelId::Figures),
            "app-block-diagram-inspector" => self.dock_tree.open_panel(PanelId::BlockDiagram),
            "app-signal-viewer" => {
                self.execute_code(
                    "t = 0:0.01:1; plot(t, sin(2*pi*5*t)); title('Signal Waveform');".to_string(),
                );
                self.dock_tree.open_panel(PanelId::Figures);
            }
            _ => {}
        }
    }

    fn open_import_dialog(&mut self) {
        self.import_dialog_open = true;
        self.import_error = None;
        if self.import_path_input.is_empty() {
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

    fn render_set_path_dialog(&mut self, ctx: &egui::Context) {
        if !self.set_path_dialog_open {
            return;
        }

        let mut open = self.set_path_dialog_open;
        Window::new("Set Path - Search Directories")
            .open(&mut open)
            .default_size([460.0, 260.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Directories in current search path for functions and scripts:");
                    ui.separator();

                    ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.label(format!(
                            "• [Current Directory] {}",
                            self.current_folder.current_dir.display()
                        ));
                        ui.label(
                            "The engine search path is fixed: the current directory and the built-in crates.",
                        );
                    });

                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.set_path_dialog_open = false;
                    }
                });
            });

        self.set_path_dialog_open = open;
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        if !self.about_dialog_open {
            return;
        }

        let mut open = self.about_dialog_open;
        Window::new("About MATSTUDIO")
            .open(&mut open)
            .fixed_size([380.0, 240.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("MATSTUDIO");
                    ui.label("A scientific IDE compatible with the MATLAB language (clean-room, not affiliated).");
                    ui.label("Version 0.1.0 (Offline Numerical Computing IDE)");
                    ui.separator();
                    ui.label("Architecture: eframe + egui | IEEE-754 Double Precision");
                    ui.label("Platform: Windows ARM64 / Linux x86_64 / macOS");
                    ui.add_space(8.0);
                    ui.label(RichText::new("Independent clean-room implementation.").italics());
                    ui.label(RichText::new("Not affiliated with MathWorks; MATLAB is a trademark of The MathWorks, Inc.").size(10.0));
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
