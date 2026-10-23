use matlab_runtime::{run_source, Session};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_csv(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "matrust_{}_{}_{}.csv",
        label,
        std::process::id(),
        nonce
    ))
}

#[test]
fn desktop_workflow_plot_in_session() {
    let mut session = Session::new();
    session
        .exec("x = [1 2 3]; y = [1 4 9]; plot(x, y);", None)
        .unwrap();
    assert_eq!(session.graphics.figures.len(), 1);
    let figure = session.graphics.current_or_new();
    assert_eq!(figure.items.len(), 1);
    match &figure.items[0] {
        matlab_runtime::PlotItem::Line { x, y, .. } => {
            assert_eq!(x, &vec![1.0, 2.0, 3.0]);
            assert_eq!(y, &vec![1.0, 4.0, 9.0]);
        }
        other => panic!("expected line plot, got {:?}", other),
    }
}

#[test]
fn desktop_workflow_after_error() {
    let mut session = Session::new();
    let err = session
        .exec("bad();", None)
        .expect_err("unknown call must fail");
    assert_eq!(err.message, "Unrecognized function or variable 'bad'.");

    session.exec("A = [10, 20];", None).unwrap();
    let a = session
        .env
        .get("A")
        .and_then(|v| v.as_matrix())
        .expect("A is a matrix");
    assert_eq!((a.rows, a.cols), (1, 2));
    assert_eq!(a.data, vec![10.0, 20.0]);
}

#[test]
fn desktop_workflow_user_function_session() {
    let mut session = Session::new();
    session
        .exec("function y = twice(x) y = 2 * x; end", None)
        .expect("function definition");
    session.exec("r = twice(21);", None).expect("function call");
    let r = session
        .env
        .get("r")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("r is scalar");
    assert_eq!(r, 42.0);
}

#[test]
fn desktop_workflow_csv_persist() {
    let path = temp_csv("desktop_csv");
    let path_text = path.to_string_lossy().to_string();

    run_source(&format!(
        "A = [10 20; 30 40]; csvwrite('{}', A);",
        path_text
    ))
    .expect("csvwrite executes");
    let back = run_source(&format!("A2 = csvread('{}');", path_text)).expect("csvread executes");
    let a2 = back
        .env
        .get("A2")
        .and_then(|v| v.as_matrix())
        .expect("A2 is a matrix");
    assert_eq!(a2.rows, 2);
    assert_eq!(a2.cols, 2);
    assert_eq!(a2.data, vec![10.0, 20.0, 30.0, 40.0]);

    std::fs::remove_file(&path).expect("temporary CSV cleaned up");
}
