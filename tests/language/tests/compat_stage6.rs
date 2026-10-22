use matlab_runtime::{run_source, Session};

/// M6 compatibility gap: complex arithmetic with matrix indexing interaction.
#[test]
fn compat_stage6_complex_indexing() {
    let r = run_source("A = complex(1,2); B = A * [1; 2]; C = B(1,1);").unwrap();
    assert!(r.env.get("C").is_some());
}

/// M6 gap: statistics interaction (mean/std on matrix with complex inputs partial).
#[test]
fn compat_stage6_stats_interaction() {
    let r = run_source("A = [1 2 3; 4 5 6]; m = mean(A(:)); s = std(A(:));").unwrap();
    assert!(r.env.get("m").is_some());
    assert!(r.env.get("s").is_some());
}

/// M6 gap: plot-state propagation through session persistence/reuse.
#[test]
fn compat_stage6_plot_state_propagation() {
    let mut session = Session::new();
    session.exec("x = [1 2 3]; y = [1 4 9]; plot(x, y);", None).unwrap();
    assert!(!session.graphics.figures.is_empty());
    // After plot creation, session remains usable (error recovery + plot persistence).
    session.exec("r = 42;", None).unwrap();
    assert!(session.env.get("r").is_some());
}
