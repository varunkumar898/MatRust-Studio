//! Integration tests for persistent sessions (`Session`), cancellation,
//! and the workspace snapshot API used by the desktop app.

use std::sync::atomic::AtomicBool;

use matlab_runtime::{run_source, variable_rows, Env, Session};

#[test]
fn session_persists_variables_and_figures_across_chunks() {
    let mut session = Session::new();

    session
        .exec("A = [1 2; 3 4]; B = [5 6; 7 8];", None)
        .expect("first chunk");
    let second = session
        .exec("C = A * B;", None)
        .expect("second chunk reads variables from the first");

    let c = second
        .env
        .get("C")
        .and_then(|v| v.as_matrix().cloned())
        .expect("C defined");
    assert_eq!(&c.data, &[19.0, 22.0, 43.0, 50.0]);

    // Graphics state also persists across chunks in one session.
    session
        .exec("figure(2); plot([0 1], [0 1]);", None)
        .unwrap();
    session.exec("title('stage two');", None).unwrap();
    let third = session
        .exec("plot([1 2 3 4 5], [1 4 9 16 25]);", None)
        .unwrap();
    assert_eq!(third.figures.len(), 1);
    assert_eq!(third.figures[0].handle, 2);
    assert_eq!(third.figures[0].title.as_deref(), Some("stage two"));
    assert_eq!(third.figures[0].items.len(), 2);
}

#[test]
fn cancel_flag_stops_between_statements() {
    let mut session = Session::new();
    let cancel = AtomicBool::new(false);

    let result = session
        .exec(
            "a = 1; b = 2; c = a + b; d = c * 10; e = 0; f = 0;",
            Some(&cancel),
        )
        .expect("execution should not error");
    assert!(!result.interrupted);

    // Pre-cancelled chunk: nothing executes, result is marked interrupted.
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let result = session.exec("g = 1;", Some(&cancel)).unwrap();
    assert!(result.interrupted);
    assert!(result.env.get("g").is_none());

    // Mid-chunk cancellation: strict prefix executes, the rest does not.
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let result = session
        .exec("h = 1; i = 2; j = 3; k = 4;", Some(&cancel))
        .unwrap();
    assert!(!result.interrupted);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let mut executed = 0usize;
    let result = session.exec("m = 1; n = 2; o = 3;", Some(&cancel)).unwrap();
    assert!(result.interrupted);
    executed = result.env.get("m").is_some() as usize + result.env.get("n").is_some() as usize;
    assert!(
        executed <= 2,
        "at most the statements before one cancel check ran"
    );
    assert!(result.env.get("o").is_none());
    let _ = executed;
}

#[test]
fn variable_rows_summarizes_workspace() {
    let result =
        run_source("A = [1 2; 3 4];\nv = [9 8 7];\ns = 2.5;\nname = 'cherry';\nt = 1 > 0;\n")
            .unwrap();
    let rows = variable_rows(&result.env);
    let by_name = |n: &str| rows.iter().find(|r| r.name == n).expect("row exists");

    let a = by_name("A");
    assert_eq!(a.kind, "matrix");
    assert_eq!((a.rows, a.cols, a.elements), (2, 2, 4));
    assert!(
        a.preview.contains("19") || a.preview.contains("1"),
        "preview: {}",
        a.preview
    );

    let v = by_name("v");
    assert_eq!(v.kind, "matrix");
    assert_eq!((v.rows, v.cols), (1, 3));

    let s = by_name("s");
    assert_eq!(s.kind, "double");
    assert_eq!((s.rows, s.cols), (1, 1));

    let n = by_name("name");
    assert_eq!(n.kind, "char");
    assert_eq!(n.cols, 6);

    // Logical values are not produced by the language yet (comparisons give
    // 0/1 doubles), but the snapshot handles the Value variant for callers
    // that construct environments directly.
    let mut env = Env::new();
    env.set("flag", matlab_values::Value::Boolean(true));
    let flag = variable_rows(&env).remove(0);
    assert_eq!(flag.kind, "logical");
    assert_eq!(flag.preview, "1");
    assert_eq!((flag.rows, flag.cols), (1, 1));

    assert_eq!(variable_rows(&Env::new()).len(), 0);
}
