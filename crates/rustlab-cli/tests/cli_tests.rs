use rustlab_cli::format::format_eval_error;
use rustlab_cli::repl::is_incomplete_input;
use rustlab_stdlib::new_interpreter;

#[test]
fn test_incomplete_input_detection() {
    // Blocks
    assert!(is_incomplete_input("for i = 1:10"));
    assert!(is_incomplete_input("if x > 0\n  y = 1;"));
    assert!(is_incomplete_input("while true"));
    assert!(is_incomplete_input("function y = my_fn(x)"));

    // Completed blocks
    assert!(!is_incomplete_input("for i = 1:10\n  x = i;\nend"));
    assert!(!is_incomplete_input("if true\n  y = 1;\nend"));

    // Unclosed matrix and cell brackets
    assert!(is_incomplete_input("A = [1, 2;"));
    assert!(is_incomplete_input("C = {42, 'text'"));
    assert!(is_incomplete_input("x = (1 + 2 *"));

    // Closed expressions
    assert!(!is_incomplete_input("A = [1, 2; 3, 4];"));
    assert!(!is_incomplete_input("C = {42, 'text'};"));
    assert!(!is_incomplete_input("x = (1 + 2) * 3;"));

    // Ellipsis line continuation
    assert!(is_incomplete_input("x = 1 + ..."));
    assert!(is_incomplete_input("A = [1, 2, ..."));
}

#[test]
fn test_diagnostic_error_formatting() {
    let mut interp = new_interpreter();

    // 1. Undefined variable
    let code1 = "y = 10 + unknown_var * 2;";
    let err1 = interp.eval_script(code1).unwrap_err();
    let formatted1 = format_eval_error(&err1, code1);

    assert!(formatted1.contains("Undefined function or variable 'unknown_var'"));
    assert!(formatted1.contains("Line 1"));
    assert!(formatted1.contains("unknown_var"));
    assert!(formatted1.contains("^^^^^^^^^^^"));

    // 2. Syntax error
    let code2 = "x = 1 + ;";
    let err2 = interp.eval_script(code2).unwrap_err();
    let formatted2 = format_eval_error(&err2, code2);

    assert!(formatted2.contains("Syntax error"));
    assert!(formatted2.contains("Line 1"));

    // 3. Mathematical dimension mismatch
    let code3 = "M = [1, 2] + [1, 2, 3];";
    let err3 = interp.eval_script(code3).unwrap_err();
    let formatted3 = format_eval_error(&err3, code3);

    assert!(formatted3.contains("Line 1"));
}

#[test]
fn test_semicolon_output_suppression() {
    let mut interp = new_interpreter();

    // Statement with semicolon: output suppressed
    interp.eval_script("x = 42;").unwrap();
    assert!(interp.output_log.is_empty());

    // Statement without semicolon: output formatted
    interp.eval_script("y = 100").unwrap();
    assert_eq!(interp.output_log.len(), 1);
    assert!(interp.output_log[0].contains("y ="));
    assert!(interp.output_log[0].contains("100"));

    interp.output_log.clear();

    // Bare expression without semicolon
    interp.eval_script("2 + 3").unwrap();
    assert_eq!(interp.output_log.len(), 1);
    assert!(interp.output_log[0].contains("ans ="));
    assert!(interp.output_log[0].contains("5"));
}
