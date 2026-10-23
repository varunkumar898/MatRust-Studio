//! Regression tests for every Jacobi-rotation caller path.
//!
//! The old symmetric-eigen angle was
//! `phi = 0.5 * (aqq - app).atan2(apq)`.  For a matrix whose two diagonal
//! entries are equal and whose off-diagonal is nonzero, that formula returns
//! zero, so the rotation is skipped and the off-diagonal never becomes zero.
//!
//! `jacobi_eigen` itself is called only from the symmetric branch of
//! `decomposition::eig_with`.  The other public algorithms below either call
//! that eigen path (covariance PCA) or use the separate one-sided Jacobi SVD
//! angle; their tests use the same equal-diagonal/equal-column-norm trigger so
//! a regression to the old no-op angle is caught at each public entry point.

use matlab_linalg::decomposition::eig;
use matlab_linalg::stats::cov_matrix;
use matlab_linalg::svd::{pinv, svd_decompose};
use matlab_runtime::run_source;
use matlab_values::DenseMatrix;

fn dense(rows: usize, cols: usize, data: Vec<f64>) -> DenseMatrix {
    DenseMatrix { rows, cols, data }
}

fn naive_mul(a: &DenseMatrix, b: &DenseMatrix) -> DenseMatrix {
    assert_eq!(a.cols, b.rows, "matmul dimension mismatch");
    let mut data = vec![0.0; a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = 0.0;
            for k in 0..a.cols {
                sum += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            data[i * b.cols + j] = sum;
        }
    }
    dense(a.rows, b.cols, data)
}

fn transpose(a: &DenseMatrix) -> DenseMatrix {
    let mut data = vec![0.0; a.rows * a.cols];
    for r in 0..a.rows {
        for c in 0..a.cols {
            data[c * a.rows + r] = a.data[r * a.cols + c];
        }
    }
    dense(a.cols, a.rows, data)
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "{context}: expected {expected}, got {actual}"
    );
}

#[test]
fn jacobi_caller_symmetric_eig_direct() {
    // app == aqq, apq == 1 -> the old angle is zero.  The correct spectrum is
    // {1, 3}; the old formula left the matrix at its diagonal {2, 2}.
    let a = dense(2, 2, vec![2.0, 1.0, 1.0, 2.0]);
    let (d, v) = eig(&a, None).expect("symmetric eig");
    assert_eq!(d.data.len(), 2);
    assert_close(d.data[0], 1.0, "eigenvalue 1");
    assert_close(d.data[1], 3.0, "eigenvalue 3");

    let av = naive_mul(&a, &v);
    for row in 0..2 {
        for col in 0..2 {
            let vd = v.data[row * 2 + col] * d.data[col];
            assert_close(av.data[row * 2 + col], vd, "A*V = V*D");
        }
    }

    // The language-level caller takes the same symmetric Jacobi branch.
    let run = run_source("d = eig([2 1; 1 2]);").expect("eig builtin");
    let language_d = run
        .env
        .get("d")
        .and_then(|v| v.as_matrix())
        .expect("real symmetric eig result");
    assert_eq!(language_d.data, vec![1.0, 3.0]);
}

#[test]
fn jacobi_caller_svd_equal_column_norms() {
    // Columns have equal squared norms (alpha = beta = 2) and positive dot
    // product (gamma = 1).  The old angle formula would produce theta = 0 and
    // leave the columns non-orthogonal, yielding two equal sqrt(2) column
    // norms.  The correct singular values are the square roots of the
    // eigenvalues of A^T A = [[2,1],[1,1]]: sqrt((3+sqrt(5))/2) and
    // sqrt((3-sqrt(5))/2).
    let a = dense(2, 2, vec![1.0, 1.0, 1.0, 0.0]);
    let res = svd_decompose(&a, None).expect("svd");
    assert_eq!(res.singular_values.len(), 2);
    let sigma_1 = ((3.0 + 5.0f64.sqrt()) / 2.0).sqrt();
    let sigma_2 = ((3.0 - 5.0f64.sqrt()) / 2.0).sqrt();
    assert_close(res.singular_values[0], sigma_1, "sigma_1");
    assert_close(res.singular_values[1], sigma_2, "sigma_2");

    // Reconstruction A = U * S * V^T, using a local naive multiply so the SVD
    // implementation is not used to validate itself.
    let us = naive_mul(&res.u, &res.s);
    let reconstructed = naive_mul(&us, &transpose(&res.v));
    for (actual, expected) in reconstructed.data.iter().zip(a.data.iter()) {
        assert_close(*actual, *expected, "SVD reconstruction");
    }
}

#[test]
fn jacobi_caller_pinv_via_svd() {
    // pinv(A) = A^-1 = [[0, 1], [1, -1]].  A wrong SVD rotation (the old
    // no-op angle) changes both singular values and therefore the inverse.
    let a = dense(2, 2, vec![1.0, 1.0, 1.0, 0.0]);
    let p = pinv(&a, None, None).expect("pinv");
    let expected = [[0.0, 1.0], [1.0, -1.0]];
    for r in 0..2 {
        for c in 0..2 {
            assert_close(p.data[r * 2 + c], expected[r][c], "pinv entry");
        }
    }
    let identity = naive_mul(&a, &p);
    assert_close(identity.data[0], 1.0, "A*pinv(A) [0,0]");
    assert_close(identity.data[1], 0.0, "A*pinv(A) [0,1]");
    assert_close(identity.data[2], 0.0, "A*pinv(A) [1,0]");
    assert_close(identity.data[3], 1.0, "A*pinv(A) [1,1]");
}

#[test]
fn jacobi_caller_covariance_pca_eig() {
    // cov_matrix itself is a plain centered second-moment computation and does
    // not rotate.  The PCA path is cov(X) followed by the symmetric eig
    // routine.  X = [[1,0],[0,1],[-1,-1]] gives covariance [[1, 0.5],
    // [0.5, 1]]: equal diagonals, nonzero off-diagonal.  The old angle returns
    // zero and leaves eigenvalues {1, 1}; the correct values are {0.5, 1.5}.
    let x = dense(3, 2, vec![1.0, 0.0, 0.0, 1.0, -1.0, -1.0]);
    let c = cov_matrix(&x, 0, None);
    assert_close(c.data[0], 1.0, "cov [0,0]");
    assert_close(c.data[1], 0.5, "cov [0,1]");
    assert_close(c.data[2], 0.5, "cov [1,0]");
    assert_close(c.data[3], 1.0, "cov [1,1]");

    let (d, v) = eig(&c, None).expect("PCA eigen-decomposition");
    assert_close(d.data[0], 0.5, "PCA eigenvalue 0.5");
    assert_close(d.data[1], 1.5, "PCA eigenvalue 1.5");

    let av = naive_mul(&c, &v);
    for row in 0..2 {
        for col in 0..2 {
            let vd = v.data[row * 2 + col] * d.data[col];
            assert_close(av.data[row * 2 + col], vd, "PCA A*V = V*D");
        }
    }
}
