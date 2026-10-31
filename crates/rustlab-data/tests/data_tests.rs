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

    // MATLAB piecewise linear interpolation: pos = q * N + 0.5
    // For [10, 20, 30, 40], N = 4:
    // q = 0.25 -> pos = 0.25 * 4 + 0.5 = 1.5 -> v[0] + 0.5 * (v[1] - v[0]) = 10 + 0.5 * 10 = 15.0
    // q = 0.75 -> pos = 0.75 * 4 + 0.5 = 3.5 -> v[2] + 0.5 * (v[3] - v[2]) = 30 + 0.5 * 10 = 35.0
    let v_interp = dbl_mat(1, 4, vec![10.0, 20.0, 30.0, 40.0]);
    let q_vec = dbl_mat(1, 2, vec![0.25, 0.75]);
    let q_res = quantile(&[v_interp.clone(), q_vec.clone()], 1).expect("quantile vector");
    if let Value::Double(arr) = &q_res[0] {
        assert_eq!(arr.shape(), &[1, 2]);
        assert_eq!(arr.as_slice(), &[15.0, 35.0]);
    } else {
        panic!("expected double array");
    }

    // Optional numpy method: h = (N - 1) * q + 1
    // q = 0.25 -> 17.5, q = 0.75 -> 32.5
    let q_np =
        quantile(&[v_interp, q_vec, str_val("method"), str_val("numpy")], 1).expect("numpy method");
    if let Value::Double(arr) = &q_np[0] {
        assert_eq!(arr.shape(), &[1, 2]);
        assert_eq!(arr.as_slice(), &[17.5, 32.5]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_prctile_matlab_rule_goldens() {
    // Golden 1: prctile([1 2 3 4 5], 25) = 1.75
    let v1 = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let p25 = prctile(&[v1.clone(), Value::scalar(25.0)], 1).expect("prctile 25");
    assert_eq!(p25[0].as_scalar_f64().expect("scalar"), 1.75);

    // Golden 2: prctile([1 2 3 4 5], 50) = 3
    let p50 = prctile(&[v1, Value::scalar(50.0)], 1).expect("prctile 50");
    assert_eq!(p50[0].as_scalar_f64().expect("scalar"), 3.0);

    // Golden 3: prctile([12.5 18.2 14.1 22 19.8 14.1 16.5], 75) = 19.4
    let v2 = dbl_mat(1, 7, vec![12.5, 18.2, 14.1, 22.0, 19.8, 14.1, 16.5]);
    let p75 = prctile(&[v2, Value::scalar(75.0)], 1).expect("prctile 75");
    assert!((p75[0].as_scalar_f64().expect("scalar") - 19.4).abs() < 1e-12);
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

#[test]
fn test_b2_display_mixed_integer_decimals_unverified() {
    // x = [1.5 22] -> in a non-integer matrix, integer elements print with decimals (.0000)
    let m = dbl_mat(1, 2, vec![1.5, 22.0]);
    let s = format!("{m}");
    assert!(
        s.contains("1.5000") && s.contains("22.0000"),
        "display: {s}"
    );

    // x = [1.5 0] -> only exact 0 prints bare
    let m0 = dbl_mat(1, 2, vec![1.5, 0.0]);
    let s0 = format!("{m0}");
    assert!(
        s0.contains("1.5000") && s0.contains(" 0"),
        "display zero: {s0}"
    );
    assert!(!s0.contains("0.0000"), "exact zero should print bare: {s0}");
}

#[test]
fn test_b2_display_sub_01_scalars_unverified() {
    // 0.001 and 0.0012 print in e-notation; 0.01 prints fixed
    let v_001 = Value::scalar(0.001);
    assert_eq!(format!("{v_001}").trim(), "1.0000e-03");

    let v_0012 = Value::scalar(0.0012);
    assert_eq!(format!("{v_0012}").trim(), "1.2000e-03");

    let v_01 = Value::scalar(0.01);
    assert_eq!(format!("{v_01}").trim(), "0.0100");

    let v_1234 = Value::scalar(1234.5678);
    assert_eq!(format!("{v_1234}").trim(), "1.2346e+03");
}

#[test]
fn test_b2_display_scale_factor_vs_enotation_unverified() {
    // [1e-16 1] -> extreme dynamic range (1e16 >= 1e9), each entry in e-notation
    let m_extreme = dbl_mat(1, 2, vec![1e-16, 1.0]);
    let s_extreme = format!("{m_extreme}");
    assert!(
        s_extreme.contains("1.0000e-16") && s_extreme.contains("1.0000e+00"),
        "{s_extreme}"
    );

    // [1 1000000] -> all integers, prints bare integers without scale factor
    let m_int = dbl_mat(1, 2, vec![1.0, 1000000.0]);
    let s_int = format!("{m_int}");
    assert!(
        !s_int.contains("1.0e+"),
        "should have no scale factor: {s_int}"
    );
    assert!(s_int.contains("1") && s_int.contains("1000000"), "{s_int}");

    // [0.001 1000] -> max is 1000, dynamic range 1e6 < 1e9, uses common scale factor
    let m_scale = dbl_mat(1, 2, vec![0.001, 1000.0]);
    let s_scale = format!("{m_scale}");
    assert!(
        s_scale.contains("1.0e+03 *"),
        "should contain scale factor: {s_scale}"
    );
}

#[test]
fn test_b2_unique_row_orientation() {
    // Row vector input 1 x N must yield row vector output 1 x M
    let row_vec = dbl_mat(1, 5, vec![5.0, 2.0, 8.0, 2.0, 1.0]);
    let u_res = unique(&[row_vec], 1).expect("unique row");
    if let Value::Double(arr) = &u_res[0] {
        assert_eq!(
            arr.shape(),
            &[1, 4],
            "unique of row vector must preserve 1 x M shape"
        );
        assert_eq!(arr.as_slice(), &[1.0, 2.0, 5.0, 8.0]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_median_edge_cases() {
    // 1. Single element
    let single = Value::scalar(42.0);
    let m = median(&[single], 1).expect("single");
    assert_eq!(m[0].as_scalar_f64().expect("scalar"), 42.0);

    // 2. Inf handling
    let v_inf = dbl_mat(1, 3, vec![1.0, f64::INFINITY, 10.0]);
    let m_inf = median(&[v_inf], 1).expect("inf");
    assert_eq!(m_inf[0].as_scalar_f64().expect("scalar"), 10.0);

    // 3. UInt8 integer class
    let u8_arr = Value::UInt8(DenseArray::row_vector(vec![10u8, 20u8, 30u8, 40u8]));
    let m_u8 = median(&[u8_arr], 1).expect("uint8");
    assert_eq!(m_u8[0].as_scalar_f64().expect("scalar"), 25.0);

    // 4. Invalid dimension error text
    let v = dbl_mat(1, 3, vec![1.0, 2.0, 3.0]);
    let err = median(&[v, Value::scalar(0.0)], 1).unwrap_err();
    assert!(format!("{err}").contains("positive integer scalar"));
}

#[test]
fn test_b2_mode_edge_cases() {
    // 1. All distinct elements -> smallest element returned
    let v_distinct = dbl_mat(1, 4, vec![40.0, 10.0, 30.0, 20.0]);
    let m = mode(&[v_distinct], 1).expect("mode distinct");
    assert_eq!(m[0].as_scalar_f64().expect("scalar"), 10.0);

    // 2. Mode along dim 2
    let mat = dbl_mat(2, 3, vec![1.0, 5.0, 2.0, 5.0, 2.0, 6.0]);
    let m2 = mode(&[mat, Value::scalar(2.0)], 1).expect("mode dim 2");
    if let Value::Double(arr) = &m2[0] {
        assert_eq!(arr.shape(), &[2, 1]);
        assert_eq!(arr.as_slice(), &[2.0, 5.0]);
    } else {
        panic!("expected double array");
    }

    // 3. Empty input
    let empty = Value::Double(DenseArray::empty());
    let m_empty = mode(&[empty], 1).expect("mode empty");
    assert!(m_empty[0].as_scalar_f64().expect("scalar").is_nan());
}

#[test]
fn test_b2_var_and_std_edge_cases() {
    // 1. Single element has variance 0
    let v_one = dbl_mat(1, 1, vec![99.0]);
    assert_eq!(
        var_fn(std::slice::from_ref(&v_one), 1).expect("var")[0]
            .as_scalar_f64()
            .expect("s"),
        0.0
    );
    assert_eq!(
        std_fn(&[v_one], 1).expect("std")[0]
            .as_scalar_f64()
            .expect("s"),
        0.0
    );

    // 2. Constant array has variance 0
    let v_const = dbl_mat(1, 6, vec![7.0; 6]);
    assert_eq!(
        var_fn(std::slice::from_ref(&v_const), 1).expect("var")[0]
            .as_scalar_f64()
            .expect("s"),
        0.0
    );
    assert_eq!(
        std_fn(&[v_const], 1).expect("std")[0]
            .as_scalar_f64()
            .expect("s"),
        0.0
    );

    // 3. Invalid flag error text
    let v = dbl_mat(1, 3, vec![1.0, 2.0, 3.0]);
    let err = var_fn(&[v, Value::scalar(2.0)], 1).unwrap_err();
    assert!(format!("{err}").contains("Normalization flag must be 0 or 1"));
}

#[test]
fn test_b2_prctile_and_quantile_edge_cases() {
    let v = dbl_mat(1, 4, vec![10.0, 20.0, 30.0, 40.0]);

    // 1. Boundary q=0.0 -> min (10.0), q=1.0 -> max (40.0)
    let p0 = prctile(&[v.clone(), Value::scalar(0.0)], 1).expect("p0");
    assert_eq!(p0[0].as_scalar_f64().expect("s"), 10.0);

    let p100 = prctile(&[v.clone(), Value::scalar(100.0)], 1).expect("p100");
    assert_eq!(p100[0].as_scalar_f64().expect("s"), 40.0);

    // 2. Out of bounds error text
    let err_p = prctile(&[v.clone(), Value::scalar(105.0)], 1).unwrap_err();
    assert!(format!("{err_p}").contains("Percentile values must be in [0, 100]"));

    let err_q = quantile(&[v, Value::scalar(1.5)], 1).unwrap_err();
    assert!(format!("{err_q}").contains("Quantile values must be in [0, 1]"));
}

#[test]
fn test_b2_cov_and_corrcoef_edge_cases() {
    // 1. Constant vector produces NaN corrcoef
    let x_const = dbl_mat(1, 4, vec![5.0; 4]);
    let y_var = dbl_mat(1, 4, vec![1.0, 2.0, 3.0, 4.0]);
    let r = corrcoef(&[x_const, y_var], 1).expect("corrcoef const");
    assert!(r[0].as_scalar_f64().expect("s").is_nan());

    // 2. Mismatched length error text
    let a = dbl_mat(1, 3, vec![1.0, 2.0, 3.0]);
    let b = dbl_mat(1, 4, vec![1.0, 2.0, 3.0, 4.0]);
    let err = cov(&[a, b], 1).unwrap_err();
    assert!(format!("{err}").contains("same number of elements"));
}

#[test]
fn test_b2_histcounts_edge_cases() {
    // 1. Values outside given edges are excluded
    let x = dbl_mat(1, 6, vec![-5.0, 1.0, 3.0, 5.0, 10.0, 20.0]);
    let edges = dbl_mat(1, 3, vec![0.0, 4.0, 8.0]);
    // Bin [0, 4) -> 1.0, 3.0 (2)
    // Bin [4, 8] -> 5.0 (1)
    let h = histcounts(&[x, edges], 1).expect("histcounts edges");
    if let Value::Double(arr) = &h[0] {
        assert_eq!(arr.as_slice(), &[2.0, 1.0]);
    } else {
        panic!("expected double array");
    }

    // 2. Automatic bin count fallback
    let x_auto = dbl_mat(1, 5, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let h_auto = histcounts(&[x_auto], 1).expect("histcounts auto");
    if let Value::Double(arr) = &h_auto[0] {
        assert_eq!(arr.shape()[0], 1);
        let sum: f64 = arr.as_slice().iter().sum();
        assert_eq!(sum, 5.0, "total count must equal numel");
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_movmean_and_movmedian_edge_cases() {
    // 1. Window length larger than vector: k = 10 on length 4
    let v = dbl_mat(1, 4, vec![2.0, 4.0, 6.0, 8.0]);
    let mm = movmean(&[v.clone(), Value::scalar(10.0)], 1).expect("movmean k>N");
    if let Value::Double(arr) = &mm[0] {
        // All windows cover the whole vector: mean is 5.0
        assert_eq!(arr.as_slice(), &[5.0, 5.0, 5.0, 5.0]);
    } else {
        panic!("expected double array");
    }

    let mmed = movmedian(&[v, Value::scalar(10.0)], 1).expect("movmedian k>N");
    if let Value::Double(arr) = &mmed[0] {
        // Median of [2, 4, 6, 8] is 5.0
        assert_eq!(arr.as_slice(), &[5.0, 5.0, 5.0, 5.0]);
    } else {
        panic!("expected double array");
    }

    // 2. Window validation error text
    let v2 = dbl_mat(1, 3, vec![1.0, 2.0, 3.0]);
    let err = movmean(&[v2, Value::scalar(-1.0)], 1).unwrap_err();
    assert!(format!("{err}").contains("positive integer scalar"));
}

#[test]
fn test_b2_cummax_and_cummin_edge_cases() {
    // 1. Monotonically decreasing input
    let dec = dbl_mat(1, 4, vec![40.0, 30.0, 20.0, 10.0]);
    let cmax = cummax(std::slice::from_ref(&dec), 1).expect("cummax dec");
    if let Value::Double(arr) = &cmax[0] {
        assert_eq!(arr.as_slice(), &[40.0, 40.0, 40.0, 40.0]);
    } else {
        panic!("expected double array");
    }

    let cmin = cummin(&[dec], 1).expect("cummin dec");
    if let Value::Double(arr) = &cmin[0] {
        assert_eq!(arr.as_slice(), &[40.0, 30.0, 20.0, 10.0]);
    } else {
        panic!("expected double array");
    }

    // 2. NaN propagation
    let v_nan = dbl_mat(1, 4, vec![5.0, f64::NAN, 10.0, 2.0]);
    let cmax_nan = cummax(&[v_nan], 1).expect("cummax nan");
    if let Value::Double(arr) = &cmax_nan[0] {
        assert_eq!(arr.as_slice()[0], 5.0);
        assert!(arr.as_slice()[1].is_nan());
        assert!(arr.as_slice()[2].is_nan());
        assert!(arr.as_slice()[3].is_nan());
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_zscore_edge_cases() {
    // 1. Constant vector -> std is 0 -> produces NaNs
    let v_const = dbl_mat(1, 4, vec![3.0; 4]);
    let z_const = zscore(&[v_const], 1).expect("zscore const");
    if let Value::Double(arr) = &z_const[0] {
        for &x in arr.as_slice() {
            assert!(x.is_nan(), "constant vector zscore must be NaN");
        }
    } else {
        panic!("expected double array");
    }

    // 2. Single element -> produces NaN
    let v_one = dbl_mat(1, 1, vec![10.0]);
    let z_one = zscore(&[v_one], 1).expect("zscore one");
    if let Value::Double(arr) = &z_one[0] {
        assert!(arr.as_slice()[0].is_nan());
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_sortrows_and_unique_edge_cases() {
    // 1. Duplicate rows
    let m = dbl_mat(3, 2, vec![2.0, 1.0, 2.0, 5.0, 3.0, 5.0]);
    let sr = sortrows(&[m], 1).expect("sortrows dup");
    if let Value::Double(arr) = &sr[0] {
        // Rows: [1 3], [2 5], [2 5]
        assert_eq!(arr.as_slice(), &[1.0, 2.0, 2.0, 3.0, 5.0, 5.0]);
    } else {
        panic!("expected double array");
    }

    // 2. Empty unique
    let empty = Value::Double(DenseArray::empty());
    let u_empty = unique(&[empty], 1).expect("unique empty");
    assert!(u_empty[0].is_empty());
}

#[test]
fn test_b2_percentiles_monotone_property() {
    let vals = vec![3.25, -2.7, 100.0, 0.0, 42.0, -10.0, 5.5, 8.8, 12.0, 99.0];
    let v = dbl_mat(1, vals.len(), vals);
    let mut prev = f64::NEG_INFINITY;
    for p in (0..=100).step_by(5) {
        let res = prctile(&[v.clone(), Value::scalar(p as f64)], 1).expect("prctile");
        let val = res[0].as_scalar_f64().expect("scalar");
        assert!(
            val >= prev,
            "percentiles must be monotonically non-decreasing: p={p}, val={val}, prev={prev}"
        );
        prev = val;
    }
}

#[test]
fn test_b2_variance_non_negative_property() {
    let test_cases = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![-10.0, 0.0, 10.0],
        vec![1e8, 1e8 + 1.0, 1e8 + 2.0],
        vec![0.001, 0.002, 0.003],
    ];
    for tc in test_cases {
        let v = dbl_mat(1, tc.len(), tc);
        let res = var_fn(&[v], 1).expect("var");
        assert!(
            res[0].as_scalar_f64().expect("scalar") >= 0.0,
            "variance must be non-negative"
        );
    }
}

#[test]
fn test_b2_corrcoef_diagonal_one_property() {
    let m = dbl_mat(
        4,
        3,
        vec![
            1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 15.0, 30.0, -5.0, -2.0, 0.0, 7.0,
        ],
    );
    let r = corrcoef(&[m], 1).expect("corrcoef");
    if let Value::Double(arr) = &r[0] {
        assert_eq!(arr.shape(), &[3, 3]);
        for i in 0..3 {
            let diag = arr.as_slice()[i + i * 3];
            assert!(
                (diag - 1.0).abs() < 1e-12,
                "diagonal must be 1.0, got {diag}"
            );
        }
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_movmean_of_constants_property() {
    for c in [0.0, 42.0, -99.5, 1e6] {
        let v = dbl_mat(1, 20, vec![c; 20]);
        for k in [1.0, 3.0, 5.0, 7.0, 25.0] {
            let mm = movmean(&[v.clone(), Value::scalar(k)], 1).expect("movmean");
            if let Value::Double(arr) = &mm[0] {
                for &val in arr.as_slice() {
                    assert!(
                        (val - c).abs() < 1e-12,
                        "movmean of constant {c} must equal {c}, got {val}"
                    );
                }
            } else {
                panic!("expected double array");
            }
        }
    }
}

#[test]
fn test_b2_csv_quoted_embedded_newlines_and_delimiters() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let temp_file = temp_dir.path().join("quoted_embedded.csv");
    let content = "1,2,\"3\"\n\"4\n\",5,6\n7,\"8,9\",10\n";
    std::fs::write(&temp_file, content).expect("write file");

    let res = readmatrix(&[str_val(&temp_file.to_string_lossy())], 1).expect("read");
    if let Value::Double(arr) = &res[0] {
        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(arr.as_slice()[0], 1.0);
        assert_eq!(arr.as_slice()[1], 4.0);
        assert_eq!(arr.as_slice()[2], 7.0);
        assert_eq!(arr.as_slice()[3], 2.0);
        assert_eq!(arr.as_slice()[4], 5.0);
        assert!(arr.as_slice()[5].is_nan()); // "8,9" cannot parse as single f64 -> NaN
        assert_eq!(arr.as_slice()[6], 3.0);
        assert_eq!(arr.as_slice()[7], 6.0);
        assert_eq!(arr.as_slice()[8], 10.0);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_csv_bom_and_crlf_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let temp_file = temp_dir.path().join("bom_crlf.csv");
    // UTF-8 BOM: 0xEF, 0xBB, 0xBF + CRLF lines
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"10,20\r\n30,40\r\n");
    std::fs::write(&temp_file, bytes).expect("write");

    let res = readmatrix(&[str_val(&temp_file.to_string_lossy())], 1).expect("read BOM CRLF");
    if let Value::Double(arr) = &res[0] {
        assert_eq!(arr.shape(), &[2, 2]);
        assert_eq!(arr.as_slice(), &[10.0, 30.0, 20.0, 40.0]);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_csv_empty_trailing_field_and_header_only() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    // 1. Empty trailing field -> NaN
    let f1 = temp_dir.path().join("trailing.csv");
    std::fs::write(&f1, "1,2,\n3,4,5\n").expect("write");
    let res1 = readmatrix(&[str_val(&f1.to_string_lossy())], 1).expect("read");
    if let Value::Double(arr) = &res1[0] {
        assert_eq!(arr.shape(), &[2, 3]);
        assert!(arr.as_slice()[4].is_nan()); // row 1, col 3 is empty -> NaN
        assert_eq!(arr.as_slice()[5], 5.0);
    } else {
        panic!("expected double array");
    }

    // 2. Header only file with NumHeaderLines=1 -> empty array
    let f2 = temp_dir.path().join("header_only.csv");
    std::fs::write(&f2, "col1,col2,col3\n").expect("write");
    let res2 = readmatrix(
        &[
            str_val(&f2.to_string_lossy()),
            str_val("NumHeaderLines"),
            Value::scalar(1.0),
        ],
        1,
    )
    .expect("read");
    assert!(res2[0].is_empty());
}

#[test]
fn test_b2_csv_nan_inf_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let f = temp_dir.path().join("nan_inf.csv");
    let orig = dbl_mat(1, 4, vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0]);
    writematrix(&[orig, str_val(&f.to_string_lossy())], 0).expect("write");
    let res = readmatrix(&[str_val(&f.to_string_lossy())], 1).expect("read");
    if let Value::Double(arr) = &res[0] {
        assert_eq!(arr.shape(), &[1, 4]);
        assert!(arr.as_slice()[0].is_nan());
        assert_eq!(arr.as_slice()[1], f64::INFINITY);
        assert_eq!(arr.as_slice()[2], f64::NEG_INFINITY);
        assert_eq!(arr.as_slice()[3], 0.0);
    } else {
        panic!("expected double array");
    }
}

#[test]
fn test_b2_csv_large_exponent_bit_exact_double_property() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let f = temp_dir.path().join("bit_exact.csv");
    let doubles = vec![
        f64::MIN_POSITIVE, // 2.2250738585072014e-308
        f64::MAX,          // 1.7976931348623157e+308
        f64::EPSILON,      // 2.220446049250313e-16
        1.2345678901234567e-100,
        9.876543210987654e+100,
    ];
    let orig = dbl_mat(1, doubles.len(), doubles.clone());
    writematrix(&[orig, str_val(&f.to_string_lossy())], 0).expect("write");
    let res = readmatrix(&[str_val(&f.to_string_lossy())], 1).expect("read");
    if let Value::Double(arr) = &res[0] {
        assert_eq!(arr.shape(), &[1, doubles.len()]);
        for (i, (&actual, &expected)) in arr.as_slice().iter().zip(doubles.iter()).enumerate() {
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "bit-exact mismatch at index {i}: actual={actual}, expected={expected}"
            );
        }
    } else {
        panic!("expected double array");
    }
}
