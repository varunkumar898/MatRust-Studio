//! MatRust Studio application: window layout, toolstrip, panels, dialogs,
//! and the bridge between the GUI and the interpreter worker.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::egui;
use egui::Id;

use matlab_diagnostics::Diagnostic;
use matlab_runtime::{Env, RunResult};

use crate::command_window::{CommandWindowState, Line, Submitted};
use crate::current_folder::{FolderAction, FolderState};
use crate::editor::CodeEditor;
use crate::execution::{Origin, Reply, Request, Worker};
use crate::figures::FiguresState;
use crate::preferences::Prefs;
use crate::statusbar::{self, RunState, StatusLine};
use crate::theme::{self, ThemeKind};
use crate::workspace_panel::{WorkspaceAction, WorkspaceState};

pub const APP_NAME: &str = "MatRust Studio";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const LINE_PLOT_SNIPPET: &str = "x = [1 2 3 4 5];\ny = [1 4 9 16 25];\nplot(x, y);\ntitle('Quadratic');\nxlabel('x');\nylabel('y');\n";

pub const EXAMPLE_SOURCE: &str = "\
% Matrix example (the computed result matches MATLAB semantics):
A = [1 2; 3 4];
B = [5 6; 7 8];

C = A * B;
disp(C);

% A plot you can zoom and pan in the Figures panel:
xs = [0 1 2 3 4 5];
ys = [0 1 4 9 16 25];
plot(xs, ys);
title('y = x^2');
xlabel('x');
ylabel('y');
grid on;
";

// ---------------------------------------------------------------- documents

pub struct Doc {
    pub path: Option<PathBuf>,
    pub title: String,
    pub editor: CodeEditor,
    pub modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    CommandWindow,
    /// Carries the doc index that requested the run (for error markers).
    Editor(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutPreset {
    Standard,
    EditorFocused,
    PlotFocused,
}

struct KeyFlags {
    run: bool,
    save: bool,
    save_as: bool,
    new: bool,
    find: bool,
    open: bool,
    close_tab: bool,
    toggle_cmd: bool,
}

// ---------------------------------------------------------------- app

pub struct MatRustApp {
    pub prefs: Prefs,
    applied_theme: Option<(ThemeKind, f32)>,
    theme_kind: ThemeKind,
    palette: theme::SyntaxPalette,

    docs: Vec<Doc>,
    active: Option<usize>,

    cmd: CommandWindowState,
    workspace: WorkspaceState,
    env_snapshot: Option<Env>,
    folder: FolderState,
    figures: FiguresState,

    worker: Worker,
    next_request_id: u64,
    pending: Option<(u64, PendingKind)>,
    /// Smoke-test hook: run this source through the real worker once on
    /// startup (set via MATRUST_AUTORUN; used by scripted UI verification).
    auto_run: Option<String>,
    running: bool,
    last_error: bool,
    last_duration: Option<Duration>,

    toolstrip: usize,
    left_split: f32,
    /// Highest figure handle seen from the session, for auto-opening the
    /// Figures panel on fresh plots.
    figures_seen: usize,
    find_open: bool,
    find_query: String,
    open_dialog: Option<String>,
    save_as_dialog: Option<String>,
    prefs_open: bool,
    about_open: bool,
    compat_open: bool,
    apps_open: bool,
    apps_selected: Option<String>,
    close_tab_pending: Option<usize>,
}

impl MatRustApp {
    pub fn new(cc: &eframe::CreationContext<'_>, prefs: Prefs) -> Self {
        let kind = prefs.theme;
        theme::apply(&cc.egui_ctx, kind, prefs.font_size);

        let cwd = prefs
            .working_dir
            .as_ref()
            .map(PathBuf::from)
            .filter(|d| d.is_dir())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut app = Self {
            applied_theme: Some((kind, prefs.font_size)),
            theme_kind: kind,
            palette: palette_for(kind),
            docs: Vec::new(),
            active: None,
            cmd: CommandWindowState::default(),
            workspace: WorkspaceState::default(),
            env_snapshot: None,
            folder: FolderState::with_directory(cwd.clone()),
            figures: FiguresState::default(),
            worker: Worker::spawn(),
            next_request_id: 1,
            pending: None,
            running: false,
            last_error: false,
            last_duration: None,
            toolstrip: 0,
            left_split: 0.35,
            auto_run: std::env::var("MATRUST_AUTORUN")
                .ok()
                .filter(|s| !s.is_empty()),
            figures_seen: 0,
            find_open: false,
            find_query: String::new(),
            open_dialog: None,
            save_as_dialog: None,
            prefs_open: false,
            about_open: false,
            compat_open: false,
            apps_open: false,
            apps_selected: None,
            close_tab_pending: None,
            prefs,
        };

        app.cmd.push(Line::Info(format!(
            "{APP_NAME} {APP_VERSION} — independent MATLAB-inspired numerical IDE"
        )));
        app.cmd.push(Line::Info(
            "Type commands at `>>` and press Enter; F5 runs the open script; `who`, `clear`, `clc`, `exit` are supported."
                .to_string(),
        ));
        app.cmd.wants_focus = true;

        for h in app.prefs.command_history.clone() {
            app.cmd.remember_history(&h);
        }

        let paths = app.prefs.open_files.clone();
        for path in &paths {
            app.open_file_quiet(Path::new(path));
        }
        let active_path = app.prefs.active_file.clone();
        if let Some(active) = &active_path {
            let target = PathBuf::from(active);
            if let Some(idx) = app
                .docs
                .iter()
                .position(|d| d.path.as_deref() == Some(target.as_path()))
            {
                app.active = Some(idx);
            }
            let _ = active;
        }

        if app.docs.is_empty() {
            // First-ever run: seed the example so the app is immediately useful.
            app.docs.push(Doc {
                path: None,
                title: "example.m".to_string(),
                editor: CodeEditor::with_text(EXAMPLE_SOURCE, "doc_example"),
                modified: true,
            });
            app.active = Some(0);
        }
        app
    }

    fn apply_theme_if_changed(&mut self, ctx: &egui::Context) {
        if self.applied_theme != Some((self.theme_kind, self.prefs.font_size)) {
            theme::apply(ctx, self.theme_kind, self.prefs.font_size);
            self.palette = palette_for(self.theme_kind);
            self.applied_theme = Some((self.theme_kind, self.prefs.font_size));
        }
    }

    fn take_keys(&self, ctx: &egui::Context) -> KeyFlags {
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.command;
            KeyFlags {
                run: i.key_pressed(egui::Key::F5),
                save: ctrl && i.key_pressed(egui::Key::S) && !i.modifiers.shift,
                save_as: ctrl && i.key_pressed(egui::Key::S) && i.modifiers.shift,
                new: ctrl && i.key_pressed(egui::Key::N),
                find: ctrl && i.key_pressed(egui::Key::F),
                open: ctrl && i.key_pressed(egui::Key::O),
                close_tab: ctrl && i.key_pressed(egui::Key::W),
                toggle_cmd: ctrl && i.key_pressed(egui::Key::Backtick),
            }
        })
    }

    fn root(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.apply_theme_if_changed(&ctx);

        let keys = self.take_keys(&ctx);
        if keys.run {
            self.run_active_script();
        }
        if keys.save {
            self.save_active();
        }
        if keys.save_as {
            self.begin_save_as();
        }
        if keys.new {
            self.new_script();
        }
        if keys.find {
            self.find_open = true;
        }
        if keys.open {
            self.open_dialog = Some(String::new());
        }
        if keys.close_tab {
            self.close_tab_pending = self.active;
        }
        if keys.toggle_cmd {
            self.prefs.layout.show_command_window = !self.prefs.layout.show_command_window;
            self.status_note(if self.prefs.layout.show_command_window {
                "Command Window shown (Ctrl+`)."
            } else {
                "Command Window hidden (Ctrl+`)."
            });
        }

        self.drain_worker(&ctx);
        // Smoke-test hook (MATRUST_AUTORUN): echo + submit through the real
        // worker path once, on the first frame, then never again.
        if let Some(source) = self.auto_run.take() {
            self.cmd.push(Line::Echo(source.clone()));
            self.submit(
                source,
                Origin::Selection {
                    name: "smoke".to_string(),
                },
                PendingKind::Editor(usize::MAX),
            );
        }

        // Starting panel proportions, scaled to the current screen. They act
        // as defaults; dragged sizes are persisted by egui itself.
        let screen = ui.available_rect_before_wrap();
        let command_default_h = (screen.height() * 0.28).clamp(180.0, 420.0);
        let left_default_w = (screen.width() * 0.22).clamp(200.0, 440.0);
        let figures_default_w = (screen.width() * 0.30).clamp(280.0, 620.0);

        self.top_panel(ui);
        // Carve order matters: bottom strips first so they span the full
        // window width; then left/right columns; central panel last.
        self.status_panel(ui);
        self.command_window_panel(ui, command_default_h);
        self.left_panel(ui, left_default_w);
        self.figures_panel(ui, figures_default_w);
        self.central_panel(ui);
        self.show_windows(ui);

        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }

    fn drain_worker(&mut self, ctx: &egui::Context) {
        let mut replies = Vec::new();
        while let Ok(reply) = self.worker.reply_rx.try_recv() {
            replies.push(reply);
        }
        for reply in replies {
            match reply {
                Reply::ExecDone {
                    id,
                    origin,
                    result,
                    duration,
                } => {
                    let kind = self
                        .pending
                        .and_then(|(pid, k)| if pid == id { Some(k) } else { None });
                    if kind.is_none() {
                        continue; // stale reply
                    }
                    self.pending = None;
                    self.running = false;
                    self.last_duration = Some(duration);
                    self.last_error = result.is_err();
                    let focus_cmd = matches!(origin, Origin::CommandWindow { .. });
                    let editor_idx = match kind {
                        Some(PendingKind::Editor(idx)) => Some(idx),
                        _ => None,
                    };
                    self.apply_run_result(&origin, result, duration, editor_idx);
                    if focus_cmd {
                        self.cmd.wants_focus = true;
                    }
                }
                Reply::WorkspaceCleared { result, .. } => {
                    self.apply_session_snapshot(&result);
                    self.cmd.push(Line::Info("Workspace cleared.".to_string()));
                }
            }
        }
        if self.running {
            ctx.request_repaint();
        }
    }

    // -------------------------------------------------------------- execution

    fn submit(&mut self, source: String, origin: Origin, kind: PendingKind) {
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.pending = Some((id, kind));
        self.running = true;
        self.last_error = false;
        let _ = self
            .worker
            .request_tx
            .send(Request::Exec { id, source, origin });
    }

    fn run_source_now(&mut self, source: &str) {
        if self.running {
            self.cmd.push(Line::Err(
                "Busy: an execution is already running.".to_string(),
            ));
            return;
        }
        self.cmd.push(Line::Echo(source.to_string()));
        self.cmd.remember_history(source);
        self.submit(
            source.to_string(),
            Origin::CommandWindow {
                echo: source.to_string(),
            },
            PendingKind::CommandWindow,
        );
    }

    fn run_active_script(&mut self) {
        if self.running {
            self.cmd.push(Line::Err(
                "Busy: an execution is already running.".to_string(),
            ));
            return;
        }
        let Some(idx) = self.active else {
            self.cmd.push(Line::Err(
                "No script is open. Use New Script or Open.".to_string(),
            ));
            return;
        };
        let doc = &mut self.docs[idx];
        let title = doc.title.clone();
        let path = doc.path.as_ref().map(|p| p.to_string_lossy().to_string());
        let source = doc.editor.text();
        doc.editor.error_line = None;
        self.cmd.push(Line::Info(format!("── run {} ──", title)));
        self.submit(
            source,
            Origin::Script { path, name: title },
            PendingKind::Editor(idx),
        );
    }

    fn run_selection(&mut self) {
        if self.running {
            self.cmd.push(Line::Err(
                "Busy: an execution is already running.".to_string(),
            ));
            return;
        }
        let Some(idx) = self.active else {
            self.cmd
                .push(Line::Err("No selection: the editor is closed.".to_string()));
            return;
        };
        let doc = &mut self.docs[idx];
        let Some(selection) = doc.editor.selection_text() else {
            self.cmd.push(Line::Err(
                "No selection: select lines in the editor first.".to_string(),
            ));
            return;
        };
        let name = doc.title.clone();
        doc.editor.error_line = None;
        self.cmd.push(Line::Info(format!(
            "── run selection ({}) ──",
            selection.lines().count()
        )));
        self.submit(
            selection,
            Origin::Selection { name },
            PendingKind::Editor(idx),
        );
    }

    fn request_stop(&mut self) {
        if self.running {
            self.worker
                .cancel
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.cmd.push(Line::Info(
                "Stop requested — interruption happens between statements; one long-running expression finishes before stopping."
                    .to_string(),
            ));
        }
    }

    fn clear_workspace(&mut self) {
        if self.running {
            self.cmd.push(Line::Err(
                "Busy: an execution is already running.".to_string(),
            ));
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        let _ = self.worker.request_tx.send(Request::ClearWorkspace { id });
    }

    fn apply_run_result(
        &mut self,
        origin: &Origin,
        result: Result<RunResult, Diagnostic>,
        duration: Duration,
        editor_idx: Option<usize>,
    ) {
        match result {
            Ok(result) => {
                self.push_outputs(&result);
                self.apply_session_snapshot(&result);
                let _ = (origin, duration);
            }
            Err(diag) => {
                self.cmd.push(Line::Err(diag_text(&diag)));
                self.mark_editor_error(&diag, editor_idx);
            }
        }
    }

    fn apply_session_snapshot(&mut self, result: &RunResult) {
        self.env_snapshot = Some(result.env.clone());
        self.workspace
            .update_from(matlab_runtime::variable_rows(&result.env));
        self.figures.replace_from(result.figures.clone());
        // Auto-open the Figures panel the first time the session owns a
        // figure (matching the desktop convention of plots opening their
        // window). Later figures keep it open; the user can hide it and it
        // only springs back for a NEW figure handle.
        let max_handle = self
            .figures
            .figures
            .iter()
            .map(|f| f.handle)
            .max()
            .unwrap_or(0);
        if max_handle > self.figures_seen {
            self.figures_seen = max_handle;
            if !self.prefs.layout.show_right {
                self.prefs.layout.show_right = true;
                self.status_note("Figures panel opened (a figure was created).");
            }
        }
    }

    /// Push interpreter output into the command window. Side-effect calls
    /// (`plot()` etc.) produce `ans = []` records that carry no text — those
    /// are filtered out, matching MATLAB behavior where plotting prints
    /// nothing.
    fn push_outputs(&mut self, result: &RunResult) {
        for line in &result.stdout {
            self.cmd.push(Line::Out(line.clone()));
        }
        for out in &result.outputs {
            let value_text = matlab_runtime::format_value(&out.value);
            let is_empty_ans = out.label.is_none()
                && out
                    .value
                    .as_matrix()
                    .map(|m| m.data.is_empty())
                    .unwrap_or(false);
            if is_empty_ans {
                continue;
            }
            let label = out.label.as_deref().unwrap_or("ans");
            let mut first = true;
            for part in value_text.split('\n') {
                if first {
                    self.cmd.push(Line::Out(format!("{} = {}", label, part)));
                    first = false;
                } else {
                    self.cmd.push(Line::Out(format!("  {}", part)));
                }
            }
        }
        if result.interrupted {
            self.cmd
                .push(Line::Err("Execution interrupted by Stop.".to_string()));
        }
    }

    /// Gutter error marker on the editors based on the diagnostics span.
    fn mark_editor_error(&mut self, diag: &Diagnostic, editor_idx: Option<usize>) {
        let Some(idx) = editor_idx.or(self.active) else {
            return;
        };
        if let Some(span) = diag.span {
            if let Some(doc) = self.docs.get_mut(idx) {
                doc.editor.error_line = Some(span.line.saturating_sub(1));
            }
        }
    }

    // -------------------------------------------------------------- files

    fn open_file_quiet(&mut self, path: &Path) -> bool {
        if let Some(existing) = self
            .docs
            .iter()
            .position(|d| d.path.as_deref() == Some(path))
        {
            self.active = Some(existing);
            return true;
        }
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let salt = format!("doc_{}", path.to_string_lossy());
                self.docs.push(Doc {
                    path: Some(path.to_path_buf()),
                    title: file_title(path),
                    editor: CodeEditor::with_text(text, salt),
                    modified: false,
                });
                self.active = Some(self.docs.len() - 1);
                true
            }
            Err(err) => {
                self.cmd.push(Line::Err(format!(
                    "Open failed: {}: {}",
                    path.display(),
                    err
                )));
                false
            }
        }
    }

    fn open_path(&mut self, raw: &str) {
        let candidate = resolve(&self.folder.cwd, raw.trim());
        self.open_dialog = None;
        if candidate.is_dir() {
            self.folder.navigate_to(&candidate);
            self.prefs.working_dir = Some(self.folder.cwd.to_string_lossy().to_string());
            self.status_note(format!("Working directory: {}", self.folder.cwd.display()));
            return;
        }
        match std::fs::metadata(&candidate) {
            Ok(_) => {
                if self.open_file_quiet(&candidate) {
                    self.folder.refresh();
                    self.save_prefs_state();
                }
            }
            Err(err) => {
                self.cmd.push(Line::Err(format!(
                    "Open failed: {} ({})",
                    candidate.display(),
                    err
                )));
            }
        }
    }

    fn save_active(&mut self) {
        let Some(idx) = self.active else {
            self.status_note("Nothing to save.");
            return;
        };
        if self.docs[idx].path.is_none() {
            self.begin_save_as();
            return;
        }
        let doc = &mut self.docs[idx];
        let path = doc.path.clone().expect("path is set when not None");
        let text = doc.editor.text();
        match std::fs::write(&path, text) {
            Ok(()) => {
                self.docs[idx].modified = false;
                self.status_note(format!("Saved {}", path.display()));
                self.folder.refresh();
                self.save_prefs_state();
            }
            Err(err) => {
                self.cmd.push(Line::Err(format!(
                    "Save failed: {} ({})",
                    path.display(),
                    err
                )));
            }
        }
    }

    fn begin_save_as(&mut self) {
        let default_name = self
            .active
            .and_then(|i| self.docs.get(i))
            .map(|d| d.title.clone())
            .unwrap_or_else(|| "untitled.m".to_string());
        self.save_as_dialog = Some(default_name);
    }

    fn save_doc_as(&mut self, name: &str) {
        let Some(idx) = self.active else {
            self.status_note("Nothing to save.");
            return;
        };
        let path = resolve(&self.folder.cwd, name);
        let text = self.docs[idx].editor.text();
        match std::fs::write(&path, text) {
            Ok(()) => {
                self.docs[idx].path = Some(path.clone());
                self.docs[idx].title = file_title(&path);
                self.docs[idx].modified = false;
                self.status_note(format!("Saved {}", path.display()));
                self.folder.refresh();
                self.save_prefs_state();
            }
            Err(err) => {
                self.cmd.push(Line::Err(format!(
                    "Save failed: {} ({})",
                    path.display(),
                    err
                )));
            }
        }
    }

    fn new_script(&mut self) {
        let n = self.docs.len() + 1;
        let title = unique_title(&self.docs, "untitled");
        self.docs.push(Doc {
            path: None,
            title,
            editor: CodeEditor::new(format!("doc_untitled{}", n)),
            modified: false,
        });
        self.active = Some(self.docs.len() - 1);
    }

    fn close_tab(&mut self, idx: usize) {
        if self.docs[idx].modified {
            self.close_tab_pending = Some(idx);
            return;
        }
        self.docs.remove(idx);
        self.active = match (self.docs.len(), self.active) {
            (0, _) => None,
            (len, Some(a)) => Some(a.min(len - 1)),
            (len, None) => Some(len - 1),
        };
        self.status_note("Tab closed.");
        self.save_prefs_state();
    }

    fn save_prefs_state(&mut self) {
        self.prefs.working_dir = Some(self.folder.cwd.to_string_lossy().to_string());
        self.prefs.open_files = self
            .docs
            .iter()
            .filter_map(|d| d.path.clone())
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        self.prefs.active_file = self
            .docs
            .get(self.active.unwrap_or(usize::MAX))
            .map(|d| d.path.clone())
            .flatten()
            .map(|p| p.to_string_lossy().to_string());
        self.prefs.command_history = self.cmd.history.clone();
    }

    fn status_note(&mut self, text: impl Into<String>) {
        self.cmd.push(Line::Info(text.into()));
    }

    // -------------------------------------------------------------- panels

    fn top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("matrust_top_v2").show(ui, |ui| {
            egui::containers::menu::MenuBar::new().ui(ui, |bar| {
                // File
                bar.menu_button("File", |ui| {
                    ui.set_min_width(200.0);
                    if ui.button("New Script (Ctrl+N)").clicked() {
                        self.new_script();
                        ui.close();
                    }
                    if ui.button("Open… (Ctrl+O)").clicked() {
                        self.open_dialog = Some(String::new());
                        ui.close();
                    }
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.save_active();
                        ui.close();
                    }
                    if ui.button("Save As… (Ctrl+Shift+S)").clicked() {
                        self.begin_save_as();
                        ui.close();
                    }
                    if ui.button("Close Tab (Ctrl+W)").clicked() {
                        self.close_tab_pending = self.active;
                        ui.close();
                    }
                });
                // Edit
                bar.menu_button("Edit", |ui| {
                    ui.set_min_width(170.0);
                    let Some(idx) = self.active else {
                        ui.weak("No editor open");
                        return;
                    };
                    if ui.button("Undo (Ctrl+Z)").clicked() {
                        self.docs[idx].editor.undo();
                        ui.close();
                    }
                    if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                        self.docs[idx].editor.redo();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Find (Ctrl+F)").clicked() {
                        self.find_open = true;
                        ui.close();
                    }
                });
                // View
                bar.menu_button("View", |ui| {
                    ui.set_min_width(200.0);
                    ui.menu_button("Layout preset", |ui| {
                        for (label, preset) in [
                            ("Standard", LayoutPreset::Standard),
                            ("Editor-focused", LayoutPreset::EditorFocused),
                            ("Plot-focused", LayoutPreset::PlotFocused),
                        ] {
                            if ui.button(label).clicked() {
                                self.apply_layout(preset);
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Find (Ctrl+F)").clicked() {
                        self.find_open = true;
                        ui.close();
                    }
                    if ui.button("Next Tab").clicked() {
                        if let Some(a) = self.active {
                            self.active = Some((a + 1) % self.docs.len().max(1));
                        }
                        ui.close();
                    }
                });
                // Window
                bar.menu_button("Window", |ui| {
                    ui.set_min_width(240.0);
                    if ui
                        .checkbox(
                            &mut self.prefs.layout.show_left,
                            "Current Folder & Workspace",
                        )
                        .clicked()
                    {
                        self.status_note("Layout updated.");
                    }
                    if ui
                        .checkbox(
                            &mut self.prefs.layout.show_command_window,
                            "Command Window (Ctrl+`)",
                        )
                        .clicked()
                    {
                        self.status_note("Layout updated.");
                    }
                    if ui
                        .checkbox(&mut self.prefs.layout.show_right, "Figures")
                        .clicked()
                    {
                        self.status_note("Layout updated.");
                    }
                    ui.separator();
                    if ui.button("Layout: Standard").clicked() {
                        self.apply_layout(LayoutPreset::Standard);
                        ui.close();
                    }
                    if ui.button("Layout: Editor-focused").clicked() {
                        self.apply_layout(LayoutPreset::EditorFocused);
                        ui.close();
                    }
                    if ui.button("Layout: Plot-focused").clicked() {
                        self.apply_layout(LayoutPreset::PlotFocused);
                        ui.close();
                    }
                });
                // Apps — scientific workflows (integrated applications).
                bar.menu_button("Apps", |ui| {
                    ui.set_min_width(170.0);
                    if ui.button("Launch apps…").clicked() {
                        self.apps_open = true;
                        ui.close();
                    }
                });
                // Help
                bar.menu_button("Help", |ui| {
                    if ui.button("About…").clicked() {
                        self.about_open = true;
                        ui.close();
                    }
                    if ui.button("Compatibility Status…").clicked() {
                        self.compat_open = true;
                        ui.close();
                    }
                });
            });
            ui.add_space(2.0);
            self.toolstrip(ui);
        });
    }

    fn apply_layout(&mut self, preset: LayoutPreset) {
        let (left, right, cmd) = match preset {
            LayoutPreset::Standard => (true, true, true),
            LayoutPreset::EditorFocused => (false, false, true),
            LayoutPreset::PlotFocused => (false, true, false),
        };
        self.prefs.layout.show_left = left;
        self.prefs.layout.show_right = right;
        self.prefs.layout.show_command_window = cmd;
        self.status_note("Layout applied.");
    }

    fn toolstrip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let mut next = self.toolstrip;
            for (i, label) in ["Home", "Plots", "Apps"].iter().enumerate() {
                let selected = self.toolstrip == i;
                let fill = if selected {
                    theme::tint(theme::accent(self.theme_kind), 0.45)
                } else {
                    egui::Color32::TRANSPARENT
                };
                let btn = egui::Button::new(*label).small().fill(fill);
                if ui.add(btn).clicked() {
                    next = i;
                }
            }
            self.toolstrip = next;
        });
        ui.separator();
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| match self.toolstrip {
            0 => self.toolstrip_home(ui),
            1 => self.toolstrip_plots(ui),
            2 => self.toolstrip_apps(ui),
            _ => {}
        });
        ui.add_space(3.0);
    }

    fn toolstrip_home(&mut self, ui: &mut egui::Ui) {
        if ui.button("New Script").clicked() {
            self.new_script();
        }
        if ui.button("Open…").clicked() {
            self.open_dialog = Some(String::new());
        }
        if ui.button("Save").clicked() {
            self.save_active();
        }
        if ui.button("Save As…").clicked() {
            self.begin_save_as();
        }
        ui.separator();
        if ui
            .add_enabled(!self.running, egui::Button::new("Run"))
            .on_hover_text("Run the active script (F5)")
            .clicked()
        {
            self.run_active_script();
        }
        if ui
            .add_enabled(!self.running, egui::Button::new("Run Selection"))
            .on_hover_text("Run the selected editor lines in the session")
            .clicked()
        {
            self.run_selection();
        }
        if ui
            .add_enabled(self.running, egui::Button::new("Stop"))
            .on_hover_text("Interrupt execution between statements")
            .clicked()
        {
            self.request_stop();
        }
        ui.separator();
        if ui.button("Clear Workspace").clicked() {
            self.clear_workspace();
        }
        if ui.button("Set Current Folder").clicked() {
            self.open_dialog = Some(self.folder.cwd.to_string_lossy().to_string());
        }
        ui.separator();
        ui.menu_button("Layout", |ui| {
            ui.set_min_width(140.0);
            for (label, preset) in [
                ("Standard", LayoutPreset::Standard),
                ("Editor-focused", LayoutPreset::EditorFocused),
                ("Plot-focused", LayoutPreset::PlotFocused),
            ] {
                if ui.button(label).clicked() {
                    self.apply_layout(preset);
                    ui.close();
                }
            }
        });
        if ui.button("Preferences").clicked() {
            self.prefs_open = true;
        }
    }

    fn toolstrip_plots(&mut self, ui: &mut egui::Ui) {
        ui.weak("Inserts code at the cursor:");
        for (label, snippet) in [
            ("Line Plot", LINE_PLOT_SNIPPET),
            (
                "Scatter Plot",
                "xs = [1 2 3 4 5];\nys = [2 4 1 3 5];\nscatter(xs, ys);\n",
            ),
            ("Bar Plot", "values = [3 5 2 6];\nbar(values);\n"),
            (
                "Histogram",
                "data = [1 2 2 3 3 3 4 4 5];\nhistogram(data, 5);\n",
            ),
        ] {
            if ui.button(label).clicked() {
                self.insert_snippet_into_active(snippet);
            }
        }
        ui.separator();
        if ui.button("Clear Figures").clicked() {
            self.run_source_now("close all;");
        }
        ui.weak("(plots render in the Figures panel →)");
    }

    fn toolstrip_apps(&mut self, ui: &mut egui::Ui) {
        if ui.button("Workspace Inspector").clicked() {
            let target = self
                .workspace
                .selected
                .clone()
                .or_else(|| self.workspace.rows.first().map(|r| r.name.clone()));
            match target {
                Some(name) => self.workspace.inspector = Some(name),
                None => self.status_note("Workspace is empty — nothing to inspect."),
            }
        }
        if ui.button("Variable Viewer").clicked() {
            self.workspace.inspector = self
                .workspace
                .selected
                .clone()
                .or_else(|| self.workspace.rows.first().map(|r| r.name.clone()));
        }
        ui.add_enabled(false, egui::Button::new("Numerical Tools"))
            .on_hover_text("Planned: statistics, formatting, more native functions");
        if ui.button("Compatibility Status").clicked() {
            self.compat_open = true;
        }
    }

    fn insert_snippet_into_active(&mut self, snippet: &str) {
        let Some(idx) = self.active else {
            self.status_note("Open a script first (New Script).");
            return;
        };
        self.docs[idx].editor.insert_snippet_at_cursor(snippet);
        self.docs[idx].modified = true;
    }

    fn left_panel(&mut self, ui: &mut egui::Ui, default_w: f32) {
        if !self.prefs.layout.show_left {
            return;
        }
        let folder_action: std::cell::RefCell<FolderAction> =
            std::cell::RefCell::new(FolderAction::None);
        let mut ws_clear = false;
        egui::Panel::left("matrust_left_v2")
            .resizable(true)
            .default_size(default_w)
            .min_size(180.0)
            .max_size(560.0)
            .show(ui, |ui| {
                let available = ui.available_height();
                // Vertical split: Current Folder on top, Workspace below.
                let folder_cap =
                    (available * self.left_split).clamp(90.0, (available - 110.0).max(100.0));

                // ---- Current Folder ----
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Current Folder").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        if ui
                            .add(egui::Button::new("+ New").small())
                            .on_hover_text("New script in this folder")
                            .clicked()
                        {
                            *folder_action.borrow_mut() = FolderAction::NewScript;
                        }
                        if ui
                            .add(egui::Button::new("Ref").small())
                            .on_hover_text("Refresh directory listing")
                            .clicked()
                        {
                            self.folder.refresh();
                        }
                        if ui
                            .add(egui::Button::new("Up").small())
                            .on_hover_text("Parent directory")
                            .clicked()
                        {
                            self.folder.go_up();
                        }
                    });
                });
                egui::ScrollArea::vertical()
                    .id_salt("folder_list")
                    .auto_shrink([false, false])
                    .max_height(folder_cap)
                    .show(ui, |ui| {
                        let body =
                            crate::current_folder::show(ui, &mut self.folder, self.theme_kind);
                        let prior: FolderAction = folder_action.borrow().clone();
                        let merged = prior.merge(body);
                        *folder_action.borrow_mut() = merged;
                    });

                self.drag_divider(ui, folder_cap, available);

                // ---- Workspace ----
                let ws_action =
                    crate::workspace_panel::show_header(ui, &mut self.workspace, self.theme_kind);
                if matches!(ws_action, WorkspaceAction::ClearWorkspace) {
                    ws_clear = true;
                }
                egui::ScrollArea::vertical()
                    .id_salt("workspace_list")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut inspector = crate::workspace_panel::show_body(
                            ui,
                            &mut self.workspace,
                            self.theme_kind,
                        );
                        if let Some(name) = inspector.take() {
                            self.workspace.inspector = Some(name);
                        }
                    });
                if let WorkspaceAction::ClearWorkspace = ws_action {
                    ws_clear = true;
                }
            });
        if ws_clear {
            self.clear_workspace();
        }
        self.handle_folder_action(folder_action.into_inner());
    }

    fn handle_folder_action(&mut self, action: FolderAction) {
        match action {
            FolderAction::None => {}
            FolderAction::Open { path } => {
                self.open_file_quiet(&path);
            }
            FolderAction::NewScript => self.new_script(),
            FolderAction::Delete { path } => {
                // Only reached when the confirm dialog succeeded earlier.
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        self.folder.refresh();
                        self.status_note(format!("Deleted {}", path.display()));
                    }
                    Err(err) => self.cmd.push(Line::Err(format!(
                        "Delete failed: {} ({})",
                        path.display(),
                        err
                    ))),
                }
            }
            FolderAction::CopyPath { path } => {
                self.cmd
                    .push(Line::Info(format!("Path copied: {}", path.display())));
            }
        }
    }

    fn drag_divider(&mut self, ui: &mut egui::Ui, used_before: f32, available: f32) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::drag());
        ui.painter().rect_filled(
            rect,
            1.0,
            theme::tint(theme::text_color_faint(self.theme_kind), 0.55),
        );
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let top = rect.min.y - used_before;
                let new = ((pos.y - top) / available.max(1.0)).clamp(0.12, 0.9);
                self.left_split = new;
            }
        }
        response.on_hover_cursor(egui::CursorIcon::ResizeVertical);
    }

    fn command_window_panel(&mut self, ui: &mut egui::Ui, default_h: f32) {
        if !self.prefs.layout.show_command_window {
            return;
        }
        egui::Panel::bottom("matrust_cmdwin_v2")
            .resizable(true)
            .default_size(default_h)
            .min_size(96.0)
            .show(ui, |ui| {
                // Header row BEFORE the scroll — safe for panel fit.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Command Window").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("Hide").small())
                            .on_hover_text(
                                "Hide the Command Window (Ctrl+` or Window ▸ Command Window restores it)",
                            )
                            .clicked()
                        {
                            self.prefs.layout.show_command_window = false;
                            self.status_note(
                                "Command Window hidden — Window ▸ Command Window restores it.",
                            );
                        }
                        if ui
                            .add(egui::Button::new("Clear").small())
                            .on_hover_text("Clear the transcript (clc)")
                            .clicked()
                        {
                            self.cmd.lines.clear();
                        }
                    });
                });
                ui.add_space(1.0);

                // Transcript + inline prompt as one scroll block. Nothing
                // may follow this call inside the panel (see module docs).
                let submitted =
                    crate::command_window::show_body(ui, &mut self.cmd, self.theme_kind);
                if let Some(submitted) = submitted {
                    match submitted {
                        Submitted::Command(source) => {
                            self.submit(
                                source.clone(),
                                Origin::CommandWindow { echo: source },
                                PendingKind::CommandWindow,
                            );
                        }
                        Submitted::Who => {
                            let names = self
                                .workspace
                                .rows
                                .iter()
                                .map(|r| r.name.clone())
                                .collect::<Vec<_>>()
                                .join("  ");
                            if names.is_empty() {
                                self.cmd
                                    .push(Line::Info("(workspace is empty)".to_string()));
                            } else {
                                self.cmd.push(Line::Out(names));
                            }
                        }
                        Submitted::ClearWorkspace => self.clear_workspace(),
                        Submitted::Exit => {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            });
    }

    fn status_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("matrust_status_v2").show(ui, |ui| {
            let status = self.build_status_line();
            statusbar::show(ui, &status, self.theme_kind);
        });
    }

    fn figures_panel(&mut self, ui: &mut egui::Ui, default_w: f32) {
        if !self.prefs.layout.show_right {
            return;
        }
        egui::Panel::right("matrust_figures_v2")
            .resizable(true)
            .default_size(default_w)
            .min_size(220.0)
            .show(ui, |ui| {
                match crate::figures::show_header(ui, &mut self.figures, self.theme_kind) {
                    crate::figures::HeaderAction::None => {}
                    crate::figures::HeaderAction::ClearAll => {
                        self.run_source_now("close all;");
                    }
                    crate::figures::HeaderAction::HidePanel => {
                        self.prefs.layout.show_right = false;
                    }
                }
                crate::figures::show_body(ui, &mut self.figures, self.theme_kind, &self.palette);
            });
    }

    /// The Figures panel is shown when the user opens it, or automatically
    /// when the session creates a new figure (see `apply_session_snapshot`).
    /// It is never a permanent reserved column: with no figures the space
    /// goes back to the editor.

    fn build_status_line(&self) -> StatusLine {
        StatusLine {
            run_state: if self.running {
                RunState::Running
            } else if self.last_error {
                RunState::Error
            } else {
                RunState::Ready
            },
            active_file: self.active.map(|i| self.docs[i].title.clone()),
            modified: self.active.map(|i| self.docs[i].modified).unwrap_or(false),
            working_dir: self.folder.cwd.to_string_lossy().to_string(),
            cursor_line: self
                .active
                .and_then(|i| self.docs.get(i))
                .map(|d| d.editor.cursor().line + 1)
                .unwrap_or(0),
            cursor_col: self
                .active
                .and_then(|i| self.docs.get(i))
                .map(|d| d.editor.cursor().col + 1)
                .unwrap_or(0),
            last_duration: self.last_duration,
            theme_name: match self.theme_kind {
                ThemeKind::Dark => "Dark",
                ThemeKind::Light => "Light",
            },
        }
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.editor_tab_bar(ui);
            if self.find_open {
                self.find_bar(ui);
            }
            let Some(idx) = self.active else {
                self.welcome(ui);
                return;
            };
            let doc = &mut self.docs[idx];
            let output = doc.editor.show(ui, self.theme_kind, &self.palette);
            if output.changed {
                doc.modified = true;
                doc.editor.error_line = None;
            }
        });
    }

    fn editor_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.horizontal(|ui| {
            for i in 0..self.docs.len() {
                let doc = &self.docs[i];
                let selected = self.active.map(|a| a == i).unwrap_or(false);
                let mut label = doc.title.clone();
                if doc.modified {
                    label.push_str(" *");
                }
                let fill = if selected {
                    theme::tint(theme::accent(self.theme_kind), 0.4)
                } else {
                    egui::Color32::TRANSPARENT
                };
                if ui
                    .add(egui::Button::new(label).small().fill(fill))
                    .clicked()
                {
                    self.active = Some(i);
                }
            }
            if ui.add(egui::Button::new("+ New").small()).clicked() {
                self.new_script();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.active.is_some() && ui.button("× Close").clicked() {
                    self.close_tab_pending = self.active;
                }
            });
        });
        ui.separator();
    }

    fn find_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Find:");
            let resp =
                ui.add(egui::TextEdit::singleline(&mut self.find_query).desired_width(190.0));
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.find_next(false);
            }
            if ui.button("← Prev").clicked() {
                self.find_next(true);
            }
            if ui.button("Next →").clicked() {
                self.find_next(false);
            }
            if ui.button("× Close").clicked() {
                self.find_open = false;
                if let Some(i) = self.active {
                    self.docs[i].editor.clear_search();
                }
            }
        });
        ui.separator();
    }

    fn find_next(&mut self, backwards: bool) {
        let Some(idx) = self.active else {
            return;
        };
        let query = self.find_query.clone();
        if query.trim().is_empty() {
            return;
        }
        self.docs[idx].editor.find_next(query.trim(), backwards);
    }

    fn welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading(APP_NAME);
            ui.weak(format!(
                "v{} — native numerical computing IDE (independent, Rust-based)",
                APP_VERSION
            ));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("New Script").clicked() {
                    self.new_script();
                }
                if ui.button("Open…").clicked() {
                    self.open_dialog = Some(String::new());
                }
                if ui.button("Load Example").clicked() {
                    self.docs.push(Doc {
                        path: None,
                        title: unique_title(&self.docs, "example"),
                        editor: CodeEditor::with_text(EXAMPLE_SOURCE, format!("doc_example{}", self.docs.len())),
                        modified: true,
                    });
                    self.active = Some(self.docs.len() - 1);
                }
            });
            ui.add_space(16.0);
            ui.weak("Implemented today: matrix literals & arithmetic (./ .* .^), comparisons, indexing + growth,");
            ui.weak("zeros/ones/eye/size/length/numel, floor/ceil/round/abs/sqrt/exp/log, sum/max/min, disp, pi,");
            ui.weak("plot/scatter/bar/histogram + figure/title/xlabel/ylabel/legend/grid/clf/close, `who`, `clear`.");
            ui.weak("Next language milestones: if/else, for/while loops, `end`, function files, strings as matrices.");
            ui.weak("The command window shares one persistent environment with the editor — variables persist.");
        });
    }

    // -------------------------------------------------------------- windows

    fn show_windows(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx();

        // Delete confirmation from the folder panel's context menu.
        if let Some(path) = crate::current_folder::show_delete_confirm(ctx, &mut self.folder) {
            self.handle_folder_action(FolderAction::Delete { path });
        }

        // Open file / set folder dialog.
        if self.open_dialog.is_some() {
            let mut open = true;
            let mut submitted: Option<String> = None;
            egui::Window::new("Open file / Set folder")
                .id(Id::unique("matrust_open_dialog"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Full path of a script (.m) or folder:");
                    ui.add_space(4.0);
                    if let Some(value) = self.open_dialog.as_mut() {
                        let resp = ui.add(egui::TextEdit::singleline(value).desired_width(340.0));
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            submitted = Some(value.trim().to_string());
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() {
                            submitted = self.open_dialog.clone();
                        }
                        if ui.button("Cancel").clicked() {
                            self.open_dialog = None;
                        }
                    });
                });
            if let Some(path) = submitted {
                if !path.is_empty() {
                    self.open_path(&path);
                }
            }
            if !open {
                self.open_dialog = None;
            }
        }

        // Save As dialog.
        if self.save_as_dialog.is_some() {
            let mut open = true;
            let mut confirm: Option<String> = None;
            egui::Window::new("Save As")
                .id(Id::unique("matrust_save_as"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Folder: {}", self.folder.cwd.display()));
                    ui.add_space(4.0);
                    if let Some(value) = self.save_as_dialog.as_mut() {
                        let resp = ui.add(egui::TextEdit::singleline(value).desired_width(340.0));
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            confirm = Some(value.trim().to_string());
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            confirm = self.save_as_dialog.clone();
                        }
                        if ui.button("Cancel").clicked() {
                            self.save_as_dialog = None;
                        }
                    });
                });
            if let Some(name) = confirm {
                if !name.is_empty() {
                    self.save_doc_as(&name);
                    self.save_as_dialog = None;
                }
            }
            if !open {
                self.save_as_dialog = None;
            }
        }

        // Preferences.
        if self.prefs_open {
            let mut open = true;
            egui::Window::new("Preferences")
                .id(Id::unique("matrust_prefs"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.strong("Theme");
                    ui.horizontal(|ui| {
                        if ui
                            .radio(self.theme_kind == ThemeKind::Dark, "Dark")
                            .clicked()
                        {
                            self.theme_kind = ThemeKind::Dark;
                        }
                        if ui
                            .radio(self.theme_kind == ThemeKind::Light, "Light")
                            .clicked()
                        {
                            self.theme_kind = ThemeKind::Light;
                        }
                    });
                    ui.add_space(6.0);
                    ui.strong("Font size");
                    let mut font = self.prefs.font_size;
                    if ui.add(egui::Slider::new(&mut font, 10.0..=20.0)).changed() {
                        self.prefs.font_size = font;
                    }
                    ui.add_space(6.0);
                    ui.strong("Layout preset");
                    ui.horizontal(|ui| {
                        for (label, preset) in [
                            ("Standard", LayoutPreset::Standard),
                            ("Editor", LayoutPreset::EditorFocused),
                            ("Plots", LayoutPreset::PlotFocused),
                        ] {
                            if ui.button(label).clicked() {
                                self.apply_layout(preset);
                            }
                        }
                    });
                    ui.add_space(6.0);
                    ui.weak("Preferences persist between restarts (layout, history, tabs).");
                });
            if !open {
                self.prefs_open = false;
            }
        }

        // About.
        if self.about_open {
            let mut open = true;
            egui::Window::new("About")
                .id(Id::unique("matrust_about"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading(APP_NAME);
                    ui.weak(format!("v{}", APP_VERSION));
                    ui.add_space(4.0);
                    ui.weak("An independent, Rust-based numerical computing IDE.");
                    ui.weak("Not affiliated with or endorsed by MathWorks.");
                    ui.weak("Workflow inspired by familiar numerical-desktop conventions.");
                });
            if !open {
                self.about_open = false;
            }
        }

        // Compatibility status.
        if self.compat_open {
            let mut open = true;
            egui::Window::new("Compatibility Status")
                .id(Id::unique("matrust_compat"))
                .open(&mut open)
                .default_width(420.0)
                .scroll([false, true])
                .show(ctx, |ui| {
                    ui.strong("Implemented");
                    ui.weak("• f64 matrices (row-major storage, 1-based column-major indexing)");
                    ui.weak("• literals, arithmetic (^* .^ ./ etc.), comparisons, transpose");
                    ui.weak("• indexing + automatic growth, assignment to index");
                    ui.weak("• zeros, ones, eye, size, length, numel, floor, ceil, round,");
                    ui.weak("  abs, sqrt, exp, log, sum, max, min, disp, pi");
                    ui.weak("• plots: plot, scatter, bar, histogram, figure, clf, close,");
                    ui.weak("  title, xlabel, ylabel, legend, grid (desktop rendering)");
                    ui.weak("• command syntax (grid on, close all), `who`, `clear`, `clc`");
                    ui.add_space(8.0);
                    ui.strong("Divergences (documented in docs/compatibility-matrix.md)");
                    ui.weak("• logical arrays are represented as 0/1 f64 doubles");
                    ui.weak("• char strings are not matrices; no string arithmetic");
                    ui.weak("• no N-D arrays, no user function files, no JIT");
                    ui.weak("• numeric-format formatting follows d/mostly style, not short/long exactly");
                    ui.weak("• round() uses Rust half-away-from-zero (matches MATLAB)");
                    ui.add_space(8.0);
                    ui.strong("Planned next");
                    ui.weak("• if/else, for/while, break/continue, `end`");
                    ui.weak("• function definitions & scopes; cell/struct soon after");
                    ui.weak("• colon ranges (a:x:b), string arrays, more toolbox functions");
                });
            if !open {
                self.compat_open = false;
            }
        }

        // Apps launcher — integrated scientific applications.
        if self.apps_open {
            let mut open = true;
            egui::Window::new("Apps — Scientific Workflows")
                .id(Id::unique("matrust_apps"))
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    ui.strong("Implemented applications");
                    ui.weak("Select an app to run its real backend workflow.");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.strong("Data & Analysis");
                        if ui.button("Launch Data Analysis").clicked() {
                            self.apps_selected = Some("data_analysis".to_string());
                            let src = "A = [1 2 3; 4 5 6]; save('/tmp/app_data.csv', A); data = load('/tmp/app_data.csv'); m = mean(data(:)); plot(1:3, data(:,1));".to_string();
                            self.submit(src, Origin::Script { path: None, name: "Data Analysis".to_string() }, PendingKind::Editor(0));
                            self.cmd.push(Line::Info("Data Analysis executed (CSV -> stats -> plot).".to_string()));
                            self.apps_open = false;
                        }
                    });
                    ui.weak("Imports CSV, computes stats, displays plot, saves/loads data.");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.strong("Simulation & Control");
                        if ui.button("Launch Simulation").clicked() {
                            self.apps_selected = Some("simulation".to_string());
                            let src = "t = 0:0.1:5; tau = 1.0; y = 1 - exp(-t / tau); plot(t, y); save('/tmp/app_sim.csv', y);".to_string();
                            self.submit(src, Origin::Script { path: None, name: "Simulation".to_string() }, PendingKind::Editor(0));
                            self.cmd.push(Line::Info("Engineering Simulation executed (first-order step response).".to_string()));
                            self.apps_open = false;
                        }
                    });
                    ui.weak("First-order step-response with time-axis plot and CSV export.");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.strong("Numerical Tools");
                        if ui.button("Launch Numerical Tools").clicked() {
                            self.apps_selected = Some("numerical".to_string());
                            let src = "A = [1 2; 3 4]; B = [5 6; 7 8]; C = A * B; disp(C);".to_string();
                            self.submit(src, Origin::Script { path: None, name: "Numerical Tools".to_string() }, PendingKind::Editor(0));
                            self.cmd.push(Line::Info("Numerical Tools executed (matrix operations).".to_string()));
                            self.apps_open = false;
                        }
                    });
                    ui.weak("Real matrix arithmetic, linear algebra, and inspection.");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.strong("Visualization");
                        if ui.button("Open Figures").clicked() {
                            self.apps_selected = Some("figures".to_string());
                            self.cmd.push(Line::Info("Figure Viewer: inspect current plots in Plots tab.".to_string()));
                            self.apps_open = false;
                        }
                    });
                    ui.weak("View plots generated by scripts, Data Analysis, Simulation, or Numerical Tools.");
                    ui.add_space(4.0);
                    ui.weak("Other applications (Signal Processing, Image Processing, ML, Robotics, Embedded) are planned; underlying algorithms not yet fully implemented.");
                });
            if !open {
                self.apps_open = false;
            }
        }

        // Confirm closing a modified tab.
        if let Some(idx) = self.close_tab_pending {
            if idx < self.docs.len() {
                let doc = &mut self.docs[idx];
                let title = doc.title.clone();
                let mut open = true;
                let mut action: Option<bool> = None;
                egui::Window::new("Unsaved changes")
                    .id(Id::unique("matrust_close_tab"))
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "'{}' has unsaved changes. Close without saving?",
                            title
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("Close without saving").clicked() {
                                action = Some(true);
                            }
                            if ui.button("Cancel").clicked() {
                                action = Some(false);
                            }
                        });
                    });
                match action {
                    Some(true) => {
                        let was_modified = self.docs[idx].modified;
                        self.close_tab_pending = None;
                        if was_modified {
                            // Force close bypassing the modified guard.
                            self.force_close_tab(idx);
                        } else {
                            self.close_tab(idx);
                        }
                    }
                    Some(false) | None => self.close_tab_pending = None,
                }
                if !open {
                    self.close_tab_pending = None;
                }
            } else {
                self.close_tab_pending = None;
            }
        }

        // Variable inspector.
        if let Some(inspect_name) = self.workspace.inspector.clone() {
            let value = self
                .env_snapshot
                .as_ref()
                .and_then(|env| env.get(&inspect_name));
            crate::workspace_panel::show_inspector(ui, &inspect_name, value, self.theme_kind);
        }
    }

    fn force_close_tab(&mut self, idx: usize) {
        self.docs[idx].modified = false;
        self.close_tab(idx);
    }
}

// -------------------------------------------------------------- helpers

fn palette_for(kind: ThemeKind) -> theme::SyntaxPalette {
    match kind {
        ThemeKind::Dark => theme::SyntaxPalette::dark(),
        ThemeKind::Light => theme::SyntaxPalette::light(),
    }
}

fn file_title(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn resolve(cwd: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn diag_text(diag: &Diagnostic) -> String {
    let mut out = String::new();
    if let Some(span) = diag.span {
        out.push_str(&format!("at line {}, column {}: ", span.line, span.column));
    }
    out.push_str(&diag.message);
    if let Some(hint) = &diag.hint {
        out.push_str(&format!("\nhint: {}", hint));
    }
    out
}

fn unique_title(docs: &[Doc], base: &str) -> String {
    let mut n = 1;
    loop {
        let title = if n == 1 {
            format!("{}.m", base)
        } else {
            format!("{}_{}.m", base, n)
        };
        if !docs.iter().any(|d| d.title == title) {
            return title;
        }
        n += 1;
    }
}

// ---------------------------------------------------------------- eframe integration

impl eframe::App for MatRustApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.root(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_prefs_state();
        let _ = &self.prefs.command_history;
        self.prefs.save(storage);
    }
}
