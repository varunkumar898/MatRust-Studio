use std::fs;
use matlab_runtime::{run_source, Session};

/// Phase D engineering simulation: basic first-order step response.
/// Uses existing array arithmetic and time-axis construction; no new domain module.
#[test]
fn engineering_stageD_first_order_response() {
    let mut session = Session::new();
    // Time vector (domain D time-based simulation)
    session.exec("t = 0:0.1:5; tau = 1.0;", None).unwrap();
    // First-order step response (analytical, independent reference: 1-exp(-t/tau))
    session.exec("y = 1 - exp(-t / tau);", None).unwrap();
    assert!(session.env.get("y").is_some());
    // Visualize result (connects to graphics model)
    session.exec("plot(t, y);", None).unwrap();
    assert!(!session.graphics.figures.is_empty());
    // Persistence / export of result
    let csv_path = "/tmp/engineering_stageD.csv";
    let _ = fs::remove_file(csv_path);
    session.exec("save('/tmp/engineering_stageD.csv', y);", None).unwrap();
    assert!(fs::metadata(csv_path).is_ok());
    assert!(session.env.get("t").is_some());
}
