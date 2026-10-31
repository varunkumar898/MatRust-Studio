use rustlab_math::*;

#[test]
fn test_minimum_2d_and_singleton_stripping() {
    let arr = DenseArray::from_vec(vec![2, 3, 1], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    // Trailing singleton dimension stripped: [2, 3, 1] becomes [2, 3]
    assert_eq!(arr.shape(), &[2, 3]);
    assert_eq!(arr.ndims(), 2);
    assert_eq!(arr.numel(), 6);
}

#[test]
fn test_1_based_indexing_column_major() {
    // 2x3 Matrix:
    // [ 10 30 50 ]
    // [ 20 40 60 ]
    let arr = DenseArray::from_vec(vec![2, 3], vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]).unwrap();

    // Linear indexing (1-based)
    assert_eq!(*arr.get_linear(1).unwrap(), 10.0);
    assert_eq!(*arr.get_linear(2).unwrap(), 20.0);
    assert_eq!(*arr.get_linear(3).unwrap(), 30.0);
    assert_eq!(*arr.get_linear(4).unwrap(), 40.0);
    assert_eq!(*arr.get_linear(5).unwrap(), 50.0);
    assert_eq!(*arr.get_linear(6).unwrap(), 60.0);

    // Out of bounds checks
    assert!(arr.get_linear(0).is_err());
    assert!(arr.get_linear(7).is_err());

    // Subscript indexing (1-based: row, col)
    assert_eq!(*arr.get_subscript(&[1, 1]).unwrap(), 10.0);
    assert_eq!(*arr.get_subscript(&[2, 1]).unwrap(), 20.0);
    assert_eq!(*arr.get_subscript(&[1, 2]).unwrap(), 30.0);
    assert_eq!(*arr.get_subscript(&[2, 2]).unwrap(), 40.0);
    assert_eq!(*arr.get_subscript(&[1, 3]).unwrap(), 50.0);
    assert_eq!(*arr.get_subscript(&[2, 3]).unwrap(), 60.0);
}

#[test]
fn test_array_growth_on_assignment() {
    let mut vec = DenseArray::row_vector(vec![1.0, 2.0]);
    vec.set_linear_grow(5, 10.0).unwrap();

    assert_eq!(vec.shape(), &[1, 5]);
    assert_eq!(vec.as_slice(), &[1.0, 2.0, 0.0, 0.0, 10.0]);

    // 2D matrix growth:
    let mut m = DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    m.set_subscript_grow(&[3, 3], 99.0).unwrap();

    assert_eq!(m.shape(), &[3, 3]);
    assert_eq!(*m.get_subscript(&[3, 3]).unwrap(), 99.0);
    assert_eq!(*m.get_subscript(&[1, 1]).unwrap(), 1.0);
    assert_eq!(*m.get_subscript(&[3, 1]).unwrap(), 0.0);
}

#[test]
fn test_broadcasting_elementwise_binary() {
    // Column vector [10; 20; 30] (3x1) + Row vector [1, 2] (1x2)
    // Results in 3x2 matrix
    let a = Value::Double(DenseArray::col_vector(vec![10.0, 20.0, 30.0]));
    let b = Value::Double(DenseArray::row_vector(vec![1.0, 2.0]));

    let c = a.add(&b).unwrap();
    assert_eq!(c.shape(), &[3, 2]);

    if let Value::Double(arr) = c {
        // Expected column-major result:
        // Col 1: 11, 21, 31
        // Col 2: 12, 22, 32
        assert_eq!(arr.as_slice(), &[11.0, 21.0, 31.0, 12.0, 22.0, 32.0]);
    } else {
        panic!("Expected double array");
    }

    // Scalar + Matrix
    let s = Value::scalar(5.0);
    let m = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());
    let sm = s.add(&m).unwrap();
    if let Value::Double(arr) = sm {
        assert_eq!(arr.as_slice(), &[6.0, 7.0, 8.0, 9.0]);
    }
}

#[test]
fn test_matrix_multiplication() {
    // A = [1 2; 3 4]
    // Col-major: 1, 3, 2, 4
    let a = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap());
    // B = [5 6; 7 8]
    // Col-major: 5, 7, 6, 8
    let b = Value::Double(DenseArray::from_vec(vec![2, 2], vec![5.0, 7.0, 6.0, 8.0]).unwrap());

    let c = a.mul(&b).unwrap();
    if let Value::Double(arr) = c {
        assert_eq!(arr.shape(), &[2, 2]);
        assert_eq!(arr.as_slice(), &[19.0, 43.0, 22.0, 50.0]);
    }

    // Incompatible inner dimensions error
    let incomp = Value::Double(DenseArray::from_vec(vec![3, 2], vec![1.0; 6]).unwrap());
    assert!(a.mul(&incomp).is_err());
}

#[test]
fn test_relational_and_logical_operations() {
    let a = Value::Double(DenseArray::row_vector(vec![1.0, 5.0, 2.0]));
    let b = Value::Double(DenseArray::row_vector(vec![3.0, 5.0, 1.0]));

    let eq_res = a.eq(&b).unwrap();
    if let Value::Logical(arr) = eq_res {
        assert_eq!(arr.as_slice(), &[false, true, false]);
    }

    let gt_res = a.gt(&b).unwrap();
    if let Value::Logical(arr) = gt_res {
        assert_eq!(arr.as_slice(), &[false, false, true]);
    }
}

#[test]
fn test_colon_range() {
    let r = colon_range(1.0, 0.5, 3.0);
    assert_eq!(r.shape(), &[1, 5]);
    assert_eq!(r.as_slice(), &[1.0, 1.5, 2.0, 2.5, 3.0]);
}

#[test]
fn test_a3_saturating_integer_arithmetic() {
    // Citation: Official MATLAB Documentation on Integer Arithmetic
    // uint8(250) + uint8(10) = 255 (saturating addition)
    let u1 = Value::UInt8(DenseArray::scalar(250));
    let u2 = Value::UInt8(DenseArray::scalar(10));
    let u_sum = u1.add(&u2).unwrap();
    assert_eq!(u_sum, Value::UInt8(DenseArray::scalar(255)));

    // uint8(3) - uint8(5) = 0 (saturating subtraction)
    let u3 = Value::UInt8(DenseArray::scalar(3));
    let u4 = Value::UInt8(DenseArray::scalar(5));
    let u_sub = u3.sub(&u4).unwrap();
    assert_eq!(u_sub, Value::UInt8(DenseArray::scalar(0)));

    // int8(127) + 1 = 127 (saturating addition with double)
    let i1 = Value::Int8(DenseArray::scalar(127));
    let d1 = Value::scalar(1.0);
    let i_sum = i1.add(&d1).unwrap();
    assert_eq!(i_sum, Value::Int8(DenseArray::scalar(127)));

    // -int8(-128) = 127
    let i_neg128 = Value::Int8(DenseArray::scalar(-128));
    let i_negated = i_neg128.neg().unwrap();
    assert_eq!(i_negated, Value::Int8(DenseArray::scalar(127)));

    // abs(int8(-128)) = 127
    let i_abs = i_neg128.abs().unwrap();
    assert_eq!(i_abs, Value::Int8(DenseArray::scalar(127)));

    // int32(7) / int32(2) = 4 (rounds half away from zero)
    let num = Value::Int32(DenseArray::scalar(7));
    let den = Value::Int32(DenseArray::scalar(2));
    let div_res = num.dot_div(&den).unwrap();
    assert_eq!(div_res, Value::Int32(DenseArray::scalar(4)));
}

#[test]
fn test_a3_mixed_type_rules_and_errors() {
    // 1. int + double -> int class (double converted with round half away from zero)
    let i = Value::Int8(DenseArray::scalar(10));
    let d = Value::scalar(2.5); // 2.5 rounds to 3
    let res = i.add(&d).unwrap();
    assert_eq!(res, Value::Int8(DenseArray::scalar(13)));

    // 2. int + different int class -> error with MATLAB message
    let i8_val = Value::Int8(DenseArray::scalar(1));
    let i16_val = Value::Int16(DenseArray::scalar(2));
    let err = i8_val.add(&i16_val).unwrap_err();
    assert_eq!(
        err,
        MathError::IntegerClassMismatch {
            identifier: "MATLAB:integerTypeMismatch"
        }
    );
    assert_eq!(
        format!("{err}"),
        "Integers can only be combined with integers of the same class, or scalar doubles."
    );

    // 3. single + double -> single
    let s = Value::Single(DenseArray::scalar(2.5f32));
    let d2 = Value::scalar(3.5);
    let s_res = s.add(&d2).unwrap();
    assert_eq!(s_res, Value::Single(DenseArray::scalar(6.0f32)));
}

#[test]
fn test_a3_exact_int64_uint64() {
    // Exactness test: intmax('int64') - 1 without f64 precision loss!
    // int64::MAX is 9223372036854775807 (> 2^53, floats lose precision)
    let max_i64 = i64::MAX;
    let v_max = Value::Int64(DenseArray::scalar(max_i64));
    let v_one = Value::Int64(DenseArray::scalar(1));
    let v_sub = v_max.sub(&v_one).unwrap();
    assert_eq!(v_sub, Value::Int64(DenseArray::scalar(max_i64 - 1)));
    assert_eq!(v_sub.as_scalar_i64().unwrap(), 9223372036854775806i64);
}

#[test]
fn test_a3_sparse_csc_duplicate_summing_and_algebra() {
    // sparse(i, j, v, m, n) with duplicate summing
    // Triplets with duplicate (2, 2)
    let rows = vec![1, 2, 2, 3];
    let cols = vec![1, 2, 2, 3];
    let vals = vec![5.0, 3.0, 7.0, 9.0]; // (2, 2) duplicates sum: 3.0 + 7.0 = 10.0
    let sp = CscMatrix::from_triplets_1based(3, 3, &rows, &cols, &vals).unwrap();

    assert_eq!(sp.shape, [3, 3]);
    assert_eq!(sp.nnz(), 3);
    assert_eq!(sp.get_1based(1, 1).unwrap(), 5.0);
    assert_eq!(sp.get_1based(2, 2).unwrap(), 10.0); // Summed!
    assert_eq!(sp.get_1based(3, 3).unwrap(), 9.0);
    assert_eq!(sp.get_1based(1, 2).unwrap(), 0.0);

    // sparse * dense
    let dense = DenseArray::from_vec(vec![3, 1], vec![1.0, 2.0, 3.0]).unwrap();
    let prod = sp.matmul_dense(&dense).unwrap();
    assert_eq!(prod.as_slice(), &[5.0, 20.0, 27.0]);

    // sparse * sparse
    let sp2 = CscMatrix::speye(3, 3);
    let sp_prod = sp.matmul_sparse(&sp2).unwrap();
    assert_eq!(sp_prod.to_dense().as_slice(), sp.to_dense().as_slice());
}

#[test]
fn test_a3_sparse_memory_efficiency_10000x10000() {
    // 10000 x 10000 with 10000 nonzeros stays small in memory!
    let n = 10000;
    let rows: Vec<usize> = (1..=n).collect();
    let cols: Vec<usize> = (1..=n).collect();
    let vals: Vec<f64> = vec![1.0; n];

    let sp = CscMatrix::from_triplets_1based(n, n, &rows, &cols, &vals).unwrap();
    assert_eq!(sp.nnz(), 10000);
    assert_eq!(sp.shape, [10000, 10000]);

    // Memory footprint of sparse matrix is (N+1)*8 + 10000*8 + 10000*8 ~ 240 KB,
    // whereas dense 10000x10000 would be 10000 * 10000 * 8 = 800 MB!
    let memory_bytes = (sp.col_ptr.len() + sp.row_ind.len()) * std::mem::size_of::<usize>()
        + sp.values.len() * std::mem::size_of::<f64>();
    assert!(
        memory_bytes < 500_000,
        "Sparse matrix must stay small in memory, got {} bytes",
        memory_bytes
    );
}

#[test]
fn test_matmul_zero_dimensions_and_mismatch() {
    use rustlab_math::ops::matmul_2d;

    // 0x0 * 0x0 -> 0x0
    let a00 = DenseArray::from_vec(vec![0, 0], Vec::new()).unwrap();
    let b00 = DenseArray::from_vec(vec![0, 0], Vec::new()).unwrap();
    let c00 = matmul_2d(&a00, &b00).unwrap();
    assert_eq!(c00.shape(), &[0, 0]);
    assert_eq!(c00.numel(), 0);

    // 0x5 * 5x3 -> 0x3
    let a05 = DenseArray::from_vec(vec![0, 5], Vec::new()).unwrap();
    let b53 = DenseArray::from_vec(vec![5, 3], vec![1.0; 15]).unwrap();
    let c03 = matmul_2d(&a05, &b53).unwrap();
    assert_eq!(c03.shape(), &[0, 3]);
    assert_eq!(c03.numel(), 0);

    // 4x0 * 0x6 -> 4x6 (all zeros)
    let a40 = DenseArray::from_vec(vec![4, 0], Vec::new()).unwrap();
    let b06 = DenseArray::from_vec(vec![0, 6], Vec::new()).unwrap();
    let c46 = matmul_2d(&a40, &b06).unwrap();
    assert_eq!(c46.shape(), &[4, 6]);
    assert_eq!(c46.numel(), 24);
    assert_eq!(c46.as_slice(), &vec![0.0; 24]);

    // 3x2 * 4x2 -> dimension mismatch (inner dimensions 2 != 4)
    let a32 = DenseArray::from_vec(vec![3, 2], vec![1.0; 6]).unwrap();
    let b42 = DenseArray::from_vec(vec![4, 2], vec![1.0; 8]).unwrap();
    let err = matmul_2d(&a32, &b42).expect_err("inner dimension mismatch must fail");
    match err {
        MathError::InnerDimensionMismatch {
            identifier,
            cols_a,
            rows_b,
        } => {
            assert_eq!(identifier, "MATLAB:innerdim");
            assert_eq!(cols_a, 2);
            assert_eq!(rows_b, 4);
        }
        other => panic!("expected InnerDimensionMismatch, got {other:?}"),
    }
}
