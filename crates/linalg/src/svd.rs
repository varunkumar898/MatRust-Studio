use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

/// Result of Singular Value Decomposition: A = U * S * V^T.
#[derive(Debug, Clone, PartialEq)]
pub struct SvdResult {
    /// Orthogonal matrix U (m x m).
    pub u: DenseMatrix,
    /// Diagonal matrix S (m x n) with singular values on the main diagonal in descending order.
    pub s: DenseMatrix,
    /// Orthogonal matrix V (n x n).
    pub v: DenseMatrix,
    /// Vector of singular values in descending order.
    pub singular_values: Vec<f64>,
}

/// Compute full Singular Value Decomposition of matrix A (m x n): A = U * S * V^T.
/// Uses one-sided Jacobi rotations (Hestenes method) with Gram-Schmidt orthogonalization.
pub fn svd_decompose(a: &DenseMatrix, span: Option<Span>) -> Result<SvdResult, Diagnostic> {
    let m = a.rows;
    let n = a.cols;

    if m == 0 || n == 0 {
        return Ok(SvdResult {
            u: DenseMatrix::empty(),
            s: DenseMatrix::empty(),
            v: DenseMatrix::empty(),
            singular_values: Vec::new(),
        });
    }

    // If m < n, compute SVD of A^T (n x m): A^T = U_t * S_t * V_t^T => A = V_t * S_t^T * U_t^T
    if m < n {
        let a_t = crate::transpose(a);
        let t_res = svd_decompose(&a_t, span)?;
        // Transpose S_t (n x m) to S (m x n)
        let mut s_data = vec![0.0; m * n];
        let k_min = m.min(n);
        for i in 0..k_min {
            s_data[i * n + i] = t_res.singular_values[i];
        }
        return Ok(SvdResult {
            u: t_res.v,
            s: DenseMatrix {
                rows: m,
                cols: n,
                data: s_data,
            },
            v: t_res.u,
            singular_values: t_res.singular_values,
        });
    }

    // Now m >= n
    let mut b_data = a.data.clone(); // m x n
    let mut v_data = DenseMatrix::eye(n).data; // n x n

    let max_sweeps = 60;
    let tol = 1e-15;

    for _sweep in 0..max_sweeps {
        let mut max_rel = 0.0f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let mut alpha = 0.0f64;
                let mut beta = 0.0f64;
                let mut gamma = 0.0f64;
                for r in 0..m {
                    let bi = b_data[r * n + i];
                    let bj = b_data[r * n + j];
                    alpha += bi * bi;
                    beta += bj * bj;
                    gamma += bi * bj;
                }

                if alpha > 0.0 && beta > 0.0 {
                    let rel = gamma.abs() / (alpha * beta).sqrt();
                    if rel > max_rel {
                        max_rel = rel;
                    }
                }

                if gamma.abs() <= tol * (alpha * beta).sqrt().max(1e-15) {
                    continue;
                }

                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = if zeta >= 0.0 {
                    1.0 / (zeta + (1.0 + zeta * zeta).sqrt())
                } else {
                    -1.0 / (-zeta + (1.0 + zeta * zeta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;

                for r in 0..m {
                    let bi = b_data[r * n + i];
                    let bj = b_data[r * n + j];
                    b_data[r * n + i] = c * bi - s * bj;
                    b_data[r * n + j] = s * bi + c * bj;
                }
                for r in 0..n {
                    let vi = v_data[r * n + i];
                    let vj = v_data[r * n + j];
                    v_data[r * n + i] = c * vi - s * vj;
                    v_data[r * n + j] = s * vi + c * vj;
                }
            }
        }
        if max_rel < tol {
            break;
        }
    }

    // Compute column norms (singular values)
    let mut sing_vals = Vec::with_capacity(n);
    for j in 0..n {
        let mut col_norm_sq = 0.0;
        for r in 0..m {
            let val = b_data[r * n + j];
            col_norm_sq += val * val;
        }
        sing_vals.push(col_norm_sq.sqrt());
    }

    // Sort singular values in descending order
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a_idx, &b_idx| {
        sing_vals[b_idx]
            .partial_cmp(&sing_vals[a_idx])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_sing_vals: Vec<f64> = indices.iter().map(|&i| sing_vals[i]).collect();

    // Reorder and normalize columns of B to form the first n columns of U
    let mut u_data = vec![0.0; m * m];
    let mut sorted_v_data = vec![0.0; n * n];

    for (new_col, &orig_col) in indices.iter().enumerate() {
        let sigma = sorted_sing_vals[new_col];
        if sigma > 1e-15 {
            for r in 0..m {
                u_data[r * m + new_col] = b_data[r * n + orig_col] / sigma;
            }
        }
        for r in 0..n {
            sorted_v_data[r * n + new_col] = v_data[r * n + orig_col];
        }
    }

    // Gram-Schmidt orthogonalization to complete U to full m x m orthonormal matrix
    let mut current_cols = 0;
    for col in 0..n {
        if sorted_sing_vals[col] > 1e-15 {
            current_cols += 1;
        } else {
            break;
        }
    }

    // If current_cols < m, generate remaining orthogonal columns from standard basis
    if current_cols < m {
        for basis in 0..m {
            if current_cols >= m {
                break;
            }
            let mut cand = vec![0.0; m];
            cand[basis] = 1.0;

            // Project out all existing columns
            for c in 0..current_cols {
                let mut dot = 0.0;
                for r in 0..m {
                    dot += cand[r] * u_data[r * m + c];
                }
                for r in 0..m {
                    cand[r] -= dot * u_data[r * m + c];
                }
            }

            let mut norm_sq = 0.0;
            for r in 0..m {
                norm_sq += cand[r] * cand[r];
            }
            let norm = norm_sq.sqrt();
            if norm > 1e-8 {
                for r in 0..m {
                    u_data[r * m + current_cols] = cand[r] / norm;
                }
                current_cols += 1;
            }
        }
    }

    // Construct S matrix (m x n)
    let mut s_data = vec![0.0; m * n];
    for i in 0..n {
        s_data[i * n + i] = sorted_sing_vals[i];
    }

    Ok(SvdResult {
        u: DenseMatrix {
            rows: m,
            cols: m,
            data: u_data,
        },
        s: DenseMatrix {
            rows: m,
            cols: n,
            data: s_data,
        },
        v: DenseMatrix {
            rows: n,
            cols: n,
            data: sorted_v_data,
        },
        singular_values: sorted_sing_vals,
    })
}

/// Single-output `s = svd(A)`: returns column vector of singular values.
pub fn svd_values(a: &DenseMatrix, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    let res = svd_decompose(a, span)?;
    let k = res.singular_values.len();
    Ok(DenseMatrix {
        rows: k,
        cols: 1,
        data: res.singular_values,
    })
}

/// Three-output `[U, S, V] = svd(A)`.
pub fn svd_full(
    a: &DenseMatrix,
    span: Option<Span>,
) -> Result<(DenseMatrix, DenseMatrix, DenseMatrix), Diagnostic> {
    let res = svd_decompose(a, span)?;
    Ok((res.u, res.s, res.v))
}

/// Matrix rank `rank(A, tol)`.
/// Returns the number of singular values greater than the specified tolerance.
pub fn rank(
    a: &DenseMatrix,
    tol: Option<f64>,
    span: Option<Span>,
) -> Result<usize, Diagnostic> {
    if a.rows == 0 || a.cols == 0 {
        return Ok(0);
    }
    let res = svd_decompose(a, span)?;
    let sigma_max = res.singular_values.first().copied().unwrap_or(0.0);
    let threshold = tol.unwrap_or_else(|| {
        (a.rows.max(a.cols) as f64) * sigma_max * f64::EPSILON
    });
    let r = res
        .singular_values
        .iter()
        .filter(|&&s| s > threshold)
        .count();
    Ok(r)
}

/// Moore-Penrose pseudoinverse `pinv(A, tol)`: computed via SVD.
/// Satisfies the four Moore-Penrose identities:
/// 1. A * pinv(A) * A ≈ A
/// 2. pinv(A) * A * pinv(A) ≈ pinv(A)
/// 3. (A * pinv(A))^T ≈ A * pinv(A)
/// 4. (pinv(A) * A)^T ≈ pinv(A) * A
pub fn pinv(
    a: &DenseMatrix,
    tol: Option<f64>,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    let m = a.rows;
    let n = a.cols;
    if m == 0 || n == 0 {
        return Ok(DenseMatrix {
            rows: n,
            cols: m,
            data: Vec::new(),
        });
    }

    let res = svd_decompose(a, span)?;
    let sigma_max = res.singular_values.first().copied().unwrap_or(0.0);
    let threshold = tol.unwrap_or_else(|| {
        (m.max(n) as f64) * sigma_max * f64::EPSILON
    });

    // S_pinv has size n x m
    // pinv(A) = V * S_pinv * U^T
    // Compute S_pinv * U^T (size n x m): row i is (1 / s_i) * (column i of U)^T
    let mut s_u_t = vec![0.0; n * m];
    for (i, &s) in res.singular_values.iter().enumerate() {
        if s > threshold {
            let inv_s = 1.0 / s;
            for j in 0..m {
                s_u_t[i * m + j] = inv_s * res.u.data[j * m + i];
            }
        }
    }

    // Compute V * (S_pinv * U^T): V is n x n, s_u_t is n x m => result is n x m
    let mut pinv_data = vec![0.0; n * m];
    for i in 0..n {
        for k in 0..n {
            let v_ik = res.v.data[i * n + k];
            if v_ik != 0.0 {
                for j in 0..m {
                    pinv_data[i * m + j] += v_ik * s_u_t[k * m + j];
                }
            }
        }
    }

    Ok(DenseMatrix {
        rows: n,
        cols: m,
        data: pinv_data,
    })
}

/// Norm type selector for `norm(A, p)`.
#[derive(Debug, Clone, PartialEq)]
pub enum NormType {
    One,
    Two,
    Inf,
    NegInf,
    Fro,
    P(f64),
}

/// Matrix and vector norm `norm(A, p)`.
pub fn norm(
    a: &DenseMatrix,
    norm_type: &NormType,
    span: Option<Span>,
) -> Result<f64, Diagnostic> {
    if a.rows == 0 || a.cols == 0 {
        return Ok(0.0);
    }

    // Scalar: return absolute value
    if a.is_scalar() {
        return Ok(a.data[0].abs());
    }

    let is_vector = a.rows == 1 || a.cols == 1;

    if is_vector {
        match norm_type {
            NormType::One => {
                let sum: f64 = a.data.iter().map(|&x| x.abs()).sum();
                Ok(sum)
            }
            NormType::Two | NormType::Fro => {
                let sum_sq: f64 = a.data.iter().map(|&x| x * x).sum();
                Ok(sum_sq.sqrt())
            }
            NormType::Inf => {
                let max_abs = a
                    .data
                    .iter()
                    .fold(0.0f64, |acc, &x| acc.max(x.abs()));
                Ok(max_abs)
            }
            NormType::NegInf => {
                let min_abs = a
                    .data
                    .iter()
                    .fold(f64::INFINITY, |acc, &x| acc.min(x.abs()));
                Ok(min_abs)
            }
            NormType::P(p) => {
                if *p <= 0.0 {
                    return Err(Diagnostic::error(
                        "Vector norm p must be positive.",
                        span,
                    ));
                }
                let sum_p: f64 = a.data.iter().map(|&x| x.abs().powf(*p)).sum();
                Ok(sum_p.powf(1.0 / p))
            }
        }
    } else {
        // 2D Matrix norms
        match norm_type {
            NormType::One => {
                // Maximum absolute column sum
                let mut max_col_sum = 0.0f64;
                for c in 0..a.cols {
                    let mut col_sum = 0.0;
                    for r in 0..a.rows {
                        col_sum += a.data[r * a.cols + c].abs();
                    }
                    if col_sum > max_col_sum {
                        max_col_sum = col_sum;
                    }
                }
                Ok(max_col_sum)
            }
            NormType::Two => {
                // Largest singular value
                let res = svd_decompose(a, span)?;
                Ok(res.singular_values.first().copied().unwrap_or(0.0))
            }
            NormType::Inf => {
                // Maximum absolute row sum
                let mut max_row_sum = 0.0f64;
                for r in 0..a.rows {
                    let mut row_sum = 0.0;
                    for c in 0..a.cols {
                        row_sum += a.data[r * a.cols + c].abs();
                    }
                    if row_sum > max_row_sum {
                        max_row_sum = row_sum;
                    }
                }
                Ok(max_row_sum)
            }
            NormType::Fro => {
                // Frobenius norm: sqrt(sum(a_ij^2))
                let sum_sq: f64 = a.data.iter().map(|&x| x * x).sum();
                Ok(sum_sq.sqrt())
            }
            NormType::NegInf | NormType::P(_) => Err(Diagnostic::error(
                "The only valid matrix norms are 1, 2, inf, and 'fro'.",
                span,
            )),
        }
    }
}
