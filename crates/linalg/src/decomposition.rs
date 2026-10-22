use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

/// Result of an LU factorization with partial pivoting: P * A = L * U.
#[derive(Debug, Clone, PartialEq)]
pub struct LuDecomp {
    /// Unit lower triangular matrix (or permuted lower triangular in 2-output form).
    pub l: DenseMatrix,
    /// Upper triangular matrix.
    pub u: DenseMatrix,
    /// Permutation matrix P such that P * A = L * U.
    pub p: DenseMatrix,
    /// Permutation vector: row i of P*A corresponds to row `p_vec[i]` of A.
    pub p_vec: Vec<usize>,
    /// Number of row swaps performed during pivoting (used for determinant sign).
    pub num_swaps: usize,
}

/// Compute LU factorization with partial row pivoting for a matrix A (m x n).
/// Produces P * A = L * U, where L is unit lower triangular and U is upper triangular.
pub fn pivoted_lu(a: &DenseMatrix, span: Option<Span>) -> Result<LuDecomp, Diagnostic> {
    let m = a.rows;
    let n = a.cols;
    if m == 0 || n == 0 {
        return Ok(LuDecomp {
            l: DenseMatrix::empty(),
            u: DenseMatrix::empty(),
            p: DenseMatrix::empty(),
            p_vec: Vec::new(),
            num_swaps: 0,
        });
    }

    let k_max = m.min(n);
    let mut u_data = a.data.clone();
    let mut p_vec: Vec<usize> = (0..m).collect();
    // L starts as zeros(m, k_max) with 1.0 on main diagonal
    let mut l_data = vec![0.0; m * k_max];
    for i in 0..k_max {
        l_data[i * k_max + i] = 1.0;
    }
    let mut num_swaps = 0;

    for k in 0..k_max {
        // Find pivot: row r >= k with maximum absolute value in column k of U
        let mut max_val = u_data[k * n + k].abs();
        let mut pivot_row = k;
        for r in (k + 1)..m {
            let val = u_data[r * n + k].abs();
            if val > max_val {
                max_val = val;
                pivot_row = r;
            }
        }

        // Swap rows if necessary
        if pivot_row != k {
            // Swap in U
            for j in 0..n {
                u_data.swap(k * n + j, pivot_row * n + j);
            }
            // Swap in P vector
            p_vec.swap(k, pivot_row);
            // Swap multipliers already in L (columns 0..k)
            for j in 0..k {
                l_data.swap(k * k_max + j, pivot_row * k_max + j);
            }
            num_swaps += 1;
        }

        let pivot = u_data[k * n + k];
        if pivot.abs() > 1e-15 {
            for i in (k + 1)..m {
                let factor = u_data[i * n + k] / pivot;
                l_data[i * k_max + k] = factor;
                u_data[i * n + k] = 0.0;
                for j in (k + 1)..n {
                    u_data[i * n + j] -= factor * u_data[k * n + j];
                }
            }
        }
    }

    // Construct permutation matrix P (m x m)
    let mut p_data = vec![0.0; m * m];
    for (i, &orig_row) in p_vec.iter().enumerate() {
        p_data[i * m + orig_row] = 1.0;
    }

    // U has dimensions k_max x n
    // Truncate U to k_max rows if m > n
    let u_rows = k_max;
    let mut final_u_data = vec![0.0; u_rows * n];
    for r in 0..u_rows {
        for c in 0..n {
            final_u_data[r * n + c] = u_data[r * n + c];
        }
    }

    Ok(LuDecomp {
        l: DenseMatrix {
            rows: m,
            cols: k_max,
            data: l_data,
        },
        u: DenseMatrix {
            rows: u_rows,
            cols: n,
            data: final_u_data,
        },
        p: DenseMatrix {
            rows: m,
            cols: m,
            data: p_data,
        },
        p_vec,
        num_swaps,
    })
}

/// Single-output `Y = lu(A)`.
/// Returns strictly lower triangular multipliers and upper triangular U in a single matrix Y.
pub fn lu_factor(a: &DenseMatrix, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            "Single-output lu(A) requires a square matrix.",
            span,
        ));
    }
    let decomp = pivoted_lu(a, span)?;
    let n = a.rows;
    let mut y_data = vec![0.0; n * n];
    for r in 0..n {
        for c in 0..n {
            if r > c {
                // strictly lower triangle of L
                y_data[r * n + c] = decomp.l.data[r * n + c];
            } else {
                // diagonal and upper triangle of U
                y_data[r * n + c] = decomp.u.data[r * n + c];
            }
        }
    }
    Ok(DenseMatrix {
        rows: n,
        cols: n,
        data: y_data,
    })
}

/// Two-output `[L, U] = lu(A)`.
/// In MATLAB, L is a permuted unit lower triangular matrix such that A = L * U.
pub fn lu_two_output(
    a: &DenseMatrix,
    span: Option<Span>,
) -> Result<(DenseMatrix, DenseMatrix), Diagnostic> {
    let decomp = pivoted_lu(a, span)?;
    // In MATLAB: [L, U] = lu(A) satisfies A = L * U, where L = P' * L_unit.
    // P * A = L_unit * U => A = P^T * L_unit * U.
    let m = decomp.l.rows;
    let k = decomp.l.cols;
    let mut l_perm = vec![0.0; m * k];
    for (i, &orig_row) in decomp.p_vec.iter().enumerate() {
        // row i of L_unit was moved from orig_row, so row orig_row of permuted L is row i of L_unit
        for c in 0..k {
            l_perm[orig_row * k + c] = decomp.l.data[i * k + c];
        }
    }
    let l_mat = DenseMatrix {
        rows: m,
        cols: k,
        data: l_perm,
    };
    Ok((l_mat, decomp.u))
}

/// Three-output `[L, U, P] = lu(A)`.
/// Satisfies P * A = L * U, where L is unit lower triangular and P is a permutation matrix.
pub fn lu_three_output(
    a: &DenseMatrix,
    span: Option<Span>,
) -> Result<(DenseMatrix, DenseMatrix, DenseMatrix), Diagnostic> {
    let decomp = pivoted_lu(a, span)?;
    Ok((decomp.l, decomp.u, decomp.p))
}

/// Determinant of a square matrix `det(A)`.
pub fn det(a: &DenseMatrix, span: Option<Span>) -> Result<f64, Diagnostic> {
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix must be square to compute determinant: {}x{}",
                a.rows, a.cols
            ),
            span,
        ));
    }
    let n = a.rows;
    if n == 0 {
        return Ok(1.0);
    }
    if n == 1 {
        return Ok(a.data[0]);
    }
    let decomp = pivoted_lu(a, span)?;
    let mut d = if decomp.num_swaps % 2 == 1 { -1.0 } else { 1.0 };
    for i in 0..n {
        let u_ii = decomp.u.data[i * n + i];
        d *= u_ii;
    }
    Ok(d)
}

/// Matrix trace `trace(A)`: sum of main diagonal elements.
pub fn trace(a: &DenseMatrix, _span: Option<Span>) -> Result<f64, Diagnostic> {
    let d = a.rows.min(a.cols);
    if d == 0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    for i in 0..d {
        sum += a.data[i * a.cols + i];
    }
    Ok(sum)
}

/// Matrix inverse `inv(A)`: computed via pivoted LU factorization and forward/backward substitution.
pub fn inv(a: &DenseMatrix, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix must be square to be inverted: {}x{}",
                a.rows, a.cols
            ),
            span,
        ));
    }
    let n = a.rows;
    if n == 0 {
        return Ok(DenseMatrix::empty());
    }
    if n == 1 {
        let val = a.data[0];
        if val == 0.0 {
            return Err(Diagnostic::error(
                "Matrix is singular to working precision.",
                span,
            ));
        }
        return Ok(DenseMatrix::scalar(1.0 / val));
    }

    let decomp = pivoted_lu(a, span)?;

    // Check singularity
    let max_abs = decomp
        .u
        .data
        .iter()
        .fold(0.0f64, |acc, &x| acc.max(x.abs()))
        .max(1.0);
    for i in 0..n {
        let diag = decomp.u.data[i * n + i].abs();
        if diag <= 1e-15 * max_abs {
            return Err(Diagnostic::error(
                "Matrix is singular to working precision.",
                span,
            ));
        }
    }

    // Solve A * X = I_n using forward and backward substitution on columns of I_n
    let mut x_data = vec![0.0; n * n];
    let mut y = vec![0.0; n];
    let mut col_sol = vec![0.0; n];

    for col in 0..n {
        // Permuted RHS b' = P * e_col
        // b'_i is 1.0 if decomp.p_vec[i] == col else 0.0
        // Forward solve: L * y = b'
        for i in 0..n {
            let b_val = if decomp.p_vec[i] == col { 1.0 } else { 0.0 };
            let mut sum = 0.0;
            for j in 0..i {
                sum += decomp.l.data[i * n + j] * y[j];
            }
            y[i] = b_val - sum;
        }

        // Backward solve: U * x = y
        for i in (0..n).rev() {
            let mut sum = 0.0;
            for j in (i + 1)..n {
                sum += decomp.u.data[i * n + j] * col_sol[j];
            }
            col_sol[i] = (y[i] - sum) / decomp.u.data[i * n + i];
        }

        for r in 0..n {
            x_data[r * n + col] = col_sol[r];
        }
    }

    Ok(DenseMatrix {
        rows: n,
        cols: n,
        data: x_data,
    })
}

/// Matrix left division `X = A \ B` (`mldivide`).
/// Solves the linear system A * X = B.
pub fn mldivide(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    // Scalar broadcasting: if A is scalar, scale B
    if a.is_scalar() {
        let s = a.data[0];
        if s == 0.0 {
            return Err(Diagnostic::error("Division by zero.", span));
        }
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| x / s).collect(),
        });
    }

    // Dimension check
    if a.rows != b.rows {
        return Err(Diagnostic::error(
            format!(
                "Matrix dimensions must agree. Left operand has {} rows, but right operand has {} rows.",
                a.rows, b.rows
            ),
            span,
        ));
    }

    let m = a.rows;
    let n = a.cols;
    let k = b.cols;

    // Square nonsingular system: use pivoted LU decomposition
    if m == n {
        let decomp = pivoted_lu(a, span)?;

        // Singularity check
        let max_abs = decomp
            .u
            .data
            .iter()
            .fold(0.0f64, |acc, &x| acc.max(x.abs()))
            .max(1.0);
        for i in 0..n {
            let diag = decomp.u.data[i * n + i].abs();
            if diag <= 1e-15 * max_abs {
                return Err(Diagnostic::error(
                    "Matrix is singular to working precision.",
                    span,
                ));
            }
        }

        let mut x_data = vec![0.0; n * k];
        let mut y = vec![0.0; n];
        let mut x_col = vec![0.0; n];

        for c in 0..k {
            // Forward solve L * y = P * b[:, c]
            for i in 0..n {
                let orig_row = decomp.p_vec[i];
                let b_val = b.data[orig_row * k + c];
                let mut sum = 0.0;
                for j in 0..i {
                    sum += decomp.l.data[i * n + j] * y[j];
                }
                y[i] = b_val - sum;
            }

            // Backward solve U * x = y
            for i in (0..n).rev() {
                let mut sum = 0.0;
                for j in (i + 1)..n {
                    sum += decomp.u.data[i * n + j] * x_col[j];
                }
                x_col[i] = (y[i] - sum) / decomp.u.data[i * n + i];
            }

            for r in 0..n {
                x_data[r * k + c] = x_col[r];
            }
        }

        return Ok(DenseMatrix {
            rows: n,
            cols: k,
            data: x_data,
        });
    }

    // Overdetermined system (m > n): QR factorization least squares
    if m > n {
        return qr_least_squares(a, b, span);
    }

    // Underdetermined system (m < n): minimum norm solution via pseudoinverse
    let p_inv = crate::svd::pinv(a, None, span)?;
    crate::matmul(&p_inv, b, span)
}

/// Solve overdetermined linear system A * X = B (m > n) in the least-squares sense using Householder QR.
fn qr_least_squares(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    let m = a.rows;
    let n = a.cols;
    let k = b.cols;

    let mut r = a.data.clone();
    let mut c = b.data.clone();

    for j in 0..n {
        // Extract column segment r[j..m, j]
        let mut norm_sq = 0.0;
        for i in j..m {
            let val = r[i * n + j];
            norm_sq += val * val;
        }
        let norm_x = norm_sq.sqrt();
        if norm_x <= 1e-15 {
            continue;
        }

        let r_jj = r[j * n + j];
        let sign = if r_jj >= 0.0 { 1.0 } else { -1.0 };
        let u1 = r_jj + sign * norm_x;

        // Construct Householder vector v of length (m - j)
        let mut v = Vec::with_capacity(m - j);
        v.push(1.0);
        for i in (j + 1)..m {
            v.push(r[i * n + j] / u1);
        }

        let mut v_norm_sq = 1.0;
        for &vi in &v[1..] {
            v_norm_sq += vi * vi;
        }
        let tau = 2.0 / v_norm_sq;

        // Apply H = I - tau * v * v^T to R[j..m, j..n]
        for col in j..n {
            let mut dot = 0.0;
            for i in 0..(m - j) {
                dot += v[i] * r[(j + i) * n + col];
            }
            for i in 0..(m - j) {
                r[(j + i) * n + col] -= tau * dot * v[i];
            }
        }

        // Apply H to C[j..m, 0..k]
        for col in 0..k {
            let mut dot = 0.0;
            for i in 0..(m - j) {
                dot += v[i] * c[(j + i) * k + col];
            }
            for i in 0..(m - j) {
                c[(j + i) * k + col] -= tau * dot * v[i];
            }
        }
    }

    // Backward solve R[0..n, 0..n] * X = C[0..n, 0..k]
    let mut x_data = vec![0.0; n * k];
    for col in 0..k {
        for i in (0..n).rev() {
            let mut sum = 0.0;
            for j in (i + 1)..n {
                sum += r[i * n + j] * x_data[j * k + col];
            }
            let diag = r[i * n + i];
            if diag.abs() <= 1e-15 {
                return Err(Diagnostic::error(
                    "Matrix is rank-deficient for least squares solve.",
                    span,
                ));
            }
            x_data[i * k + col] = (c[i * k + col] - sum) / diag;
        }
    }

    Ok(DenseMatrix {
        rows: n,
        cols: k,
        data: x_data,
    })
}

/// Element-wise left division `A .\ B` (`elementwise_ldiv`).
/// Equivalent to `B ./ A`.
pub fn elementwise_ldiv(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        if s == 0.0 {
            return Err(Diagnostic::error("Division by zero.", span));
        }
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| x / s).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a
                .data
                .iter()
                .map(|&x| if x == 0.0 { f64::INFINITY } else { s / x })
                .collect(),
        });
    }
    if a.rows != b.rows || a.cols != b.cols {
        return Err(Diagnostic::error(
            format!(
                "Arrays have incompatible sizes for this operation: {}x{} vs {}x{}",
                a.rows, a.cols, b.rows, b.cols
            ),
            span,
        ));
    }
    let data = a
        .data
        .iter()
        .zip(b.data.iter())
        .map(|(&x, &y)| if x == 0.0 { f64::INFINITY } else { y / x })
        .collect();
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}
