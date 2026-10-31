#![allow(clippy::collapsible_match)]

use rustlab_parse::*;

#[test]
fn test_operator_precedence() {
    let stmts = parse("y = 1 + 2 * 3 ^ 4;").unwrap();
    assert_eq!(stmts.len(), 1);

    // Should parse as: 1 + (2 * (3 ^ 4))
    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::Binary {
            op, left, right, ..
        } = value
        {
            assert_eq!(*op, BinaryOp::Add);
            assert!(matches!(**left, Expr::Number(1.0, _)));
            if let Expr::Binary {
                op: op2,
                left: left2,
                right: right2,
                ..
            } = &**right
            {
                assert_eq!(*op2, BinaryOp::Mul);
                assert!(matches!(**left2, Expr::Number(2.0, _)));
                if let Expr::Binary {
                    op: op3,
                    left: left3,
                    right: right3,
                    ..
                } = &**right2
                {
                    assert_eq!(*op3, BinaryOp::Pow);
                    assert!(matches!(**left3, Expr::Number(3.0, _)));
                    assert!(matches!(**right3, Expr::Number(4.0, _)));
                } else {
                    panic!("Expected power binary expr");
                }
            } else {
                panic!("Expected mul binary expr");
            }
        } else {
            panic!("Expected add binary expr");
        }
    } else {
        panic!("Expected assignment");
    }
}

#[test]
fn test_power_right_associativity() {
    // 2 ^ 3 ^ 4 in MATLAB is 2 ^ (3 ^ 4)
    let stmts = parse("x = 2 ^ 3 ^ 4;").unwrap();
    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::Binary {
            op, left, right, ..
        } = value
        {
            assert_eq!(*op, BinaryOp::Pow);
            assert!(matches!(**left, Expr::Number(2.0, _)));
            if let Expr::Binary {
                op: r_op,
                left: r_left,
                right: r_right,
                ..
            } = &**right
            {
                assert_eq!(*r_op, BinaryOp::Pow);
                assert!(matches!(**r_left, Expr::Number(3.0, _)));
                assert!(matches!(**r_right, Expr::Number(4.0, _)));
            } else {
                panic!("Expected right-associative power");
            }
        }
    }
}

#[test]
fn test_colon_range_expressions() {
    let stmts = parse("v = 1:10; r = 1:0.5:10;").unwrap();
    assert_eq!(stmts.len(), 2);

    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::Range {
            start, step, end, ..
        } = value
        {
            assert!(matches!(**start, Expr::Number(1.0, _)));
            assert!(step.is_none());
            assert!(matches!(**end, Expr::Number(10.0, _)));
        } else {
            panic!("Expected two-argument range");
        }
    }

    if let Stmt::Assignment { value, .. } = &stmts[1] {
        if let Expr::Range {
            start, step, end, ..
        } = value
        {
            assert!(matches!(**start, Expr::Number(1.0, _)));
            assert!(matches!(
                step.as_ref().map(|s| &**s),
                Some(Expr::Number(0.5, _))
            ));
            assert!(matches!(**end, Expr::Number(10.0, _)));
        } else {
            panic!("Expected three-argument range");
        }
    }
}

#[test]
fn test_matrix_literals_and_whitespace() {
    // Standard matrix
    let stmts = parse("A = [1, 2; 3, 4];").unwrap();
    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::Matrix { rows, .. } = value {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 2);
        } else {
            panic!("Expected matrix literal");
        }
    }

    // Space separated elements
    let stmts2 = parse("B = [10 20 30; 40 50 60];").unwrap();
    if let Stmt::Assignment { value, .. } = &stmts2[0] {
        if let Expr::Matrix { rows, .. } = value {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 3);
            assert_eq!(rows[1].len(), 3);
        } else {
            panic!("Expected matrix literal");
        }
    }

    // Whitespace disambiguation:
    // [1 - 2] is a 1x1 matrix: [ (1 - 2) ]
    // [1 -2] is a 1x2 matrix: [ 1, -2 ]
    let m1 = parse("x = [1 - 2];").unwrap();
    if let Stmt::Assignment { value, .. } = &m1[0] {
        if let Expr::Matrix { rows, .. } = value {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 1);
            assert!(matches!(
                &rows[0][0],
                Expr::Binary {
                    op: BinaryOp::Sub,
                    ..
                }
            ));
        }
    }

    let m2 = parse("y = [1 -2];").unwrap();
    if let Stmt::Assignment { value, .. } = &m2[0] {
        if let Expr::Matrix { rows, .. } = value {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
            assert!(matches!(&rows[0][0], Expr::Number(1.0, _)));
            assert!(matches!(
                &rows[0][1],
                Expr::Unary {
                    op: UnaryOp::Minus,
                    ..
                }
            ));
        }
    }
}

#[test]
fn test_indexing_and_end() {
    let stmts = parse("val = A(2:end, :);").unwrap();
    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::Index { callee, args, .. } = value {
            assert!(matches!(**callee, Expr::Ident(ref s, _) if s == "A"));
            assert_eq!(args.len(), 2);
            if let Expr::Range { start, end, .. } = &args[0] {
                assert!(matches!(**start, Expr::Number(2.0, _)));
                assert!(matches!(**end, Expr::End(_)));
            } else {
                panic!("Expected range with end");
            }
            assert!(matches!(args[1], Expr::Colon(_)));
        } else {
            panic!("Expected index expr");
        }
    }
}

#[test]
fn test_multi_assignment() {
    let stmts = parse("[m, n] = size(A);").unwrap();
    if let Stmt::Assignment { target, value, .. } = &stmts[0] {
        if let AssignTarget::MultiAssign(targets, _) = target {
            assert_eq!(targets.len(), 2);
            assert!(matches!(&targets[0], AssignTarget::Ident(s, _) if s == "m"));
            assert!(matches!(&targets[1], AssignTarget::Ident(s, _) if s == "n"));
        } else {
            panic!("Expected multi assignment target");
        }
        assert!(matches!(value, Expr::Index { .. }));
    }
}

#[test]
fn test_control_flow() {
    let code = "
    if x > 0
        y = 1;
    elseif x == 0
        y = 0;
    else
        y = -1;
    end

    for i = 1:5
        disp(i);
    end

    while count < 10
        count = count + 1;
    end
    ";
    let stmts = parse(code).unwrap();
    assert_eq!(stmts.len(), 3);
    assert!(matches!(stmts[0], Stmt::If { .. }));
    assert!(matches!(stmts[1], Stmt::For { .. }));
    assert!(matches!(stmts[2], Stmt::While { .. }));
}

#[test]
fn test_function_def() {
    let code = "
    function [out1, out2] = my_add_mul(a, b)
        out1 = a + b;
        out2 = a * b;
    end
    ";
    let stmts = parse(code).unwrap();
    assert_eq!(stmts.len(), 1);
    if let Stmt::Function(f) = &stmts[0] {
        assert_eq!(f.name, "my_add_mul");
        assert_eq!(f.inputs, vec!["a", "b"]);
        assert_eq!(f.outputs, vec!["out1", "out2"]);
        assert_eq!(f.body.len(), 2);
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_anonymous_function() {
    let stmts = parse("f = @(x, y) x.^2 + y.^2;").unwrap();
    if let Stmt::Assignment { value, .. } = &stmts[0] {
        if let Expr::AnonymousFunction { params, body, .. } = value {
            assert_eq!(params, &vec!["x".to_string(), "y".to_string()]);
            assert!(matches!(
                **body,
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        } else {
            panic!("Expected anonymous function");
        }
    }
}

#[test]
fn test_parser_nesting_depth_limit() {
    use rustlab_parse::error::ParseError;

    // Construct 100,000 nested parentheses: "((((... 1 ...))))"
    let n = 100_000;
    let mut s = String::with_capacity(n * 2 + 10);
    for _ in 0..n {
        s.push('(');
    }
    s.push('1');
    for _ in 0..n {
        s.push(')');
    }

    let res = parse(&s);
    assert!(res.is_err(), "100k nested parens must return an error");
    match res.unwrap_err() {
        ParseError::RecursionLimitExceeded { .. } => {}
        other => panic!("expected RecursionLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_dos_100k_terms_addition() {
    use rustlab_parse::error::ParseError;
    // 100k terms: 1 + 1 + 1 + ...
    let n = 100_000;
    let mut s = String::with_capacity(n * 4);
    s.push('1');
    for _ in 0..n {
        s.push_str(" + 1");
    }

    let res = parse(&s);
    assert!(res.is_err(), "100k terms addition must return an error");
    match res.unwrap_err() {
        ParseError::RecursionLimitExceeded { .. } => {}
        other => panic!("expected RecursionLimitExceeded, got {other:?}"),
    }
}

#[test]
fn test_dos_nested_brackets_and_braces() {
    use rustlab_parse::error::ParseError;
    // Nested brackets [ [ [ ... 1 ... ] ] ]
    let n = 500;
    let mut s_mat = String::new();
    for _ in 0..n {
        s_mat.push('[');
    }
    s_mat.push('1');
    for _ in 0..n {
        s_mat.push(']');
    }
    let res_mat = parse(&s_mat);
    assert!(res_mat.is_err(), "nested brackets must return an error");
    assert!(matches!(
        res_mat.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));

    // Nested braces { { { ... 1 ... } } }
    let mut s_cell = String::new();
    for _ in 0..n {
        s_cell.push('{');
    }
    s_cell.push('1');
    for _ in 0..n {
        s_cell.push('}');
    }
    let res_cell = parse(&s_cell);
    assert!(res_cell.is_err(), "nested braces must return an error");
    assert!(matches!(
        res_cell.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));
}

#[test]
fn test_dos_nested_anon_and_unary_minus() {
    use rustlab_parse::error::ParseError;
    // Nested @(x) @(x) ... 1
    let n = 500;
    let mut s_anon = String::new();
    for _ in 0..n {
        s_anon.push_str("@(x) ");
    }
    s_anon.push('1');
    let res_anon = parse(&s_anon);
    assert!(res_anon.is_err(), "nested @() must return an error");
    assert!(matches!(
        res_anon.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));

    // Nested unary minus - - - ... 1
    let mut s_minus = String::new();
    for _ in 0..n {
        s_minus.push_str("- ");
    }
    s_minus.push('1');
    let res_minus = parse(&s_minus);
    assert!(
        res_minus.is_err(),
        "nested unary minus must return an error"
    );
    assert!(matches!(
        res_minus.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));
}

#[test]
fn test_dos_long_dot_and_call_chains() {
    use rustlab_parse::error::ParseError;
    // Long .a.a.a chain
    let n = 500;
    let mut s_dot = String::from("x");
    for _ in 0..n {
        s_dot.push_str(".a");
    }
    let res_dot = parse(&s_dot);
    assert!(res_dot.is_err(), "long .a chain must return an error");
    assert!(matches!(
        res_dot.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));

    // Long (1)(1)(1) chain
    let mut s_call = String::from("x");
    for _ in 0..n {
        s_call.push_str("(1)");
    }
    let res_call = parse(&s_call);
    assert!(res_call.is_err(), "long (1) chain must return an error");
    assert!(matches!(
        res_call.unwrap_err(),
        ParseError::RecursionLimitExceeded { .. }
    ));
}
