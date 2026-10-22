use std::fs;
use matlab_runtime::{run_source, Session};

/// Phase C integrated workflow: dataset -> stats -> plot -> persistence.
/// This verifies that existing CSV I/O, statistics, plotting model, and
/// session persistence work together as a coherent scientific workflow.
#[test]
fn data_workflow_csv_stats_plot_persist() {
    let csv_path = "/tmp/workflow_stageC_data.csv";
    let _ = fs::remove_file(csv_path);

    let mut session = Session::new();

    // 1. Create a dataset and save to CSV (data I/O domain B).
    session.exec("data = [10 20 30; 40 50 60; 70 80 90]; save('/tmp/workflow_stageC_data.csv', data);", None).unwrap();
    assert!(fs::metadata(csv_path).is_ok());

    // 2. Load back (persistence domain B).
    session.exec("loaded = load('/tmp/workflow_stageC_data.csv');", None).unwrap();
    assert!(session.env.get("loaded").is_some());

    // 3. Statistics (domain A / statistics).
    session.exec("m = mean(loaded(:)); s = std(loaded(:));", None).unwrap();
    assert!(session.env.get("m").is_some());
    assert!(session.env.get("s").is_some());

    // 4. Plot the statistic result (domain C / visualization).
    session.exec("plot([1 2 3], loaded(:,1));", None).unwrap();
    assert!(!session.graphics.figures.is_empty());

    // 5. Check that session remains usable after plot + stats (domain E / session).
    session.exec("final_check = m + s;", None).unwrap();
    assert!(session.env.get("final_check").is_some());
}
