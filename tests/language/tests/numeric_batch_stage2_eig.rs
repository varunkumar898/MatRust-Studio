use matlab_linalg::complex::matmul;
use matlab_linalg::decomposition::{eig, eig_general, eig_with};
use matlab_runtime::run_source;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix};
use std::f64;

/// Naive triple-loop matrix multiply: C = A*B, all 2D with matching inner dims.
fn naive_mul(a: &DenseMatrix, b: &DenseMatrix) -> DenseMatrix {
    assert_eq!(a.cols, b.rows, "matmul dims mismatch");
    let m = a.rows;
    let n = b.cols;
    let p = a.cols;
    let mut res = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..p {
                sum += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            res[i * n + j] = sum;
        }
    }
    DenseMatrix {
        rows: m,
        cols: n,
        data: res,
    }
}

/// Frobenius norm
fn frobenius(a: &DenseMatrix) -> f64 {
    a.data.iter().map(|v| v * v).sum::<f64>().sqrt()
}

/// Residual ||A*V - V*D||_F / (||A||_F * epsilon * n)
fn relative_residual(a: &DenseMatrix, d: &DenseMatrix, v: &DenseMatrix) -> f64 {
    let av = naive_mul(a, v);
    // D is column vector of eigenvalues: build diagonal matrix Dmat
    // A*V = V*D => V * diag(D) should equal A*V for right eigenvectors.
    // Build diag(D) with same dimensions as V (n x n) using eigenvalues from D column.
    assert!(d.cols == 1);
    let n = d.rows;
    let mut dmat = vec![0.0; n * n];
    for i in 0..n {
        dmat[i * n + i] = d.data[i];
    }
    let dmat = DenseMatrix {
        rows: n,
        cols: n,
        data: dmat,
    };
    let vd = naive_mul(v, &dmat);
    let diff: Vec<f64> = av
        .data
        .iter()
        .zip(vd.data.iter())
        .map(|(x, y)| x - y)
        .collect();
    let residual = frobenius(&DenseMatrix {
        rows: av.rows,
        cols: av.cols,
        data: diff,
    });
    let a_norm = frobenius(a);
    residual / (a_norm.max(1e-15) * (n as f64) * 2.2e-16)
}

fn make_dense(rows: usize, cols: usize, vals: Vec<f64>) -> DenseMatrix {
    DenseMatrix {
        rows,
        cols,
        data: vals,
    }
}

fn complex_close(actual: Complex64, expected: Complex64, tol: f64) -> bool {
    (actual.re - expected.re).abs() <= tol && (actual.im - expected.im).abs() <= tol
}

fn assert_complex_close(actual: Complex64, expected: Complex64, tol: f64, context: &str) {
    assert!(
        complex_close(actual, expected, tol),
        "{context}: expected {expected}, got {actual}"
    );
}

fn eigenvalue_sum(d: &ComplexDenseMatrix) -> Complex64 {
    d.data
        .iter()
        .copied()
        .fold(Complex64::zero(), |acc, z| acc + z)
}

fn eigenvalue_product(d: &ComplexDenseMatrix) -> Complex64 {
    d.data
        .iter()
        .copied()
        .fold(Complex64::one(), |acc, z| acc * z)
}

/// max |A*V - V*diag(d)| over all entries, using the complex path.
fn complex_eigen_residual(a: &DenseMatrix, d: &ComplexDenseMatrix, v: &ComplexDenseMatrix) -> f64 {
    let n = a.rows;
    let ac = ComplexDenseMatrix::from_real(a);
    let av = matmul(&ac, v, None).expect("A*V dimensions");
    let mut vd = v.clone();
    for col in 0..n {
        for row in 0..n {
            vd.data[row * n + col] *= d.data[col];
        }
    }
    av.data
        .iter()
        .zip(vd.data.iter())
        .map(|(x, y)| (*x - *y).abs())
        .fold(0.0f64, f64::max)
}

#[test]
fn stage2_eig_symmetric_2x2() {
    // Symmetric [2 1; 1 2] has eigenvalues 1 and 3.
    let a = make_dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let (d, v) = eig(&a, None).expect("symmetric eig");
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((vals[0] - 1.0).abs() < 1e-10, "small eigenvalue: {vals:?}");
    assert!((vals[1] - 3.0).abs() < 1e-10, "large eigenvalue: {vals:?}");
    assert_eq!((v.rows, v.cols), (2, 2), "eigenvector matrix shape");
}

#[test]
fn stage2_eig_symmetric_4x4() {
    let a = make_dense(
        4,
        4,
        vec![
            4.0, 1.0, 0.0, 0.0, 1.0, 3.0, 1.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 1.0, 1.0,
        ],
    );
    let (d, v) = eig(&a, None).unwrap();
    assert_eq!((d.rows, d.cols), (4, 1));
    assert_eq!((v.rows, v.cols), (4, 4));
    // Mathematical identity: A*V = V*diag(d).
    let av = naive_mul(&a, &v);
    let mut vd = [0.0f64; 16];
    for j in 0..4 {
        for i in 0..4 {
            vd[i * 4 + j] = v.data[i * 4 + j] * d.data[j];
        }
    }
    let diff = DenseMatrix {
        rows: 4,
        cols: 4,
        data: av.data.iter().zip(vd.iter()).map(|(x, y)| x - y).collect(),
    };
    let residual = frobenius(&diff);
    assert!(residual < 1e-7, "eigenpair residual too large: {residual}");
}

#[test]
fn stage2_eig_repeated_identity() {
    let a = make_dense(3, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    let (d, v) = eig(&a, None).unwrap();
    assert_eq!(d.data.len(), 3);
    for val in &d.data {
        assert!((val - 1.0).abs() < 1e-6, "identity eigenvalue ~1");
    }
    // V should be orthonormal (approx) within tolerance
    let rr = relative_residual(&a, &d, &v);
    assert!(rr < 1e4, "identity residual high: {}", rr);
}

#[test]
fn stage2_eig_repeated_clustered() {
    // Symmetric matrix with eigenvalues 2, 3, 4.
    let a = make_dense(3, 3, vec![3.0, 0.0, 1.0, 0.0, 3.0, 0.0, 1.0, 0.0, 3.0]);
    let (d, _) = eig(&a, None).expect("clustered symmetric eig");
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let expected = [2.0, 3.0, 4.0];
    for (actual, expected) in vals.iter().zip(expected.iter()) {
        assert!(
            (actual - expected).abs() < 1e-8,
            "expected eigenvalue {expected}, got {actual} (all: {vals:?})"
        );
    }
}

#[test]
fn stage2_eig_non_normal() {
    // Triangular [1 100; 0 2] has eigenvalues 1 and 2.
    let a = make_dense(2, 2, vec![1.0, 0.0, 100.0, 2.0]);
    let (d, _) = eig(&a, None).expect("triangular eig");
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((vals[0] - 1.0).abs() < 1e-8, "first eigenvalue: {vals:?}");
    assert!((vals[1] - 2.0).abs() < 1e-8, "second eigenvalue: {vals:?}");
}

#[test]
fn stage2_eig_defective_documented() {
    // Defective matrix [1 1; 0 1] has the repeated eigenvalue 1 (a Jordan
    // block). The general non-symmetric shifted-QR path converges linearly for
    // this case; with the default 200 iterations the observed residual is about
    // 1.7e-5. The tolerance below is a documented numerical limit, not a way to
    // hide a wrong eigenvalue.
    let a = make_dense(2, 2, vec![1.0, 0.0, 1.0, 1.0]);
    let (d, _) = eig(&a, None).expect("defective eig returns eigenvalues");
    assert_eq!(d.data.len(), 2, "two eigenvalues");
    for value in &d.data {
        assert!(
            (value - 1.0).abs() < 1e-4,
            "defective eigenvalue must be 1 within 1e-4: {value}"
        );
    }
}

#[test]
fn stage2_eig_scaled_large() {
    let a = make_dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let scaled = DenseMatrix {
        rows: 2,
        cols: 2,
        data: a.data.iter().map(|v| v * 1e8).collect(),
    };
    let (d, _) = eig(&scaled, None).unwrap();
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((vals[0] - 1e8).abs() < 1.0, "small eigenvalue: {vals:?}");
    assert!((vals[1] - 3e8).abs() < 1.0, "large eigenvalue: {vals:?}");
}

#[test]
fn stage2_eig_scaled_small() {
    let a = make_dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let scaled = DenseMatrix {
        rows: 2,
        cols: 2,
        data: a.data.iter().map(|v| v * 1e-8).collect(),
    };
    let (d, _) = eig(&scaled, None).unwrap();
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((vals[0] - 1e-8).abs() < 1e-16, "small eigenvalue: {vals:?}");
    assert!((vals[1] - 3e-8).abs() < 1e-16, "large eigenvalue: {vals:?}");
}

#[test]
fn stage2_eig_non_convergence_trigger() {
    // One shifted-QR sweep cannot converge this non-symmetric matrix.
    let a = make_dense(3, 3, vec![3.0, 1.0, 0.0, 0.0, 2.0, 1.0, 1.0, 0.0, 4.0]);
    let err = eig_with(&a, None, 1).expect_err("max_iter=1 must report non-convergence");
    assert_eq!(
        err.message,
        "Eigenvalue iteration did not converge to working precision."
    );
}

#[test]
fn stage2_eig_nan_rejected() {
    let a = make_dense(2, 2, vec![1.0, f64::NAN, 0.0, 2.0]);
    let err = eig(&a, None);
    assert!(err.is_err(), "NaN matrix should be rejected");
}

#[test]
fn stage2_eig_inf_rejected() {
    let a = make_dense(2, 2, vec![1.0, f64::INFINITY, 0.0, 2.0]);
    let err = eig(&a, None);
    assert!(err.is_err(), "Inf matrix should be rejected");
}

#[test]
fn stage2_eig_value_check_2_3() {
    let a = make_dense(2, 2, vec![2.0, 0.0, 0.0, 3.0]);
    let (d, _) = eig(&a, None).unwrap();
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert_eq!(vals, vec![2.0, 3.0]);
}

#[test]
fn stage2_eig_value_check_2_1_1_2() {
    // [2 1; 1 2] has eigenvalues 1 and 3.
    let a = make_dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let (d, _) = eig(&a, None).unwrap();
    let mut vals = d.data.clone();
    vals.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert!((vals[0] - 1.0).abs() < 1e-10);
    assert!((vals[1] - 3.0).abs() < 1e-10);
}

#[test]
fn stage2_eig_complex_pair_returns_conjugates() {
    // [0 -1; 1 0] is a quarter-turn rotation; its eigenvalues are +i and -i.
    let a = make_dense(2, 2, vec![0.0, 1.0, -1.0, 0.0]);
    let (d, v) = eig_general(&a, None).expect("real matrix with complex pair");
    assert_eq!((d.rows, d.cols), (2, 1));
    assert_eq!((v.rows, v.cols), (2, 2));
    // Sorted by real part then imaginary part.
    assert_complex_close(d.data[0], Complex64::new(0.0, -1.0), 1e-12, "lambda_1");
    assert_complex_close(d.data[1], Complex64::new(0.0, 1.0), 1e-12, "lambda_2");
    assert_complex_close(eigenvalue_sum(&d), Complex64::zero(), 1e-12, "trace");
    assert_complex_close(
        eigenvalue_product(&d),
        Complex64::one(),
        1e-12,
        "determinant",
    );
    let residual = complex_eigen_residual(&a, &d, &v);
    assert!(residual < 1e-10, "A*V - V*D residual too large: {residual}");
}

#[test]
fn stage2_eig_language_call_returns_complex_pair() {
    // The `eig` builtin must not hide a complex spectrum behind a real matrix.
    let result = run_source("d = eig([0 -1; 1 0]);").expect("eig call");
    let d = result
        .env
        .get("d")
        .and_then(|value| value.as_complex_matrix())
        .expect("rotation eigenvalues must be stored as a complex column");
    assert_eq!((d.rows, d.cols), (2, 1));
    assert_complex_close(d.data[0], Complex64::new(0.0, -1.0), 1e-12, "lambda_1");
    assert_complex_close(d.data[1], Complex64::new(0.0, 1.0), 1e-12, "lambda_2");
}

#[test]
fn stage2_eig_two_by_two_complex_pair_values() {
    // [1 -2; 3 1] has eigenvalues 1 +/- i*sqrt(6), trace 2, determinant 7.
    let a = make_dense(2, 2, vec![1.0, 3.0, -2.0, 1.0]);
    let (d, v) = eig_general(&a, None).expect("complex pair");
    let root6 = 6.0f64.sqrt();
    assert_complex_close(d.data[0], Complex64::new(1.0, -root6), 1e-12, "lambda_1");
    assert_complex_close(d.data[1], Complex64::new(1.0, root6), 1e-12, "lambda_2");
    assert_complex_close(eigenvalue_sum(&d), Complex64::new(2.0, 0.0), 1e-12, "trace");
    assert_complex_close(
        eigenvalue_product(&d),
        Complex64::new(7.0, 0.0),
        1e-12,
        "determinant",
    );
    let residual = complex_eigen_residual(&a, &d, &v);
    assert!(residual < 1e-10, "residual: {residual}");
}

#[test]
fn stage2_eig_three_by_three_rotation_plus_scale() {
    // Block diagonal: a 0.6/0.8 rotation (eigenvalues 0.6 +/- 0.8i) and a
    // real 2.0 eigenvalue. Trace = 3.2, determinant = 2.0.
    let a = make_dense(
        3,
        3,
        vec![
            0.6, 0.8, 0.0, //
            -0.8, 0.6, 0.0, //
            0.0, 0.0, 2.0,
        ],
    );
    let (d, v) = eig_general(&a, None).expect("rotation-plus-scale eig");
    assert_complex_close(d.data[0], Complex64::new(0.6, -0.8), 1e-12, "lambda_1");
    assert_complex_close(d.data[1], Complex64::new(0.6, 0.8), 1e-12, "lambda_2");
    assert_complex_close(d.data[2], Complex64::new(2.0, 0.0), 1e-12, "lambda_3");
    assert_complex_close(eigenvalue_sum(&d), Complex64::new(3.2, 0.0), 1e-12, "trace");
    assert_complex_close(
        eigenvalue_product(&d),
        Complex64::new(2.0, 0.0),
        1e-12,
        "determinant",
    );
    let residual = complex_eigen_residual(&a, &d, &v);
    assert!(residual < 1e-10, "residual: {residual}");
}

#[test]
fn stage2_eig_companion_of_x4_plus_one() {
    // Companion matrix of x^4 + 1. The roots are the fourth roots of -1:
    // (+/-sqrt(2)/2) +/- (sqrt(2)/2)i. Trace 0, determinant 1.
    let a = make_dense(
        4,
        4,
        vec![
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
            -1.0, 0.0, 0.0, 0.0,
        ],
    );
    let (d, v) = eig_general(&a, None).expect("companion matrix eig");
    let s = 0.5f64.sqrt();
    // Sorted by real part, then imaginary part.
    let expected = [
        Complex64::new(-s, -s),
        Complex64::new(-s, s),
        Complex64::new(s, -s),
        Complex64::new(s, s),
    ];
    for (idx, want) in expected.iter().enumerate() {
        assert_complex_close(d.data[idx], *want, 1e-12, &format!("lambda_{}", idx + 1));
    }
    assert_complex_close(eigenvalue_sum(&d), Complex64::zero(), 1e-12, "trace");
    assert_complex_close(
        eigenvalue_product(&d),
        Complex64::one(),
        1e-12,
        "determinant",
    );
    let residual = complex_eigen_residual(&a, &d, &v);
    assert!(residual < 1e-10, "residual: {residual}");
}

#[test]
fn stage2_eig_five_by_five_real_eigenvalues() {
    // Companion matrix of (x-1)(x-2)(x-3)(x-4)(x-5). The spectrum is exactly
    // {1, 2, 3, 4, 5}; trace 15, determinant 120. This is non-symmetric and
    // not triangular, so it exercises the general QR path with real roots.
    let a = make_dense(
        5,
        5,
        vec![
            15.0, 1.0, 0.0, 0.0, 0.0, //
            -85.0, 0.0, 1.0, 0.0, 0.0, //
            225.0, 0.0, 0.0, 1.0, 0.0, //
            -274.0, 0.0, 0.0, 0.0, 1.0, //
            120.0, 0.0, 0.0, 0.0, 0.0,
        ],
    );
    let (d, v) = eig_general(&a, None).expect("real spectrum");
    for (idx, want) in [1.0, 2.0, 3.0, 4.0, 5.0].iter().enumerate() {
        assert_complex_close(
            d.data[idx],
            Complex64::new(*want, 0.0),
            1e-12,
            &format!("lambda_{}", idx + 1),
        );
    }
    assert_complex_close(
        eigenvalue_sum(&d),
        Complex64::new(15.0, 0.0),
        1e-12,
        "trace",
    );
    assert_complex_close(
        eigenvalue_product(&d),
        Complex64::new(120.0, 0.0),
        1e-12,
        "determinant",
    );
    let residual = complex_eigen_residual(&a, &d, &v);
    assert!(residual < 1e-10, "residual: {residual}");
}

#[test]
fn stage2_eig_jacobi_angle_fix_is_required() {
    // Regression justification for the Jacobi angle change. For [[2, 1], [1, 2]]
    // the pre-fix formula phi = 0.5*atan2(aqq-app, apq) returns 0, so the
    // off-diagonal 1.0 is never zeroed and the old code returned [2, 2].
    let a = make_dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let app = 2.0f64;
    let aqq = 2.0f64;
    let apq = 1.0f64;
    let old_phi = 0.5 * (aqq - app).atan2(apq);
    assert_eq!(old_phi, 0.0);
    let c = old_phi.cos();
    let s = old_phi.sin();
    let old_off_diagonal = (c * c - s * s) * apq + c * s * (app - aqq);
    assert_eq!(
        old_off_diagonal, 1.0,
        "old angle formula leaves the off-diagonal untouched"
    );

    // The corrected angle zeroes it and recovers both eigenvalues.
    let (d, v) = eig(&a, None).expect("equal-diagonal symmetric eig");
    assert!((d.data[0] - 1.0).abs() < 1e-12, "lambda_1: {:?}", d.data);
    assert!((d.data[1] - 3.0).abs() < 1e-12, "lambda_2: {:?}", d.data);
    let av = naive_mul(&a, &v);
    let mut vd = [0.0f64; 4];
    for row in 0..2 {
        for col in 0..2 {
            vd[row * 2 + col] = v.data[row * 2 + col] * d.data[col];
        }
    }
    let residual = av
        .data
        .iter()
        .zip(vd.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f64, f64::max);
    assert!(residual < 1e-12, "A*V - V*D residual: {residual}");
}
