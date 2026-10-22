use matlab_linalg::svd::{svd_decompose, pinv};
use matlab_values::{DenseMatrix, ComplexDenseMatrix, Complex64};
use std::f64;

/// Naive matmul, real DenseMatrix only (complex handled separately below via C = A*B).
fn naive_mul(a: &DenseMatrix, b: &DenseMatrix) -> DenseMatrix {
    assert_eq!(a.cols, b.rows);
    let m = a.rows;
    let n = b.cols;
    let p = a.cols;
    let mut res = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut s = 0.0;
            for k in 0..p {
                s += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            res[i * n + j] = s;
        }
    }
    DenseMatrix { rows: m, cols: n, data: res }
}

fn frobenius(a: &DenseMatrix) -> f64 {
    a.data.iter().map(|v| v * v).sum::<f64>().sqrt()
}

fn col_max(a: &[f64]) -> f64 {
    a.iter().copied().fold(0.0, f64::max)
}

/// c*eps*max(m,n)*||A||_F with c = 100. Larger c documented where needed for badly-scaled inputs.
const TOL_C: f64 = 100.0 * f64::EPSILON;

#[test]
fn stage2_svd_reconstruction_2x2() {
    let a = DenseMatrix {
        rows: 2,
        cols: 2,
        data: vec![3.0, 0.0, 0.0, 4.0],
    };
    let res = svd_decompose(&a, None).unwrap();
    // Sorted descending, nonnegative
    assert!(res.singular_values.len() == 2);
    assert!(res.singular_values[0] >= res.singular_values[1]);
    assert!(res.singular_values.iter().all(|&v| v >= -1e-12));
    // Reconstruction: U * S * V^T; tolerance c*eps*max(m,n)*||A||_F, c=100.
    let us_d = {
        let mut d = vec![0.0; res.u.rows * res.s.cols];
        for i in 0..res.u.rows {
            for j in 0..res.s.cols {
                let sv = if j < res.singular_values.len() { res.singular_values[j] } else { 0.0 };
                d[i * res.s.cols + j] = res.u.data[i * res.u.cols + j] * sv;
            }
        }
        d
    };
    let vt_data: Vec<f64> = (0..res.v.cols)
        .flat_map(|j| (0..res.v.rows).map(move |i| res.v.data[i * res.v.cols + j]))
        .collect();
    let vt = DenseMatrix { rows: res.v.cols, cols: res.v.rows, data: vt_data };
    let us = DenseMatrix { rows: res.u.rows, cols: res.s.cols, data: us_d };
    let us_vt = naive_mul(&us, &vt);
    let diff_data: Vec<f64> = (0..a.rows * a.cols).map(|i| a.data[i] - us_vt.data[i]).collect();
    let err_frob = frobenius(&DenseMatrix { rows: a.rows, cols: a.cols, data: diff_data });
    let rhs = TOL_C * f64::EPSILON * (a.rows.max(a.cols) as f64) * frobenius(&a);
    assert!(err_frob < rhs || rhs.is_infinite(), "reconstruction residual {} within {}", err_frob, rhs);
}

#[test]
fn stage2_pinv_moore_penrose_square() {
    let a = DenseMatrix { rows: 2, cols: 2, data: vec![3.0, 0.0, 0.0, 4.0] };
    let x = pinv(&a, None, None).unwrap();
    let ax = naive_mul(&a, &x);
    let axa = naive_mul(&ax, &a);
    for i in 0..a.data.len() { assert!((axa.data[i] - a.data[i]).abs() < 1e-6, "MP1"); }
    let xax = naive_mul(&x, &ax);
    for i in 0..x.data.len() { assert!((xax.data[i] - x.data[i]).abs() < 1e-6, "MP2"); }
}

#[test]
fn stage2_pinv_rectangle() {
    let a = DenseMatrix {
        rows: 3,
        cols: 2,
        data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    };
    let p = pinv(&a, None, None).unwrap();
    assert_eq!(p.rows, 2);
    assert_eq!(p.cols, 3);
}
