//! Integration test suite for rustlab-data crate (B2.1).

use rustlab_data::csv_io::{readmatrix, writematrix};
use rustlab_data::stats::{
    corrcoef, cov, cummax, cummin, histcounts, median, mode, movmean, movmedian, prctile, quantile,
    sortrows, std_fn, unique, var_fn, zscore,
};
use rustlab_math::{DenseArray, Value};
use std::path::Path;

fn str_val(s: &str) -> Value {
    Value::String(DenseArray::scalar(s.to_string()))
}

fn dbl_mat(rows: usize, cols: usize, data: Vec<f64>) -> Value {
    Value::Double(DenseArray::from_vec_unchecked(vec![rows, cols], data))
}

#[test]
fn test_b2_median_vector_matrix_nd_omitnan() {
    // 1. Odd length vector
    let v_odd = dbl_mat(1, 5, vec![1.0, 5.0, 2.0, 8.0, 7.0]);
    let res = median(&[v_odd], 1).expect("median odd");
    assert_eq!(res[0].as_scalar_f64().expect("scalar"), 5.0);

    // 2. Even length vector (average of middle elements)
    let v_even = dbl_mat(1, 4, vec![1.0, 2.0, 5.0, 6.0]);
    let res = median(&[v_even], 1).expect("median even");
    assert_eq!(res[0].as_scalar_f64().expect("scalar"), 3.5);

    // 3. 2-D matrix along dim 1 and dim 2
    // Matrix [1 6; 3 4; 5 2] (3x2, column-major: 1, 3, 5, 6, 4, 2)
    let mat = dbl_mat(3, 2, vec![1.0, 3.0, 5.0, 6.0, 4.0, 2.0]);
    let res_dim1 = median(&[mat.clone(), Value::scalar(1.0)], 1).expect("dim 1");
    if let Value::Double(arr) = &res_dim1[0] {
        assert_eq!(arr.shape(), &[1, 2]);
        assert_eq!(arr.as_slice(), &[3.0, 4.0]);
    } else {
        panic!("expected double array");
    }

    let res_dim2 = median(&[mat, Value::scalar(2.0)], 1).expect("dim 2");
    if let Value::Double(arr) = &res_dim2[0] {
        assert_eq!(arr.shape(), &[3, 1]);
        assert_eq!(arr.as_slice(), &[3.5, 3.5, 3.5]);
    } else {
        panic!("expected double array");
    }

    // 4. NaN propagation and 'omitnan'
    let v_nan = dbl_mat(1, 3, vec![1.0, f64::NAN, 3.0]);
    let res_nan = median(std::slice::from_ref(&v_nan), 1).expect("median nan");
    assert!(res_nan[0].as_scalar_f64().expect("scalar").is_nan());

    let res_omit = median(&[v_nan, str_val("omitnan")], 1).expect("median omitnan");
    assert_eq!(res_omit[0].as_scalar_f64().expect("scalar"), 2.0);

    // 5. Empty array
    let empty = Value::Double(DenseArray::empty());
    let res_empty = median(&[empty], 1).expect("median empty");
    assert!(res_empty[0].as_scalar_f64().expect("scalar").is_nan());
}

#[test]
fn test_b2_mode_tiebreak_smallest() {
    // Tie between 1.0, 2.0, 3.0 (each frequency 2) -> must pick smallest (1.0)
    let v = dbl_mat(1, 6, vec![3.0, 1.0, 2.0, 3.0, 2.0, 1.0]);
    let res = mode(&[v], 1).expect("mode");
    assert_eq!(res[0].as_scalar_f64().expect("scalar"), 1.0);

    // Single winner
    let v2 = dbl_mat(1, 5, vec![4.0, 2.0, 4.0, 1.0, 4.0]);
    let res2 = mode(&[v2], 1).expect("mode 4");
    assert_eq!(res2[0].as_scalar_f64().expect("scalar"), 4.0);

    // Omitnan with ties
    let v_nan = dbl_mat(1, 4, vec![5.0, f64::NAN, 5.0, 2.0]);
    let res_nan = mode(std::slice::from_ref(&v_nan), 1).expect("mode nan");
    assert!(res_nan[0].as_scalar_f64().expect("scalar").is_nan());

    let res_omit = mode(&[v_nan, str_val("omitnan")], 1).expect("mode omitnan");
    assert_eq!(res_omit[0].as_scalar_f64().expect("scalar"), 5.0);
}

#[test]
fn test_b2_var_and_std_flags_dims_integer_classes() {
    let v = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    // Flag 0 (N - 1 = 4)
    let v0 = var_fn(&[v.clone(), Value::scalar(0.0)], 1).expect("var flag 0");
    assert_eq!(v0[0].as_scalar_f64().expect("scalar"), 2.5);

    let s0 = std_fn(&[v.clone(), Value::scalar(0.0)], 1).expect("std flag 0");
    assert!((s0[0].as_scalar_f64().expect("scalar") - (2.5f64).sqrt()).abs() < 1e-12);

    // Flag 1 (N = 5)
    let v1 = var_fn(&[v.clone(), Value::scalar(1.0)], 1).expect("var flag 1");
    assert_eq!(v1[0].as_scalar_f64().expect("scalar"), 2.0);

    let s1 = std_fn(&[v, Value::scalar(1.0)], 1).expect("std flag 1");
    assert!((s1[0].as_scalar_f64().expect("scalar") - (2.0f64).sqrt()).abs() < 1e-12);

    // Integer class: UInt8([10, 20, 30])
    let u8_arr = Value::UInt8(DenseArray::row_vector(vec![10u8, 20u8, 30u8]));
    let v_u8 = var_fn(&[u8_arr], 1).expect("var uint8");
    assert_eq!(v_u8[0].as_scalar_f64().expect("scalar"), 100.0);

    // Empty matrix
    let empty = Value::Double(DenseArray::empty());
    let v_empty = var_fn(&[empty], 1).expect("var empty");
    assert!(v_empty[0].as_scalar_f64().expect("scalar").is_nan());

    // omitnan
    let v_nan = dbl_mat(1, 4, vec![1.0, f64::NAN, 3.0, f64::NAN]);
    let v_omit = var_fn(&[v_nan, Value::scalar(0.0), str_val("omitnan")], 1).expect("var omitnan");
    assert_eq!(v_omit[0].as_scalar_f64().expect("scalar"), 2.0);
}

#[test]
fn test_b2_prctile_and_quantile_interpolation() {
    let v = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    // Median via quantile and prctile
    let q50 = quantile(&[v.clone(), Value::scalar(0.5)], 1).expect("quantile 0.5");
    assert_eq!(q50[0].as_scalar_f64().expect("scalar"), 3.0);

    let p50 = prctile(&[v.clone(), Value::scalar(50.0)], 1).expect("prctile 50");
    assert_eq!(p50[0].as_scalar_f64().expect("scalar"), 3.0);

    // Linear interpolation test: [10, 20, 30, 40], N = 4
    // q = 0.25 -> h = (4 - 1)*0.25 + 1 = 1.75 -> v[0] + 0.75*(v[1]-v[0]) = 10 + 0.75*10 = 17.5
    // q = 0.75 -> h = (4 - 1)*0.75 + 1 = 3.25 -> v[2] + 0.25*(v[3]-v[2]) = 30 + 0.25*10 = 32.5
    let v_interp = dbl_mat(1, 4, vec![10.0, 20.0, 30.0, 40.0]);
    let q_vec = dbl_mat(1, 2, vec![0.25, 0.75]);
    let q_res = quantile(&[v_interp, q_vec], 1).expect("quantile vector");
    if let Value::Double(arr) = &q_res[0] {
        assert_eq!(arr.shape(), &[1, 2]);
        assert_eq!(arr.as_slice(), &[17.5, 32.5]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_cov_and_corrcoef() {
    let x = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let y = dbl_mat(1, 5, vec![2.0, 4.0, 6.0, 8.0, 10.0]);

    // cov(x, y) = 2 * var(x) = 2 * 2.5 = 5.0
    let c = cov(&[x.clone(), y.clone()], 1).expect("cov(x,y)");
    assert_eq!(c[0].as_scalar_f64().expect("scalar"), 5.0);

    // corrcoef(x, y) = 1.0 (perfect linear correlation)
    let r = corrcoef(&[x, y], 1).expect("corrcoef(x,y)");
    assert_eq!(r[0].as_scalar_f64().expect("scalar"), 1.0);

    // 2-D matrix corrcoef: diagonal must be exactly 1.0
    // Matrix [1 2; 3 5; 4 6]
    let m = dbl_mat(3, 2, vec![1.0, 3.0, 4.0, 2.0, 5.0, 6.0]);
    let r_mat = corrcoef(&[m], 1).expect("corrcoef matrix");
    if let Value::Double(arr) = &r_mat[0] {
        assert_eq!(arr.shape(), &[2, 2]);
        assert_eq!(arr.as_slice()[0], 1.0);
        assert_eq!(arr.as_slice()[3], 1.0);
        let off_diag = arr.as_slice()[1];
        assert!(off_diag > 0.9 && off_diag <= 1.0);
        assert_eq!(arr.as_slice()[2], off_diag); // symmetric
    } else {
        panic!("expected double matrix");
    }
}

#[test]
fn test_b2_histcounts() {
    let x = dbl_mat(1, 8, vec![0.5, 1.2, 2.0, 2.8, 3.5, 4.0, 5.5, 6.0]);
    let edges = dbl_mat(1, 4, vec![0.0, 2.0, 4.0, 6.0]);

    // Bin 1: [0.0, 2.0) -> 0.5, 1.2 (count: 2)
    // Bin 2: [2.0, 4.0) -> 2.0, 2.8, 3.5 (count: 3)
    // Bin 3: [4.0, 6.0] -> 4.0, 5.5, 6.0 (count: 3)
    let h = histcounts(&[x, edges], 1).expect("histcounts");
    if let Value::Double(arr) = &h[0] {
        assert_eq!(arr.shape(), &[1, 3]);
        assert_eq!(arr.as_slice(), &[2.0, 3.0, 3.0]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_movmean_and_movmedian() {
    let v = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    // Centered window length 3:
    // i=0: [1, 2] -> mean 1.5
    // i=1: [1, 2, 3] -> mean 2.0
    // i=2: [2, 3, 4] -> mean 3.0
    // i=3: [3, 4, 5] -> mean 4.0
    // i=4: [4, 5] -> mean 4.5
    let mm = movmean(&[v.clone(), Value::scalar(3.0)], 1).expect("movmean");
    if let Value::Double(arr) = &mm[0] {
        assert_eq!(arr.as_slice(), &[1.5, 2.0, 3.0, 4.0, 4.5]);
    } else {
        panic!("expected double array");
    }

    let mmed = movmedian(&[v, Value::scalar(3.0)], 1).expect("movmedian");
    if let Value::Double(arr) = &mmed[0] {
        assert_eq!(arr.as_slice(), &[1.5, 2.0, 3.0, 4.0, 4.5]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_cummax_and_cummin() {
    let v = dbl_mat(1, 7, vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0]);

    let cmax = cummax(std::slice::from_ref(&v), 1).expect("cummax");
    if let Value::Double(arr) = &cmax[0] {
        assert_eq!(arr.as_slice(), &[3.0, 3.0, 4.0, 4.0, 5.0, 9.0, 9.0]);
    } else {
        panic!("expected double array");
    }

    let cmin = cummin(&[v], 1).expect("cummin");
    if let Value::Double(arr) = &cmin[0] {
        assert_eq!(arr.as_slice(), &[3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_zscore() {
    let v = dbl_mat(5, 1, vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    let z = zscore(&[v], 1).expect("zscore");
    if let Value::Double(arr) = &z[0] {
        let mean = arr.as_slice().iter().sum::<f64>() / 5.0;
        assert!(mean.abs() < 1e-12, "zscore mean should be ~0");
        let var: f64 = arr.as_slice().iter().map(|x| x * x).sum::<f64>() / 4.0;
        assert!((var - 1.0).abs() < 1e-12, "zscore variance should be ~1");
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_sortrows_and_unique_ia_ic() {
    // 1. sortrows multi-key
    // [1 4; 2 3; 1 2] -> sorted by col 1 asc, col 2 desc:
    // col 1 ties: [1 4] and [1 2]. With -2: [1 4] comes before [1 2].
    // Result: [1 4; 1 2; 2 3]
    let m = dbl_mat(3, 2, vec![1.0, 2.0, 1.0, 4.0, 3.0, 2.0]);
    let cols = dbl_mat(1, 2, vec![1.0, -2.0]);
    let sr = sortrows(&[m, cols], 2).expect("sortrows");
    if let Value::Double(arr) = &sr[0] {
        assert_eq!(arr.as_slice(), &[1.0, 1.0, 2.0, 4.0, 2.0, 3.0]);
    } else {
        panic!("expected double array");
    }

    // 2. unique with ia, ic
    let u_vec = dbl_mat(1, 6, vec![9.0, 2.0, 9.0, 5.0, 2.0, 7.0]);
    let u_res = unique(&[u_vec], 3).expect("unique");
    if let (Value::Double(c), Value::Double(ia), Value::Double(ic)) =
        (&u_res[0], &u_res[1], &u_res[2])
    {
        // Unique sorted: [2.0, 5.0, 7.0, 9.0]
        assert_eq!(c.as_slice(), &[2.0, 5.0, 7.0, 9.0]);
        // ia: 1-based indices of first occurrences in original array
        // 2.0 is at idx 2, 5.0 is at idx 4, 7.0 is at idx 6, 9.0 is at idx 1
        assert_eq!(ia.as_slice(), &[2.0, 4.0, 6.0, 1.0]);
        // ic: indices in c to reconstruct original: [4, 1, 4, 2, 1, 3]
        assert_eq!(ic.as_slice(), &[4.0, 1.0, 4.0, 2.0, 1.0, 3.0]);
    } else {
        panic!("expected double arrays");
    }
}

#[test]
fn test_b2_csv_options_delimiter_header_bom_crlf() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");

    // 1. simple.csv
    let f_simple = data_dir.join("simple.csv");
    let res_simple = readmatrix(&[str_val(&f_simple.to_string_lossy())], 1).expect("read simple");
    if let Value::Double(arr) = &res_simple[0] {
        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(
            arr.as_slice(),
            &[1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0]
        );
    } else {
        panic!("expected double array");
    }

    // 2. header_comments.csv with 'NumHeaderLines', 2
    let f_hdr = data_dir.join("header_comments.csv");
    let res_hdr = readmatrix(
        &[
            str_val(&f_hdr.to_string_lossy()),
            str_val("NumHeaderLines"),
            Value::scalar(2.0),
        ],
        1,
    )
    .expect("read header");
    if let Value::Double(arr) = &res_hdr[0] {
        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(arr.as_slice()[0], 0.0);
        assert_eq!(arr.as_slice()[1], 0.5);
        assert_eq!(arr.as_slice()[2], 1.0);
    } else {
        panic!("expected double array");
    }

    // 3. empty_fields.csv -> empty maps to NaN
    let f_empty = data_dir.join("empty_fields.csv");
    let res_empty =
        readmatrix(&[str_val(&f_empty.to_string_lossy())], 1).expect("read empty fields");
    if let Value::Double(arr) = &res_empty[0] {
        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(arr.as_slice()[0], 10.0);
        assert!(arr.as_slice()[1].is_nan()); // row 2 col 1 was empty
        assert_eq!(arr.as_slice()[2], 70.0);
        assert!(arr.as_slice()[3].is_nan()); // row 1 col 2 was empty
    } else {
        panic!("expected double array");
    }

    // 4. quoted.csv
    let f_quoted = data_dir.join("quoted.csv");
    let res_quoted = readmatrix(&[str_val(&f_quoted.to_string_lossy())], 1).expect("read quoted");
    if let Value::Double(arr) = &res_quoted[0] {
        assert_eq!(arr.shape(), &[2, 3]);
        assert_eq!(arr.as_slice()[0], 1.5);
        assert_eq!(arr.as_slice()[1], 4.25);
    } else {
        panic!("expected double array");
    }

    // 5. bom_crlf.csv
    let f_bom = data_dir.join("bom_crlf.csv");
    let res_bom = readmatrix(&[str_val(&f_bom.to_string_lossy())], 1).expect("read bom");
    if let Value::Double(arr) = &res_bom[0] {
        assert_eq!(arr.shape(), &[2, 3]);
        assert_eq!(arr.as_slice()[0], 100.5);
        assert_eq!(arr.as_slice()[1], 400.75);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_csv_ragged_row_error_with_line_number() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let f_ragged = data_dir.join("ragged.csv");
    let err = readmatrix(&[str_val(&f_ragged.to_string_lossy())], 1).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Row 2") && msg.contains("expected 3"),
        "error should identify Row 2 and expected columns: {msg}"
    );
}

#[test]
fn test_b2_readmatrix_and_writematrix_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let temp_file = temp_dir.path().join("roundtrip.csv");
    let filename = temp_file.to_string_lossy().to_string();

    let orig_mat = dbl_mat(
        3,
        4,
        vec![
            1.23456789012345,
            -987.654321,
            0.0,
            1e-15,
            f64::NAN,
            1e20,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -0.5,
            100.0,
            200.0,
            300.0,
        ],
    );

    // Write matrix
    writematrix(&[orig_mat, str_val(&filename)], 0).expect("writematrix");

    // Read back
    let read_res = readmatrix(&[str_val(&filename)], 1).expect("readmatrix");
    if let Value::Double(arr) = &read_res[0] {
        assert_eq!(arr.shape(), &[3, 4]);
        // Verify values
        assert_eq!(arr.as_slice()[0], 1.23456789012345);
        assert_eq!(arr.as_slice()[1], -987.654321);
        assert_eq!(arr.as_slice()[2], 0.0);
        assert_eq!(arr.as_slice()[3], 1e-15);
        assert!(arr.as_slice()[4].is_nan());
        assert_eq!(arr.as_slice()[5], 1e20);
        assert_eq!(arr.as_slice()[6], f64::INFINITY);
        assert_eq!(arr.as_slice()[7], f64::NEG_INFINITY);
    } else {
        panic!("expected double array");
    }
}
