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
