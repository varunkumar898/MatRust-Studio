//! MatRust Studio application: window layout, toolstrip, panels, dialogs,
//! and the bridge between the GUI and the interpreter worker.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::egui;
use egui::Id;

use matlab_diagnostics::Diagnostic;
use matlab_runtime::{Env, RunResult};
use matlabr_apps::curve_fitter::FitResult;
use matlabr_apps::data_analysis::{ColumnKind, ColumnStats, CsvTable, PlotKind, PlotRequest};
use matlabr_apps::simulation::{InputSignal, SimulationParams, SimulationResult, SystemModel};

use matlabr_apps::{Status, REGISTRY};


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
    apps: bool,
}

/// Which Data Analysis worker request is currently outstanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataPending {
    Preview,
    Run,
    Export,
}

/// UI state for the Data Analysis app. Kept alive when the window is closed so
/// switching apps never discards the loaded table or computed results.
struct DataApp {
    path: String,
    table: Option<CsvTable>,
    selected: Vec<usize>,
    stats: Vec<ColumnStats>,
    correlation: Option<f64>,
    plot_kind: PlotKind,
    x_col: usize,
    y_col: usize,
    export_path: String,
    export_message: Option<(bool, String)>,
    error: Option<String>,
    picker_open: bool,
    picker_dir: PathBuf,
    confirm_overwrite: bool,
}

impl DataApp {
    fn new(cwd: &Path) -> Self {
        Self {
            path: String::new(),
            table: None,
            selected: Vec::new(),
            stats: Vec::new(),
            correlation: None,
            plot_kind: PlotKind::Column,
            x_col: 0,
            y_col: 0,
            export_path: cwd
                .join("data_analysis_stats.csv")
                .to_string_lossy()
                .to_string(),
            export_message: None,
            error: None,
            picker_open: false,
            picker_dir: cwd.to_path_buf(),
            confirm_overwrite: false,
        }
    }

    fn configure_table(&mut self, table: CsvTable) {
        let numeric: Vec<usize> = table
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == ColumnKind::Numeric)
            .map(|(i, _)| i)
            .collect();
        self.selected = numeric.clone();
        self.x_col = numeric.first().copied().unwrap_or(0);
        self.y_col = numeric.get(1).copied().unwrap_or(self.x_col);
        self.stats.clear();
        self.correlation = None;
        self.error = None;
        self.confirm_overwrite = false;
        self.export_message = None;
        let stem = table
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "data_analysis".to_string());
        self.export_path = self
            .picker_dir
            .join(format!("{stem}_stats.csv"))
            .to_string_lossy()
            .to_string();
        self.table = Some(table);
    }
}

/// Which Curve Fitter worker request is currently outstanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurvePending {
    Preview,
    Fit,
    Export,
}

/// UI state for the Curve Fitter app. Kept alive when the window is closed so
/// switching apps never discards the fitted results.
struct CurveFitterApp {
    path: String,
    table: Option<CsvTable>,
    x_col: usize,
    y_col: usize,
    degree: usize,
    fit: Option<FitResult>,
    /// x values of the fitted rows, for the fitted-values export.
    xs: Vec<f64>,
    export_path: String,
    fitted_export_path: String,
    export_message: Option<(bool, String)>,
    error: Option<String>,
    picker_open: bool,
    picker_dir: PathBuf,
    confirm_overwrite: bool,
}

impl CurveFitterApp {
    fn new(cwd: &Path) -> Self {
        Self {
            path: String::new(),
            table: None,
            x_col: 0,
            y_col: 1,
            degree: 1,
            fit: None,
            xs: Vec::new(),
            export_path: cwd
                .join("curve_fit_coefficients.csv")
                .to_string_lossy()
                .to_string(),
            fitted_export_path: cwd
                .join("curve_fit_fitted.csv")
                .to_string_lossy()
                .to_string(),
            export_message: None,
            error: None,
            picker_open: false,
            picker_dir: cwd.to_path_buf(),
            confirm_overwrite: false,
        }
    }

    fn configure_table(&mut self, table: CsvTable) {
        let numeric: Vec<usize> = table
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == ColumnKind::Numeric)
            .map(|(i, _)| i)
            .collect();
        self.x_col = numeric.first().copied().unwrap_or(0);
        self.y_col = numeric.get(1).copied().unwrap_or(self.x_col);
        self.fit = None;
        self.xs.clear();
        self.error = None;
        self.confirm_overwrite = false;
        self.export_message = None;
        let stem = table
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "curve_fit".to_string());
        self.export_path = self
            .picker_dir
            .join(format!("{stem}_coefficients.csv"))
            .to_string_lossy()
            .to_string();
        self.fitted_export_path = self
            .picker_dir
            .join(format!("{stem}_fitted.csv"))
            .to_string_lossy()
            .to_string();
        self.table = Some(table);
    }
}

fn parse_matrix_text(text: &str, name: &str) -> Result<matlab_values::DenseMatrix, Diagnostic> {
    let mut rows_data: Vec<Vec<f64>> = Vec::new();
    for row_str in text.split(';') {
        let trimmed = row_str.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut row = Vec::new();
        for token in trimmed.split(|c: char| c.is_whitespace() || c == ',') {
            let t = token.trim();
            if t.is_empty() {
                continue;
            }
            let val: f64 = t.parse().map_err(|_| {
                Diagnostic::error(
                    format!("Invalid number '{t}' in matrix {name}."),
                    None,
                )
            })?;
            row.push(val);
        }
        if !row.is_empty() {
            rows_data.push(row);
        }
    }
    if rows_data.is_empty() {
        return Err(Diagnostic::error(
            format!("Matrix {name} cannot be empty."),
            None,
        ));
    }
    let cols = rows_data[0].len();
    for row in &rows_data {
        if row.len() != cols {
            return Err(Diagnostic::error(
                format!("Matrix {name} has inconsistent row lengths."),
                None,
            ));
        }
    }
    let rows = rows_data.len();
    let mut data = Vec::with_capacity(rows * cols);
    for row in rows_data {
        data.extend(row);
    }
    Ok(matlab_values::DenseMatrix { rows, cols, data })
}

fn parse_vector_text(text: &str, name: &str) -> Result<Vec<f64>, Diagnostic> {
    let mut vec = Vec::new();
    for token in text.split(|c: char| c.is_whitespace() || c == ';' || c == ',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let val: f64 = trimmed.parse().map_err(|_| {
            Diagnostic::error(
                format!("Invalid number '{trimmed}' in vector {name}."),
                None,
            )
        })?;
        vec.push(val);
    }
    if vec.is_empty() {
        return Err(Diagnostic::error(
            format!("Vector {name} cannot be empty."),
            None,
        ));
    }
    Ok(vec)
}

/// Which Simulation worker request is currently outstanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimulationPending {
    Run,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimModelKind {
    FirstOrder,
    SecondOrder,
    StateSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SimSignalKind {
    Step,
    Impulse,
    Ramp,
    Sine,
    Zero,
}

/// UI state for the Simulation and Control app. Kept alive when the window is closed.
struct SimulationApp {
    model_kind: SimModelKind,
    // First-order
    tau: f64,
    first_order_gain: f64,
    first_order_y0: f64,
    // Second-order
    omega_n: f64,
    zeta: f64,
    second_order_gain: f64,
    second_order_y0: f64,
    second_order_v0: f64,
    // State-Space (text matrix entry)
    ss_a: String,
    ss_b: String,
    ss_c: String,
    ss_d: f64,
    ss_x0: String,
    // Input signal
    signal_kind: SimSignalKind,
    step_amplitude: f64,
    step_t0: f64,
    impulse_magnitude: f64,
    ramp_slope: f64,
    sine_amplitude: f64,
    sine_frequency: f64,
    sine_phase_deg: f64,
    // Simulation parameters
    t_final: f64,
    dt: f64,
    // Preset index
    preset_idx: usize,
    // Results
    result: Option<SimulationResult>,
    export_path: String,
    export_message: Option<(bool, String)>,
    confirm_overwrite: bool,
    error: Option<String>,
}

impl SimulationApp {
    fn new(cwd: &Path) -> Self {
        Self {
            model_kind: SimModelKind::SecondOrder,
            tau: 1.0,
            first_order_gain: 1.0,
            first_order_y0: 0.0,
            omega_n: 5.0,
            zeta: 0.3,
            second_order_gain: 1.0,
            second_order_y0: 0.0,
            second_order_v0: 0.0,
            ss_a: "0 1; -25 -3".to_string(),
            ss_b: "0; 25".to_string(),
            ss_c: "1 0".to_string(),
            ss_d: 0.0,
            ss_x0: "0 0".to_string(),
            signal_kind: SimSignalKind::Step,
            step_amplitude: 1.0,
            step_t0: 0.0,
            impulse_magnitude: 1.0,
            ramp_slope: 1.0,
            sine_amplitude: 1.0,
            sine_frequency: 1.0,
            sine_phase_deg: 0.0,
            t_final: 5.0,
            dt: 0.01,
            preset_idx: 1, // Underdamped 2nd-Order
            result: None,
            export_path: cwd
                .join("simulation_response.csv")
                .to_string_lossy()
                .to_string(),
            export_message: None,
            confirm_overwrite: false,
            error: None,
        }
    }

    fn apply_preset(&mut self, idx: usize) {
        self.preset_idx = idx;
        match idx {
            0 => {
                self.model_kind = SimModelKind::FirstOrder;
                self.tau = 1.0;
                self.first_order_gain = 1.0;
                self.first_order_y0 = 0.0;
                self.signal_kind = SimSignalKind::Step;
                self.step_amplitude = 1.0;
                self.step_t0 = 0.0;
                self.t_final = 5.0;
                self.dt = 0.01;
            }
            1 => {
                self.model_kind = SimModelKind::SecondOrder;
                self.omega_n = 5.0;
                self.zeta = 0.3;
                self.second_order_gain = 1.0;
                self.second_order_y0 = 0.0;
                self.second_order_v0 = 0.0;
                self.signal_kind = SimSignalKind::Step;
                self.step_amplitude = 1.0;
                self.step_t0 = 0.0;
                self.t_final = 4.0;
                self.dt = 0.005;
            }
            2 => {
                self.model_kind = SimModelKind::SecondOrder;
                self.omega_n = 4.0;
                self.zeta = 1.0;
                self.second_order_gain = 1.0;
                self.second_order_y0 = 0.0;
                self.second_order_v0 = 0.0;
                self.signal_kind = SimSignalKind::Step;
                self.step_amplitude = 1.0;
                self.step_t0 = 0.0;
                self.t_final = 5.0;
                self.dt = 0.01;
            }
            3 => {
                self.model_kind = SimModelKind::SecondOrder;
                self.omega_n = 3.0;
                self.zeta = 2.0;
                self.second_order_gain = 1.0;
                self.second_order_y0 = 0.0;
                self.second_order_v0 = 0.0;
                self.signal_kind = SimSignalKind::Step;
                self.step_amplitude = 1.0;
                self.step_t0 = 0.0;
                self.t_final = 6.0;
                self.dt = 0.01;
            }
            4 => {
                self.model_kind = SimModelKind::SecondOrder;
                self.omega_n = 2.0 * std::f64::consts::PI;
                self.zeta = 0.0;
                self.second_order_gain = 1.0;
                self.second_order_y0 = 1.0;
                self.second_order_v0 = 0.0;
                self.signal_kind = SimSignalKind::Zero;
                self.t_final = 3.0;
                self.dt = 0.005;
            }
            5 => {
                self.model_kind = SimModelKind::StateSpace;
                self.ss_a = "0 1; -9 -4".to_string();
                self.ss_b = "0; 9".to_string();
                self.ss_c = "1 0".to_string();
                self.ss_d = 0.0;
                self.ss_x0 = "0 0".to_string();
                self.signal_kind = SimSignalKind::Step;
                self.step_amplitude = 1.0;
                self.step_t0 = 0.0;
                self.t_final = 5.0;
                self.dt = 0.01;
            }
            _ => {}
        }
    }

    fn build_model(&self) -> Result<SystemModel, Diagnostic> {
        match self.model_kind {
            SimModelKind::FirstOrder => Ok(SystemModel::FirstOrder {
                tau: self.tau,
                gain: self.first_order_gain,
                y0: self.first_order_y0,
            }),
            SimModelKind::SecondOrder => Ok(SystemModel::SecondOrder {
                omega_n: self.omega_n,
                zeta: self.zeta,
                gain: self.second_order_gain,
                y0: self.second_order_y0,
                v0: self.second_order_v0,
            }),
            SimModelKind::StateSpace => {
                let a = parse_matrix_text(&self.ss_a, "A")?;
                let b = parse_matrix_text(&self.ss_b, "B")?;
                let c = parse_matrix_text(&self.ss_c, "C")?;
                let x0 = parse_vector_text(&self.ss_x0, "x0")?;
                Ok(SystemModel::StateSpace {
                    a,
                    b,
                    c,
                    d: self.ss_d,
                    x0,
                })
            }
        }
    }

    fn build_input(&self) -> InputSignal {
        match self.signal_kind {
            SimSignalKind::Step => InputSignal::Step {
                amplitude: self.step_amplitude,
                t_start: self.step_t0,
            },
            SimSignalKind::Impulse => InputSignal::Impulse {
                magnitude: self.impulse_magnitude,
                t_start: self.step_t0,
            },
            SimSignalKind::Ramp => InputSignal::Ramp {
                slope: self.ramp_slope,
                t_start: self.step_t0,
            },
            SimSignalKind::Sine => InputSignal::Sine {
                amplitude: self.sine_amplitude,
                frequency: self.sine_frequency,
                phase_rad: self.sine_phase_deg.to_radians(),
            },
            SimSignalKind::Zero => InputSignal::Zero,
        }
    }

    fn build_params(&self) -> SimulationParams {
        SimulationParams {
            t_final: self.t_final,
            dt: self.dt,
        }
    }
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
    /// Scripted Data Analysis smoke hook: file to open on startup
    /// (`MATRUST_OPEN_APP=data_analysis`, `MATRUST_APP_CSV=...`).
    auto_app_csv: Option<PathBuf>,
    auto_app_export: Option<PathBuf>,
    auto_app_run: bool,
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
    data_app: Option<DataApp>,
    data_app_open: bool,
    data_pending: Option<(u64, DataPending)>,
    curve_app: Option<CurveFitterApp>,
    curve_app_open: bool,
    curve_pending: Option<(u64, CurvePending)>,
    sim_app: Option<SimulationApp>,
    sim_app_open: bool,
    sim_pending: Option<(u64, SimulationPending)>,
    /// Scripted Curve Fitter smoke hook: fit through the worker after the
    /// CSV preview arrives (`MATRUST_OPEN_APP=curve_fitter`).
    auto_curve_run: bool,
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
            auto_app_csv: std::env::var("MATRUST_APP_CSV")
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from),
            auto_app_export: std::env::var("MATRUST_APP_EXPORT")
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from),
            auto_app_run: false,
            figures_seen: 0,
            find_open: false,
            find_query: String::new(),
            open_dialog: None,
            save_as_dialog: None,
            prefs_open: false,
            about_open: false,
            compat_open: false,
            apps_open: false,
            data_app: None,
            data_app_open: false,
            data_pending: None,
            curve_app: None,
            curve_app_open: false,
            curve_pending: None,
            sim_app: None,
            sim_app_open: false,
            sim_pending: None,
            auto_curve_run: false,
            close_tab_pending: None,
            prefs,
        };

        // Scripted Data Analysis smoke hook (used for live-GUI verification in
        // a non-interactive harness). The normal app always starts closed.
        match std::env::var("MATRUST_OPEN_APP").ok().as_deref() {
            Some("data_analysis") => {
                let mut data = DataApp::new(&cwd);
                if let Some(path) = &app.auto_app_csv {
                    data.path = path.to_string_lossy().to_string();
                }
                app.data_app = Some(data);
                app.data_app_open = true;
            }
            Some("curve_fitter") => {
                let mut curve = CurveFitterApp::new(&cwd);
                if let Some(path) = &app.auto_app_csv {
                    curve.path = path.to_string_lossy().to_string();
                }
                app.curve_app = Some(curve);
                app.curve_app_open = true;
            }
            Some("simulation") => {
                let sim = SimulationApp::new(&cwd);
                app.sim_app = Some(sim);
                app.sim_app_open = true;
            }
            _ => {}
        }


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
                apps: ctrl && i.modifiers.shift && i.key_pressed(egui::Key::A),
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
        if keys.apps {
            self.apps_open = true;
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
        // Scripted app smoke path: open -> preview -> compute -> export, all
        // through the same worker and backend calls the UI uses. The target
        // app is whichever smoke window is open.
        if let Some(path) = self.auto_app_csv.take() {
            if self.curve_app_open {
                self.auto_curve_run = true;
                self.curve_submit_preview(path);
            } else {
                self.auto_app_run = true;
                self.data_submit_preview(path);
            }
        }
        if self.auto_app_run && !self.running {
            if let Some(app) = self.data_app.as_ref() {
                if let Some(table) = &app.table {
                    let selected = app.selected.clone();
                    let plot = PlotRequest {
                        kind: app.plot_kind,
                        x: if app.plot_kind == PlotKind::Scatter {
                            app.x_col
                        } else {
                            app.y_col
                        },
                        y: app.y_col,
                    };
                    let path = table.path.clone();
                    self.auto_app_run = false;
                    self.data_submit_run(path, selected, plot);
                }
            }
        }
        if self.auto_curve_run && !self.running {
            if let Some(app) = self.curve_app.as_ref() {
                if let Some(table) = &app.table {
                    let path = table.path.clone();
                    let x_col = app.x_col;
                    let y_col = app.y_col;
                    let degree = app.degree;
                    self.auto_curve_run = false;
                    self.curve_submit_fit(path, x_col, y_col, degree);
                }
            }
        }
        if let Some(export) = self.auto_app_export.clone() {
            if !self.running {
                if self.curve_app_open {
                    if let Some(app) = self.curve_app.as_ref() {
                        if let Some(fit) = &app.fit {
                            let fitted = Self::derive_fitted_path(&export);
                            let xs = app.xs.clone();
                            let fit = fit.clone();
                            self.auto_app_export = None;
                            self.curve_submit_export(export, fitted, fit, xs, true);
                        }
                    }
                } else if let Some(app) = self.data_app.as_ref() {
                    if !app.stats.is_empty() {
                        let corr = app.correlation.and_then(|r| {
                            let table = app.table.as_ref()?;
                            let x = table.columns.get(app.x_col)?.name.clone();
                            let y = table.columns.get(app.y_col)?.name.clone();
                            Some((x, y, r))
                        });
                        let stats = app.stats.clone();
                        self.auto_app_export = None;
                        self.data_submit_export(export, stats, corr, true);
                    }
                }
            }
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
                Reply::DataAnalysisPreview { id, result } => {
                    if !matches!(self.data_pending, Some((pid, DataPending::Preview)) if pid == id)
                    {
                        continue;
                    }
                    self.data_pending = None;
                    self.running = false;
                    let mut info = None;
                    if let Some(app) = self.data_app.as_mut() {
                        match result {
                            Ok(table) => {
                                app.configure_table(table);
                                info = Some("CSV parsed for preview.".to_string());
                            }
                            Err(diag) => {
                                app.table = None;
                                app.error = Some(diag_text(&diag));
                            }
                        }
                    }
                    if let Some(info) = info {
                        self.cmd.push(Line::Info(info));
                    }
                }
                Reply::DataAnalysisRun {
                    id,
                    result,
                    snapshot,
                } => {
                    if !matches!(self.data_pending, Some((pid, DataPending::Run)) if pid == id) {
                        continue;
                    }
                    self.data_pending = None;
                    self.running = false;
                    self.apply_session_snapshot(&snapshot);
                    if let Some(app) = self.data_app.as_mut() {
                        match result {
                            Ok(outcome) => {
                                app.table = Some(outcome.table);
                                app.stats = outcome.stats;
                                app.correlation = outcome.correlation;
                                app.error = None;
                                app.confirm_overwrite = false;
                                app.export_message = None;
                            }
                            Err(diag) => app.error = Some(diag_text(&diag)),
                        }
                    }
                }
                Reply::DataAnalysisExport { id, result } => {
                    if !matches!(self.data_pending, Some((pid, DataPending::Export)) if pid == id) {
                        continue;
                    }
                    self.data_pending = None;
                    self.running = false;
                    if let Some(app) = self.data_app.as_mut() {
                        match result {
                            Ok(path) => {
                                app.confirm_overwrite = false;
                                app.export_message =
                                    Some((true, format!("Exported {}", path.display())));
                            }
                            Err(diag) => {
                                app.export_message = Some((false, diag_text(&diag)));
                            }
                        }
                    }
                }
                Reply::CurveFitDone {
                    id,
                    result,
                    snapshot,
                } => {
                    if !matches!(self.curve_pending, Some((pid, CurvePending::Fit)) if pid == id) {
                        continue;
                    }
                    self.curve_pending = None;
                    self.running = false;
                    self.apply_session_snapshot(&snapshot);
                    if let Some(app) = self.curve_app.as_mut() {
                        match result {
                            Ok((fit, xs, _ys, table)) => {
                                app.table = Some(table);
                                app.xs = xs;
                                app.fit = Some(fit);
                                app.error = None;
                                app.confirm_overwrite = false;
                                app.export_message = None;
                            }
                            Err(diag) => app.error = Some(diag_text(&diag)),
                        }
                    }
                }
                Reply::CurvePreview { id, result } => {
                    if !matches!(self.curve_pending, Some((pid, CurvePending::Preview)) if pid == id)
                    {
                        continue;
                    }
                    self.curve_pending = None;
                    self.running = false;
                    let mut info = None;
                    if let Some(app) = self.curve_app.as_mut() {
                        match result {
                            Ok(table) => {
                                app.configure_table(table);
                                info = Some("CSV parsed for Curve Fitter.".to_string());
                            }
                            Err(diag) => {
                                app.table = None;
                                app.error = Some(diag_text(&diag));
                            }
                        }
                    }
                    if let Some(info) = info {
                        self.cmd.push(Line::Info(info));
                    }
                }
                Reply::CurveFitExport { id, result } => {
                    if !matches!(self.curve_pending, Some((pid, CurvePending::Export)) if pid == id)
                    {
                        continue;
                    }
                    self.curve_pending = None;
                    self.running = false;
                    if let Some(app) = self.curve_app.as_mut() {
                        match result {
                            Ok((coeff, fitted)) => {
                                app.confirm_overwrite = false;
                                app.export_message = Some((
                                    true,
                                    format!(
                                        "Exported {} and {}",
                                        coeff.display(),
                                        fitted.display()
                                    ),
                                ));
                            }
                            Err(diag) => {
                                app.export_message = Some((false, diag_text(&diag)));
                            }
                        }
                    }
                }
                Reply::SimulationDone {
                    id,
                    result,
                    snapshot,
                } => {
                    if !matches!(self.sim_pending, Some((pid, SimulationPending::Run)) if pid == id) {
                        continue;
                    }
                    self.sim_pending = None;
                    self.running = false;
                    self.apply_session_snapshot(&snapshot);
                    if let Some(app) = self.sim_app.as_mut() {
                        match result {
                            Ok(res) => {
                                app.result = Some(res);
                                app.error = None;
                                app.confirm_overwrite = false;
                                app.export_message = None;
                            }
                            Err(diag) => app.error = Some(diag_text(&diag)),
                        }
                    }
                }
                Reply::SimulationExport { id, result } => {
                    if !matches!(self.sim_pending, Some((pid, SimulationPending::Export)) if pid == id) {
                        continue;
                    }
                    self.sim_pending = None;
                    self.running = false;
                    if let Some(app) = self.sim_app.as_mut() {
                        match result {
                            Ok(path) => {
                                app.confirm_overwrite = false;
                                app.export_message =
                                    Some((true, format!("Exported {}", path.display())));
                            }
                            Err(diag) => {
                                app.export_message = Some((false, diag_text(&diag)));
                            }
                        }
                    }
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

    fn open_data_analysis(&mut self) {
        if self.data_app.is_none() {
            self.data_app = Some(DataApp::new(&self.folder.cwd));
        }
        self.data_app_open = true;
        self.apps_open = false;
    }

    fn open_curve_fitter(&mut self) {
        if self.curve_app.is_none() {
            self.curve_app = Some(CurveFitterApp::new(&self.folder.cwd));
        }
        self.curve_app_open = true;
        self.apps_open = false;
    }

    fn open_simulation(&mut self) {
        if self.sim_app.is_none() {
            self.sim_app = Some(SimulationApp::new(&self.folder.cwd));
        }
        self.sim_app_open = true;
        self.apps_open = false;
    }


    /// The fitted-values export path implied by a coefficients export path
    /// (`<stem>_fitted.csv` next to `<stem>.csv`).
    fn derive_fitted_path(coeff_path: &Path) -> PathBuf {
        let fitted = coeff_path
            .file_stem()
            .map(|stem| {
                let mut name = stem.to_string_lossy().to_string();
                name.push_str("_fitted");
                let ext = coeff_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_else(|| "csv".to_string());
                coeff_path.with_file_name(format!("{name}.{ext}"))
            })
            .unwrap_or_else(|| coeff_path.with_file_name("curve_fit_fitted.csv"));
        fitted
    }

    fn data_submit_preview(&mut self, path: PathBuf) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.data_pending = Some((id, DataPending::Preview));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.data_app.as_mut() {
            app.error = None;
            app.stats.clear();
            app.correlation = None;
            app.export_message = None;
        }
        let _ = self
            .worker
            .request_tx
            .send(Request::DataAnalysisPreview { id, path });
    }

    fn data_submit_run(&mut self, path: PathBuf, selected: Vec<usize>, plot: PlotRequest) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.data_pending = Some((id, DataPending::Run));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.data_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::DataAnalysisRun {
            id,
            path,
            selected,
            plot,
        });
    }

    fn data_submit_export(
        &mut self,
        path: PathBuf,
        stats: Vec<ColumnStats>,
        correlation: Option<(String, String, f64)>,
        overwrite: bool,
    ) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.data_pending = Some((id, DataPending::Export));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.data_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::DataAnalysisExport {
            id,
            path,
            stats,
            correlation,
            overwrite,
        });
    }

    fn curve_submit_preview(&mut self, path: PathBuf) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.curve_pending = Some((id, CurvePending::Preview));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.curve_app.as_mut() {
            app.error = None;
            app.fit = None;
            app.xs.clear();
            app.export_message = None;
        }
        let _ = self
            .worker
            .request_tx
            .send(Request::CurvePreview { id, path });
    }

    fn curve_submit_fit(&mut self, path: PathBuf, x_col: usize, y_col: usize, degree: usize) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.curve_pending = Some((id, CurvePending::Fit));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.curve_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::CurveFit {
            id,
            path,
            x_col,
            y_col,
            degree,
        });
    }

    fn curve_submit_export(
        &mut self,
        coeff_path: PathBuf,
        fitted_path: PathBuf,
        fit: FitResult,
        xs: Vec<f64>,
        overwrite: bool,
    ) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.curve_pending = Some((id, CurvePending::Export));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.curve_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::CurveFitExport {
            id,
            coeff_path,
            fitted_path,
            fit,
            xs,
            overwrite,
        });
    }

    fn sim_submit_run(
        &mut self,
        model: SystemModel,
        input: InputSignal,
        params: SimulationParams,
    ) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.sim_pending = Some((id, SimulationPending::Run));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.sim_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::SimulationRun {
            id,
            model,
            input,
            params,
        });
    }

    fn sim_submit_export(
        &mut self,
        path: PathBuf,
        result: SimulationResult,
        overwrite: bool,
    ) {
        if self.running {
            return;
        }
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.sim_pending = Some((id, SimulationPending::Export));
        self.running = true;
        self.last_error = false;
        if let Some(app) = self.sim_app.as_mut() {
            app.error = None;
            app.export_message = None;
        }
        let _ = self.worker.request_tx.send(Request::SimulationExport {
            id,
            path,
            result,
            overwrite,
        });
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
                    if ui.button("Launch apps…  (Ctrl+Shift+A)").clicked() {
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

        // Apps launcher — registry-driven, truthful status, disabled when not Ready.
        if self.apps_open {
            let mut open = true;
            egui::Window::new("Apps — Scientific Workflows")
                .id(Id::unique("matrust_apps"))
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.strong("Application registry");
                    ui.weak(
                        "Status and verification come from matlabr_apps::REGISTRY. Only Ready apps can launch.",
                    );
                    ui.add_space(6.0);
                    for app in REGISTRY {
                        let color = match app.status {
                            Status::Ready => egui::Color32::from_rgb(46, 160, 67),
                            Status::Partial => egui::Color32::from_rgb(210, 153, 34),
                            Status::Planned => egui::Color32::GRAY,
                            Status::Blocked => egui::Color32::from_rgb(215, 58, 73),
                        };
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(app.status_text())
                                    .strong()
                                    .color(color),
                            );
                            ui.strong(app.name);
                            ui.weak(app.verification_text());
                        });
                        ui.weak(app.description);
                        ui.horizontal(|ui| {
                            if app.launchable() {
                                if ui.button("Launch").clicked() {
                                    if app.id == "data_analysis" {
                                        self.open_data_analysis();
                                    }
                                    if app.id == "curve_fitter" {
                                        self.open_curve_fitter();
                                    }
                                    if app.id == "simulation" {
                                        self.open_simulation();
                                    }
                                }

                            } else {
                                ui.add_enabled(false, egui::Button::new("Unavailable"));
                                ui.weak("not yet implemented");
                            }
                        });
                        ui.separator();
                    }
                    ui.weak(
                        "Planned/Blocked apps are disabled and never run placeholder computations.",
                    );
                });
            if !open {
                self.apps_open = false;
            }
        }

        // Data Analysis app window.
        if self.data_app_open {
            let mut open = true;
            let mut preview_path: Option<PathBuf> = None;
            let mut run_request: Option<(PathBuf, Vec<usize>, PlotRequest)> = None;
            let mut export_request: Option<(
                PathBuf,
                Vec<ColumnStats>,
                Option<(String, String, f64)>,
                bool,
            )> = None;
            let busy_here = self.running && self.data_pending.is_some();
            let cancel_flag = std::sync::Arc::clone(&self.worker.cancel);

            if let Some(app) = self.data_app.as_mut() {
                egui::Window::new("Data Analysis — Ready · Unit-tested · Not reference-verified")
                    .id(Id::unique("matrust_data_analysis"))
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(true)
                    .default_width(920.0)
                    .default_height(680.0)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("CSV file:");
                            ui.add(
                                egui::TextEdit::singleline(&mut app.path)
                                    .desired_width(420.0)
                                    .hint_text("path to a .csv file"),
                            );
                            if ui.button("Browse…").clicked() {
                                app.picker_open = !app.picker_open;
                            }
                            let can_open = !app.path.trim().is_empty() && !busy_here;
                            if ui
                                .add_enabled(can_open, egui::Button::new("Open"))
                                .clicked()
                            {
                                preview_path = Some(PathBuf::from(app.path.trim()));
                            }
                            if busy_here {
                                ui.spinner();
                                ui.weak("worker running…");
                                if ui.button("Stop").clicked() {
                                    cancel_flag
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        });

                        if app.picker_open {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Up").clicked() {
                                        if let Some(parent) = app.picker_dir.parent() {
                                            app.picker_dir = parent.to_path_buf();
                                        }
                                    }
                                    ui.weak(app.picker_dir.display().to_string());
                                });
                                let mut entries: Vec<(PathBuf, bool)> = Vec::new();
                                if let Ok(read_dir) = std::fs::read_dir(&app.picker_dir) {
                                    for entry in read_dir.flatten() {
                                        let path = entry.path();
                                        let is_dir = path.is_dir();
                                        let is_csv = path
                                            .extension()
                                            .map(|e| e.eq_ignore_ascii_case("csv"))
                                            .unwrap_or(false);
                                        if is_dir || is_csv {
                                            entries.push((path, is_dir));
                                        }
                                    }
                                }
                                entries.sort_by(|a, b| {
                                    b.1.cmp(&a.1).then_with(|| a.0.file_name().cmp(&b.0.file_name()))
                                });
                                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                                    for (path, is_dir) in entries {
                                        let label = if is_dir {
                                            format!("[dir] {}", path.display())
                                        } else {
                                            path.display().to_string()
                                        };
                                        if ui.small_button(label).clicked() {
                                            if is_dir {
                                                app.picker_dir = path;
                                            } else {
                                                app.path = path.to_string_lossy().to_string();
                                                app.picker_open = false;
                                                preview_path = Some(path);
                                            }
                                        }
                                    }
                                });
                            });
                        }

                        if let Some(err) = &app.error {
                            ui.colored_label(egui::Color32::from_rgb(215, 58, 73), err);
                        }

                        if let Some(table) = &app.table {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.strong(format!(
                                    "{} rows × {} columns",
                                    table.row_count, table.col_count
                                ));
                                ui.weak(format!(
                                    "delimiter '{}', header: {}, ragged: {}",
                                    table.delimiter,
                                    if table.has_header { "yes" } else { "no" },
                                    if table.ragged { "yes" } else { "no" }
                                ));
                            });
                            egui::CollapsingHeader::new("Preview (first 20 rows)")
                                .default_open(true)
                                .show(ui, |ui| {
                                    egui::ScrollArea::both().max_height(220.0).show(ui, |ui| {
                                        egui::Grid::new("data_preview_grid")
                                            .striped(true)
                                            .show(ui, |ui| {
                                                for col in &table.columns {
                                                    ui.strong(&col.name);
                                                }
                                                ui.end_row();
                                                for col in &table.columns {
                                                    let kind = match col.kind {
                                                        ColumnKind::Numeric => "numeric",
                                                        ColumnKind::Text => "text",
                                                    };
                                                    ui.weak(format!(
                                                        "{kind}, missing {}",
                                                        col.missing
                                                    ));
                                                }
                                                ui.end_row();
                                                for row in table.rows.iter().take(20) {
                                                    for cell in row {
                                                        let shown = if cell.chars().count() > 40 {
                                                            format!(
                                                                "{}…",
                                                                cell.chars().take(40).collect::<String>()
                                                            )
                                                        } else if cell.is_empty() {
                                                            "∅".to_string()
                                                        } else {
                                                            cell.clone()
                                                        };
                                                        ui.label(shown);
                                                    }
                                                    ui.end_row();
                                                }
                                            });
                                    });
                                });

                            ui.add_space(4.0);
                            ui.strong("Columns to analyze");
                            let columns = table.columns.clone();
                            ui.horizontal_wrapped(|ui| {
                                for (idx, col) in columns.iter().enumerate() {
                                    let mut checked = app.selected.contains(&idx);
                                    let numeric = col.kind == ColumnKind::Numeric;
                                    let response = ui.add_enabled(
                                        numeric,
                                        egui::Checkbox::new(
                                            &mut checked,
                                            format!("{} ({})", col.name, col.missing),
                                        ),
                                    );
                                    if response.changed() {
                                        if checked {
                                            if !app.selected.contains(&idx) {
                                                app.selected.push(idx);
                                            }
                                        } else {
                                            app.selected.retain(|&c| c != idx);
                                        }
                                        app.selected.sort_unstable();
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.strong("Plot:");
                                ui.selectable_value(
                                    &mut app.plot_kind,
                                    PlotKind::Column,
                                    "column vs row",
                                );
                                ui.selectable_value(
                                    &mut app.plot_kind,
                                    PlotKind::Scatter,
                                    "scatter",
                                );
                                if app.plot_kind == PlotKind::Scatter {
                                    ui.label("x:");
                                    egui::ComboBox::from_id_salt("data_x_col")
                                        .selected_text(
                                            columns
                                                .get(app.x_col)
                                                .map(|c| c.name.clone())
                                                .unwrap_or_default(),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (idx, col) in columns.iter().enumerate() {
                                                if col.kind == ColumnKind::Numeric {
                                                    ui.selectable_value(
                                                        &mut app.x_col,
                                                        idx,
                                                        &col.name,
                                                    );
                                                }
                                            }
                                        });
                                }
                                ui.label("y:");
                                egui::ComboBox::from_id_salt("data_y_col")
                                    .selected_text(
                                        columns
                                            .get(app.y_col)
                                            .map(|c| c.name.clone())
                                            .unwrap_or_default(),
                                    )
                                    .show_ui(ui, |ui| {
                                        for (idx, col) in columns.iter().enumerate() {
                                            if col.kind == ColumnKind::Numeric {
                                                ui.selectable_value(&mut app.y_col, idx, &col.name);
                                            }
                                        }
                                    });
                                let can_run = !busy_here && !app.selected.is_empty();
                                if ui
                                    .add_enabled(
                                        can_run,
                                        egui::Button::new("Compute statistics and plot"),
                                    )
                                    .clicked()
                                {
                                    app.error = None;
                                    let path = PathBuf::from(app.path.trim());
                                    let x = if app.plot_kind == PlotKind::Scatter {
                                        app.x_col
                                    } else {
                                        app.y_col
                                    };
                                    run_request = Some((
                                        path,
                                        app.selected.clone(),
                                        PlotRequest {
                                            kind: app.plot_kind,
                                            x,
                                            y: app.y_col,
                                        },
                                    ));
                                }
                            });

                            if !app.stats.is_empty() {
                                ui.add_space(4.0);
                                ui.strong("Statistics");
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    egui::Grid::new("data_stats_grid")
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for label in [
                                                "column", "count", "missing", "mean", "median",
                                                "std", "min", "max", "q25", "q50", "q75",
                                            ] {
                                                ui.strong(label);
                                            }
                                            ui.end_row();
                                            for s in &app.stats {
                                                ui.label(&s.name);
                                                ui.label(s.count.to_string());
                                                ui.label(s.missing.to_string());
                                                ui.label(format!("{:.6}", s.mean));
                                                ui.label(format!("{:.6}", s.median));
                                                ui.label(format!("{:.6}", s.std_dev));
                                                ui.label(format!("{:.6}", s.min));
                                                ui.label(format!("{:.6}", s.max));
                                                ui.label(format!("{:.6}", s.q25));
                                                ui.label(format!("{:.6}", s.q50));
                                                ui.label(format!("{:.6}", s.q75));
                                                ui.end_row();
                                            }
                                        });
                                });
                                if let Some(r) = app.correlation {
                                    ui.label(format!("Pearson correlation: {:.9}", r));
                                }
                                ui.weak(
                                    "Results were written to data_analysis_* workspace variables and a figure was pushed to the Plots panel.",
                                );
                            }

                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label("Export results:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.export_path)
                                        .desired_width(360.0),
                                );
                                let can_export = !busy_here && !app.stats.is_empty();
                                let overwrite = app.confirm_overwrite;
                                let button = if overwrite {
                                    "Overwrite and export"
                                } else {
                                    "Export results"
                                };
                                if ui
                                    .add_enabled(can_export, egui::Button::new(button))
                                    .clicked()
                                {
                                    let path = PathBuf::from(app.export_path.trim());
                                    if path.exists() && !overwrite {
                                        app.confirm_overwrite = true;
                                        app.export_message = Some((
                                            false,
                                            format!(
                                                "{} exists. Click again to confirm overwrite.",
                                                path.display()
                                            ),
                                        ));
                                    } else {
                                        let corr = app.correlation.map(|r| {
                                            let x = columns
                                                .get(app.x_col)
                                                .map(|c| c.name.clone())
                                                .unwrap_or_default();
                                            let y = columns
                                                .get(app.y_col)
                                                .map(|c| c.name.clone())
                                                .unwrap_or_default();
                                            (x, y, r)
                                        });
                                        export_request = Some((
                                            path,
                                            app.stats.clone(),
                                            corr,
                                            overwrite,
                                        ));
                                        app.confirm_overwrite = false;
                                    }
                                }
                            });
                            if let Some((ok, message)) = &app.export_message {
                                let color = if *ok {
                                    egui::Color32::from_rgb(46, 160, 67)
                                } else {
                                    egui::Color32::from_rgb(215, 58, 73)
                                };
                                ui.colored_label(color, message);
                            }
                        } else {
                            ui.add_space(8.0);
                            ui.weak("Open a CSV file to preview columns, types, and missing counts.");
                        }
                    });
            }
            if !open {
                self.data_app_open = false;
            }
            if let Some(path) = preview_path {
                self.data_submit_preview(path);
            }
            if let Some((path, selected, plot)) = run_request {
                self.data_submit_run(path, selected, plot);
            }
            if let Some((path, stats, corr, overwrite)) = export_request {
                self.data_submit_export(path, stats, corr, overwrite);
            }
        }

        // Curve Fitter app window.
        if self.curve_app_open {
            let mut open = true;
            let mut preview_path: Option<PathBuf> = None;
            let mut fit_request: Option<(PathBuf, usize, usize, usize)> = None;
            let mut export_request: Option<(PathBuf, PathBuf, FitResult, Vec<f64>, bool)> = None;
            let busy_here = self.running && self.curve_pending.is_some();
            let cancel_flag = std::sync::Arc::clone(&self.worker.cancel);

            if let Some(app) = self.curve_app.as_mut() {
                egui::Window::new("Curve Fitter — Ready · Unit-tested · Not reference-verified")
                    .id(Id::unique("matrust_curve_fitter"))
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(true)
                    .default_width(880.0)
                    .default_height(640.0)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("CSV file:");
                            ui.add(
                                egui::TextEdit::singleline(&mut app.path)
                                    .desired_width(420.0)
                                    .hint_text("path to a .csv file"),
                            );
                            if ui.button("Browse…").clicked() {
                                app.picker_open = !app.picker_open;
                            }
                            let can_open = !app.path.trim().is_empty() && !busy_here;
                            if ui
                                .add_enabled(can_open, egui::Button::new("Open"))
                                .clicked()
                            {
                                preview_path = Some(PathBuf::from(app.path.trim()));
                            }
                            if busy_here {
                                ui.spinner();
                                ui.weak("worker running…");
                                if ui.button("Stop").clicked() {
                                    cancel_flag
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        });

                        if app.picker_open {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Up").clicked() {
                                        if let Some(parent) = app.picker_dir.parent() {
                                            app.picker_dir = parent.to_path_buf();
                                        }
                                    }
                                    ui.weak(app.picker_dir.display().to_string());
                                });
                                let mut entries: Vec<(PathBuf, bool)> = Vec::new();
                                if let Ok(read_dir) = std::fs::read_dir(&app.picker_dir) {
                                    for entry in read_dir.flatten() {
                                        let path = entry.path();
                                        let is_dir = path.is_dir();
                                        let is_csv = path
                                            .extension()
                                            .map(|e| e.eq_ignore_ascii_case("csv"))
                                            .unwrap_or(false);
                                        if is_dir || is_csv {
                                            entries.push((path, is_dir));
                                        }
                                    }
                                }
                                entries.sort_by(|a, b| {
                                    b.1.cmp(&a.1).then_with(|| a.0.file_name().cmp(&b.0.file_name()))
                                });
                                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                                    for (path, is_dir) in entries {
                                        let label = if is_dir {
                                            format!("[dir] {}", path.display())
                                        } else {
                                            path.display().to_string()
                                        };
                                        if ui.small_button(label).clicked() {
                                            if is_dir {
                                                app.picker_dir = path;
                                            } else {
                                                app.path = path.to_string_lossy().to_string();
                                                app.picker_open = false;
                                                preview_path = Some(path);
                                            }
                                        }
                                    }
                                });
                            });
                        }

                        if let Some(err) = &app.error {
                            ui.colored_label(egui::Color32::from_rgb(215, 58, 73), err);
                        }

                        if let Some(table) = &app.table {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.strong(format!(
                                    "{} rows × {} columns",
                                    table.row_count, table.col_count
                                ));
                                ui.weak(format!(
                                    "delimiter '{}', header: {}, ragged: {}",
                                    table.delimiter,
                                    if table.has_header { "yes" } else { "no" },
                                    if table.ragged { "yes" } else { "no" }
                                ));
                            });
                            egui::CollapsingHeader::new("Preview (first 20 rows)")
                                .default_open(false)
                                .show(ui, |ui| {
                                    egui::ScrollArea::both().max_height(180.0).show(ui, |ui| {
                                        egui::Grid::new("curve_preview_grid")
                                            .striped(true)
                                            .show(ui, |ui| {
                                                for col in &table.columns {
                                                    ui.strong(&col.name);
                                                }
                                                ui.end_row();
                                                for col in &table.columns {
                                                    let kind = match col.kind {
                                                        ColumnKind::Numeric => "numeric",
                                                        ColumnKind::Text => "text",
                                                    };
                                                    ui.weak(format!(
                                                        "{kind}, missing {}",
                                                        col.missing
                                                    ));
                                                }
                                                ui.end_row();
                                                for row in table.rows.iter().take(20) {
                                                    for cell in row {
                                                        ui.label(cell.clone());
                                                    }
                                                    ui.end_row();
                                                }
                                            });
                                    });
                                });

                            ui.add_space(4.0);
                            ui.strong("Fit variables");
                            let columns = table.columns.clone();
                            ui.horizontal(|ui| {
                                ui.label("x:");
                                egui::ComboBox::from_id_salt("curve_x_col")
                                    .selected_text(
                                        columns
                                            .get(app.x_col)
                                            .map(|c| c.name.clone())
                                            .unwrap_or_default(),
                                    )
                                    .show_ui(ui, |ui| {
                                        for (idx, col) in columns.iter().enumerate() {
                                            if col.kind == ColumnKind::Numeric {
                                                ui.selectable_value(&mut app.x_col, idx, &col.name);
                                            }
                                        }
                                    });
                                ui.label("y:");
                                egui::ComboBox::from_id_salt("curve_y_col")
                                    .selected_text(
                                        columns
                                            .get(app.y_col)
                                            .map(|c| c.name.clone())
                                            .unwrap_or_default(),
                                    )
                                    .show_ui(ui, |ui| {
                                        for (idx, col) in columns.iter().enumerate() {
                                            if col.kind == ColumnKind::Numeric {
                                                ui.selectable_value(&mut app.y_col, idx, &col.name);
                                            }
                                        }
                                    });
                                ui.label("Degree:");
                                ui.add(
                                    egui::DragValue::new(&mut app.degree)
                                        .range(0..=12)
                                        .prefix("d = "),
                                );
                                let can_fit = !busy_here && app.x_col != app.y_col;
                                if ui
                                    .add_enabled(can_fit, egui::Button::new("Fit"))
                                    .clicked()
                                {
                                    app.error = None;
                                    fit_request = Some((
                                        PathBuf::from(app.path.trim()),
                                        app.x_col,
                                        app.y_col,
                                        app.degree,
                                    ));
                                }
                            });
                            if app.x_col == app.y_col {
                                ui.colored_label(
                                    egui::Color32::from_rgb(210, 153, 34),
                                    "x and y must be different columns.",
                                );
                            }

                            if let Some(fit) = &app.fit {
                                ui.add_space(4.0);
                                ui.strong("Fit result");
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    egui::Grid::new("curve_coeff_grid")
                                        .striped(true)
                                        .show(ui, |ui| {
                                            for label in
                                                ["coefficient", "value", "power of x"]
                                            {
                                                ui.strong(label);
                                            }
                                            ui.end_row();
                                            for (i, c) in fit.coefficients.iter().enumerate() {
                                                ui.label(format!("c{i}"));
                                                ui.label(format!("{c:.10}"));
                                                ui.label(if i == 0 {
                                                    "0 (constant)".to_string()
                                                } else {
                                                    format!("x^{i}")
                                                });
                                                ui.end_row();
                                            }
                                        });
                                });
                                ui.label(format!(
                                    "R² = {:.10}, RMSE = {:.10}, points = {}",
                                    fit.r2, fit.rmse, fit.points
                                ));
                                ui.weak(
                                    "Results were written to the curve_fit_* workspace variables; a data+fit figure and a residuals figure were pushed to the Plots panel.",
                                );
                            }

                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label("Export coefficients:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.export_path)
                                        .desired_width(300.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Export fitted values:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.fitted_export_path)
                                        .desired_width(300.0),
                                );
                                let can_export = !busy_here && app.fit.is_some();
                                let overwrite = app.confirm_overwrite;
                                let button = if overwrite {
                                    "Overwrite and export"
                                } else {
                                    "Export fit"
                                };
                                if ui
                                    .add_enabled(can_export, egui::Button::new(button))
                                    .clicked()
                                {
                                    if let Some(fit) = app.fit.clone() {
                                        let coeff = PathBuf::from(app.export_path.trim());
                                        let fitted =
                                            PathBuf::from(app.fitted_export_path.trim());
                                        if (coeff.exists() || fitted.exists()) && !overwrite {
                                            app.confirm_overwrite = true;
                                            app.export_message = Some((
                                                false,
                                                "Output file(s) exist. Click again to confirm overwrite.".to_string(),
                                            ));
                                        } else {
                                            export_request =
                                                Some((coeff, fitted, fit, app.xs.clone(), overwrite));
                                            app.confirm_overwrite = false;
                                        }
                                    }
                                }
                            });
                            if let Some((ok, message)) = &app.export_message {
                                let color = if *ok {
                                    egui::Color32::from_rgb(46, 160, 67)
                                } else {
                                    egui::Color32::from_rgb(215, 58, 73)
                                };
                                ui.colored_label(color, message);
                            }
                        } else {
                            ui.add_space(8.0);
                            ui.weak(
                                "Open a CSV, pick two numeric columns and a degree, then Fit.",
                            );
                        }
                    });
            }
            if !open {
                self.curve_app_open = false;
            }
            if let Some(path) = preview_path {
                self.curve_submit_preview(path);
            }
            if let Some((path, x, y, degree)) = fit_request {
                self.curve_submit_fit(path, x, y, degree);
            }
            if let Some((coeff, fitted, fit, xs, overwrite)) = export_request {
                self.curve_submit_export(coeff, fitted, fit, xs, overwrite);
            }
        }

        // Simulation and Control app window.
        if self.sim_app_open {
            let mut open = true;
            let mut run_request: Option<(SystemModel, InputSignal, SimulationParams)> = None;
            let mut export_request: Option<(PathBuf, SimulationResult, bool)> = None;
            let busy_here = self.running && self.sim_pending.is_some();
            let cancel_flag = std::sync::Arc::clone(&self.worker.cancel);

            if let Some(app) = self.sim_app.as_mut() {
                egui::Window::new("Simulation and Control — Ready · Unit-tested · Not reference-verified")
                    .id(Id::unique("matrust_simulation"))
                    .open(&mut open)
                    .collapsible(false)
                    .resizable(true)
                    .default_width(920.0)
                    .default_height(680.0)
                    .show(ctx, |ui| {
                        // Presets bar
                        ui.horizontal_wrapped(|ui| {
                            ui.strong("Presets:");
                            if ui.button("1st-Order (tau=1)").clicked() {
                                app.apply_preset(0);
                            }
                            if ui.button("Underdamped 2nd-Order").clicked() {
                                app.apply_preset(1);
                            }
                            if ui.button("Critically Damped").clicked() {
                                app.apply_preset(2);
                            }
                            if ui.button("Overdamped").clicked() {
                                app.apply_preset(3);
                            }
                            if ui.button("Harmonic Oscillator").clicked() {
                                app.apply_preset(4);
                            }
                            if ui.button("State-Space Double Int").clicked() {
                                app.apply_preset(5);
                            }
                        });
                        ui.separator();

                        // Model selection
                        ui.horizontal(|ui| {
                            ui.strong("Model Type:");
                            ui.radio_value(&mut app.model_kind, SimModelKind::FirstOrder, "First-Order ODE");
                            ui.radio_value(&mut app.model_kind, SimModelKind::SecondOrder, "Second-Order Canonical");
                            ui.radio_value(&mut app.model_kind, SimModelKind::StateSpace, "Linear State-Space");
                        });

                        // Model parameters
                        ui.group(|ui| {
                            match app.model_kind {
                                SimModelKind::FirstOrder => {
                                    ui.weak("tau * dy/dt + y(t) = K * u(t)");
                                    ui.horizontal(|ui| {
                                        ui.label("Time constant tau:");
                                        ui.add(egui::DragValue::new(&mut app.tau).speed(0.05).range(0.001..=100.0).prefix("tau = "));
                                        ui.label("DC Gain K:");
                                        ui.add(egui::DragValue::new(&mut app.first_order_gain).speed(0.1).prefix("K = "));
                                        ui.label("Initial y(0):");
                                        ui.add(egui::DragValue::new(&mut app.first_order_y0).speed(0.1).prefix("y0 = "));
                                    });
                                }
                                SimModelKind::SecondOrder => {
                                    ui.weak("d²y/dt² + 2*zeta*omega_n*dy/dt + omega_n²*y = omega_n²*K*u(t)");
                                    ui.horizontal(|ui| {
                                        ui.label("Natural freq wn:");
                                        ui.add(egui::DragValue::new(&mut app.omega_n).speed(0.1).range(0.001..=1000.0).suffix(" rad/s"));
                                        ui.label("Damping ratio zeta:");
                                        ui.add(egui::DragValue::new(&mut app.zeta).speed(0.02).range(0.0..=50.0));
                                        ui.label("Gain K:");
                                        ui.add(egui::DragValue::new(&mut app.second_order_gain).speed(0.1));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Initial displacement y(0):");
                                        ui.add(egui::DragValue::new(&mut app.second_order_y0).speed(0.1));
                                        ui.label("Initial velocity v(0):");
                                        ui.add(egui::DragValue::new(&mut app.second_order_v0).speed(0.1));
                                    });
                                }
                                SimModelKind::StateSpace => {
                                    ui.weak("dx/dt = A x + B u,   y = C x + D u (matrices separated by semicolons/spaces)");
                                    ui.horizontal(|ui| {
                                        ui.label("A matrix:");
                                        ui.add(egui::TextEdit::singleline(&mut app.ss_a).desired_width(180.0));
                                        ui.label("B vector:");
                                        ui.add(egui::TextEdit::singleline(&mut app.ss_b).desired_width(80.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("C vector:");
                                        ui.add(egui::TextEdit::singleline(&mut app.ss_c).desired_width(120.0));
                                        ui.label("D feedthrough:");
                                        ui.add(egui::DragValue::new(&mut app.ss_d).speed(0.1));
                                        ui.label("x0 state:");
                                        ui.add(egui::TextEdit::singleline(&mut app.ss_x0).desired_width(100.0));
                                    });
                                }
                            }
                        });

                        // Input Signal selection
                        ui.horizontal(|ui| {
                            ui.strong("Input Signal u(t):");
                            ui.radio_value(&mut app.signal_kind, SimSignalKind::Step, "Step");
                            ui.radio_value(&mut app.signal_kind, SimSignalKind::Impulse, "Impulse");
                            ui.radio_value(&mut app.signal_kind, SimSignalKind::Ramp, "Ramp");
                            ui.radio_value(&mut app.signal_kind, SimSignalKind::Sine, "Sine");
                            ui.radio_value(&mut app.signal_kind, SimSignalKind::Zero, "Zero / Free");
                        });

                        // Input parameters
                        ui.horizontal(|ui| {
                            match app.signal_kind {
                                SimSignalKind::Step => {
                                    ui.label("Amplitude:");
                                    ui.add(egui::DragValue::new(&mut app.step_amplitude).speed(0.1));
                                    ui.label("Step time t0:");
                                    ui.add(egui::DragValue::new(&mut app.step_t0).speed(0.1).range(0.0..=100.0).suffix(" s"));
                                }
                                SimSignalKind::Impulse => {
                                    ui.label("Magnitude (area):");
                                    ui.add(egui::DragValue::new(&mut app.impulse_magnitude).speed(0.1));
                                    ui.label("Pulse time t0:");
                                    ui.add(egui::DragValue::new(&mut app.step_t0).speed(0.1).range(0.0..=100.0).suffix(" s"));
                                }
                                SimSignalKind::Ramp => {
                                    ui.label("Slope:");
                                    ui.add(egui::DragValue::new(&mut app.ramp_slope).speed(0.1));
                                    ui.label("Start time t0:");
                                    ui.add(egui::DragValue::new(&mut app.step_t0).speed(0.1).range(0.0..=100.0).suffix(" s"));
                                }
                                SimSignalKind::Sine => {
                                    ui.label("Amplitude:");
                                    ui.add(egui::DragValue::new(&mut app.sine_amplitude).speed(0.1));
                                    ui.label("Frequency:");
                                    ui.add(egui::DragValue::new(&mut app.sine_frequency).speed(0.1).range(0.01..=1000.0).suffix(" Hz"));
                                    ui.label("Phase:");
                                    ui.add(egui::DragValue::new(&mut app.sine_phase_deg).speed(5.0).suffix("°"));
                                }
                                SimSignalKind::Zero => {
                                    ui.weak("Autonomous response driven entirely by initial conditions (x0).");
                                }
                            }
                        });

                        // Simulation parameters and Run button
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.strong("Simulation:");
                            ui.label("Duration:");
                            ui.add(egui::DragValue::new(&mut app.t_final).speed(0.5).range(0.05..=1000.0).suffix(" s"));
                            ui.label("Step size dt:");
                            ui.add(egui::DragValue::new(&mut app.dt).speed(0.001).range(0.0001..=1.0).suffix(" s"));

                            let can_run = !busy_here;
                            if ui.add_enabled(can_run, egui::Button::new("Simulate (RK4)")).clicked() {
                                app.error = None;
                                match app.build_model() {
                                    Ok(model) => {
                                        let input = app.build_input();
                                        let params = app.build_params();
                                        run_request = Some((model, input, params));
                                    }
                                    Err(diag) => {
                                        app.error = Some(diag_text(&diag));
                                    }
                                }
                            }

                            if busy_here {
                                ui.spinner();
                                ui.weak("simulating on worker…");
                                if ui.button("Stop").clicked() {
                                    cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        });

                        if let Some(err) = &app.error {
                            ui.colored_label(egui::Color32::from_rgb(215, 58, 73), err);
                        }

                        // Simulation results
                        if let Some(res) = &app.result {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.strong(format!("Results: {} — {}", res.model_name, res.input_label));
                            ui.horizontal(|ui| {
                                let m = &res.metrics;
                                let stab_color = if m.is_stable {
                                    egui::Color32::from_rgb(46, 160, 67)
                                } else {
                                    egui::Color32::from_rgb(215, 58, 73)
                                };
                                ui.label(egui::RichText::new(if m.is_stable { "STABLE" } else { "UNSTABLE / MARGINAL" }).strong().color(stab_color));
                                ui.weak(format!("({} steps, dt = {}s)", res.time.len(), app.dt));
                            });

                            egui::Grid::new("sim_metrics_grid").striped(true).show(ui, |ui| {
                                ui.strong("Metric");
                                ui.strong("Value");
                                ui.strong("Details");
                                ui.end_row();

                                ui.label("Poles (eigenvalues of A)");
                                let pole_strs: Vec<String> = res.metrics.poles.iter().map(|(re, im)| {
                                    if im.abs() < 1e-6 {
                                        format!("{re:.4}")
                                    } else if *im > 0.0 {
                                        format!("{re:.4} + {im:.4}i")
                                    } else {
                                        format!("{re:.4} - {:.4}i", im.abs())
                                    }
                                }).collect();
                                ui.label(pole_strs.join(", "));
                                ui.label(if res.metrics.is_stable { "Re(lambda) < 0" } else { "Re(lambda) >= 0" });
                                ui.end_row();

                                ui.label("Steady-State Value (y_ss)");
                                ui.label(format!("{:.6}", res.metrics.steady_state));
                                ui.label("mean of final 5% trajectory");
                                ui.end_row();

                                ui.label("Peak Output (y_peak)");
                                ui.label(format!("{:.6}", res.metrics.peak_value));
                                ui.label(format!("at t = {:.4}s", res.metrics.peak_time));
                                ui.end_row();

                                ui.label("Percent Overshoot (Mp)");
                                match res.metrics.overshoot_percent {
                                    Some(ov) => { ui.label(format!("{ov:.2}%")); ui.label("(y_peak - y_ss) / delta"); }
                                    None => { ui.label("N/A"); ui.label("not a step or no steady state"); }
                                }
                                ui.end_row();

                                ui.label("Rise Time (tr, 10%-90%)");
                                match res.metrics.rise_time {
                                    Some(tr) => { ui.label(format!("{tr:.4}s")); ui.label("time from 10% to 90% change"); }
                                    None => { ui.label("N/A"); ui.label("non-monotonic or non-step"); }
                                }
                                ui.end_row();

                                ui.label("Settling Time (ts, 2% band)");
                                match res.metrics.settling_time {
                                    Some(ts) => { ui.label(format!("{ts:.4}s")); ui.label("remains within 2% of steady state"); }
                                    None => { ui.label("N/A"); ui.label("never entered 2% band"); }
                                }
                                ui.end_row();
                            });

                            ui.weak("Variables sim_t, sim_y, sim_u, sim_x written to workspace; Time Response and Phase Portrait plots pushed to Figures panel.");

                            // Export section
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Export CSV:");
                                ui.add(egui::TextEdit::singleline(&mut app.export_path).desired_width(320.0));

                                let can_export = !busy_here;
                                let overwrite = app.confirm_overwrite;
                                let btn_text = if overwrite { "Overwrite and export" } else { "Export CSV" };

                                if ui.add_enabled(can_export, egui::Button::new(btn_text)).clicked() {
                                    let path = PathBuf::from(app.export_path.trim());
                                    if path.exists() && !overwrite {
                                        app.confirm_overwrite = true;
                                        app.export_message = Some((
                                            false,
                                            "Target file exists. Click again to confirm overwrite.".to_string(),
                                        ));
                                    } else {
                                        export_request = Some((path, res.clone(), overwrite));
                                        app.confirm_overwrite = false;
                                    }
                                }
                            });

                            if let Some((ok, message)) = &app.export_message {
                                let color = if *ok {
                                    egui::Color32::from_rgb(46, 160, 67)
                                } else {
                                    egui::Color32::from_rgb(215, 58, 73)
                                };
                                ui.colored_label(color, message);
                            }
                        } else {
                            ui.add_space(10.0);
                            ui.weak("Choose a preset or configure the system dynamics and input signal, then click 'Simulate (RK4)'.");
                        }
                    });
            }
            if !open {
                self.sim_app_open = false;
            }
            if let Some((model, input, params)) = run_request {
                self.sim_submit_run(model, input, params);
            }
            if let Some((path, res, overwrite)) = export_request {
                self.sim_submit_export(path, res, overwrite);
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
