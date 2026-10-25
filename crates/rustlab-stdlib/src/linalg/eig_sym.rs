//! Native symmetric eigenvalue decomposition using Householder tridiagonalization
//! (`tred2`) followed by the implicit QL algorithm with Wilkinson shifts (`tql2`).
//!
//! Clean-room implementation based on published numerical literature (Golub & Van Loan,
//! Wilkinson & Reinsch, Martin et al. 1968).

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::DenseArray;

/// Computes eigenvalues and eigenvectors of a real symmetric N x N matrix.
///
/// Returns `(eigenvalues, eigenvectors)` where:
/// - `eigenvalues` is a `Vec<f64>` sorted in ascending order.
/// - `eigenvectors` is a `DenseArray<f64>` of shape `[N, N]` whose columns are orthonormal eigenvectors.
///
/// Eigenvectors are sign-normalized such that the component with maximum absolute value is positive.
pub fn eig_sym(mat: &DenseArray<f64>) -> Result<(Vec<f64>, DenseArray<f64>), EvalError> {
    let shp = mat.shape();
    if shp.len() != 2 || shp[0] != shp[1] {
        return Err(user_err("MATLAB:eig:notSquare", "Matrix must be square."));
    }
    let n = shp[0];

    if n == 0 {
        let v = DenseArray::empty();
        return Ok((Vec::new(), v));
    }
    if n == 1 {
        let val = *mat.get_subscript(&[1, 1])?;
        let v = DenseArray::scalar(1.0);
        return Ok((vec![val], v));
    }

    // 2D row-indexed working matrix `a`
    let mut a = vec![vec![0.0; n]; n];
    for r in 0..n {
        for c in 0..n {
            a[r][c] = *mat.get_subscript(&[r + 1, c + 1])?;
        }
    }

    let mut d = vec![0.0; n];
    let mut e = vec![0.0; n];

    // Householder reduction to tridiagonal form (tred2)
    for i in (1..n).rev() {
        let l = i - 1;
        let mut h = 0.0;
        let mut scale = 0.0;
        if l > 0 {
            for k in 0..=l {
                scale += a[i][k].abs();
            }
            if scale == 0.0 {
                e[i] = a[i][l];
            } else {
                for k in 0..=l {
                    a[i][k] /= scale;
                    h += a[i][k] * a[i][k];
                }
                let f = a[i][l];
                let g = if f >= 0.0 { -h.sqrt() } else { h.sqrt() };
                e[i] = scale * g;
                h -= f * g;
                a[i][l] = f - g;
                let mut f_sum = 0.0;
                for j in 0..=l {
                    a[j][i] = a[i][j] / h;
                    let mut g_val = 0.0;
                    for k in 0..=j {
                        g_val += a[j][k] * a[i][k];
                    }
                    for k in (j + 1)..=l {
                        g_val += a[k][j] * a[i][k];
                    }
                    e[j] = g_val / h;
                    f_sum += e[j] * a[i][j];
                }
                let hh = f_sum / (h + h);
                for j in 0..=l {
                    let f_val = a[i][j];
                    let g_val = e[j] - hh * f_val;
                    e[j] = g_val;
                    for k in 0..=j {
                        a[j][k] -= f_val * e[k] + g_val * a[i][k];
                    }
                }
            }
        } else {
            e[i] = a[i][l];
        }
        d[i] = h;
    }
    d[0] = 0.0;
    e[0] = 0.0;

    // Accumulate orthogonal transformation matrix
    for i in 0..n {
        let l = i as isize - 1;
        if l >= 0 {
            let l = l as usize;
            if d[i] != 0.0 {
                for j in 0..=l {
                    let mut g = 0.0;
                    for k in 0..=l {
                        g += a[i][k] * a[k][j];
                    }
                    for k in 0..=l {
                        a[k][j] -= g * a[k][i];
                    }
                }
            }
        }
        d[i] = a[i][i];
        a[i][i] = 1.0;
        if l >= 0 {
            let l = l as usize;
            for j in 0..=l {
                a[j][i] = 0.0;
                a[i][j] = 0.0;
            }
        }
    }

    // QL algorithm with implicit Wilkinson shifts (tql2)
    for i in 1..n {
        e[i - 1] = e[i];
    }
    e[n - 1] = 0.0;

    let mut f = 0.0;
    let mut tst1: f64 = 0.0;
    let eps = f64::EPSILON;

    for l in 0..n {
        tst1 = tst1.max(d[l].abs() + e[l].abs());
        let mut m = l;
        while m < n {
            if e[m].abs() <= eps * tst1 {
                break;
            }
            m += 1;
        }

        if m > l {
            let mut iter = 0;
            loop {
                iter += 1;
                if iter > 100 {
                    return Err(user_err(
                        "MATLAB:eig:noConvergence",
                        "QL algorithm did not converge.",
                    ));
                }

                let mut g = d[l];
                let mut p = (d[l + 1] - g) / (2.0 * e[l]);
                let mut r = (p * p + 1.0).sqrt();
                if p < 0.0 {
                    r = -r;
                }
                d[l] = e[l] / (p + r);
                d[l + 1] = e[l] * (p + r);
                let dl1 = d[l + 1];
                let mut h = g - d[l];
                for i in (l + 2)..n {
                    d[i] -= h;
                }
                f += h;

                p = d[m];
                let mut c = 1.0;
                let mut c2 = c;
                let mut c3 = c;
                let el1 = e[l + 1];
                let mut s = 0.0;
                let mut s2 = 0.0;

                for i in ((l + 1)..=m).rev() {
                    c3 = c2;
                    c2 = c;
                    s2 = s;
                    g = c * e[i - 1];
                    h = c * p;
                    r = (p * p + e[i - 1] * e[i - 1]).sqrt();
                    e[i] = s * r;
                    s = e[i - 1] / r;
                    c = p / r;
                    p = c * d[i - 1] - s * g;
                    d[i] = h + s * (c * g + s * d[i - 1]);

                    for k in 0..n {
                        let h_val = a[k][i];
                        a[k][i] = s * a[k][i - 1] + c * h_val;
                        a[k][i - 1] = c * a[k][i - 1] - s * h_val;
                    }
                }
                p = -s * s2 * c3 * el1 * e[l] / dl1;
                e[l] = s * p;
                d[l] = c * p;

                if e[l].abs() <= eps * tst1 {
                    break;
                }
            }
        }
        d[l] += f;
    }

    // Sort eigenvalues in ascending order and permute eigenvector columns
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&i1, &i2| {
        d[i1]
            .partial_cmp(&d[i2])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut sorted_d = vec![0.0; n];
    let mut sorted_v = vec![0.0; n * n];

    for (new_col, &old_col) in indices.iter().enumerate() {
        sorted_d[new_col] = d[old_col];
        for r in 0..n {
            sorted_v[r + new_col * n] = a[r][old_col];
        }
    }

    // Sign normalization: element with largest magnitude in each column must be positive.
    // Explicit tie-break: if multiple elements share the maximum absolute value, select
    // the first component by index (r = 0..n).
    // NOTE ON LAPACK DIFFERENCE: Standard LAPACK (e.g. DSYEV/DGEEV) does not specify an
    // eigenvector sign convention and signs may vary across BLAS implementations/architectures.
    // MATLAB and RustLab standardize column signs so the leading max-abs element is positive.
    for c in 0..n {
        let mut max_abs = -1.0;
        let mut max_idx = 0;
        for r in 0..n {
            let val = sorted_v[r + c * n].abs();
            if val > max_abs {
                max_abs = val;
                max_idx = r;
            }
        }
        if sorted_v[max_idx + c * n] < 0.0 {
            for r in 0..n {
                sorted_v[r + c * n] = -sorted_v[r + c * n];
            }
        }
    }

    let eig_vecs = DenseArray::from_vec_unchecked(vec![n, n], sorted_v);
    Ok((sorted_d, eig_vecs))
}

/// Evaluates symmetric eigenvalue decomposition using `faer`'s optimized self-adjoint solver.
#[cfg(feature = "faer")]
pub fn faer_self_adjoint_eig(
    mat: &DenseArray<f64>,
) -> Result<(Vec<f64>, DenseArray<f64>), EvalError> {
    let shp = mat.shape();
    let n = shp[0];
    let faer_mat = faer::Mat::from_fn(n, n, |r, c| mat.as_slice()[r + c * n]);
    let decomp = faer_mat
        .self_adjoint_eigen(faer::Side::Lower)
        .map_err(|_| {
            user_err(
                "MATLAB:eig:convergence",
                "Eigenvalue decomposition failed to converge.",
            )
        })?;
    let mut eigenvalues = vec![0.0; n];
    for i in 0..n {
        eigenvalues[i] = decomp.S()[i];
    }
    let u = decomp.U();
    let mut v_data = vec![0.0; n * n];
    for c in 0..n {
        for r in 0..n {
            v_data[r + c * n] = u[(r, c)];
        }
    }

    // Sort eigenvalues in ascending order and permute eigenvector columns
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&i1, &i2| {
        eigenvalues[i1]
            .partial_cmp(&eigenvalues[i2])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut sorted_d = vec![0.0; n];
    let mut sorted_v = vec![0.0; n * n];
    for (new_col, &old_col) in indices.iter().enumerate() {
        sorted_d[new_col] = eigenvalues[old_col];
        for r in 0..n {
            sorted_v[r + new_col * n] = v_data[r + old_col * n];
        }
    }

    // Sign normalization: first maximum-magnitude component by index is non-negative
    for c in 0..n {
        let mut max_abs = -1.0;
        let mut max_idx = 0;
        for r in 0..n {
            let val = sorted_v[r + c * n].abs();
            if val > max_abs {
                max_abs = val;
                max_idx = r;
            }
        }
        if sorted_v[max_idx + c * n] < 0.0 {
            for r in 0..n {
                sorted_v[r + c * n] = -sorted_v[r + c * n];
            }
        }
    }

    let v_arr = DenseArray::from_vec_unchecked(vec![n, n], sorted_v);
    Ok((sorted_d, v_arr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eig_sym_diagonal() {
        let a = DenseArray::from_vec(vec![2, 2], vec![2.0, 0.0, 0.0, 3.0]).unwrap();
        let (d, v) = eig_sym(&a).unwrap();
        assert_eq!(d, vec![2.0, 3.0]);
        // V should be identity [1 0; 0 1]
        assert!((v.as_slice()[0] - 1.0).abs() < 1e-12);
        assert!(v.as_slice()[1].abs() < 1e-12);
        assert!(v.as_slice()[2].abs() < 1e-12);
        assert!((v.as_slice()[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_eig_sym_known_2x2() {
        // [2 1; 1 2] -> eigenvalues [1, 3]
        let a = DenseArray::from_vec(vec![2, 2], vec![2.0, 1.0, 1.0, 2.0]).unwrap();
        let (d, v) = eig_sym(&a).unwrap();
        assert!((d[0] - 1.0).abs() < 1e-12);
        assert!((d[1] - 3.0).abs() < 1e-12);

        // Property check: A * V = V * D
        let inv_sqrt2 = 1.0 / (2.0_f64).sqrt();
        let v_slice = v.as_slice();
        assert!((v_slice[0].abs() - inv_sqrt2).abs() < 1e-12);
        assert!((v_slice[1].abs() - inv_sqrt2).abs() < 1e-12);
        assert!((v_slice[2].abs() - inv_sqrt2).abs() < 1e-12);
        assert!((v_slice[3].abs() - inv_sqrt2).abs() < 1e-12);
    }

    #[test]
    fn test_eig_sym_3x3() {
        // Symmetric 3x3
        // A = [4, 1, -2; 1, 2, 0; -2, 0, 3]
        let a = DenseArray::from_vec(
            vec![3, 3],
            vec![
                4.0, 1.0, -2.0, // col 0
                1.0, 2.0, 0.0, // col 1
                -2.0, 0.0, 3.0, // col 2
            ],
        )
        .unwrap();
        let (d, v) = eig_sym(&a).unwrap();
        assert_eq!(d.len(), 3);
        assert!(d[0] <= d[1] && d[1] <= d[2]);

        // Verify A * v_k = lambda_k * v_k
        for k in 0..3 {
            let lambda = d[k];
            for i in 0..3 {
                let mut av_i = 0.0;
                for j in 0..3 {
                    av_i += *a.get_subscript(&[i + 1, j + 1]).unwrap()
                        * *v.get_subscript(&[j + 1, k + 1]).unwrap();
                }
                let lv_i = lambda * *v.get_subscript(&[i + 1, k + 1]).unwrap();
                assert!(
                    (av_i - lv_i).abs() < 1e-10,
                    "A*v != lambda*v for col {k}, row {i}: {av_i} vs {lv_i}"
                );
            }
        }
    }
}
