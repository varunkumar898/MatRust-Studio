use rustlab_eval::*;
use rustlab_math::Value;

#[test]
fn test_basic_arithmetic_evaluation() {
    let mut interp = Interpreter::new();
    let val = interp.eval_script("x = 1 + 2 * 3 ^ 2;").unwrap().unwrap();
    // 1 + 2 * 9 = 19
    assert_eq!(val.as_scalar_f64().unwrap(), 19.0);
    assert_eq!(interp.env.get("x").unwrap().as_scalar_f64().unwrap(), 19.0);
}

#[test]
fn test_matrix_creation_and_multiplication() {
    let mut interp = Interpreter::new();
    let code = "
    A = [1 2; 3 4];
    x = [2; 1];
    y = A * x;
    ";
    interp.eval_script(code).unwrap();
    let y = interp.env.get("y").unwrap();
    // [1*2 + 2*1; 3*2 + 4*1] = [4; 10]
    assert_eq!(y.shape(), &[2, 1]);
    if let Value::Double(arr) = y {
        assert_eq!(arr.as_slice(), &[4.0, 10.0]);
    }
}

#[test]
fn test_indexing_with_end() {
    let mut interp = Interpreter::new();
    let code = "
    v = [10, 20, 30, 40, 50];
    last = v(end);
    sub = v(2:end-1);
    ";
    interp.eval_script(code).unwrap();
    let last = interp.env.get("last").unwrap().as_scalar_f64().unwrap();
    assert_eq!(last, 50.0);

    let sub = interp.env.get("sub").unwrap();
    assert_eq!(sub.shape(), &[1, 3]);
    if let Value::Double(arr) = sub {
        assert_eq!(arr.as_slice(), &[20.0, 30.0, 40.0]);
    }
}

#[test]
fn test_array_growth_on_assignment() {
    let mut interp = Interpreter::new();
    let code = "
    x = [1, 2];
    x(5) = 10;
    ";
    interp.eval_script(code).unwrap();
    let x = interp.env.get("x").unwrap();
    assert_eq!(x.shape(), &[1, 5]);
    if let Value::Double(arr) = x {
        assert_eq!(arr.as_slice(), &[1.0, 2.0, 0.0, 0.0, 10.0]);
    }
}

#[test]
fn test_if_elseif_else() {
    let mut interp = Interpreter::new();
    let code = "
    val = 15;
    if val < 10
        res = 1;
    elseif val == 15
        res = 2;
    else
        res = 3;
    end
    ";
    interp.eval_script(code).unwrap();
    assert_eq!(interp.env.get("res").unwrap().as_scalar_f64().unwrap(), 2.0);
}

#[test]
fn test_for_loop_and_column_iteration() {
    let mut interp = Interpreter::new();
    // In MATLAB, 'for col = matrix' iterates over each column of the matrix
    let code = "
    total = 0;
    for i = 1:5
        total = total + i;
    end

    matrix_sum = 0;
    for col = [1 2 3; 4 5 6]
        matrix_sum = matrix_sum + col(1) + col(2);
    end
    ";
    interp.eval_script(code).unwrap();
    // 1+2+3+4+5 = 15
    assert_eq!(
        interp.env.get("total").unwrap().as_scalar_f64().unwrap(),
        15.0
    );
    // (1+4) + (2+5) + (3+6) = 21
    assert_eq!(
        interp
            .env
            .get("matrix_sum")
            .unwrap()
            .as_scalar_f64()
            .unwrap(),
        21.0
    );
}

#[test]
fn test_while_loop_with_break() {
    let mut interp = Interpreter::new();
    let code = "
    n = 1;
    while n < 100
        n = n * 2;
        if n > 20
            break;
        end
    end
    ";
    interp.eval_script(code).unwrap();
    // 1 -> 2 -> 4 -> 8 -> 16 -> 32 -> breaks!
    assert_eq!(interp.env.get("n").unwrap().as_scalar_f64().unwrap(), 32.0);
}

#[test]
fn test_user_defined_functions_and_multiple_returns() {
    let mut interp = Interpreter::new();
    let code = "
    function [s, p] = sum_and_prod(a, b)
        s = a + b;
        p = a * b;
    end

    [my_sum, my_prod] = sum_and_prod(3, 4);
    ";
    interp.eval_script(code).unwrap();
    assert_eq!(
        interp.env.get("my_sum").unwrap().as_scalar_f64().unwrap(),
        7.0
    );
    assert_eq!(
        interp.env.get("my_prod").unwrap().as_scalar_f64().unwrap(),
        12.0
    );
}

#[test]
fn test_global_scoping() {
    let mut interp = Interpreter::new();
    let code = "
    global G;
    G = 100;

    function modify_global()
        global G;
        G = G + 50;
    end

    modify_global();
    ";
    interp.eval_script(code).unwrap();
    assert_eq!(interp.env.get("G").unwrap().as_scalar_f64().unwrap(), 150.0);
}

#[test]
fn test_dos_runtime_recursion_limit() {
    let mut interp = Interpreter::new();
    let code = "
    function r = rec(x)
        r = rec(x + 1);
    end
    rec(1);
    ";
    let res = interp.eval_script(code);
    assert!(res.is_err(), "infinite recursion must return an error");
    match res.unwrap_err() {
        rustlab_eval::EvalError::UserError { identifier, .. } => {
            assert_eq!(identifier, "MATLAB:recursionLimitExceeded");
        }
        other => panic!("expected MATLAB:recursionLimitExceeded, got {other:?}"),
    }
}
