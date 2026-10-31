//! Matrix exponential via Padé approximation with scaling and squaring.
//!
//! Based on Higham (2005) "The Scaling and Squaring Method for the Matrix Exponential
//! Revisited" and Golub & Van Loan "Matrix Computations".

use crate::helpers::{math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::DenseArray;

/// Helper: multiply two square N x N DenseArray matrices.
fn matmul_square(a: &DenseArray<f64>, b: &DenseArray<f64>, n: usize) -> DenseArray<f64> {
    let mut out = vec![0.0; n * n];
    let a_s = a.as_slice();
    let b_s = b.as_slice();

    for j in 0..n {
        for i in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a_s[i + k * n] * b_s[k + j * n];
            }
            out[i + j * n] = sum;
        }
    }
    DenseArray::from_vec_unchecked(vec![n, n], out)
}

/// Helper: adds two N x N matrices with scalar weights: `alpha * A + beta * B`.
fn mat_add_weighted(
    a: &DenseArray<f64>,
    alpha: f64,
    b: &DenseArray<f64>,
    beta: f64,
    n: usize,
) -> DenseArray<f64> {
    let mut out = vec![0.0; n * n];
    let a_s = a.as_slice();
    let b_s = b.as_slice();
    for i in 0..(n * n) {
        out[i] = alpha * a_s[i] + beta * b_s[i];
    }
    DenseArray::from_vec_unchecked(vec![n, n], out)
}

/// Helper: solve linear system D * Y = N via Gauss-Jordan elimination with partial pivoting.
fn solve_matrix_eq(
    d: &DenseArray<f64>,
    n_mat: &DenseArray<f64>,
    n: usize,
) -> Result<DenseArray<f64>, EvalError> {
    let total_cols = 2 * n;
    let mut aug = vec![0.0; n * total_cols];

    for r in 0..n {
        for c in 0..n {
            aug[r * total_cols + c] = *d.get_subscript(&[r + 1, c + 1])?;
            aug[r * total_cols + n + c] = *n_mat.get_subscript(&[r + 1, c + 1])?;
        }
    }

    for col in 0..n {
        let mut pivot_row = col;
        let mut max_val = aug[col * total_cols + col].abs();
        for r in (col + 1)..n {
            let val = aug[r * total_cols + col].abs();
            if val > max_val {
                max_val = val;
                pivot_row = r;
            }
        }

        if max_val < 1e-15 {
            return Err(user_err(
                "MATLAB:expm:singularSystem",
                "Matrix denominator in Padé approximation is singular.",
            ));
        }

        if pivot_row != col {
            for c in 0..total_cols {
                aug.swap(col * total_cols + c, pivot_row * total_cols + c);
            }
        }

        let pivot = aug[col * total_cols + col];
        for c in col..total_cols {
            aug[col * total_cols + c] /= pivot;
        }

        for r in 0..n {
            if r != col {
                let factor = aug[r * total_cols + col];
                if factor != 0.0 {
                    for c in col..total_cols {
                        let sub = factor * aug[col * total_cols + c];
                        aug[r * total_cols + c] -= sub;
                    }
                }
            }
        }
    }

    let mut y_data = vec![0.0; n * n];
    for c in 0..n {
        for r in 0..n {
            y_data[r + c * n] = aug[r * total_cols + n + c];
        }
    }

    DenseArray::from_vec(vec![n, n], y_data).map_err(math_err)
}

/// Computes the matrix exponential `e^A` of a square matrix A.
pub fn expm(mat: &DenseArray<f64>) -> Result<DenseArray<f64>, EvalError> {
    let shp = mat.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:expm:notSquare", "Matrix must be square."));
    }
    let n = shp[0];

    if n == 0 {
        return Ok(DenseArray::from_vec_unchecked(vec![0, 0], Vec::new()));
    }
    if n == 1 {
        let val = *mat.get_subscript(&[1, 1])?;
        return Ok(DenseArray::from_vec_unchecked(vec![1, 1], vec![val.exp()]));
    }

    // Step 1: Infinity norm ||A||_inf = max_i sum_j |A_ij|
    let mut norm_inf: f64 = 0.0;
    for r in 0..n {
        let mut row_sum = 0.0;
        for c in 0..n {
            row_sum += mat.get_subscript(&[r + 1, c + 1])?.abs();
        }
        if row_sum > norm_inf {
            norm_inf = row_sum;
        }
    }

    // Step 2: Scaling parameter s such that ||A / 2^s||_inf <= 0.5
    let mut s: usize = 0;
    if norm_inf > 0.5 {
        let ratio = norm_inf / 0.5;
        s = ratio.log2().ceil() as usize;
    }
    let scale = 0.5_f64.powi(s as i32);

    let mut scaled_data = mat.as_slice().to_vec();
    for x in &mut scaled_data {
        *x *= scale;
    }
    let a_scaled = DenseArray::from_vec_unchecked(vec![n, n], scaled_data);

    // Step 3: Compute matrix powers X, X^2, X^3, X^4, X^5, X^6
    // Padé coefficients for order p = 6:
    // c_k = (2p - k)! * p! / ((2p)! * k! * (p - k)!)
    let c0 = 1.0;
    let c1 = 1.0 / 2.0;
    let c2 = 5.0 / 44.0;
    let c3 = 1.0 / 66.0;
    let c4 = 1.0 / 792.0;
    let c5 = 1.0 / 15840.0;
    let c6 = 1.0 / 665280.0;

    let mut eye_data = vec![0.0; n * n];
    for i in 0..n {
        eye_data[i + i * n] = 1.0;
    }
    let eye = DenseArray::from_vec_unchecked(vec![n, n], eye_data);

    let x1 = a_scaled;
    let x2 = matmul_square(&x1, &x1, n);
    let x3 = matmul_square(&x2, &x1, n);
    let x4 = matmul_square(&x2, &x2, n);
    let x5 = matmul_square(&x4, &x1, n);
    let x6 = matmul_square(&x4, &x2, n);

    // Even powers part: c0*I + c2*X^2 + c4*X^4 + c6*X^6
    let even = mat_add_weighted(&eye, c0, &x2, c2, n);
    let even = mat_add_weighted(&even, 1.0, &x4, c4, n);
    let even = mat_add_weighted(&even, 1.0, &x6, c6, n);

    // Odd powers part: c1*X + c3*X^3 + c5*X^5
    let odd = mat_add_weighted(&x1, c1, &x3, c3, n);
    let odd = mat_add_weighted(&odd, 1.0, &x5, c5, n);

    // N_6(X) = even + odd
    let num = mat_add_weighted(&even, 1.0, &odd, 1.0, n);
    // D_6(X) = even - odd
    let den = mat_add_weighted(&even, 1.0, &odd, -1.0, n);

    // Step 4: Solve D_6 * Y = N_6
    let mut y = solve_matrix_eq(&den, &num, n)?;

    // Step 5: Squaring s times: Y = Y^(2^s)
    for _ in 0..s {
        y = matmul_square(&y, &y, n);
    }

    Ok(y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expm_zero_matrix() {
        let a = DenseArray::from_vec_unchecked(vec![2, 2], vec![0.0, 0.0, 0.0, 0.0]);
        let res = expm(&a).unwrap();
        let s = res.as_slice();
        assert!((s[0] - 1.0).abs() < 1e-12);
        assert!(s[1].abs() < 1e-12);
        assert!(s[2].abs() < 1e-12);
        assert!((s[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_expm_diagonal_matrix() {
        let a = DenseArray::from_vec_unchecked(vec![2, 2], vec![1.0, 0.0, 0.0, 2.0]);
        let res = expm(&a).unwrap();
        let s = res.as_slice();
        assert!((s[0] - 1.0_f64.exp()).abs() < 1e-12);
        assert!(s[1].abs() < 1e-12);
        assert!(s[2].abs() < 1e-12);
        assert!((s[3] - 2.0_f64.exp()).abs() < 1e-12);
    }
}
