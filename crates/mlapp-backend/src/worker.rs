//! Persistent worker process management for an external backend.
//!
//! A `Worker` owns one child process with piped stdin/stdout. Requests are
//! sent as length-prefixed JSON frames (see `crate::framing`) and responses
//! are read synchronously with a per-call timeout. The worker can be
//! interrupted (killed) and restarted after a crash; a worker crash never
//! panics the host — it surfaces as `BackendError::WorkerCrashed`.

use crate::framing::{read_frame, write_frame, FrameError};
use crate::protocol::{Request, Response};
use std::io::BufReader;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use thiserror::Error;

/// Errors produced by the worker layer.
#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("failed to spawn worker process: {0}")]
    ProcessStart(String),
    #[error("worker process I/O error: {0}")]
    ProcessIo(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("execution timed out after {0} ms")]
    Timeout(u64),
    #[error("worker process crashed; restart required")]
    WorkerCrashed,
}

/// A persistent worker process connected to a backend over framed stdin/stdout.
pub struct Worker {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    /// Receives response payloads from the reader thread.
    rx: mpsc::Receiver<Vec<u8>>,
}

impl Worker {
    /// Spawn a backend process with piped stdin/stdout and return a `Worker`.
    /// A reader thread is spawned to consume framed responses from stdout.
    pub fn spawn(mut command: Command) -> Result<Self, WorkerError> {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|e| WorkerError::ProcessStart(e.to_string()))?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let (stdin, stdout) = match (stdin, stdout) {
            (Some(stdin), Some(stdout)) => (stdin, BufReader::new(stdout)),
            _ => {
                return Err(WorkerError::ProcessStart(
                    "worker process did not provide stdin/stdout".to_string(),
                ))
            }
        };

        let (tx, rx) = mpsc::channel();
        // Reader thread: continuously read frames and forward payloads.
        std::thread::spawn(move || {
            reader_loop(stdout, tx);
        });

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            rx,
        })
    }

    /// Send one request and read one response, enforcing `timeout_ms`.
    /// Returns `WorkerCrashed` if the child has exited or the pipe broke.
    pub fn request(
        &mut self,
        req: &Request,
        timeout_ms: u64,
    ) -> Result<Response, WorkerError> {
        self.ensure_alive()?;

        let payload =
            serde_json::to_vec(req).map_err(|e| WorkerError::Protocol(e.to_string()))?;
        let stdin = self.stdin.as_mut().ok_or(WorkerError::WorkerCrashed)?;
        write_frame(stdin, &payload).map_err(|e| WorkerError::Protocol(e.to_string()))?;

        let payload = self
            .rx
            .recv_timeout(Duration::from_millis(timeout_ms))
            .map_err(|_| WorkerError::Timeout(timeout_ms))?;

        serde_json::from_slice(&payload).map_err(|e| WorkerError::Protocol(e.to_string()))
    }

    /// Interrupt the running computation by killing the worker process.
    /// The host must call `restart` (or `spawn` a new worker) afterwards.
    pub fn interrupt(&mut self) -> Result<(), WorkerError> {
        let child = self.child.as_mut().ok_or(WorkerError::WorkerCrashed)?;
        child.kill().map_err(|e| WorkerError::ProcessIo(e.to_string()))
    }

    /// Shut down the worker gracefully: send `Shutdown`, wait briefly, then
    /// kill if it did not exit.
    pub fn shutdown(&mut self) -> Result<(), WorkerError> {
        let _ = self.request(&Request::Shutdown, 2_000);
        let child = self.child.take();
        if let Some(mut child) = child {
            if let Ok(None) = child.try_wait() {
                let _ = child.kill();
            }
        }
        self.stdin = None;
        Ok(())
    }

    /// Restart the worker: kill the old child (if any) and spawn a new one
    /// with the same command.
    pub fn restart(&mut self, command: Command) -> Result<(), WorkerError> {
        let _ = self.interrupt();
        let mut new = Self::spawn(command)?;
        std::mem::swap(self, &mut new);
        let _ = new.shutdown();
        Ok(())
    }

    /// Returns true if the worker child process is still running.
    pub fn is_alive(&mut self) -> bool {
        matches!(
            self.child.as_mut().and_then(|c| c.try_wait().ok()),
            Some(None)
        )
    }

    fn ensure_alive(&mut self) -> Result<(), WorkerError> {
        match self
            .child
            .as_mut()
            .and_then(|c| c.try_wait().ok())
        {
            Some(Some(_)) | None => Err(WorkerError::WorkerCrashed),
            Some(None) => Ok(()),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Reader thread loop: read frames from stdout and forward payloads on `tx`.
/// Ends when the stream closes (child exited or pipe broken).
fn reader_loop(mut stdout: BufReader<ChildStdout>, tx: mpsc::Sender<Vec<u8>>) {
    loop {
        let payload = match read_frame(&mut stdout) {
            Ok(p) => p,
            Err(FrameError::UnexpectedEof) => return,
            Err(FrameError::PayloadTooLarge(_, _)) => return,
            Err(FrameError::Io(_)) => return,
        };
        if tx.send(payload).is_err() {
            // Receiver gone; nothing more to do.
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_error_display() {
        let err = WorkerError::Timeout(5000);
        assert!(err.to_string().contains("5000"));
    }
}
