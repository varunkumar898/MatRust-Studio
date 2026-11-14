//! In-process mock backend implementing `ExternalBackend` for tests.
//!
//! The mock requires no Octave or Julia installation. It canned-responds to
//! `eval`/`call` and supports programmable behaviors (delay, error, crash)
//! used by the K2 protocol fuzz, timeout, kill, and restart tests.

use crate::{BackendError, BackendInfo, BackendOutput, ExternalBackend, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A canned response the mock returns for a given operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Canned {
    /// Return a scalar double result.
    Double(f64),
    /// Return an error with the given id and message.
    Error { id: String, message: String },
    /// Sleep `ms` milliseconds before responding (used for timeout tests).
    DelayMs(u64),
    /// Panic the mock's internal handler (simulates a worker crash).
    Crash,
}

/// In-process mock backend. Every call is handled synchronously on the
/// calling thread; there is no subprocess, so `interrupt`/`shutdown` are
/// no-ops that return successfully.
pub struct MockBackend {
    id: String,
    started: bool,
    responses: HashMap<String, Canned>,
    /// When true, `call`/`eval` return `WorkerCrashed` (simulates death).
    crashed: Arc<AtomicBool>,
}

impl MockBackend {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            started: false,
            responses: HashMap::new(),
            crashed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Register a canned response for a function name (used by `call`).
    #[allow(dead_code)]
    fn with_call(mut self, fn_name: &str, canned: Canned) -> Self {
        self.responses.insert(fn_name.to_string(), canned);
        self
    }

    /// Register a canned response for `eval` (keyed by "__eval__").
    #[allow(dead_code)]
    fn with_eval(mut self, canned: Canned) -> Self {
        self.responses.insert("__eval__".to_string(), canned);
        self
    }

    /// Mark the mock as crashed so subsequent calls fail with `WorkerCrashed`.
    pub fn crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }

    /// Clear the crashed state.
    pub fn uncrash(&self) {
        self.crashed.store(false, Ordering::SeqCst);
    }

    fn handle_call(&self, fn_name: &str, _args: &[Value], nargout: usize) -> Result<Vec<Value>, BackendError> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(BackendError::WorkerCrashed);
        }
        match self.responses.get(fn_name) {
            Some(Canned::Double(v)) => {
                let mut out = Vec::with_capacity(nargout.max(1));
                out.push(Value::scalar(*v));
                while out.len() < nargout {
                    out.push(Value::scalar(0.0));
                }
                Ok(out)
            }
            Some(Canned::Error { id, message }) => Err(BackendError::BackendException {
                id: id.clone(),
                message: message.clone(),
            }),
            Some(Canned::DelayMs(ms)) => {
                std::thread::sleep(std::time::Duration::from_millis(*ms));
                Ok(vec![Value::scalar(1.0); nargout.max(1)])
            }
            Some(Canned::Crash) => Err(BackendError::WorkerCrashed),
            None => Ok(vec![Value::scalar(0.0); nargout.max(1)]),
        }
    }

    fn handle_eval(&self, _code: &str, nargout: usize) -> Result<Vec<Value>, BackendError> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(BackendError::WorkerCrashed);
        }
        match self.responses.get("__eval__") {
            Some(Canned::Double(v)) => Ok(vec![Value::scalar(*v); nargout.max(1)]),
            Some(Canned::Error { id, message }) => Err(BackendError::BackendException {
                id: id.clone(),
                message: message.clone(),
            }),
            Some(Canned::DelayMs(ms)) => {
                std::thread::sleep(std::time::Duration::from_millis(*ms));
                Ok(vec![Value::scalar(1.0); nargout.max(1)])
            }
            Some(Canned::Crash) => Err(BackendError::WorkerCrashed),
            None => Ok(vec![Value::scalar(0.0); nargout.max(1)]),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new("mock")
    }
}

impl ExternalBackend for MockBackend {
    fn id(&self) -> &str {
        &self.id
    }

    fn detect(&self) -> Option<BackendInfo> {
        Some(BackendInfo {
            path: std::path::PathBuf::from("mock"),
            version: "mock-0.1.0".to_string(),
        })
    }

    fn start(&mut self) -> Result<(), BackendError> {
        if self.crashed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(BackendError::WorkerCrashed);
        }
        self.started = true;
        Ok(())
    }

    fn eval(&mut self, code: &str, nargout: usize) -> Result<BackendOutput, BackendError> {
        let values = self.handle_eval(code, nargout)?;
        Ok(BackendOutput {
            stdout: String::new(),
            stderr: String::new(),
            values,
        })
    }

    fn call(
        &mut self,
        fn_name: &str,
        args: Vec<Value>,
        nargout: usize,
    ) -> Result<Vec<Value>, BackendError> {
        self.handle_call(fn_name, &args, nargout)
    }

    fn interrupt(&mut self) -> Result<(), BackendError> {
        // In-process mock has no process to kill; interrupt is a no-op.
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), BackendError> {
        self.started = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_call_returns_double() {
        let mut m = MockBackend::new("m").with_call("sin", Canned::Double(0.5));
        m.start().unwrap();
        let out = m.call("sin", vec![Value::scalar(0.5)], 1).unwrap();
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0], Value::Double(_)));
    }

    #[test]
    fn mock_call_error_propagates() {
        let mut m = MockBackend::new("m").with_call("bad", Canned::Error { id: "TEST:Err".into(), message: "boom".into() });
        m.start().unwrap();
        let err = m.call("bad", vec![], 1).unwrap_err();
        assert!(matches!(err, BackendError::BackendException { .. }));
    }

    #[test]
    fn mock_crash_propagates_worker_crashed() {
        let mut m = MockBackend::new("m");
        m.start().unwrap();
        m.crash();
        let err = m.call("sin", vec![], 1).unwrap_err();
        assert!(matches!(err, BackendError::WorkerCrashed));
        m.uncrash();
        let _ = m.call("sin", vec![], 1).unwrap();
    }

    #[test]
    fn mock_detect_returns_info() {
        let m = MockBackend::new("m");
        let info = m.detect().unwrap();
        assert_eq!(info.path, std::path::PathBuf::from("mock"));
        assert_eq!(info.version, "mock-0.1.0");
    }
}
