//! Execution worker: the GUI sends requests over a channel; a dedicated
//! thread owns the single persistent `Session` (environment + graphics) and
//! streams results back. The UI thread never blocks on evaluation and stays
//! responsive during long scripts; Stop works via a cancellation flag checked
//! between statements.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use matlab_diagnostics::Diagnostic;
use matlab_runtime::{RunResult, Session};

use matlabr_apps::curve_fitter::{self, FitResult};
use matlabr_apps::data_analysis::{self, AnalysisOutcome, ColumnStats, CsvTable, PlotRequest};
use matlabr_apps::simulation::{
    self, InputSignal, SimulationParams, SimulationResult, SystemModel,
};

/// Where a piece of executed source came from (for status messages and
/// focus handling).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Typed into the command window (`echo` is the raw typed line).
    CommandWindow { echo: String },
    /// The whole active script.
    Script { path: Option<String>, name: String },
    /// The editor selection.
    Selection { name: String },
}

#[derive(Debug)]
pub enum Request {
    Exec {
        id: u64,
        source: String,
        origin: Origin,
    },
    ClearWorkspace {
        id: u64,
    },
    /// Parse a CSV for the Data Analysis preview (runs on the worker thread).
    DataAnalysisPreview {
        id: u64,
        path: std::path::PathBuf,
    },
    /// Parse a CSV for the Curve Fitter preview (runs on the worker thread).
    CurvePreview {
        id: u64,
        path: std::path::PathBuf,
    },
    /// Compute statistics, write workspace variables, and push a figure.
    DataAnalysisRun {
        id: u64,
        path: std::path::PathBuf,
        selected: Vec<usize>,
        plot: PlotRequest,
    },
    /// Export previously computed statistics to a CSV.
    DataAnalysisExport {
        id: u64,
        path: std::path::PathBuf,
        stats: Vec<ColumnStats>,
        correlation: Option<(String, String, f64)>,
        overwrite: bool,
    },
    /// Fit a polynomial on two CSV columns: parse, QR least-squares solve,
    /// write workspace variables, and push data+fit and residual figures.
    CurveFit {
        id: u64,
        path: std::path::PathBuf,
        x_col: usize,
        y_col: usize,
        degree: usize,
    },
    /// Export a completed fit's coefficients and fitted values.
    CurveFitExport {
        id: u64,
        coeff_path: std::path::PathBuf,
        fitted_path: std::path::PathBuf,
        fit: FitResult,
        xs: Vec<f64>,
        overwrite: bool,
    },
    /// Run an RK4 continuous-time simulation: writes workspace variables,
    /// pushes time-response and phase-portrait figures into the session.
    SimulationRun {
        id: u64,
        model: SystemModel,
        input: InputSignal,
        params: SimulationParams,
    },
    /// Export simulation results and state trajectories to a CSV file.
    SimulationExport {
        id: u64,
        path: std::path::PathBuf,
        result: SimulationResult,
        overwrite: bool,
    },
}

#[derive(Debug)]
pub enum Reply {
    ExecDone {
        id: u64,
        origin: Origin,
        result: Result<RunResult, Diagnostic>,
        duration: Duration,
    },
    WorkspaceCleared {
        result: RunResult,
    },
    DataAnalysisPreview {
        id: u64,
        result: Result<CsvTable, Diagnostic>,
    },
    CurvePreview {
        id: u64,
        result: Result<CsvTable, Diagnostic>,
    },
    DataAnalysisRun {
        id: u64,
        result: Result<AnalysisOutcome, Diagnostic>,
        snapshot: RunResult,
    },
    DataAnalysisExport {
        id: u64,
        result: Result<std::path::PathBuf, Diagnostic>,
    },
    CurveFitDone {
        id: u64,
        result: Result<(FitResult, Vec<f64>, Vec<f64>, CsvTable), Diagnostic>,
        snapshot: RunResult,
    },
    CurveFitExport {
        id: u64,
        result: Result<(std::path::PathBuf, std::path::PathBuf), Diagnostic>,
    },
    SimulationDone {
        id: u64,
        result: Result<SimulationResult, Diagnostic>,
        snapshot: RunResult,
    },
    SimulationExport {
        id: u64,
        result: Result<std::path::PathBuf, Diagnostic>,
    },
}

pub struct Worker {
    pub request_tx: Sender<Request>,
    pub reply_rx: Receiver<Reply>,
    pub cancel: Arc<AtomicBool>,
}

impl Worker {
    /// Spawn the worker thread that owns the persistent interpreter session.
    pub fn spawn() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Request>();
        let (reply_tx, reply_rx) = mpsc::channel::<Reply>();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_thread = Arc::clone(&cancel);

        std::thread::Builder::new()
            .name("matrust-exec".into())
            .spawn(move || {
                let mut session = Session::new();
                loop {
                    let req = match request_rx.recv() {
                        Ok(req) => req,
                        Err(_) => return, // UI closed
                    };
                    cancel_thread.store(false, Ordering::Relaxed);
                    match req {
                        Request::Exec { id, source, origin } => {
                            let started = Instant::now();
                            reply_tx
                                .send(Reply::ExecDone {
                                    id,
                                    origin,
                                    result: session.exec(&source, Some(&cancel_thread)),
                                    duration: started.elapsed(),
                                })
                                .ok();
                        }
                        Request::ClearWorkspace { id } => {
                            session.env.clear();
                            let _ = id;
                            reply_tx
                                .send(Reply::WorkspaceCleared {
                                    result: RunResult {
                                        env: session.env.clone(),
                                        outputs: Vec::new(),
                                        stdout: Vec::new(),
                                        figures: session.graphics.figures.clone(),
                                        interrupted: false,
                                    },
                                })
                                .ok();
                        }
                        Request::DataAnalysisPreview { id, path } => {
                            let result = data_analysis::parse_csv(&path, Some(&cancel_thread));
                            reply_tx
                                .send(Reply::DataAnalysisPreview { id, result })
                                .ok();
                        }
                        Request::CurvePreview { id, path } => {
                            let result = data_analysis::parse_csv(&path, Some(&cancel_thread));
                            reply_tx.send(Reply::CurvePreview { id, result }).ok();
                        }
                        Request::DataAnalysisRun {
                            id,
                            path,
                            selected,
                            plot,
                        } => {
                            let result = data_analysis::analyze(
                                &mut session,
                                &path,
                                &selected,
                                plot,
                                Some(&cancel_thread),
                            );
                            let snapshot = RunResult {
                                env: session.env.clone(),
                                outputs: Vec::new(),
                                stdout: Vec::new(),
                                figures: session.graphics.figures.clone(),
                                interrupted: cancel_thread.load(Ordering::Relaxed),
                            };
                            reply_tx
                                .send(Reply::DataAnalysisRun {
                                    id,
                                    result,
                                    snapshot,
                                })
                                .ok();
                        }
                        Request::DataAnalysisExport {
                            id,
                            path,
                            stats,
                            correlation,
                            overwrite,
                        } => {
                            let corr = correlation
                                .as_ref()
                                .map(|(x, y, r)| (x.as_str(), y.as_str(), *r));
                            let result =
                                data_analysis::export_stats_csv(&path, &stats, corr, overwrite)
                                    .map(|()| path);
                            reply_tx.send(Reply::DataAnalysisExport { id, result }).ok();
                        }
                        Request::CurveFit {
                            id,
                            path,
                            x_col,
                            y_col,
                            degree,
                        } => {
                            let result = data_analysis::parse_csv(&path, Some(&cancel_thread))
                                .and_then(|table| {
                                    curve_fitter::run_fit(
                                        &mut session,
                                        &table,
                                        x_col,
                                        y_col,
                                        degree,
                                        Some(&cancel_thread),
                                    )
                                    .map(|fit| {
                                        let xs: Vec<f64> = table
                                            .numeric
                                            .iter()
                                            .map(|row| row[x_col].unwrap_or(f64::NAN))
                                            .collect();
                                        let ys: Vec<f64> = table
                                            .numeric
                                            .iter()
                                            .map(|row| row[y_col].unwrap_or(f64::NAN))
                                            .collect();
                                        (fit, xs, ys, table)
                                    })
                                });
                            let snapshot = RunResult {
                                env: session.env.clone(),
                                outputs: Vec::new(),
                                stdout: Vec::new(),
                                figures: session.graphics.figures.clone(),
                                interrupted: cancel_thread.load(Ordering::Relaxed),
                            };
                            reply_tx
                                .send(Reply::CurveFitDone {
                                    id,
                                    result,
                                    snapshot,
                                })
                                .ok();
                        }
                        Request::CurveFitExport {
                            id,
                            coeff_path,
                            fitted_path,
                            fit,
                            xs,
                            overwrite,
                        } => {
                            let result = curve_fitter::export_fit_csv(
                                &coeff_path,
                                &fitted_path,
                                &fit,
                                &xs,
                                overwrite,
                            )
                            .map(|()| (coeff_path, fitted_path));
                            reply_tx.send(Reply::CurveFitExport { id, result }).ok();
                        }
                        Request::SimulationRun {
                            id,
                            model,
                            input,
                            params,
                        } => {
                            let result = simulation::run_simulation(
                                &mut session,
                                &model,
                                &input,
                                &params,
                                Some(&cancel_thread),
                            );
                            let snapshot = RunResult {
                                env: session.env.clone(),
                                outputs: Vec::new(),
                                stdout: Vec::new(),
                                figures: session.graphics.figures.clone(),
                                interrupted: cancel_thread.load(Ordering::Relaxed),
                            };
                            reply_tx
                                .send(Reply::SimulationDone {
                                    id,
                                    result,
                                    snapshot,
                                })
                                .ok();
                        }
                        Request::SimulationExport {
                            id,
                            path,
                            result,
                            overwrite,
                        } => {
                            let export_res =
                                simulation::export_simulation_csv(&path, &result, overwrite)
                                    .map(|()| path);
                            reply_tx
                                .send(Reply::SimulationExport {
                                    id,
                                    result: export_res,
                                })
                                .ok();
                        }
                    }
                }
            })
            .expect("failed to spawn execution worker");

        Self {
            request_tx,
            reply_rx,
            cancel,
        }
    }
}
