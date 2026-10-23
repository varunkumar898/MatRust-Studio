use matlab_diagnostics::DiagnosticSeverity;
use matlab_runtime::{run_source, Session};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique path in the OS temp directory; never a fixed global filename.
fn temp_csv_path(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "matrust_{}_{}_{}.csv",
        label,
        std::process::id(),
        nonce
    ))
}

/// M5 persistence regression: basic save/load round-trip with real values.
#[test]
fn persist_stage5_save_load_roundtrip() {
    let file_path = temp_csv_path("save_load");
    let path_text = file_path.to_string_lossy().to_string();

    let r = run_source(&format!("A = [1 2; 3 4]; save('{}');", path_text)).expect("save executes");
    assert_eq!(r.env.get("A").and_then(|v| v.as_matrix()).unwrap().rows, 2);

    let saved = fs::read_to_string(&file_path).expect("save created a readable file");
    assert!(
        saved.contains("A\n1,2\n3,4\n"),
        "saved file layout must be the documented name/rows format, got:\n{}",
        saved
    );

    let r2 = run_source(&format!("data = load('{}');", path_text)).expect("load executes");
    let data = r2
        .env
        .get("data")
        .and_then(|v| v.as_matrix())
        .expect("loaded variable is a matrix");
    assert_eq!((data.rows, data.cols), (2, 2));
    assert_eq!(data.data, vec![1.0, 2.0, 3.0, 4.0]);

    fs::remove_file(&file_path).expect("cleanup removes the temporary file");
}

/// M5 regression: a missing file produces a clear load error and the session
/// remains usable.
#[test]
fn persist_stage5_load_error_recovery() {
    let missing = temp_csv_path("does_not_exist");
    assert!(!missing.exists());

    let mut session = Session::new();
    let err = session
        .exec(&format!("data = load('{}');", missing.display()), None)
        .expect_err("loading a missing file must fail");
    assert_eq!(err.severity, DiagnosticSeverity::Error);
    assert!(
        err.message.contains("cannot read"),
        "load error should identify the failed read, got: {}",
        err.message
    );

    session
        .exec("recovered = 41 + 1;", None)
        .expect("session remains usable after a load error");
    let recovered = session
        .env
        .get("recovered")
        .and_then(|v| v.as_matrix())
        .expect("recovered is a matrix");
    assert_eq!(recovered.data, vec![42.0]);
}
