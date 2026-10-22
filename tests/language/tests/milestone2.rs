use matlab_runtime::run_source;

#[test]
fn milestone_2_conditional() {
    let r = run_source("x = 10; if x > 5; result = 1; else; result = 0; end").unwrap();
    assert!(
        r.env
            .get("result")
            .and_then(|v| v.as_matrix())
            .map_or(false, |m| m.data[0] == 1.0),
        "conditional result expected 1"
    );
}

#[test]
fn milestone_2_for_loop() {
    let r = run_source("total = 0; for i = 1:5; total = total + i; end").unwrap();
    assert!(
        r.env
            .get("total")
            .and_then(|v| v.as_matrix())
            .map_or(false, |m| m.data[0] == 15.0),
        "for-loop total expected 15"
    );
}

#[test]
fn milestone_2_while_loop() {
    let r =
        run_source("i = 1; total = 0; while i <= 5; total = total + i; i = i + 1; end").unwrap();
    assert!(
        r.env
            .get("total")
            .and_then(|v| v.as_matrix())
            .map_or(false, |m| m.data[0] == 15.0),
        "while-loop total expected 15"
    );
}

#[test]
fn milestone_2_nested_continue() {
    let r = run_source("total = 0; for i = 1:4; if i == 3; continue; end; total = total + i; end")
        .unwrap();
    assert!(
        r.env
            .get("total")
            .and_then(|v| v.as_matrix())
            .map_or(false, |m| m.data[0] == 7.0),
        "continue-loop total expected 7"
    );
}

#[test]
fn milestone_2_negative_step_range() {
    let r = run_source("disp(5:-1:1)").unwrap();
    assert!(!r.stdout.is_empty());
}

#[test]
fn milestone_2_empty_range() {
    let r = run_source("r = 1:-1:5; disp(size(r));").unwrap();
    assert!(!r.stdout.is_empty());
}

#[test]
fn milestone_2_break() {
    let r = run_source("total = 0; for i = 1:10; if i > 4; break; end; total = total + i; end")
        .unwrap();
    assert!(
        r.env
            .get("total")
            .and_then(|v| v.as_matrix())
            .map_or(false, |m| m.data[0] == 10.0),
        "break-loop total expected 10"
    );
}

#[test]
fn milestone_2_break_outside_loop_errors() {
    assert!(run_source("break;").is_err());
}

#[test]
fn milestone_2_string_condition_errors() {
    assert!(run_source("if \"hi\"; disp(1); end").is_err());
}
