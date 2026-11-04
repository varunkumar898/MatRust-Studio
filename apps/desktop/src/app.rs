//! Main MatStudioApp application implementing eframe::App.
//!
//! Provides the complete scientific computing IDE shell:
//! - Multi-tab toolstrip ribbon (HOME, PLOTS, APPS, contextual EDITOR, PUBLISH, VIEW)
//! - Global search box (Ctrl+Shift+Space)
//! - Address bar navigation with editable working directory
//! - Multi-panel dock (Current Folder, Workspace, Editor, Command Window, Figures, History, Help, Add-Ons, Block Diagram Editor)
//! - Status bar with engine status, active file, line/col, working dir, and zoom
//! - Offline clean-room Help & Documentation panel

use crate::actions::AppAction;
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
use crate::statusbar::{EngineStatus, StatusAction, StatusBar};
use crate::theme::{Theme, ThemeMode};
use crate::workspace_panel::WorkspacePanel;
use egui::{Key, RichText, ScrollArea, TextEdit, Ui, WidgetText, Window};
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
    pub confirm_clear_workspace: bool,
    pub cloud_dialog_open: bool,
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
            confirm_clear_workspace: false,
            cloud_dialog_open: false,
        };
        app.try_load_dock_layout();
        // Screenshot/automation hook: choose the ribbon tab at startup without
        // changing any layout or behaviour of the normal HOME startup.
        if let Ok(content) = std::env::var("MATSTUDIO_START_CONTENT") {
            if let Some(doc) = app.editor.active_doc_mut() {
                doc.content = content.replace("\\n", "\n");
                doc.is_dirty = true;
            }
        }
        if let Ok(tab) = std::env::var("MATSTUDIO_START_TAB") {
            app.active_top_tab = match tab.to_ascii_lowercase().as_str() {
                "plots" => TopTab::Plots,
                "apps" => TopTab::Apps,
                "editor" => TopTab::Editor,
                "publish" => TopTab::Publish,
                "view" => TopTab::View,
                _ => TopTab::Home,
            };
        }
        if std::env::var_os("MATSTUDIO_START_THIS_PC").is_some() {
            app.current_folder.show_this_pc = true;
            app.current_folder.refresh_locations();
        }
        app
    }
}

impl eframe::App for MatStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // One-shot viewport diagnostics. Set MATSTUDIO_DEBUG_VIEWPORT=1 when
        // checking DPI/window geometry; normal runs print nothing.
        if std::env::var_os("MATSTUDIO_DEBUG_VIEWPORT").is_some() {
            ctx.request_repaint();
            use std::sync::atomic::{AtomicUsize, Ordering};
            static FRAME: AtomicUsize = AtomicUsize::new(0);
            let frame = FRAME.fetch_add(1, Ordering::Relaxed);
            if frame < 3 || frame.is_multiple_of(20) {
                eprintln!(
                    "MATSTUDIO viewport frame {frame}: screen_rect={:?} pixels_per_point={} inner_rect={:?}",
                    ctx.screen_rect(),
                    ctx.pixels_per_point(),
                    ctx.input(|input| input.viewport().inner_rect)
                );
            }
        }

        // Start the per-pass control/id audit before any panel builds widgets.
        crate::ui_registry::begin_frame(ctx);

        // Ctrl+= / Ctrl+- / Ctrl+0 belong to the code editor font, not the
        // whole-window egui zoom (GUI-EDITOR requirement 4).
        ctx.options_mut(|options| options.zoom_with_keyboard = false);

        // Poll for asynchronous worker responses
        self.poll_worker();

        // Apply theme visuals
        ctx.style_mut(|s| self.theme.apply_to_visuals(&mut s.visuals));

        // Global Keyboard Shortcuts
        self.handle_shortcuts(ctx);

        self.render_ui(ctx);
    }
}

impl MatStudioApp {
    /// Draw the complete application shell. This is public so headless GUI
    /// tests can exercise the real panels without an eframe window.
    pub fn render_ui(&mut self, ctx: &egui::Context) {
        // One drop is handled per frame. Directories become the project root;
        // .m files open in the editor; .csv files open the importer.
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        if let Some(path) = dropped.first() {
            if path.is_dir() {
                self.current_folder.open_project(path.clone());
                self.command_window
                    .push_output(format!("Opened project folder: {}\n", path.display()));
            } else {
                if let Some(parent) = path.parent() {
                    self.current_folder.set_directory(parent.to_path_buf());
                }
                let extension = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if extension == "m" {
                    let _ = self.editor.open_file(path.clone());
                    self.command_window
                        .push_output(format!("Opened script: {}\n", path.display()));
                } else if extension == "csv" {
                    self.open_import_dialog();
                    self.import_path_input = path.display().to_string();
                    self.update_import_preview();
                    self.command_window
                        .push_output(format!("Ready to import CSV: {}\n", path.display()));
                } else {
                    self.command_window.push_output(format!(
                        "Selected folder for dropped file: {}\n",
                        path.parent().unwrap_or(path).display()
                    ));
                }
            }
        }

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

        // Bottom Status Bar. Every field is filled from the live editor/session
        // before the bar is drawn, so it cannot lag behind the active tab.
        if let Some(doc) = self.editor.active_doc() {
            self.statusbar.active_file = doc.path.clone();
            self.statusbar.active_title = doc.title.clone();
            self.statusbar.cursor_line = doc.cursor_line;
            self.statusbar.cursor_col = doc.cursor_col;
        } else {
            self.statusbar.active_file = None;
            self.statusbar.active_title.clear();
            self.statusbar.cursor_line = 1;
            self.statusbar.cursor_col = 1;
        }
        self.statusbar.working_dir = self.current_folder.current_dir.clone();
        self.statusbar.error_count = self.editor.diagnostics().len();
        self.statusbar.warning_count = 0;
        let mut status_action = None;
        egui::TopBottomPanel::bottom("bottom_status_bar").show(ctx, |ui| {
            status_action = self.statusbar.show(ui, &self.theme);
        });
        if status_action == Some(StatusAction::JumpToFirstError) {
            self.editor.jump_to_first_error(ctx);
        }

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
            let mut on_stop = false;
            let mut panel_action: Option<AppAction> = None;
            let mut on_close_figure: Option<usize> = None;

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
                            crate::editor::EditorCallbacks {
                                run_script: &mut on_run_script,
                                run_selection: &mut on_run_selection,
                                run_section: &mut on_run_section,
                                stop: &mut on_stop,
                            },
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
                        command_window.show(body_ui, &theme, &mut exec_cmd, &mut panel_action);
                    }
                    PanelId::Figures => {
                        figures_panel.show(body_ui, &theme, graphics, &mut on_close_figure);
                    }
                    PanelId::History => {
                        command_window.show_history_panel(
                            body_ui,
                            &theme,
                            &mut exec_cmd,
                            &mut panel_action,
                        );
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
                self.perform_action(AppAction::ClearWorkspace);
            }
            if on_refresh_workspace {
                self.worker.refresh_workspace();
            }
            if on_stop {
                self.worker.stop();
                self.statusbar.status = EngineStatus::Ready;
            }
            if let Some(id) = on_close_figure {
                self.graphics.close_figure(id);
                if let Some(first) = self.graphics.figures.first() {
                    self.figures_panel.selected_figure_id = first.id;
                }
            }
            if let Some(action) = panel_action {
                self.perform_action(action);
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

        // Cloud Drives dialog modal
        self.render_cloud_dialog(ctx);

        // Clear-workspace confirmation
        self.render_clear_workspace_dialog(ctx);
    }

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

    /// Execute one named action. Menus, ribbon buttons and panel buttons all
    /// call this function, so labels and behaviour cannot diverge.
    pub fn perform_action(&mut self, action: AppAction) {
        match action {
            AppAction::ClearCommandWindow => {
                self.command_window.clear();
            }
            AppAction::ClearWorkspace => {
                if self.variables.is_empty() {
                    self.worker.clear();
                } else {
                    self.confirm_clear_workspace = true;
                }
            }
            AppAction::ClearHistory => {
                self.command_window.history.clear();
            }
            AppAction::CloseFigure => {
                let id = self.figures_panel.selected_figure_id;
                self.graphics.close_figure(id);
                if let Some(first) = self.graphics.figures.first() {
                    self.figures_panel.selected_figure_id = first.id;
                }
            }
        }
    }

    fn render_clear_workspace_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_clear_workspace {
            return;
        }
        let mut open = true;
        egui::Window::new("Clear Workspace")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Remove every variable from the workspace?");
                ui.label("This cannot be undone. Saved files are not affected.");
                ui.horizontal(|ui| {
                    if ui.button("Clear Workspace").clicked() {
                        self.worker.clear();
                        self.confirm_clear_workspace = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.confirm_clear_workspace = false;
                    }
                });
            });
        if !open {
            self.confirm_clear_workspace = false;
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
        // Ctrl+= / Ctrl+- / Ctrl+0 = editor font size (not global UI zoom).
        self.editor.handle_font_shortcuts(ctx);

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
            if let Some(code) = self.editor.code_for_selection(ctx) {
                self.execute_code(code);
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

        // Ctrl+C = Stop execution only while the engine is busy. When idle the
        // same shortcut must reach the focused TextEdit so copy keeps working.
        let ctrl_c = ctx.input(|i| i.modifiers.command && i.key_pressed(Key::C));
        let paused = matches!(&self.statusbar.status, EngineStatus::Paused(_));
        let running = matches!(&self.statusbar.status, EngineStatus::Running(_));
        if ctrl_c && (paused || running) {
            if paused {
                self.worker.debug_stop();
            } else {
                self.worker.stop();
            }
            self.statusbar.status = EngineStatus::Ready;
            let _ = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::C));
        }

        // Ctrl+N = New Script
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::N)) {
            self.editor.new_script();
        }

        // Ctrl+S = Save
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            self.save_active_doc();
        }
    }

    /// Save the active editor document and refresh the Current Folder panel.
    /// The panel's list re-reads the disk every frame, so a file written by
    /// the app appears by itself; the invalidation makes the whole-project
    /// totals recount on the next frame too.
    pub fn save_active_doc(&mut self) {
        if let Some(doc) = self.editor.active_doc_mut() {
            let _ = doc.save();
        }
        self.current_folder.invalidate_totals();
    }

    fn render_menu_bar(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                if ui.button("New Script (Ctrl+N)").clicked() {
                    self.editor.new_script();
                    ui.close_menu();
                }
                if ui.button("Save (Ctrl+S)").clicked() {
                    self.save_active_doc();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Import Data (CSV)...").clicked() {
                    self.open_import_dialog();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cloud Drives...").clicked() {
                    self.cloud_dialog_open = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Close Tab (Ctrl+W)").clicked() {
                    self.editor.close_active_tab();
                    ui.close_menu();
                }
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Edit Menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Clear Command Window").clicked() {
                    self.perform_action(AppAction::ClearCommandWindow);
                    ui.close_menu();
                }
                if ui.button("Clear Workspace").clicked() {
                    self.perform_action(AppAction::ClearWorkspace);
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
                if ui.button("Run (F5)").clicked() {
                    if let Some(doc) = self.editor.active_doc() {
                        self.execute_code(doc.content.clone());
                    }
                    ui.close_menu();
                }
                if ui.button("Run Section (Ctrl+Enter)").clicked() {
                    if let Some(doc) = self.editor.active_doc() {
                        let sec = extract_active_section(&doc.content, doc.cursor_line);
                        self.execute_code(sec);
                    }
                    ui.close_menu();
                }
                if ui.button("Run Selection (F9)").clicked() {
                    if let Some(code) = self.editor.code_for_selection(ui.ctx()) {
                        self.execute_code(code);
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Stop (Ctrl+C)").clicked() {
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
                    .hint_text("Search documentation and commands")
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
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.spacing_mut().item_spacing.y = 4.0;

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

    /// Register a ribbon button so the GUI-CLEAN rect audit sees it.
    fn ribbon_button(ui: &mut Ui, id: &str, text: impl Into<WidgetText>) -> egui::Response {
        let response = ui.button(text);
        let _ = crate::ui_registry::register(ui.ctx(), id, response.id, response.rect);
        response
    }

    /// Register a ribbon menu button.
    fn ribbon_menu_button(
        ui: &mut Ui,
        id: &str,
        text: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui),
    ) -> egui::Response {
        let response = ui.menu_button(text, add_contents).response;
        let _ = crate::ui_registry::register(ui.ctx(), id, response.id, response.rect);
        response
    }

    /// Render only the active ribbon, for the headless rect/viewport audit.
    pub fn render_toolstrip_for_test(&mut self, ui: &mut Ui) {
        self.render_toolstrip(ui);
    }

    fn render_home_toolstrip(&mut self, ui: &mut Ui) {
        // Flat, wrapping button row. Group frames made egui reserve a huge
        // empty toolbar band at 1280 px; separators keep the scopes readable
        // without changing the layout height.
        if Self::ribbon_button(ui, "ribbon.home.new_script", "New Script")
            .on_hover_text("Create a new script file (Ctrl+N)")
            .clicked()
        {
            self.editor.new_script();
        }
        ui.separator();
        if Self::ribbon_button(ui, "ribbon.home.import_data", "Import Data")
            .on_hover_text("Import tabular or numeric CSV data into workspace")
            .clicked()
        {
            self.open_import_dialog();
        }
        if Self::ribbon_button(ui, "ribbon.home.save_workspace", "Save Workspace")
            .on_hover_text("Save workspace variables to MAT-file")
            .clicked()
        {
            self.execute_code("save('workspace.mat');".to_string());
        }
        let clear_workspace =
            Self::ribbon_button(ui, "ribbon.home.clear_workspace", "Clear Workspace")
                .on_hover_text("Remove all workspace variables (asks for confirmation)");
        AppAction::ClearWorkspace.register_control(
            ui.ctx(),
            clear_workspace.id,
            clear_workspace.rect,
        );
        if clear_workspace.clicked() {
            self.perform_action(AppAction::ClearWorkspace);
        }
        ui.separator();
        let clear_commands = Self::ribbon_button(
            ui,
            "ribbon.home.clear_command_window",
            "Clear Command Window",
        )
        .on_hover_text("Clear only the Command Window log");
        AppAction::ClearCommandWindow.register_control(
            ui.ctx(),
            clear_commands.id,
            clear_commands.rect,
        );
        if clear_commands.clicked() {
            self.perform_action(AppAction::ClearCommandWindow);
        }
        ui.separator();
        if Self::ribbon_button(ui, "ribbon.home.block_diagram", "Block Diagram")
            .on_hover_text("Open the block diagram editor with solver")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::BlockDiagram);
        }
        ui.separator();
        let _ = Self::ribbon_menu_button(ui, "ribbon.home.layout", "Layout", |ui| {
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
        if Self::ribbon_button(ui, "ribbon.home.set_path", "Set Path")
            .on_hover_text("View and configure script search paths")
            .clicked()
        {
            self.set_path_dialog_open = true;
        }
        if Self::ribbon_button(ui, "ribbon.home.addons", "Add-Ons")
            .on_hover_text("Open Add-On Explorer and package catalog")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::AddOns);
        }
        ui.separator();
        if Self::ribbon_button(ui, "ribbon.home.help", "Help")
            .on_hover_text("Toggle Help and Documentation panel")
            .clicked()
        {
            if self.dock_tree.is_open(PanelId::Help) {
                self.dock_tree.root.remove_panel(PanelId::Help);
                self.dock_tree.root.prune_empty();
            } else {
                self.dock_tree.open_panel(PanelId::Help);
            }
        }
        if Self::ribbon_button(ui, "ribbon.home.learn", "Learn")
            .on_hover_text("Open getting started documentation")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::Help);
            self.help_panel.select_function("log2");
        }
    }

    fn render_plots_toolstrip(&mut self, ui: &mut Ui) {
        if Self::ribbon_button(ui, "ribbon.plots.line", "Line Plot")
            .on_hover_text("2-D line plot (plot)")
            .clicked()
        {
            self.execute_code(
                "t = 0:0.05:10; plot(t, sin(t)); title('Line Plot'); grid on;".to_string(),
            );
            self.dock_tree.open_panel(PanelId::Figures);
        }
        if Self::ribbon_button(ui, "ribbon.plots.scatter", "Scatter")
            .on_hover_text("Scatter plot (scatter)")
            .clicked()
        {
            self.execute_code(
                "x = rand(1, 50); y = rand(1, 50); scatter(x, y); title('Scatter Plot'); grid on;"
                    .to_string(),
            );
            self.dock_tree.open_panel(PanelId::Figures);
        }
        if Self::ribbon_button(ui, "ribbon.plots.bar", "Bar")
            .on_hover_text("Bar chart (bar)")
            .clicked()
        {
            self.execute_code("bar([12 34 25 45 19]); title('Bar Chart');".to_string());
            self.dock_tree.open_panel(PanelId::Figures);
        }
        if Self::ribbon_button(ui, "ribbon.plots.histogram", "Histogram")
            .on_hover_text("Histogram distribution (hist)")
            .clicked()
        {
            self.execute_code("hist(randn(1, 200), 15); title('Histogram');".to_string());
            self.dock_tree.open_panel(PanelId::Figures);
        }
    }

    fn render_apps_toolstrip(&mut self, ui: &mut Ui) {
        if Self::ribbon_button(ui, "ribbon.apps.data_importer", "Data Importer")
            .on_hover_text("Open CSV Data Importer and Cleaner")
            .clicked()
        {
            self.open_import_dialog();
        }
        if Self::ribbon_button(ui, "ribbon.apps.plot_explorer", "Plot Explorer")
            .on_hover_text("Open interactive curve and plot analyzer")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::Figures);
        }
        if Self::ribbon_button(ui, "ribbon.apps.signal_viewer", "Signal Viewer")
            .on_hover_text("Open signal waveform analyzer")
            .clicked()
        {
            self.execute_code(
                "t = 0:0.01:1; plot(t, sin(2*pi*5*t)); title('Signal Waveform');".to_string(),
            );
            self.dock_tree.open_panel(PanelId::Figures);
        }
        if Self::ribbon_button(ui, "ribbon.apps.inspector", "Inspector")
            .on_hover_text("Open block diagram model inspector")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::BlockDiagram);
        }
        if Self::ribbon_button(ui, "ribbon.apps.addon_explorer", "Add-On Explorer")
            .on_hover_text("Browse package and app catalog")
            .clicked()
        {
            self.dock_tree.open_panel(PanelId::AddOns);
        }
    }

    fn render_editor_toolstrip(&mut self, ui: &mut Ui) {
        if Self::ribbon_button(ui, "ribbon.editor.run", "Run")
            .on_hover_text("Run active script (F5)")
            .clicked()
        {
            if let Some(doc) = self.editor.active_doc() {
                self.execute_debug_code(doc.content.clone());
            }
        }
        if Self::ribbon_button(ui, "ribbon.editor.run_section", "Run Section")
            .on_hover_text("Run section between %% markers (Ctrl+Enter)")
            .clicked()
        {
            if let Some(doc) = self.editor.active_doc() {
                let sec = extract_active_section(&doc.content, doc.cursor_line);
                self.execute_code(sec);
            }
        }
        if Self::ribbon_button(ui, "ribbon.editor.run_selection", "Run Selection")
            .on_hover_text("Run selected text, or the current line (F9)")
            .clicked()
        {
            if let Some(code) = self.editor.code_for_selection(ui.ctx()) {
                self.execute_code(code);
            }
        }
        if Self::ribbon_button(ui, "ribbon.editor.stop", "Stop")
            .on_hover_text("Interrupt computation (Ctrl+C)")
            .clicked()
        {
            self.worker.stop();
        }
        ui.separator();
        if Self::ribbon_button(ui, "ribbon.editor.save", "Save")
            .on_hover_text("Save file (Ctrl+S)")
            .clicked()
        {
            self.save_active_doc();
        }
    }

    fn render_publish_toolstrip(&mut self, ui: &mut Ui) {
        ui.label("Publishing is not available in this build.");
    }

    fn render_view_toolstrip(&mut self, ui: &mut Ui) {
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
            let response = ui.checkbox(&mut is_open, panel.title());
            let control_id = format!("ribbon.view.panel.{}", panel.title());
            let _ = crate::ui_registry::register(ui.ctx(), &control_id, response.id, response.rect);
            if response.clicked() {
                if is_open {
                    self.dock_tree.open_panel(panel);
                } else {
                    self.dock_tree.root.remove_panel(panel);
                    self.dock_tree.root.prune_empty();
                }
            }
        }
    }

    /// Renders the Address Bar below the toolstrip ribbon.
    fn render_address_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Back button
            if ui
                .button("<")
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
                .button(">")
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
            if ui.button("Up").on_hover_text("Go up one folder").clicked() {
                if let Some(parent) = self.current_folder.current_dir.parent() {
                    let parent_buf = parent.to_path_buf();
                    self.current_folder.set_directory(parent_buf.clone());
                    self.current_folder_input = parent_buf.to_string_lossy().to_string();
                    self.folder_history.push(parent_buf);
                    self.folder_history_idx = self.folder_history.len() - 1;
                }
            }

            ui.label("Dir:");

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
                            "- [Current Directory] {}",
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

    /// Cloud Drives dialog. One place to see every supported provider, open
    /// its synced folder in one click, or pick any folder manually. The
    /// sign-in itself happens in the provider's own app or in the browser
    /// via the official links - MATSTUDIO never sees or stores credentials.
    fn render_cloud_dialog(&mut self, ctx: &egui::Context) {
        if !self.cloud_dialog_open {
            return;
        }
        let theme = self.theme.clone();
        let mut open = self.cloud_dialog_open;
        let mut open_project: Option<PathBuf> = None;
        Window::new("Cloud Drives")
            .open(&mut open)
            .id(egui::Id::new("cloud_drives_dialog"))
            .fixed_size([520.0, 380.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(
                        "Cloud files are read from the provider's synced folder on this PC. \
                         Sign-in happens inside the provider's own app or in your browser - \
                         MATSTUDIO never asks for and never stores your password or tokens.",
                    )
                    .small()
                    .color(theme.text_secondary),
                );
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("cloud_drives_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for provider in crate::cloud_accounts::PROVIDERS {
                            ui.push_id(provider.label, |ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(provider.label);
                                    let detected =
                                        crate::cloud_accounts::live_sync_folder(provider);
                                    match detected {
                                        Some(folder) => {
                                            ui.label(
                                                RichText::new("Synced folder detected")
                                                    .small()
                                                    .color(theme.accent),
                                            );
                                            let open_btn = ui
                                                .button("Open")
                                                .on_hover_text(folder.display().to_string());
                                            let _ = crate::ui_registry::register(
                                                ui.ctx(),
                                                "cloud.open_sync_folder",
                                                open_btn.id,
                                                open_btn.rect,
                                            );
                                            if open_btn.clicked() {
                                                open_project = Some(folder);
                                            }
                                        }
                                        None => {
                                            ui.label(
                                                RichText::new("Not detected")
                                                    .small()
                                                    .color(theme.text_secondary),
                                            );
                                            ui.hyperlink_to(
                                                "Sign in / install",
                                                provider.official_url,
                                            );
                                        }
                                    }
                                });
                            });
                        }
                        ui.separator();
                        let browse = ui
                            .button("Open any folder...")
                            .on_hover_text("Pick any local or synced folder as the project");
                        let _ = crate::ui_registry::register(
                            ui.ctx(),
                            "cloud.browse_any",
                            browse.id,
                            browse.rect,
                        );
                        if browse.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                open_project = Some(folder);
                            }
                        }
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Drag and drop a folder onto the Current Folder panel to open \
                                 it as a project at any time.",
                            )
                            .small()
                            .color(theme.text_secondary),
                        );
                    });
            });
        self.cloud_dialog_open = open;
        if let Some(folder) = open_project {
            self.current_folder.open_project(folder);
            self.command_window
                .push_output("Opened cloud/local folder as project.\n".to_string());
        }
    }

    pub fn save_dock_layout(&self) {
        if let Ok(json) = self.dock_tree.to_json() {
            let temp_dir = std::env::temp_dir();
            let path = temp_dir.join("matrust_dock_layout.json");
            let _ = std::fs::write(path, json);
        }
    }

    pub fn try_load_dock_layout(&mut self) {
        if std::env::var_os("MATSTUDIO_RESET_LAYOUT").is_some() {
            return;
        }
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("matrust_dock_layout.json");
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(tree) = DockTree::from_json(&json) {
                self.dock_tree = tree;
            }
        }
    }
}
