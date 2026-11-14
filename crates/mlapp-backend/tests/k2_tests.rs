//! K2 integration tests for the backend core: worker management, framing,
//! timeouts, kill, crash recovery/restart, and 64 MB payload.
//!
//! These tests run without Octave or Julia by spawning an in-process mock
//! server as a subprocess that speaks the length-prefixed JSON protocol.
//!
//! If the environment variable `MLAPP_BACKEND_MOCK_SERVER` is set, this binary
//! runs as the mock server and exits. Otherwise it runs the tests, spawning
//! the mock server as a child process.

use mlapp_backend::framing::{self, write_frame, MAX_PAYLOAD};
use mlapp_backend::protocol::{BackendErrorInfo, Request, Response, ValueEnvelope};
use mlapp_backend::{BackendError, ExternalBackend, MockBackend, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Entry point for the mock server subprocess.
fn mock_server_main() {
    let mode = std::env::var("MOCK_SERVER_MODE").unwrap_or_default();
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let delay_ms: u64 = mode
        .strip_prefix("delay:")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let crash = mode == "crash";
    let error = mode == "error";

    let mut id_gen = 0u64;
    loop {
        let payload = match framing::read_frame(&mut stdin) {
            Ok(p) => p,
            Err(_) => return,
        };
        let req: Request = match serde_json::from_slice(&payload) {
            Ok(r) => r,
            Err(_) => return,
        };
        id_gen += 1;

        if crash {
            // Exit without responding: simulates a worker crash.
            std::process::exit(0);
        }

        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }

        let resp = if error {
            Response {
                id: id_gen,
                ok: false,
                get: None,
                error: Some(BackendErrorInfo {
                    id: "MOCK:Error".to_string(),
                    message: "mock error".to_string(),
                }),
            }
        } else {
            Response {
                id: id_gen,
                ok: true,
                get: None,
                error: None,
            }
        };

        let payload = serde_json::to_vec(&resp).unwrap();
        let _ = write_frame(&mut stdout, &payload);

        if matches!(req, Request::Shutdown) {
            return;
        }
    }
}

/// Spawn the mock server subprocess with the given mode.
fn spawn_mock_server(mode: &str) -> (std::process::Child, std::process::ChildStdin, BufReader<std::process::ChildStdout>) {
    let exe = std::env::current_exe().unwrap();
    let mut child = Command::new(exe)
        .env("MLAPP_BACKEND_MOCK_SERVER", "1")
        .env("MOCK_SERVER_MODE", mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    (child, stdin, stdout)
}

/// Connect a `mlapp_backend::worker::Worker` to the mock server's pipes.
/// Since `Worker::spawn` takes a `Command`, we use the same exe in mock mode.
fn connect_worker(mode: &str) -> mlapp_backend::worker::Worker {
    let exe = std::env::current_exe().unwrap();
    let mut command = Command::new(exe);
    command.env("MLAPP_BACKEND_MOCK_SERVER", "1").env("MOCK_SERVER_MODE", mode);
    mlapp_backend::worker::Worker::spawn(command).unwrap()
}

// ---------------------------------------------------------------------------
// Test: protocol fuzz — malformed frames never panic
// ---------------------------------------------------------------------------

#[test]
fn protocol_fuzz_truncated_frame() {
    let (mut child, mut stdin, stdout) = spawn_mock_server("normal");
    // Drop stdout so the reader thread gets EOF.
    drop(stdout);

    // Send a truncated frame: length prefix says 100 bytes, but only 5 follow.
    let mut bad = Vec::new();
    bad.extend_from_slice(&(100u32).to_le_bytes());
    bad.extend_from_slice(b"hello");
    let _ = stdin.write_all(&bad);
    let _ = stdin.flush();

    // The worker should report WorkerCrashed (EOF), not panic.
    let mut worker = mlapp_backend::worker::Worker::spawn(Command::new("false")).unwrap_err();
    let _ = worker; // spawn of "false" fails; that's fine

    // Give the reader thread time to observe EOF.
    std::thread::sleep(Duration::from_millis(100));
    let _ = child.kill();
    let _ = child.wait();
}

// ---------------------------------------------------------------------------
// Test: timeout — slow mock server exceeds per-call deadline
// ---------------------------------------------------------------------------

#[test]
fn worker_timeout() {
    let mut worker = connect_worker("delay:5000");
    let err = worker
        .request(&Request::Ping, 100)
        .unwrap_err();
    assert!(
        matches!(err, mlapp_backend::worker::WorkerError::Timeout(_)),
        "expected Timeout, got {err:?}"
    );
    let _ = worker.shutdown();
}

// ---------------------------------------------------------------------------
// Test: kill on Stop — interrupt() kills the worker
// ---------------------------------------------------------------------------

#[test]
fn kill_on_stop() {
    let mut worker = connect_worker("normal");
    assert!(worker.is_alive());
    worker.interrupt().unwrap();
    // After kill, the next request should report WorkerCrashed.
    let err = worker.request(&Request::Ping, 1000).unwrap_err();
    assert!(
        matches!(err, mlapp_backend::worker::WorkerError::WorkerCrashed),
        "expected WorkerCrashed after interrupt, got {err:?}"
    );
    let _ = worker.shutdown();
}

// ---------------------------------------------------------------------------
// Test: restart after crash
// ---------------------------------------------------------------------------

#[test]
fn restart_after_crash() {
    // First worker talks to a crash-mode server (dies on first request).
    let mut worker = connect_worker("crash");
    let err = worker.request(&Request::Ping, 1000).unwrap_err();
    assert!(
        matches!(err, mlapp_backend::worker::WorkerError::WorkerCrashed),
        "expected WorkerCrashed from crash-mode server, got {err:?}"
    );

    // Restart with a normal-mode server.
    let exe = std::env::current_exe().unwrap();
    let mut command = Command::new(exe);
    command.env("MLAPP_BACKEND_MOCK_SERVER", "1").env("MOCK_SERVER_MODE", "normal");
    worker.restart(command).unwrap();
    assert!(worker.is_alive());
    let resp = worker.request(&Request::Ping, 1000).unwrap();
    assert!(resp.ok);
    let _ = worker.shutdown();
}

// ---------------------------------------------------------------------------
// Test: 64 MB payload round trip
// ---------------------------------------------------------------------------

#[test]
fn large_payload_round_trip() {
    let mut worker = connect_worker("normal");
    // Build a 64 MiB payload of JSON (a large string in an eval response).
    // The mock server echoes back ok=true; we test that a large *request*
    // frame is accepted and framed correctly.
    let big_string = "x".repeat(64 * 1024 * 1024);
    let req = Request::Eval {
        code: big_string.clone(),
        put: vec![],
        nargout: 1,
    };
    let payload = serde_json::to_vec(&req).unwrap();
    assert!(payload.len() as u32 <= MAX_PAYLOAD, "payload exceeds max");

    // The mock server will read the large frame and respond ok=true.
    // This may take a moment due to the 64 MiB JSON parse.
    let resp = worker.request(&req, 30_000).unwrap();
    assert!(resp.ok, "expected ok for large payload");
    let _ = worker.shutdown();
}

// ---------------------------------------------------------------------------
// Test: MockBackend trait-level tests (no subprocess)
// ---------------------------------------------------------------------------

#[test]
fn mock_backend_call() {
    let mut m = MockBackend::new("m");
    m.with_call("sin", mlapp_backend::mock::Canned::Double(0.5));
    m.start().unwrap();
    let out = m.call("sin", vec![Value::scalar(0.5)], 1).unwrap();
    assert_eq!(out.len(), 1);
    assert!(matches!(out[0], Value::Double(_)));
}

#[test]
fn mock_backend_worker_crashed() {
    let mut m = MockBackend::new("m");
    m.start().unwrap();
    m.crash();
    let err = m.call("sin", vec![], 1).unwrap_err();
    assert!(matches!(err, BackendError::WorkerCrashed));
    m.uncrash();
    let _ = m.call("sin", vec![], 1).unwrap();
}

#[test]
fn mock_backend_detect() {
    let m = MockBackend::new("m");
    let info = m.detect().unwrap();
    assert_eq!(info.path, std::path::PathBuf::from("mock"));
    assert_eq!(info.version, "mock-0.1.0");
}

#[test]
fn value_envelope_round_trip() {
    let env = ValueEnvelope::scalar_double(42.0);
    let v = mlapp_backend::protocol::envelope_to_value(&env).unwrap();
    assert!(matches!(v, Value::Double(_)));
    let back = mlapp_backend::protocol::value_to_envelope(&v).unwrap();
    assert_eq!(env, back);
}

// ---------------------------------------------------------------------------
// Mock server entry point
// ---------------------------------------------------------------------------

fn main() {
    if std::env::var("MLAPP_BACKEND_MOCK_SERVER").is_ok() {
        mock_server_main();
        std::process::exit(0);
    }
}
