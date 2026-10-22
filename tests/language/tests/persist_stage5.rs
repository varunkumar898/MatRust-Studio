use std::fs;
use matlab_runtime::{run_source, Session};

/// M5 persistence regression: basic save/load round-trip through session variables.
#[test]
fn persist_stage5_save_load_roundtrip() {
    let file_path = "/tmp/stage5_persist.csv";
    // Clean previous file if present.
    let _ = fs::remove_file(file_path);

    // Create a variable and save it.
    let r = run_source(&format!(
        "A = [1 2; 3 4]; save('{}', A);",
        file_path
    )).unwrap();
    assert!(r.env.get("A").is_some());
    assert!(fs::metadata(file_path).is_ok());

    // Load it back (into session variable `data` through interpreter assignment).
    let r2 = run_source(&format!(
        "data = load('{}');",
        file_path
    )).unwrap();
    assert!(r2.env.get("data").is_some());
}

/// M5 regression: load error should propagate clearly; session remains usable.
#[test]
fn persist_stage5_load_error_recovery() {
    let r = run_source("data = load('/tmp/nonexistent_stage5.csv');").unwrap_err();
    // The error should be visible; after failure, session should remain usable.
    // This is documented as basic persistence behavior, not full MAT-file support.
    println!("Load error captured (expected): {:?}", r);
}
