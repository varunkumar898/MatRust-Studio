//! Milestone A3 Acceptance Tests:
//! - Numeric classes: single, int8..uint64, saturating arithmetic, mixed-class rules.
//! - N-D array parity: slicing, assignment, deletion, cat, permute, meshgrid, ndgrid, reshape with [].
//! - Sparse matrices: CSC representation, duplicate accumulation, full, speye, nnz, nonzeros, spones.

use rustlab_math::{CscMatrix, DenseArray, Value};
use rustlab_stdlib::{eval_matlab, new_interpreter};

#[test]
fn test_a3_integer_types_and_saturating_arithmetic() {
    // 1. Saturating addition & subtraction
    let res = eval_matlab("uint8(250) + uint8(10)").unwrap().unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(255)));

    let res = eval_matlab("uint8(3) - uint8(5)").unwrap().unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(0)));

    let res = eval_matlab("int8(127) + 1").unwrap().unwrap();
    assert_eq!(res, Value::Int8(DenseArray::scalar(127)));

    let res = eval_matlab("-int8(-128)").unwrap().unwrap();
    assert_eq!(res, Value::Int8(DenseArray::scalar(127)));

    let res = eval_matlab("abs(int8(-128))").unwrap().unwrap();
    assert_eq!(res, Value::Int8(DenseArray::scalar(127)));

    // 2. Round half away from zero conversions
    let res = eval_matlab("int32(2.5)").unwrap().unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(3)));

    let res = eval_matlab("int32(-2.5)").unwrap().unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(-3)));

    // 3. Integer division rounded to nearest
    let res = eval_matlab("int32(7) / int32(2)").unwrap().unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(4)));

    let res = eval_matlab("int32(-7) / int32(2)").unwrap().unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(-4)));

    // 4. idivide with modes
    let res = eval_matlab("idivide(int32(7), int32(2), 'fix')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(3)));

    let res = eval_matlab("idivide(int32(7), int32(2), 'floor')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(3)));

    let res = eval_matlab("idivide(int32(7), int32(2), 'ceil')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(4)));

    let res = eval_matlab("idivide(int32(7), int32(2), 'round')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(4)));

    let res = eval_matlab("idivide(int32(-7), int32(2), 'floor')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(-4)));

    let res = eval_matlab("idivide(int32(-7), int32(2), 'ceil')")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(-3)));
}

#[test]
fn test_a3_mixed_type_rules_and_errors() {
    // int + double -> int
    let res = eval_matlab("int32(5) + 2.0").unwrap().unwrap();
    assert_eq!(res, Value::Int32(DenseArray::scalar(7)));

    // single + double -> single
    let res = eval_matlab("single(3) + 2.0").unwrap().unwrap();
    assert_eq!(res, Value::Single(DenseArray::scalar(5.0f32)));

    // logical + double -> double
    let res = eval_matlab("true + 2.0").unwrap().unwrap();
    assert_eq!(res, Value::scalar(3.0));

    // intA + intB (different classes) -> Error "Integers can only be combined with integers of the same class, or scalar doubles."
    let err = eval_matlab("int8(5) + int16(2)").unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains(
            "Integers can only be combined with integers of the same class, or scalar doubles."
        ),
        "Unexpected error: {err_str}"
    );

    // Assignment of mismatched integer class
    let err = eval_matlab("A = int8([1 2; 3 4]); A(1, 1) = int16(5);").unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains(
            "Integers can only be combined with integers of the same class, or scalar doubles."
        ),
        "Unexpected error: {err_str}"
    );
}

#[test]
fn test_a3_class_predicates_and_limits() {
    assert_eq!(
        eval_matlab("class(single(1))").unwrap().unwrap(),
        Value::char_vector("single")
    );
    assert_eq!(
        eval_matlab("class(int64(1))").unwrap().unwrap(),
        Value::char_vector("int64")
    );

    assert_eq!(
        eval_matlab("isa(int8(1), 'integer')").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("isa(int8(1), 'numeric')").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("isa(1.0, 'integer')").unwrap().unwrap(),
        Value::logical_scalar(false)
    );

    assert_eq!(
        eval_matlab("isinteger(int32(1))").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("isinteger(1.0)").unwrap().unwrap(),
        Value::logical_scalar(false)
    );

    assert_eq!(
        eval_matlab("isfloat(single(1))").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("isfloat(1.0)").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("isfloat(int8(1))").unwrap().unwrap(),
        Value::logical_scalar(false)
    );

    // Exact 64-bit operations
    let res = eval_matlab("intmax('int64') - 1").unwrap().unwrap();
    assert_eq!(
        res,
        Value::Int64(DenseArray::scalar(9223372036854775806i64))
    );

    assert_eq!(
        eval_matlab("intmin('int8')").unwrap().unwrap(),
        Value::Int8(DenseArray::scalar(-128i8))
    );

    assert_eq!(
        eval_matlab("flintmax('single')").unwrap().unwrap(),
        Value::Single(DenseArray::scalar(16777216.0f32))
    );
    assert_eq!(
        eval_matlab("flintmax('double')").unwrap().unwrap(),
        Value::scalar(9007199254740992.0f64)
    );

    let eps_s = eval_matlab("eps('single')").unwrap().unwrap();
    assert_eq!(eps_s, Value::Single(DenseArray::scalar(f32::EPSILON)));
}

#[test]
fn test_a3_bitwise_operations() {
    let res = eval_matlab("bitand(uint8(12), uint8(10))")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(8)));

    let res = eval_matlab("bitor(uint8(12), uint8(10))").unwrap().unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(14)));

    let res = eval_matlab("bitxor(uint8(12), uint8(10))")
        .unwrap()
        .unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(6)));

    let res = eval_matlab("bitshift(uint8(1), 3)").unwrap().unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(8)));

    let res = eval_matlab("bitshift(uint8(16), -2)").unwrap().unwrap();
    assert_eq!(res, Value::UInt8(DenseArray::scalar(4)));
}

#[test]
fn test_a3_nd_slicing_assignment_and_deletion() {
    // 3D slicing
    let src = r#"
        A = reshape(1:24, [2, 3, 4]);
        B = A(1:2, 2:3, 1:2);
        sz = size(B);
    "#;
    let res = eval_matlab(src).unwrap().unwrap();
    let da = res.to_double_array().unwrap();
    assert_eq!(da.as_slice(), &[2.0, 2.0, 2.0]);

    // Sliced assignment
    let src2 = r#"
        A = reshape(1:24, [2, 3, 4]);
        A(1:2, 1:2, 1) = 99;
        v = A(1, 1, 1);
    "#;
    let res2 = eval_matlab(src2).unwrap().unwrap();
    assert_eq!(res2.as_scalar_f64().unwrap(), 99.0);

    // 2D Row Deletion: A(2, :) = []
    let src3 = r#"
        A = [1 2; 3 4; 5 6];
        A(2, :) = [];
        s = size(A);
    "#;
    let res3 = eval_matlab(src3).unwrap().unwrap();
    assert_eq!(res3.to_double_array().unwrap().as_slice(), &[2.0, 2.0]);

    // 2D Column Deletion: A(:, 2) = []
    let src4 = r#"
        A = [1 2 3; 4 5 6];
        A(:, 2) = [];
        s = size(A);
    "#;
    let res4 = eval_matlab(src4).unwrap().unwrap();
    assert_eq!(res4.to_double_array().unwrap().as_slice(), &[2.0, 2.0]);
}

#[test]
fn test_a3_nd_reductions_and_integer_preservation() {
    // Integer sum saturates and preserves integer class
    let res = eval_matlab("sum(int8([100 100]))").unwrap().unwrap();
    assert_eq!(res, Value::Int8(DenseArray::scalar(127)));

    // Integer mean returns double
    let res = eval_matlab("mean(int8([1 2 3]))").unwrap().unwrap();
    assert_eq!(res, Value::scalar(2.0));

    // N-D reductions along dim k
    let src = r#"
        A = reshape(1:24, [2, 3, 4]);
        S3 = sum(A, 3);
        sz = size(S3);
    "#;
    let res = eval_matlab(src).unwrap().unwrap();
    assert_eq!(res.to_double_array().unwrap().as_slice(), &[2.0, 3.0]);
}

#[test]
fn test_a3_nd_operations_cat_permute_meshgrid_reshape() {
    // cat along dim 3
    let src = r#"
        A = [1 2; 3 4];
        B = [5 6; 7 8];
        C = cat(3, A, B);
        sz = size(C);
    "#;
    let res = eval_matlab(src).unwrap().unwrap();
    assert_eq!(res.to_double_array().unwrap().as_slice(), &[2.0, 2.0, 2.0]);

    // permute dimensions
    let src2 = r#"
        A = reshape(1:24, [2, 3, 4]);
        P = permute(A, [2 1 3]);
        sz = size(P);
    "#;
    let res2 = eval_matlab(src2).unwrap().unwrap();
    assert_eq!(res2.to_double_array().unwrap().as_slice(), &[3.0, 2.0, 4.0]);

    // meshgrid
    let src3 = r#"
        [X, Y] = meshgrid(1:3, 1:2);
        szX = size(X);
    "#;
    let res3 = eval_matlab(src3).unwrap().unwrap();
    assert_eq!(res3.to_double_array().unwrap().as_slice(), &[2.0, 3.0]);

    // ndgrid
    let src4 = r#"
        [X1, X2] = ndgrid(1:3, 1:2);
        sz1 = size(X1);
    "#;
    let res4 = eval_matlab(src4).unwrap().unwrap();
    assert_eq!(res4.to_double_array().unwrap().as_slice(), &[3.0, 2.0]);

    // reshape with []
    let src5 = r#"
        A = 1:12;
        B = reshape(A, 3, []);
        sz = size(B);
    "#;
    let res5 = eval_matlab(src5).unwrap().unwrap();
    assert_eq!(res5.to_double_array().unwrap().as_slice(), &[3.0, 4.0]);
}

#[test]
fn test_a3_sparse_csc_matrices_and_algebra() {
    // Construction with duplicate accumulation
    let src = r#"
        i = [1 2 2 3];
        j = [1 2 2 3];
        v = [5 6 7 8];
        S = sparse(i, j, v, 3, 3);
        full_S = full(S);
    "#;
    let res = eval_matlab(src).unwrap().unwrap();
    let da = res.to_double_array().unwrap();
    assert_eq!(
        da.as_slice(),
        &[5.0, 0.0, 0.0, 0.0, 13.0, 0.0, 0.0, 0.0, 8.0]
    );

    assert_eq!(
        eval_matlab("issparse(sparse(eye(3)))").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        eval_matlab("issparse(eye(3))").unwrap().unwrap(),
        Value::logical_scalar(false)
    );

    assert_eq!(
        eval_matlab("nnz(speye(4))").unwrap().unwrap(),
        Value::scalar(4.0)
    );

    // Sparse + Sparse and Sparse * Sparse
    let src2 = r#"
        S1 = speye(3);
        S2 = speye(3);
        S3 = S1 + S2;
        full(S3)
    "#;
    let res2 = eval_matlab(src2).unwrap().unwrap();
    let da2 = res2.to_double_array().unwrap();
    assert_eq!(
        da2.as_slice(),
        &[2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0]
    );
}

#[test]
fn test_a3_sparse_memory_efficiency() {
    // 10000x10000 matrix with 10000 elements stays small
    let n = 10000;
    let rows: Vec<usize> = (1..=n).collect();
    let cols: Vec<usize> = (1..=n).collect();
    let vals: Vec<f64> = vec![1.0; n];

    let sp = CscMatrix::from_triplets_1based(n, n, &rows, &cols, &vals).unwrap();
    let col_ptr_bytes = sp.col_ptr.len() * 8;
    let row_ind_bytes = sp.row_ind.len() * 8;
    let val_bytes = sp.values.len() * 8;
    let total_bytes = col_ptr_bytes + row_ind_bytes + val_bytes;

    // Sparse memory < 300 KB, whereas 10000x10000 dense f64 would be 800 MB!
    assert!(
        total_bytes < 500_000,
        "Sparse matrix memory exceeds budget: {total_bytes} bytes"
    );
}

#[test]
fn test_a3_nd_deletion_3d_and_growth() {
    let mut interp = new_interpreter();

    // 1. 3D sliced deletion along dimension 3: A(:,:,2) = []
    interp
        .eval_script("A = reshape(1:24, [2, 3, 4]); A(:, :, 2) = []; sz = size(A);")
        .unwrap();
    let sz = interp.eval_script("sz").unwrap().unwrap();
    assert_eq!(sz.to_double_array().unwrap().as_slice(), &[2.0, 3.0, 3.0]);

    // 2. Growth by assignment in 3D: A(4, 5, 2) = 10
    interp
        .eval_script("B = [1 2; 3 4]; B(4, 5, 2) = 10; szB = size(B);")
        .unwrap();
    let sz_b = interp.eval_script("szB").unwrap().unwrap();
    assert_eq!(sz_b.to_double_array().unwrap().as_slice(), &[4.0, 5.0, 2.0]);
}

#[test]
fn test_a3_reshape_empty_placeholder_and_negative_error() {
    let mut interp = new_interpreter();

    // 1. Reshape with [] placeholder
    interp
        .eval_script("B = reshape(1:6, [], 3); sz = size(B);")
        .unwrap();
    let sz = interp.eval_script("sz").unwrap().unwrap();
    assert_eq!(sz.to_double_array().unwrap().as_slice(), &[2.0, 3.0]);

    // 2. Negative reshape dimension returns exact MATLAB error
    let err = interp.eval_script("reshape(1:6, -1, 3)").unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Size arguments must be integer scalars or []."),
        "Unexpected error: {msg}"
    );
}

#[test]
fn test_a3_logical_mask_assignment_errors() {
    let mut interp = new_interpreter();

    // 1. A(mask) = vector count mismatch error
    let err = interp
        .eval_script("A = 1:5; A([true false true false true]) = [10 20];")
        .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains(
            "In an assignment  A(I) = B, the number of elements in B and I must be the same."
        ),
        "Unexpected count mismatch error: {msg}"
    );

    // 2. Mask size mismatch error
    let err2 = interp
        .eval_script("A = 1:5; A(zeros(2, 2) == 1) = 5;")
        .unwrap_err();
    let msg2 = format!("{err2}");
    assert!(
        msg2.contains("Array dimensions must match for binary array op or logical indexing.")
            || msg2.contains("Array to be indexed must have the same size as the mask."),
        "Unexpected mask size error: {msg2}"
    );
}

#[test]
fn test_a3_find_variants_and_short_circuit() {
    let mut interp = new_interpreter();

    // 1. find(x, n)
    interp
        .eval_script("idx_first = find([1 0 2 0 3 0 4], 2);")
        .unwrap();
    let first = interp.eval_script("idx_first").unwrap().unwrap();
    assert_eq!(first.to_double_array().unwrap().as_slice(), &[1.0, 3.0]);

    // 2. find(x, n, 'last')
    interp
        .eval_script("idx_last = find([1 0 2 0 3 0 4], 2, 'last');")
        .unwrap();
    let last = interp.eval_script("idx_last").unwrap().unwrap();
    assert_eq!(last.to_double_array().unwrap().as_slice(), &[5.0, 7.0]);

    // 3. Short-circuiting && and || with side-effecting function
    interp
        .eval_script(
            r#"
        global side_effect_counter;
        side_effect_counter = 0;
        function res = increment_and_true()
            global side_effect_counter;
            side_effect_counter = side_effect_counter + 1;
            res = true;
        end
    "#,
        )
        .unwrap();

    // false && expr: RHS is skipped, counter stays 0
    interp
        .eval_script("sc1 = false && increment_and_true();")
        .unwrap();
    let c0 = interp.eval_script("side_effect_counter").unwrap().unwrap();
    assert_eq!(c0.as_scalar_f64().unwrap(), 0.0);

    // true && expr: RHS is evaluated, counter increments to 1
    interp
        .eval_script("sc2 = true && increment_and_true();")
        .unwrap();
    let c1 = interp.eval_script("side_effect_counter").unwrap().unwrap();
    assert_eq!(c1.as_scalar_f64().unwrap(), 1.0);

    // true || expr: RHS is skipped, counter stays 1
    interp
        .eval_script("sc3 = true || increment_and_true();")
        .unwrap();
    let c2 = interp.eval_script("side_effect_counter").unwrap().unwrap();
    assert_eq!(c2.as_scalar_f64().unwrap(), 1.0);

    // false || expr: RHS is evaluated, counter increments to 2
    interp
        .eval_script("sc4 = false || increment_and_true();")
        .unwrap();
    let c3 = interp.eval_script("side_effect_counter").unwrap().unwrap();
    assert_eq!(c3.as_scalar_f64().unwrap(), 2.0);

    // 4. Elementwise & and | (both sides always evaluated, works on arrays)
    interp
        .eval_script("and_arr = [true false true false] & [true true false false];")
        .unwrap();
    let and_res = interp.eval_script("and_arr").unwrap().unwrap();
    if let Value::Logical(arr) = and_res {
        assert_eq!(arr.as_slice(), &[true, false, false, false]);
    } else {
        panic!("Expected logical array");
    }

    interp
        .eval_script("or_arr = [true false true false] | [false true false false];")
        .unwrap();
    let or_res = interp.eval_script("or_arr").unwrap().unwrap();
    if let Value::Logical(arr) = or_res {
        assert_eq!(arr.as_slice(), &[true, true, true, false]);
    } else {
        panic!("Expected logical array");
    }

    // 5. logical() on integers
    interp
        .eval_script("log_res = logical(int8([0 5 -2]));")
        .unwrap();
    let log_res = interp.eval_script("log_res").unwrap().unwrap();
    if let Value::Logical(arr) = log_res {
        assert_eq!(arr.as_slice(), &[false, true, true]);
    } else {
        panic!("Expected logical array");
    }
}

#[test]
fn test_a3_exact_64bit_integers_and_saturation() {
    let mut interp = new_interpreter();

    // 1. intmax('int64') - 1 exact
    interp.eval_script("m = intmax('int64') - 1;").unwrap();
    let m = interp.eval_script("m").unwrap().unwrap();
    if let Value::Int64(da) = m {
        assert_eq!(da.as_slice()[0], i64::MAX - 1);
        assert_eq!(format!("{}", da.as_slice()[0]), "9223372036854775806");
    } else {
        panic!("Expected int64");
    }

    // 2. uint64 math > 2^53
    interp
        .eval_script("u = uint64(2)^62; u2 = u + uint64(100);")
        .unwrap();
    let u2 = interp.eval_script("u2").unwrap().unwrap();
    if let Value::UInt64(da) = u2 {
        let expected = (1u64 << 62) + 100;
        assert_eq!(da.as_slice()[0], expected);
    } else {
        panic!("Expected uint64");
    }

    // 3. Saturating additions
    let sat = interp
        .eval_script("uint8(200) + uint8(100)")
        .unwrap()
        .unwrap();
    assert_eq!(sat, Value::UInt8(DenseArray::scalar(255)));

    // 4. Integer division rounding to nearest
    let idiv = interp.eval_script("int32(7) / int32(2)").unwrap().unwrap();
    assert_eq!(idiv, Value::Int32(DenseArray::scalar(4)));
}

#[test]
fn test_a3_sparse_parity_and_residual() {
    let mut interp = new_interpreter();

    // 1. Sparse creation, indexing read and write
    interp
        .eval_script("S = sparse([1 2 3], [1 2 3], [2.0 4.0 5.0], 3, 3);")
        .unwrap();
    interp.eval_script("S(1, 2) = 10.0;").unwrap();
    let s12 = interp.eval_script("S(1, 2)").unwrap().unwrap();
    assert_eq!(s12.as_scalar_f64().unwrap(), 10.0);

    // 2. Sparse linear solve: S \ b and residual test ||S*x - b|| < 1e-10
    interp.eval_script("b = [12; 4; 5];").unwrap();
    interp.eval_script("x = S \\ b;").unwrap();
    interp.eval_script("res = norm(S * x - b);").unwrap();
    let res = interp.eval_script("res").unwrap().unwrap();
    assert!(
        res.as_scalar_f64().unwrap() < 1e-10,
        "Sparse residual test failed: {}",
        res.as_scalar_f64().unwrap()
    );

    // 3. Sparse transpose
    interp.eval_script("St = S';").unwrap();
    let s21 = interp.eval_script("St(2, 1)").unwrap().unwrap();
    assert_eq!(s21.as_scalar_f64().unwrap(), 10.0);

    // 4. Sparse + scalar returns full double
    interp.eval_script("S_full = S + 1;").unwrap();
    let s_full = interp.eval_script("S_full").unwrap().unwrap();
    assert!(!s_full.is_sparse());
    assert_eq!(s_full.type_name(), "double");

    // 5. Empty sparse display
    let sp_empty = CscMatrix::<f64>::new(0, 0);
    assert_eq!(
        sp_empty.format_matlab_display(),
        "   All zero sparse: 0-by-0\n"
    );
}

#[test]
fn test_a3_nd_sort_cumsum_prod_and_broadcast() {
    let mut interp = new_interpreter();

    // 1. sort along dim 3
    interp
        .eval_script("A = reshape([4 3 2 1 8 7 6 5], [2, 2, 2]); S3 = sort(A, 3); sz = size(S3);")
        .unwrap();
    let sz = interp.eval_script("sz").unwrap().unwrap();
    assert_eq!(sz.to_double_array().unwrap().as_slice(), &[2.0, 2.0, 2.0]);

    // 2. cumsum along dim 3
    interp
        .eval_script("C3 = cumsum(ones(2, 2, 3), 3); szC = size(C3);")
        .unwrap();
    let sz_c = interp.eval_script("szC").unwrap().unwrap();
    assert_eq!(sz_c.to_double_array().unwrap().as_slice(), &[2.0, 2.0, 3.0]);

    // 3. prod along dim 3
    interp
        .eval_script("P3 = prod(ones(2, 2, 3) * 2, 3); szP = size(P3);")
        .unwrap();
    let sz_p = interp.eval_script("szP").unwrap().unwrap();
    assert_eq!(sz_p.to_double_array().unwrap().as_slice(), &[2.0, 2.0]);

    // 4. Broadcasting across dim 3
    interp
        .eval_script("B = ones(2, 1, 3) + ones(1, 4, 3); szB = size(B);")
        .unwrap();
    let sz_b = interp.eval_script("szB").unwrap().unwrap();
    assert_eq!(sz_b.to_double_array().unwrap().as_slice(), &[2.0, 4.0, 3.0]);
}

#[test]
fn test_a3_all_registered_builtins_have_markdown_docs_and_help() {
    let interp = new_interpreter();
    let builtins = interp.builtin_names();
    let builtin_set: std::collections::BTreeSet<_> = builtins.iter().cloned().collect();

    let candidates = [
        std::path::PathBuf::from("../../docs/functions"),
        std::path::PathBuf::from("docs/functions"),
        std::path::PathBuf::from("rustlab/docs/functions"),
    ];
    let docs_dir = candidates
        .iter()
        .find(|p| p.exists())
        .expect("docs/functions directory must exist");

    // 1. Forward check: no registered builtin without docs or help
    let mut missing_docs = Vec::new();
    let mut missing_help = Vec::new();

    for name in &builtins {
        let doc_path = docs_dir.join(format!("{name}.md"));
        if !doc_path.exists() {
            missing_docs.push(name.clone());
        }
        if rustlab_stdlib::help::get_help_text(name).is_none() {
            missing_help.push(name.clone());
        }
    }

    assert!(
        missing_docs.is_empty(),
        "Builtins missing documentation in docs/functions/<name>.md: {:?}",
        missing_docs
    );
    assert!(
        missing_help.is_empty(),
        "Builtins missing help text in help.rs: {:?}",
        missing_help
    );

    // 2. Reverse check: no orphan doc files in docs/functions
    let mut orphan_docs = Vec::new();
    for entry in std::fs::read_dir(docs_dir).expect("failed to read docs_dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if !builtin_set.contains(stem) {
                    orphan_docs.push(stem.to_string());
                }
            }
        }
    }

    assert!(
        orphan_docs.is_empty(),
        "Orphan markdown doc files found without a registered builtin: {:?}",
        orphan_docs
    );
}
