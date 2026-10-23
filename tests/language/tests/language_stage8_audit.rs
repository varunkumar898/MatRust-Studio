use matlab_runtime::{run_source, Session};

fn scalar(session: &Session, name: &str) -> f64 {
    session
        .env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{} should be a scalar", name))
}

#[test]
fn lang_index_end_context() {
    let r = run_source("A = [1 2 3; 4 5 6]; x = A(1, end);").unwrap();
    let x = r
        .env
        .get("x")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("x is scalar");
    assert_eq!(x, 3.0);
}

#[test]
fn lang_range_and_colon() {
    let r = run_source("A = [10 20 30]; B = A(1:2);").unwrap();
    let b = r.env.get("B").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((b.rows, b.cols), (1, 2));
    assert_eq!(b.data, vec![10.0, 20.0]);
}

#[test]
fn lang_multi_output_order() {
    let mut session = Session::new();
    session
        .exec("function [p, q] = g(x) p = x; q = x + 1; end", None)
        .unwrap();
    session.exec("[first, second] = g(4);", None).unwrap();
    assert_eq!(scalar(&session, "first"), 4.0);
    assert_eq!(scalar(&session, "second"), 5.0);
}

#[test]
fn lang_scope_isolation() {
    let mut session = Session::new();
    session
        .exec("function y = f(x) hidden = 99; y = x + hidden; end", None)
        .unwrap();
    let run = session.exec("result = f(1);", None).unwrap();
    let result = run
        .env
        .get("result")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("result is scalar");
    assert_eq!(result, 100.0);
    assert!(session.env.get("hidden").is_none());
}

#[test]
fn lang_return_exits_function() {
    let mut session = Session::new();
    session
        .exec(
            "function y = f(x) if x > 0, y = 1; return; end; y = 2; end",
            None,
        )
        .unwrap();
    session
        .exec("positive = f(1); negative = f(-1);", None)
        .unwrap();
    assert_eq!(scalar(&session, "positive"), 1.0);
    assert_eq!(scalar(&session, "negative"), 2.0);
}

#[test]
fn lang_switch_case() {
    let r = run_source(
        "switch 2 case 1, disp('one'); case 2, disp('two'); otherwise, disp('other'); end",
    )
    .unwrap();
    assert_eq!(r.stdout, vec!["two".to_string()]);
}

#[test]
fn lang_for_loop() {
    let r = run_source("s = 0; for i = 1:3, s = s + i; end").unwrap();
    let s = r
        .env
        .get("s")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("s is scalar");
    assert_eq!(s, 6.0);
}

#[test]
fn lang_while_break() {
    let r = run_source("i = 0; while true, i = i + 1; if i >= 3, break; end; end").unwrap();
    let i = r
        .env
        .get("i")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("i is scalar");
    assert_eq!(i, 3.0);
}

#[test]
fn lang_function_persistence() {
    let mut session = Session::new();
    session
        .exec("function y = square(x) y = x * x; end", None)
        .unwrap();
    session.exec("r = square(9);", None).unwrap();
    assert_eq!(scalar(&session, "r"), 81.0);
}
