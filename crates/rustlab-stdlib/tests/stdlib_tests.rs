use rustlab_stdlib::{eval_matlab, new_interpreter};

#[test]
fn test_array_generators() {
    let z = eval_matlab("Z = zeros(2, 3);").unwrap().unwrap();
    assert_eq!(z.shape(), &[2, 3]);
    let da = z.to_double_array().unwrap();
    assert_eq!(da.as_slice(), &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    let o = eval_matlab("O = ones(3, 2);").unwrap().unwrap();
    assert_eq!(o.shape(), &[3, 2]);
    let da_o = o.to_double_array().unwrap();
    assert_eq!(da_o.as_slice(), &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

    let i = eval_matlab("I = eye(3);").unwrap().unwrap();
    assert_eq!(i.shape(), &[3, 3]);
    let da_i = i.to_double_array().unwrap();
    assert_eq!(da_i.as_slice()[0], 1.0);
    assert_eq!(da_i.as_slice()[1], 0.0);
    assert_eq!(da_i.as_slice()[4], 1.0);

    let lin = eval_matlab("v = linspace(0, 10, 5);").unwrap().unwrap();
    assert_eq!(lin.shape(), &[1, 5]);
    let da_lin = lin.to_double_array().unwrap();
    assert_eq!(da_lin.as_slice(), &[0.0, 2.5, 5.0, 7.5, 10.0]);

    let mag = eval_matlab("M = magic(3);").unwrap().unwrap();
    assert_eq!(mag.shape(), &[3, 3]);
    // Sum of rows and columns in magic(3) must all be 15
    let col_sum = eval_matlab("M = magic(3); s = sum(M);").unwrap().unwrap();
    let da_sum = col_sum.to_double_array().unwrap();
    assert_eq!(da_sum.as_slice(), &[15.0, 15.0, 15.0]);
}

#[test]
fn test_shape_and_inspection() {
    let res = eval_matlab("A = [1 2 3; 4 5 6]; s = size(A);")
        .unwrap()
        .unwrap();
    let da_s = res.to_double_array().unwrap();
    assert_eq!(da_s.as_slice(), &[2.0, 3.0]);

    let len = eval_matlab("A = [1 2 3; 4 5 6]; l = length(A);")
        .unwrap()
        .unwrap();
    assert_eq!(len.as_scalar_f64().unwrap(), 3.0);

    let num = eval_matlab("A = [1 2 3; 4 5 6]; n = numel(A);")
        .unwrap()
        .unwrap();
    assert_eq!(num.as_scalar_f64().unwrap(), 6.0);

    let reshaped = eval_matlab("A = [1 2 3; 4 5 6]; B = reshape(A, 3, 2);")
        .unwrap()
        .unwrap();
    assert_eq!(reshaped.shape(), &[3, 2]);

    let rep = eval_matlab("A = [1 2; 3 4]; B = repmat(A, 2, 3);")
        .unwrap()
        .unwrap();
    assert_eq!(rep.shape(), &[4, 6]);

    let d = eval_matlab("A = [10 0; 0 20]; v = diag(A);")
        .unwrap()
        .unwrap();
    assert_eq!(d.shape(), &[2, 1]);
    let da_d = d.to_double_array().unwrap();
    assert_eq!(da_d.as_slice(), &[10.0, 20.0]);
}

#[test]
fn test_elementwise_elementary_math() {
    let res = eval_matlab("x = abs([-5, 3; -2.5, 0]);").unwrap().unwrap();
    let da = res.to_double_array().unwrap();
    assert_eq!(da.as_slice(), &[5.0, 2.5, 3.0, 0.0]);

    let sq = eval_matlab("s = sqrt([4, 9; 16, 25]);").unwrap().unwrap();
    let da_sq = sq.to_double_array().unwrap();
    assert_eq!(da_sq.as_slice(), &[2.0, 4.0, 3.0, 5.0]);

    // Negative sqrt promotes to complex
    let sq_neg = eval_matlab("z = sqrt(-4);").unwrap().unwrap();
    if let rustlab_math::Value::ComplexDouble(c_arr) = sq_neg {
        assert_eq!(c_arr.as_slice()[0].re, 0.0);
        assert_eq!(c_arr.as_slice()[0].im, 2.0);
    } else {
        panic!("Expected ComplexDouble from sqrt(-4)");
    }

    let trig = eval_matlab("y = sin(0) + cos(0);").unwrap().unwrap();
    assert!((trig.as_scalar_f64().unwrap() - 1.0).abs() < 1e-12);

    let m = eval_matlab("r = mod(17, 5);").unwrap().unwrap();
    assert_eq!(m.as_scalar_f64().unwrap(), 2.0);
}

#[test]
fn test_reductions_and_statistics() {
    let s = eval_matlab("A = [1 2 3; 4 5 6]; s = sum(A, 1);")
        .unwrap()
        .unwrap();
    assert_eq!(s.shape(), &[1, 3]);
    let da_s = s.to_double_array().unwrap();
    assert_eq!(da_s.as_slice(), &[5.0, 7.0, 9.0]);

    let m = eval_matlab("A = [1 2 3; 4 5 6]; m = mean(A, 1);")
        .unwrap()
        .unwrap();
    let da_m = m.to_double_array().unwrap();
    assert_eq!(da_m.as_slice(), &[2.5, 3.5, 4.5]);

    let c = eval_matlab("A = [1 2 3; 4 5 6]; cs = cumsum(A, 1);")
        .unwrap()
        .unwrap();
    let da_c = c.to_double_array().unwrap();
    assert_eq!(da_c.as_slice(), &[1.0, 5.0, 2.0, 7.0, 3.0, 9.0]);

    let d = eval_matlab("x = [1, 3, 6, 10]; y = diff(x);")
        .unwrap()
        .unwrap();
    let da_d = d.to_double_array().unwrap();
    assert_eq!(da_d.as_slice(), &[2.0, 3.0, 4.0]);
}

#[test]
fn test_linear_algebra_and_system_solve() {
    // Solve Ax = b:
    // [2 1; 1 3] * [x1; x2] = [4; 7]
    // Solution is [1; 2] because 2(1) + 1(2) = 4, and 1(1) + 3(2) = 7.
    let sol = eval_matlab("A = [2 1; 1 3]; b = [4; 7]; x = A \\ b;")
        .unwrap()
        .unwrap();
    assert_eq!(sol.shape(), &[2, 1]);
    let da_sol = sol.to_double_array().unwrap();
    assert!((da_sol.as_slice()[0] - 1.0).abs() < 1e-12);
    assert!((da_sol.as_slice()[1] - 2.0).abs() < 1e-12);

    // Determinant
    let det_val = eval_matlab("A = [2 1; 1 3]; d = det(A);").unwrap().unwrap();
    // det([2 1; 1 3]) = 2*3 - 1*1 = 5
    assert!((det_val.as_scalar_f64().unwrap() - 5.0).abs() < 1e-12);

    // Matrix inverse
    let inv_a = eval_matlab("A = [2 1; 1 3]; invA = inv(A); prod = A * invA;")
        .unwrap()
        .unwrap();
    let da_prod = inv_a.to_double_array().unwrap();
    assert!((da_prod.as_slice()[0] - 1.0).abs() < 1e-12);
    assert!(da_prod.as_slice()[1].abs() < 1e-12);
    assert!(da_prod.as_slice()[2].abs() < 1e-12);
    assert!((da_prod.as_slice()[3] - 1.0).abs() < 1e-12);

    // Trace
    let tr = eval_matlab("A = [2 1; 1 3]; t = trace(A);")
        .unwrap()
        .unwrap();
    assert_eq!(tr.as_scalar_f64().unwrap(), 5.0);

    // Vector and Matrix norms
    let n2 = eval_matlab("v = [3, 4]; n = norm(v);").unwrap().unwrap();
    assert!((n2.as_scalar_f64().unwrap() - 5.0).abs() < 1e-12);

    // Cholesky decomposition of symmetric positive definite matrix [4 2; 2 5]
    // R' * R = [4 2; 2 5], R = [2 1; 0 2]
    let r_mat = eval_matlab("A = [4 2; 2 5]; R = chol(A); prod = R' * R;")
        .unwrap()
        .unwrap();
    let da_r_prod = r_mat.to_double_array().unwrap();
    assert!((da_r_prod.as_slice()[0] - 4.0).abs() < 1e-12);
    assert!((da_r_prod.as_slice()[1] - 2.0).abs() < 1e-12);
    assert!((da_r_prod.as_slice()[2] - 2.0).abs() < 1e-12);
    assert!((da_r_prod.as_slice()[3] - 5.0).abs() < 1e-12);
}

#[test]
fn test_multiple_return_values() {
    let mut interp = rustlab_stdlib::new_interpreter();
    interp
        .eval_script("[nrows, ncols] = size(zeros(4, 7));")
        .unwrap();
    let r = interp.env.get("nrows").unwrap();
    let c = interp.env.get("ncols").unwrap();
    assert_eq!(r.as_scalar_f64().unwrap(), 4.0);
    assert_eq!(c.as_scalar_f64().unwrap(), 7.0);

    interp
        .eval_script("[val, idx] = min([40, 10, 50, 20]);")
        .unwrap();
    let v = interp.env.get("val").unwrap();
    let i = interp.env.get("idx").unwrap();
    assert_eq!(v.as_scalar_f64().unwrap(), 10.0);
    assert_eq!(i.as_scalar_f64().unwrap(), 2.0);

    interp.eval_script("[L, U, P] = lu([1 2; 3 4]);").unwrap();
    let l_val = interp.env.get("L").unwrap();
    let u_val = interp.env.get("U").unwrap();
    let p_val = interp.env.get("P").unwrap();
    assert_eq!(l_val.shape(), &[2, 2]);
    assert_eq!(u_val.shape(), &[2, 2]);
    assert_eq!(p_val.shape(), &[2, 2]);
}

#[test]
fn test_signal_and_fft() {
    // FFT of [1, 1, 1, 1]: DC component is 4, all other bins are 0
    let f = eval_matlab("X = fft([1, 1, 1, 1]);").unwrap().unwrap();
    if let rustlab_math::Value::ComplexDouble(da) = f {
        let s = da.as_slice();
        assert!((s[0].re - 4.0).abs() < 1e-12);
        assert!(s[0].im.abs() < 1e-12);
        assert!(s[1].norm() < 1e-12);
        assert!(s[2].norm() < 1e-12);
        assert!(s[3].norm() < 1e-12);
    } else {
        panic!("Expected ComplexDouble from fft");
    }

    // Convolution of [1, 2] and [3, 4, 5]:
    // [1*3, 1*4+2*3, 1*5+2*4, 2*5] = [3, 10, 13, 10]
    let c = eval_matlab("c = conv([1, 2], [3, 4, 5]);")
        .unwrap()
        .unwrap();
    let da_c = c.to_double_array().unwrap();
    assert_eq!(da_c.as_slice(), &[3.0, 10.0, 13.0, 10.0]);
}

#[test]
fn test_search_and_sort() {
    let s = eval_matlab("v = sort([5, 1, 9, 3]);").unwrap().unwrap();
    let da_s = s.to_double_array().unwrap();
    assert_eq!(da_s.as_slice(), &[1.0, 3.0, 5.0, 9.0]);

    let f = eval_matlab("idx = find([0, 5, 0, 8, 9]);")
        .unwrap()
        .unwrap();
    let da_f = f.to_double_array().unwrap();
    assert_eq!(da_f.as_slice(), &[2.0, 4.0, 5.0]);

    let u = eval_matlab("u = unique([4, 1, 4, 2, 1, 3]);")
        .unwrap()
        .unwrap();
    let da_u = u.to_double_array().unwrap();
    assert_eq!(da_u.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_cell_arrays() {
    // Cell literal creation and dereferencing
    let val = eval_matlab("C = {42, 'hello'; [1, 2], 99}; x = C{1, 1};")
        .unwrap()
        .unwrap();
    assert_eq!(val.as_scalar_f64().unwrap(), 42.0);

    let val2 = eval_matlab("C = {42, 'hello'; [1, 2], 99}; y = C{2, 1};")
        .unwrap()
        .unwrap();
    let da = val2.to_double_array().unwrap();
    assert_eq!(da.as_slice(), &[1.0, 2.0]);

    // Cell assignment with dynamic growth
    let val3 = eval_matlab("C = cell(2, 2); C{1, 2} = 100; z = C{1, 2};")
        .unwrap()
        .unwrap();
    assert_eq!(val3.as_scalar_f64().unwrap(), 100.0);
}

#[test]
fn test_struct_arrays() {
    // Struct creation via builtin and dot access
    let s_val = eval_matlab("s = struct('name', 'Alice', 'age', 30); a = s.age;")
        .unwrap()
        .unwrap();
    assert_eq!(s_val.as_scalar_f64().unwrap(), 30.0);

    // Dynamic dot assignment: s.city = 'London'
    let s_mod =
        eval_matlab("s = struct('name', 'Alice'); s.city = 'London'; has = isfield(s, 'city');")
            .unwrap()
            .unwrap();
    assert_eq!(s_mod, rustlab_math::Value::logical_scalar(true));

    // Dynamic dot access: s.('name')
    let s_dyn = eval_matlab("s = struct('score', 95); f = 'score'; res = s.(f);")
        .unwrap()
        .unwrap();
    assert_eq!(s_dyn.as_scalar_f64().unwrap(), 95.0);

    // fieldnames
    let mut interp = rustlab_stdlib::new_interpreter();
    interp
        .eval_script("s = struct('x', 1, 'y', 2); fn = fieldnames(s);")
        .unwrap();
    let fn_val = interp.env.get("fn").unwrap();
    assert_eq!(fn_val.shape(), &[2, 1]);
}

#[test]
fn test_strings_and_chars() {
    // String concatenation with '+'
    let s_concat = eval_matlab("s = \"Hello\" + \" World\";").unwrap().unwrap();
    assert_eq!(s_concat, rustlab_math::Value::string_scalar("Hello World"));

    // String length
    let len = eval_matlab("l = strlength(\"RustLab\");").unwrap().unwrap();
    assert_eq!(len.as_scalar_f64().unwrap(), 7.0);

    // Char vector concatenation: ['abc', 'def']
    let c_concat = eval_matlab("c = ['abc', 'def'];").unwrap().unwrap();
    assert_eq!(c_concat, rustlab_math::Value::char_vector("abcdef"));
}

#[test]
fn test_matio_builtin_load_save() {
    let tmp_path = std::env::temp_dir().join("rustlab_test_load_save.mat");
    let tmp_str = tmp_path.to_str().unwrap().replace('\\', "/");

    let mut interp = new_interpreter();
    let script = format!(
        "s = struct('alpha', 123, 'beta', [1, 2; 3, 4]);\nsave('{tmp_str}', s);\nloaded = load('{tmp_str}');"
    );
    interp.eval_script(&script).unwrap();

    let loaded = interp.env.get("loaded").unwrap();
    if let rustlab_math::Value::Struct(s) = loaded {
        assert!(s.get_field("alpha").is_some());
        assert!(s.get_field("beta").is_some());
        let alpha = s.get_field("alpha").unwrap();
        assert_eq!(alpha.as_scalar_f64().unwrap(), 123.0);
    } else {
        panic!("Expected loaded to be a struct");
    }

    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_b2_inv_and_mldivide_singular_matrix_returns_inf() {
    let mut interp = new_interpreter();
    // inv([1 2; 2 4]) produces warning and returns Inf matrix without error
    let res_inv = interp
        .eval_script("inv([1 2; 2 4])")
        .expect("inv on singular matrix must succeed")
        .expect("val");
    if let rustlab_math::Value::Double(arr) = res_inv {
        assert_eq!(arr.shape(), &[2, 2]);
        for &x in arr.as_slice() {
            assert!(
                x.is_infinite(),
                "inv of singular matrix entries must be Inf"
            );
        }
    } else {
        panic!("expected double matrix");
    }

    // [1 2; 2 4] \ [1; 1] produces warning and returns Inf matrix without error
    let res_mldiv = interp
        .eval_script("[1 2; 2 4] \\ [1; 1]")
        .expect("mldivide on singular matrix must succeed")
        .expect("val");
    if let rustlab_math::Value::Double(arr) = res_mldiv {
        assert_eq!(arr.shape(), &[2, 1]);
        for &x in arr.as_slice() {
            assert!(
                x.is_infinite(),
                "mldivide of singular matrix entries must be Inf"
            );
        }
    } else {
        panic!("expected double matrix");
    }
}
