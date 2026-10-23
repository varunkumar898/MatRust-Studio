//! Gate 0 user-defined function coverage.
//!
//! These tests exercise real parsed function definitions and assert numeric
//! results (not merely that a variable exists).

use matlab_runtime::{run_source, Session};

fn scalar(session: &Session, name: &str) -> f64 {
    let matrix = session
        .env
        .get(name)
        .and_then(|value| value.as_matrix())
        .unwrap_or_else(|| panic!("variable '{}' should be a matrix", name));
    matrix
        .as_scalar()
        .unwrap_or_else(|| panic!("variable '{}' should be scalar", name))
}

fn assert_scalar_close(session: &Session, name: &str, expected: f64) {
    let actual = scalar(session, name);
    assert!(
        (actual - expected).abs() < 1e-12,
        "{} should be {}, got {}",
        name,
        expected,
        actual
    );
}

#[test]
fn stage3_single_output_call() {
    let mut session = Session::new();
    session
        .exec("function y = triple(x) y = 3 * x; end", None)
        .expect("function definition parses and installs");
    session.exec("A = triple(4);", None).expect("function call");

    assert_scalar_close(&session, "A", 12.0);
}

#[test]
fn stage3_multi_output_assigned() {
    let mut session = Session::new();
    session
        .exec("function [p, q] = g(x) p = x; q = x + 1; end", None)
        .expect("multi-output definition");
    session
        .exec("[p, q] = g(5);", None)
        .expect("multi-output call");

    assert_scalar_close(&session, "p", 5.0);
    assert_scalar_close(&session, "q", 6.0);
}

#[test]
fn stage3_nargout_reflects_caller() {
    let mut session = Session::new();
    session
        .exec(
            "function [a, b] = probe(x) a = nargout; b = nargin; end",
            None,
        )
        .expect("definition");
    session
        .exec("[requested, supplied] = probe(7);", None)
        .expect("two-output call");
    assert_scalar_close(&session, "requested", 2.0);
    assert_scalar_close(&session, "supplied", 1.0);

    // A single-output call asks for one output, not the declared maximum.
    session
        .exec("only = probe(7);", None)
        .expect("one-output call");
    assert_scalar_close(&session, "only", 1.0);
}

#[test]
fn stage3_missing_argument_errors() {
    let mut session = Session::new();
    session
        .exec("function y = f(x) y = x; end", None)
        .expect("definition");
    let err = session
        .exec("f();", None)
        .expect_err("missing argument must fail");
    assert_eq!(err.message, "Not enough input arguments.");
    assert_eq!(err.severity, matlab_diagnostics::DiagnosticSeverity::Error);
}

#[test]
fn stage3_scope_isolation_local_env() {
    let mut session = Session::new();
    session
        .exec("function y = f(x) local_var = 99; y = x; end", None)
        .expect("definition");
    let run = session.exec("f(1)", None).expect("call");

    assert!(
        session.env.get("local_var").is_none(),
        "function-local variables must not leak into the caller"
    );
    assert_eq!(run.outputs.len(), 1);
    let returned = run.outputs[0]
        .value
        .as_matrix()
        .and_then(|m| m.as_scalar())
        .expect("function return is a scalar");
    assert_eq!(returned, 1.0);
}

#[test]
fn stage3_callee_sees_session_functions() {
    let mut session = Session::new();
    session
        .exec(
            "function y = helper(x) y = x + 1; end\n\
             function y = outer(x) y = helper(x) * 2; end",
            None,
        )
        .expect("definitions");
    session.exec("r = outer(4);", None).expect("nested call");

    assert_scalar_close(&session, "r", 10.0);
}

#[test]
fn stage3_factorial_value() {
    let mut session = Session::new();
    session
        .exec(
            "function y = fact(n) if n <= 1 y = 1; else y = n * fact(n-1); end end",
            None,
        )
        .expect("recursive definition");
    session.exec("r = fact(5);", None).expect("recursive call");

    assert_scalar_close(&session, "r", 120.0);
}

#[test]
fn stage3_factorial_20_on_large_interpreter_stack() {
    let mut session = Session::new();
    session
        .exec(
            "function y = fact(n) if n <= 1 y = 1; else y = n * fact(n-1); end end",
            None,
        )
        .expect("recursive definition");
    session
        .exec("r = fact(20);", None)
        .expect("20 nested calls");

    // 20! is exactly representable as an f64 (2^18 times an odd integer
    // below 2^53), so this is an exact-value assertion, not a tolerance.
    assert_scalar_close(&session, "r", 2_432_902_008_176_640_000.0);
}

#[test]
fn stage3_fibonacci_value() {
    let mut session = Session::new();
    session
        .exec(
            "function y = fib(n) if n <= 1 y = n; else y = fib(n-1) + fib(n-2); end end",
            None,
        )
        .expect("recursive definition");
    session.exec("r = fib(10);", None).expect("recursive call");

    assert_scalar_close(&session, "r", 55.0);
}

#[test]
fn stage3_redefinition_uses_latest_definition() {
    let mut session = Session::new();
    session
        .exec("function y = f(x) y = 2 * x; end", None)
        .expect("first definition");
    session.exec("a = f(3);", None).expect("first call");

    session
        .exec("function y = f(x) y = 4 * x; end", None)
        .expect("redefinition");
    session.exec("b = f(3);", None).expect("second call");

    assert_scalar_close(&session, "a", 6.0);
    assert_scalar_close(&session, "b", 12.0);
}

#[test]
fn stage3_recursion_limit_returns_error() {
    let mut session = Session::new();
    session
        .exec(
            "function y = deep_rec(x) if x <= 0 y = 0; else y = deep_rec(x-1) + 1; end end",
            None,
        )
        .expect("recursive definition");
    let err = session
        .exec("deep_rec(1000);", None)
        .expect_err("runaway recursion must hit the guard");
    assert_eq!(
        err.message,
        format!(
            "Maximum recursion limit ({}) exceeded.",
            matlab_runtime::interp::MAX_RECURSION_DEPTH
        )
    );
}

#[test]
fn stage3_persistence_across_exec_calls() {
    let mut session = Session::new();
    session.exec("A = 42;", None).expect("first exec");
    session.exec("B = A + 1;", None).expect("second exec");
    assert_scalar_close(&session, "A", 42.0);
    assert_scalar_close(&session, "B", 43.0);

    let run = run_source(
        "function y = add_one(x) y = x + 1; end\n\
         C = add_one(41);",
    )
    .expect("one-shot execution");
    let c = run
        .env
        .get("C")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("C is scalar");
    assert_eq!(c, 42.0);
}
