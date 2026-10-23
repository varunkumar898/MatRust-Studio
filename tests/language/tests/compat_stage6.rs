use matlab_runtime::{run_source, Session};

fn scalar(run: &matlab_runtime::RunResult, name: &str) -> f64 {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{} should be a scalar", name))
}

/// Complex arithmetic with matrix indexing: B = (1+2i) * [1;2], so
/// B(1,1) = 1+2i.
#[test]
fn compat_stage6_complex_indexing() {
    let r = run_source("A = complex(1,2); B = A * [1; 2]; C = B(1,1);").unwrap();
    let c = r
        .env
        .get("C")
        .and_then(|v| v.as_complex_matrix())
        .expect("C should be a complex scalar");
    assert_eq!((c.rows, c.cols), (1, 1));
    assert!((c.data[0].re - 1.0).abs() < 1e-12);
    assert!((c.data[0].im - 2.0).abs() < 1e-12);
}

/// mean(1:6) = 3.5; sample std = sqrt(3.5).
#[test]
fn compat_stage6_stats_interaction() {
    let r = run_source("A = [1 2 3; 4 5 6]; m = mean(A(:)); s = std(A(:));").unwrap();
    assert!((scalar(&r, "m") - 3.5).abs() < 1e-12);
    assert!((scalar(&r, "s") - 3.5_f64.sqrt()).abs() < 1e-12);
}

/// Plot state stays attached to the session and survives a later command.
#[test]
fn compat_stage6_plot_state_propagation() {
    let mut session = Session::new();
    session
        .exec("x = [1 2 3]; y = [1 4 9]; plot(x, y);", None)
        .unwrap();
    assert_eq!(session.graphics.figures.len(), 1);
    let figure = &session.graphics.figures[0];
    assert_eq!(figure.items.len(), 1);
    match &figure.items[0] {
        matlab_runtime::PlotItem::Line { x, y, .. } => {
            assert_eq!(x, &vec![1.0, 2.0, 3.0]);
            assert_eq!(y, &vec![1.0, 4.0, 9.0]);
        }
        other => panic!("expected a line plot, got {:?}", other),
    }

    session.exec("r = 42;", None).unwrap();
    assert_eq!(
        session
            .env
            .get("r")
            .and_then(|v| v.as_matrix())
            .and_then(|m| m.as_scalar()),
        Some(42.0)
    );
    assert_eq!(session.graphics.figures[0].items.len(), 1);
}
