//! # `mlapp-backend`
//!
//! External backend trait, persistent worker process, framed IPC protocol, and
//! mock backend for RustLab's Octave/Julia bridge architecture.
//!
//! Design (see `docs/backends/K1-design.md`):
//! - One `ExternalBackend` trait implemented by Octave, Julia, and a mock.
//! - A persistent worker process per backend, not one process per call.
//! - Length-prefixed JSON framing over stdin/stdout (`crate::framing`).
//! - Data marshalled through `.mat` files in a per-session temp dir
//!   (`rustlab-matio`), bit-exact for doubles.
//! - Timeouts, interrupt/kill switch, and crash recovery/restart.
//!
//! Library crates deny `unwrap`/`expect`/`panic`/`todo`/`unimplemented`
//! (AGENTS.md R11). This crate has no `unsafe`.

pub mod framing;
pub mod mock;
pub mod protocol;
pub mod worker;

use rustlab_math::Value;
use std::path::PathBuf;
use thiserror::Error;

/// Information about a detected backend installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInfo {
    /// Resolved path to the backend executable.
    pub path: PathBuf,
    /// Version string reported by the backend.
    pub version: String,
}

/// Output of an `eval` operation: captured stdout/stderr plus returned values.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendOutput {
    /// Stdout captured from the backend.
    pub stdout: String,
    /// Stderr captured from the backend.
    pub stderr: String,
    /// Values returned by the evaluation (nargout entries).
    pub values: Vec<Value>,
}

/// Errors produced by the backend layer. Ids follow the prefix convention
/// (AGENTS.md R12, backends prompt O10): `MATLAB:`, `Octave:`, `Julia:`,
/// `RUSTLAB:` for bridge-internal failures.
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("backend '{0}' is not installed or could not be found")]
    NotAvailable(String),
    #[error("failed to start backend process: {0}")]
    ProcessStart(String),
    #[error("backend process I/O error: {0}")]
    ProcessIo(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("execution timed out after {0} ms")]
    Timeout(u64),
    #[error("worker process crashed; restart required")]
    WorkerCrashed,
    #[error("marshalling error: {0}")]
    Marshalling(String),
    #[error("backend exception [{id}]: {message}")]
    BackendException { id: String, message: String },
}

/// Every backend must implement this trait. Real backends (Octave, Julia)
/// spawn a persistent worker process; the mock backend runs in-process for
/// tests (no Octave or Julia required).
pub trait ExternalBackend: Send + Sync {
    /// Stable backend identifier, e.g. `"octave"`, `"julia"`.
    fn id(&self) -> &str;

    /// Detect the backend executable and report its version, without starting
    /// a worker. Returns `None` when the backend is not installed.
    fn detect(&self) -> Option<BackendInfo>;

    /// Start a persistent worker process. Must be called before `eval`/`call`.
    fn start(&mut self) -> Result<(), BackendError>;

    /// Evaluate source code in the backend workspace.
    fn eval(&mut self, code: &str, nargout: usize) -> Result<BackendOutput, BackendError>;

    /// Call a named function with arguments, returning `nargout` values.
    fn call(
        &mut self,
        fn_name: &str,
        args: Vec<Value>,
        nargout: usize,
    ) -> Result<Vec<Value>, BackendError>;

    /// Interrupt the currently running computation (kill switch bound to Stop).
    fn interrupt(&mut self) -> Result<(), BackendError>;

    /// Shut down the worker process and release resources.
    fn shutdown(&mut self) -> Result<(), BackendError>;
}
