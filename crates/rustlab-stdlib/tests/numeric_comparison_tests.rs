//! Comparison tests verifying numerical equivalence between:
//! 1. `matrixmultiply::dgemm` vs fallback column-major GEMM
//! 2. `faer` self-adjoint eigendecomposition vs native Householder+QL eigendecomposition
//! 3. `faer` SVD vs native Jacobi SVD

use rustlab_math::{DenseArray, Value};
use rustlab_stdlib::linalg::eig_sym::{eig_sym, faer_self_adjoint_eig};
use rustlab_stdlib::linalg::svd::{jacobi_svd_native, svd_fn};

fn fallback_dgemm(m: usize, k: usize, n: usize, a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut c = vec![0.0; m * n];
    for col in 0..n {
        let c_col = &mut c[col * m..(col + 1) * m];
        for p in 0..k {
            let b_val = b[col * k + p];
            if b_val != 0.0 {
                let a_col = &a[p * m..(p + 1) * m];
                for (c_elem, a_elem) in c_col.iter_mut().zip(a_col.iter()) {
                    *c_elem += a_elem * b_val;
                }
            }
        }
    }
    c
}

fn matrixmultiply_dgemm(m: usize, k: usize, n: usize, a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut c = vec![0.0; m * n];
    unsafe {
        matrixmultiply::dgemm(
            m,
            k,
            n,
            1.0,
            a.as_ptr(),
            1,
            m as isize,
            b.as_ptr(),
            1,
            k as isize,
            0.0,
            c.as_mut_ptr(),
            1,
            m as isize,
        );
    }
    c
}

#[test]
fn test_numeric_comparison_matrixmultiply_vs_fallback_gemm() {
    let m = 16;
    let k = 24;
    let n = 12;

    let mut a_data = Vec::with_capacity(m * k);
    for col in 0..k {
        for row in 0..m {
            a_data.push(((row + 1) as f64 * 1.5) - ((col + 1) as f64 * 0.7));
        }
    }

    let mut b_data = Vec::with_capacity(k * n);
    for col in 0..n {
        for row in 0..k {
            b_data.push(((row + 1) as f64 * 0.3) + ((col + 1) as f64 * 2.1));
        }
    }

    let c_fallback = fallback_dgemm(m, k, n, &a_data, &b_data);
    let c_matrixmultiply = matrixmultiply_dgemm(m, k, n, &a_data, &b_data);

    assert_eq!(c_fallback.len(), c_matrixmultiply.len());
    let mut max_err = 0.0f64;
    for (f, m) in c_fallback.iter().zip(c_matrixmultiply.iter()) {
        let diff = (f - m).abs();
        if diff > max_err {
            max_err = diff;
        }
    }

    assert!(
        max_err < 1e-12,
        "Matrix multiply mismatch between fallback and matrixmultiply: max error = {max_err}"
    );
}

#[test]
fn test_numeric_comparison_faer_vs_native_eig_sym() {
    // Construct a symmetric 4x4 matrix with distinct eigenvalues
    let a_data = vec![
        10.0, 2.0, 1.0, 0.5, 2.0, 8.0, 3.0, 1.0, 1.0, 3.0, 6.0, 2.0, 0.5, 1.0, 2.0, 4.0,
    ];
    let a = DenseArray::from_vec_unchecked(vec![4, 4], a_data);

    let (d_native, v_native) = eig_sym(&a).expect("native eig_sym failed");
    let (d_faer, v_faer) = faer_self_adjoint_eig(&a).expect("faer eig failed");

    assert_eq!(d_native.len(), 4);
    assert_eq!(d_faer.len(), 4);

    // Eigenvalues must match to within 1e-12
    for i in 0..4 {
        let err = (d_native[i] - d_faer[i]).abs();
        assert!(
            err < 1e-12,
            "Eigenvalue mismatch at index {i}: native={}, faer={}, diff={err}",
            d_native[i],
            d_faer[i]
        );
    }

    // Verify reconstruction A = V * D * V^T for both
    for (name, v, d) in [("native", &v_native, &d_native), ("faer", &v_faer, &d_faer)] {
        for r in 0..4 {
            for c in 0..4 {
                let mut sum = 0.0f64;
                for (k, &d_k) in d.iter().enumerate() {
                    let v_rk = *v.get_subscript(&[r + 1, k + 1]).unwrap();
                    let v_ck = *v.get_subscript(&[c + 1, k + 1]).unwrap();
                    sum += v_rk * d_k * v_ck;
                }
                let expected: f64 = *a.get_subscript(&[r + 1, c + 1]).unwrap();
                let residual = (sum - expected).abs();
                assert!(
                    residual < 1e-11,
                    "{name} reconstruction failed at ({r},{c}): got {sum}, expected {expected}"
                );
            }
        }
    }
}

#[test]
fn test_numeric_comparison_faer_vs_native_svd() {
    // 4x3 rectangular matrix
    let a_data = vec![1.0, 4.0, 7.0, 2.0, 2.0, 5.0, 8.0, 1.0, 3.0, 6.0, 9.0, 4.0];
    let a = DenseArray::from_vec_unchecked(vec![4, 3], a_data);

    // Native Jacobi SVD
    let (s_native, u_native, v_native) = jacobi_svd_native(&a).expect("jacobi_svd_native failed");

    // Faer SVD via svd_fn built-in route
    let res = svd_fn(&[Value::Double(a.clone())], 3).expect("svd_fn failed");
    let (u_faer, s_faer, v_faer) = match (&res[0], &res[1], &res[2]) {
        (Value::Double(u), Value::Double(s), Value::Double(v)) => (u, s, v),
        _ => panic!("Expected double arrays from svd_fn"),
    };

    assert_eq!(s_native.len(), 3);
    assert_eq!(u_native.shape(), &[4, 3]);
    assert_eq!(v_native.shape(), &[3, 3]);

    // Compare singular values between faer and native Jacobi SVD
    for (i, &s_nat) in s_native.iter().enumerate() {
        let faer_s_val = *s_faer.get_subscript(&[i + 1, i + 1]).unwrap();
        let diff = (s_nat - faer_s_val).abs();
        assert!(
            diff < 1e-11,
            "Singular value mismatch at index {i}: native={}, faer={}, diff={diff}",
            s_nat,
            faer_s_val
        );
    }

    // Verify reconstruction for native: A = U * S * V^T
    for r in 0..4 {
        for c in 0..3 {
            let mut sum_native = 0.0f64;
            let mut sum_faer = 0.0f64;
            for (k, &s_nat) in s_native.iter().enumerate() {
                let u_nk = *u_native.get_subscript(&[r + 1, k + 1]).unwrap();
                let v_ck = *v_native.get_subscript(&[c + 1, k + 1]).unwrap();
                sum_native += u_nk * s_nat * v_ck;

                let u_fk = *u_faer.get_subscript(&[r + 1, k + 1]).unwrap();
                let s_fk = *s_faer.get_subscript(&[k + 1, k + 1]).unwrap();
                let v_fk = *v_faer.get_subscript(&[c + 1, k + 1]).unwrap();
                sum_faer += u_fk * s_fk * v_fk;
            }
            let expected: f64 = *a.get_subscript(&[r + 1, c + 1]).unwrap();
            let err_native = (sum_native - expected).abs();
            let err_faer = (sum_faer - expected).abs();
            assert!(
                err_native < 1e-11,
                "Native SVD reconstruction error at ({r},{c}): got {sum_native}, expected {expected}, diff={err_native}"
            );
            assert!(
                err_faer < 1e-11,
                "Faer SVD reconstruction error at ({r},{c}): got {sum_faer}, expected {expected}, diff={err_faer}"
            );
        }
    }
}
