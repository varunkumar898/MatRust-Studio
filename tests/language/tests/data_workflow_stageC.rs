use matlab_runtime::Session;
use std::time::{SystemTime, UNIX_EPOCH};

/// Phase C integrated workflow: dataset -> stats -> plot -> persistence.
/// Values are checked against hand-computed results, not mere existence.
#[test]
fn data_workflow_csv_stats_plot_persist() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let csv_path = std::env::temp_dir().join(format!(
        "matrust_phase_c_{}_{}.csv",
        std::process::id(),
        nonce
    ));
    let path_text = csv_path.to_string_lossy().to_string();

    let mut session = Session::new();

    session
        .exec(
            &format!(
                "data = [10 20 30; 40 50 60; 70 80 90]; save('{}');",
                path_text
            ),
            None,
        )
        .expect("save dataset");
    let written = std::fs::read_to_string(&csv_path).expect("dataset file exists");
    assert_eq!(written, "data\n10,20,30\n40,50,60\n70,80,90\n");

    session
        .exec(&format!("loaded = load('{}');", path_text), None)
        .expect("load dataset");
    let loaded = session
        .env
        .get("loaded")
        .and_then(|v| v.as_matrix())
        .expect("loaded is a matrix");
    assert_eq!((loaded.rows, loaded.cols), (3, 3));
    assert_eq!(
        loaded.data,
        vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]
    );

    session
        .exec("m = mean(loaded(:)); s = std(loaded(:));", None)
        .expect("statistics");
    let m = session
        .env
        .get("m")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("m is scalar");
    let s = session
        .env
        .get("s")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("s is scalar");
    assert!((m - 50.0).abs() < 1e-12, "mean should be 50, got {}", m);
    let expected_std = 750.0_f64.sqrt(); // sample std of 10,20,...,90
    assert!(
        (s - expected_std).abs() < 1e-12,
        "std should be {}, got {}",
        expected_std,
        s
    );

    session
        .exec("plot([1 2 3], loaded(:,1));", None)
        .expect("plot first column");
    assert_eq!(session.graphics.figures.len(), 1);
    assert_eq!(session.graphics.figures[0].items.len(), 1);

    session
        .exec("final_check = m + s;", None)
        .expect("session remains usable after plot + stats");
    let final_check = session
        .env
        .get("final_check")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("final_check is scalar");
    assert!((final_check - (m + s)).abs() < 1e-12);

    std::fs::remove_file(&csv_path).expect("temporary file cleaned up");
}
