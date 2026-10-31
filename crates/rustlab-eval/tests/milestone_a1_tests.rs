//! Tests for Milestone A1:
//! - Context stack for `end` (`A(1:end-B(end))`, `A(end, end)`, `C{end}`, `s.field(end)`)
//! - Boolean mask indexing and masked assignment (`A(A > 2)`, `A(mask) = 0`)
//! - Function handles and lexical closures (`@sin`, `f = @(x) x + a`)
//! - `cellfun`, `arrayfun`, `structfun`
//! - `nargin`, `nargout`, `varargin`, `varargout`
//! - `try ... catch err` MException struct fields (`err.identifier`, `err.message`)

use rustlab_eval::Interpreter;
use rustlab_math::Value;

#[test]
fn test_nested_end_contexts() {
    let mut interp = Interpreter::new();

    // 1. Nested indexing: A(1:end - B(end))
    // A has 10 elements: 10, 20, ..., 100.
    // B has 4 elements: 1, 2, 3, 2.
    // B(end) = B(4) = 2.
    // A(1:end - B(end)) = A(1:10 - 2) = A(1:8).
    let script1 = r#"
        A = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        B = [1, 2, 3, 2];
        res1 = A(1:end - B(end));
    "#;
    interp.eval_script(script1).unwrap();
    let res1 = interp.env.get("res1").unwrap();
    let da1 = res1.to_double_array().unwrap();
    assert_eq!(da1.shape(), &[1, 8]);
    assert_eq!(
        da1.as_slice(),
        &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]
    );

    // 2. 2D subscript indexing: A(end, end)
    let script2 = r#"
        M = [1, 2, 3; 4, 5, 6; 7, 8, 9];
        last_elem = M(end, end);
    "#;
    interp.eval_script(script2).unwrap();
    let last_elem = interp.env.get("last_elem").unwrap();
    assert_eq!(last_elem.as_scalar_f64().unwrap(), 9.0);

    // 3. Cell array dereferencing: C{end}
    let script3 = r#"
        C = {10, 'mid', 999};
        last_cell = C{end};
    "#;
    interp.eval_script(script3).unwrap();
    let last_cell = interp.env.get("last_cell").unwrap();
    assert_eq!(last_cell.as_scalar_f64().unwrap(), 999.0);

    // 4. Struct array field: s.field(end)
    let script4 = r#"
        s.vec = [100, 200, 300, 400];
        last_field = s.vec(end);
    "#;
    interp.eval_script(script4).unwrap();
    let last_field = interp.env.get("last_field").unwrap();
    assert_eq!(last_field.as_scalar_f64().unwrap(), 400.0);
}

#[test]
fn test_boolean_mask_indexing_and_assignment() {
    let mut interp = Interpreter::new();

    // 1. Boolean mask extraction: A(A > 2)
    // MATLAB reference: A = [1, 5, 2, 8]; A(A > 2) -> [5; 8]
    let script1 = r#"
        A = [1, 5, 2, 8];
        greater = A(A > 2);
    "#;
    interp.eval_script(script1).unwrap();
    let greater = interp.env.get("greater").unwrap();
    let da_g = greater.to_double_array().unwrap();
    assert_eq!(da_g.shape(), &[1, 2]);
    assert_eq!(da_g.as_slice(), &[5.0, 8.0]);

    // 2. Boolean mask scalar assignment: A(A > 2) = 0
    let script2 = r#"
        M = [1, 5; 2, 8];
        M(M > 2) = 0;
    "#;
    interp.eval_script(script2).unwrap();
    let m = interp.env.get("M").unwrap();
    let da_m = m.to_double_array().unwrap();
    assert_eq!(da_m.as_slice(), &[1.0, 2.0, 0.0, 0.0]); // column-major [1, 2; 0, 0]
}

#[test]
fn test_function_handles_and_lexical_closures() {
    let mut interp = Interpreter::new();

    // 1. Anonymous function with lexical closure capture
    // When f is defined, 'a' is 10.
    // Later, mutating 'a' to 100 must NOT affect 'f'!
    let script1 = r#"
        a = 10;
        f = @(x) x + a;
        a = 100;
        y = f(5);
    "#;
    interp.eval_script(script1).unwrap();
    let y = interp.env.get("y").unwrap();
    // 5 + 10 = 15 (if closure captured 'a' by value). NOT 105!
    assert_eq!(y.as_scalar_f64().unwrap(), 15.0);

    // 2. Storing function handles in cells and executing them
    let script2 = r#"
        add_two = @(x) x + 2;
        mul_three = @(x) x * 3;
        pipeline = {add_two, mul_three};
        step1 = pipeline{1}(10);
        step2 = pipeline{2}(step1);
    "#;
    interp.eval_script(script2).unwrap();
    let step1 = interp.env.get("step1").unwrap();
    assert_eq!(step1.as_scalar_f64().unwrap(), 12.0);
    let step2 = interp.env.get("step2").unwrap();
    assert_eq!(step2.as_scalar_f64().unwrap(), 36.0);
}

#[test]
fn test_cellfun_arrayfun_structfun() {
    let mut interp = Interpreter::new();

    // 1. cellfun with UniformOutput = true (default)
    let script1 = r#"
        C = {1, 2, 3, 4};
        sq = cellfun(@(x) x.^2, C);
    "#;
    interp.eval_script(script1).unwrap();
    let sq = interp.env.get("sq").unwrap();
    let da_sq = sq.to_double_array().unwrap();
    assert_eq!(da_sq.as_slice(), &[1.0, 4.0, 9.0, 16.0]);

    // 2. cellfun with UniformOutput = false
    let script2 = r#"
        words = {'alpha', 'beta'};
        prefixed = cellfun(@(w) ['prefix_', w], words, 'UniformOutput', false);
        w1 = prefixed{1};
        w2 = prefixed{2};
    "#;
    interp.eval_script(script2).unwrap();
    let w1 = interp.env.get("w1").unwrap();
    assert_eq!(w1, Value::char_vector("prefix_alpha"));
    let w2 = interp.env.get("w2").unwrap();
    assert_eq!(w2, Value::char_vector("prefix_beta"));

    // 3. arrayfun
    let script3 = r#"
        A = [1, 2, 3];
        cube = arrayfun(@(x) x.^3, A);
    "#;
    interp.eval_script(script3).unwrap();
    let cube = interp.env.get("cube").unwrap();
    let da_cube = cube.to_double_array().unwrap();
    assert_eq!(da_cube.as_slice(), &[1.0, 8.0, 27.0]);

    // 4. structfun
    let script4 = r#"
        s.val1 = 10;
        s.val2 = 20;
        s_res = structfun(@(v) v * 2, s);
        r1 = s_res.val1;
        r2 = s_res.val2;
    "#;
    interp.eval_script(script4).unwrap();
    let r1 = interp.env.get("r1").unwrap();
    assert_eq!(r1.as_scalar_f64().unwrap(), 20.0);
    let r2 = interp.env.get("r2").unwrap();
    assert_eq!(r2.as_scalar_f64().unwrap(), 40.0);
}

#[test]
fn test_varargin_varargout_nargin_nargout() {
    let mut interp = Interpreter::new();

    // 1. varargin and nargin
    let script1 = r#"
        function [count, total] = my_sum(varargin)
            count = nargin;
            total = 0;
            for i = 1:count
                total = total + varargin{i};
            end
        end

        [c, t] = my_sum(10, 20, 30, 40);
    "#;
    interp.eval_script(script1).unwrap();
    let c = interp.env.get("c").unwrap();
    assert_eq!(c.as_scalar_f64().unwrap(), 4.0);
    let t = interp.env.get("t").unwrap();
    assert_eq!(t.as_scalar_f64().unwrap(), 100.0);

    // 2. varargout and nargout
    let script2 = r#"
        function varargout = multi_gen(n)
            varargout = {10, 20, 30};
        end

        [o1, o2, o3] = multi_gen(3);
    "#;
    interp.eval_script(script2).unwrap();
    let o1 = interp.env.get("o1").unwrap();
    assert_eq!(o1.as_scalar_f64().unwrap(), 10.0);
    let o2 = interp.env.get("o2").unwrap();
    assert_eq!(o2.as_scalar_f64().unwrap(), 20.0);
    let o3 = interp.env.get("o3").unwrap();
    assert_eq!(o3.as_scalar_f64().unwrap(), 30.0);
}

#[test]
fn test_try_catch_mexception() {
    let mut interp = Interpreter::new();

    let script = r#"
        try
            x = undefined_variable_name;
        catch err
            id = err.identifier;
            msg = err.message;
        end
    "#;
    interp.eval_script(script).unwrap();
    let id = interp.env.get("id").unwrap();
    assert_eq!(id, Value::char_vector("MATLAB:UndefinedFunction"));
    let msg = interp.env.get("msg").unwrap();
    if let Value::Char(ca) = msg {
        let msg_str: String = ca.as_slice().iter().collect();
        assert!(msg_str.contains("undefined_variable_name"));
    } else {
        panic!("Expected char vector message");
    }
}
