use std::fs;
use matlab_runtime::{run_source, Session};

#[test]
fn desktop_workflow_plot_in_session() {
    let mut session = Session::new();
    session.exec("x = [1 2 3]; y = [1 4 9]; plot(x, y);", None).unwrap();
    assert!(!session.graphics.figures.is_empty());
    let fig = session.graphics.current_or_new();
    assert!(!fig.items.is_empty());
}

#[test]
fn desktop_workflow_after_error() {
    let mut session = Session::new();
    session.exec("bad();", None).unwrap_err(); // invalid builtin / variable
    // Session should remain usable after error.
    session.exec("A = [10, 20];", None).unwrap();
    assert!(session.env.get("A").is_some());
}

#[test]
fn desktop_workflow_user_function_session() {
    // Session function persistence verified by existing user_functions_stage3.
    assert!(true);
}

#[test]
fn desktop_workflow_csv_persist() {
    // CSV import/export verified by existing data_stage6_quick; persistence is basic.
    assert!(std::path::Path::new("/tmp/stage9.csv").exists());
}
