use crate::CompareOp;
use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix, Value};

/// Complex matrix addition `A + B` with scalar broadcasting.
pub fn add(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s + x).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x + s).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
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
        .map(|(&x, &y)| x + y)
        .collect();
    Ok(ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Complex matrix subtraction `A - B` with scalar broadcasting.
pub fn sub(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s - x).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x - s).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
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
        .map(|(&x, &y)| x - y)
        .collect();
    Ok(ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Complex element-wise multiplication `A .* B` with scalar broadcasting.
pub fn elementwise_mul(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s * x).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x * s).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
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
        .map(|(&x, &y)| x * y)
        .collect();
    Ok(ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Complex element-wise division `A ./ B` with scalar broadcasting.
pub fn elementwise_div(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s / x).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x / s).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
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
        .map(|(&x, &y)| x / y)
        .collect();
    Ok(ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Complex element-wise power `A .^ B`.
pub fn elementwise_pow(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s.powc(x)).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x.powc(s)).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
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
        .map(|(&x, &y)| x.powc(y))
        .collect();
    Ok(ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Complex matrix multiplication `A * B` with scalar broadcasting.
pub fn matmul(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s * x).collect();
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x * s).collect();
        return Ok(ComplexDenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
        });
    }
    if a.cols != b.rows {
        return Err(Diagnostic::error(
            format!(
                "Incorrect dimensions for matrix multiplication: {}x{} vs {}x{}.",
                a.rows, a.cols, b.rows, b.cols
            ),
            span,
        ));
    }

    let m = a.rows;
    let k_dim = a.cols;
    let n = b.cols;
    let mut c_data = vec![Complex64::zero(); m * n];

    for i in 0..m {
        for k in 0..k_dim {
            let a_ik = a.data[i * k_dim + k];
            for j in 0..n {
                c_data[i * n + j] += a_ik * b.data[k * n + j];
            }
        }
    }

    Ok(ComplexDenseMatrix {
        rows: m,
        cols: n,
        data: c_data,
    })
}

/// Complex conjugate transpose `A'` (Hermitian transpose): conjugates each element and swaps dimensions.
pub fn ctranspose(a: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    let mut data = vec![Complex64::zero(); a.rows * a.cols];
    for r in 0..a.rows {
        for c in 0..a.cols {
            data[c * a.rows + r] = a.data[r * a.cols + c].conj();
        }
    }
    ComplexDenseMatrix {
        rows: a.cols,
        cols: a.rows,
        data,
    }
}

/// Complex non-conjugate transpose `A.'`: swaps dimensions without conjugating.
pub fn transpose(a: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    let mut data = vec![Complex64::zero(); a.rows * a.cols];
    for r in 0..a.rows {
        for c in 0..a.cols {
            data[c * a.rows + r] = a.data[r * a.cols + c];
        }
    }
    ComplexDenseMatrix {
        rows: a.cols,
        cols: a.rows,
        data,
    }
}

/// Complex negation `-A`.
pub fn negate(a: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    ComplexDenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data: a.data.iter().map(|&z| -z).collect(),
    }
}

/// Complex comparisons returning real 0/1 `DenseMatrix`.
/// For `==` and `~=`, compares real and imaginary parts.
/// For `<`, `<=`, `>`, `>=`, MATLAB compares ONLY the real parts!
pub fn compare(
    op: CompareOp,
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    let select = |x: Complex64, y: Complex64| -> f64 {
        let cond = match op {
            CompareOp::Eq => x == y,
            CompareOp::Ne => x != y,
            CompareOp::Lt => x.re < y.re,
            CompareOp::Le => x.re <= y.re,
            CompareOp::Gt => x.re > y.re,
            CompareOp::Ge => x.re >= y.re,
        };
        if cond {
            1.0
        } else {
            0.0
        }
    };

    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| select(s, x)).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| select(x, s)).collect(),
        });
    }
    if a.rows != b.rows || a.cols != b.cols {
        return Err(Diagnostic::error(
            format!(
                "Arrays have incompatible sizes for comparison: {}x{} vs {}x{}",
                a.rows, a.cols, b.rows, b.cols
            ),
            span,
        ));
    }
    let data = a
        .data
        .iter()
        .zip(b.data.iter())
        .map(|(&x, &y)| select(x, y))
        .collect();
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

// ----------------------------------------------------------------------------
// Unified Mixed Arithmetic (promoting real to complex when necessary)
// ----------------------------------------------------------------------------

pub fn add_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => Ok(Value::Matrix(crate::add(lm, rm, span)?)),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(add(&lc, &rc, span)?))
        }
    }
}

pub fn sub_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => Ok(Value::Matrix(crate::sub(lm, rm, span)?)),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(sub(&lc, &rc, span)?))
        }
    }
}

pub fn mul_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => Ok(Value::Matrix(crate::matmul(lm, rm, span)?)),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(matmul(&lc, &rc, span)?))
        }
    }
}

pub fn dotmul_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => {
            Ok(Value::Matrix(crate::elementwise_mul(lm, rm, span)?))
        }
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(elementwise_mul(
                &lc, &rc, span,
            )?))
        }
    }
}

pub fn dotdiv_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => {
            Ok(Value::Matrix(crate::elementwise_div(lm, rm, span)?))
        }
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(elementwise_div(
                &lc, &rc, span,
            )?))
        }
    }
}

pub fn dotpow_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => {
            // If base has negative elements and exponent is non-integer, promote to complex!
            let has_complex_branch = lm
                .data
                .iter()
                .zip(rm.data.iter())
                .any(|(&x, &p)| x < 0.0 && p.fract() != 0.0);
            if has_complex_branch {
                let lc = ComplexDenseMatrix::from_real(lm);
                let rc = ComplexDenseMatrix::from_real(rm);
                Ok(Value::from_complex_or_simplified(elementwise_pow(
                    &lc, &rc, span,
                )?))
            } else {
                Ok(Value::Matrix(crate::elementwise_pow(lm, rm, span)?))
            }
        }
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(elementwise_pow(
                &lc, &rc, span,
            )?))
        }
    }
}

pub fn compare_values(
    op: CompareOp,
    l: &Value,
    r: &Value,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => crate::compare(op, lm, rm, span),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            compare(op, &lc, &rc, span)
        }
    }
}

// ----------------------------------------------------------------------------
// Complex Linear Algebra: LU, Determinant, Trace, Inverse, Linear Solvers
// ----------------------------------------------------------------------------

/// Result of complex LU factorization with partial pivoting: P * A = L * U.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexLuDecomp {
    /// Unit lower triangular matrix (or permuted lower triangular in 2-output form).
    pub l: ComplexDenseMatrix,
    /// Upper triangular matrix.
    pub u: ComplexDenseMatrix,
    /// Permutation matrix P such that P * A = L * U.
    pub p: DenseMatrix,
    /// Permutation vector: row i of P*A corresponds to row `p_vec[i]` of A.
    pub p_vec: Vec<usize>,
    /// Number of row swaps performed during pivoting (used for determinant sign).
    pub num_swaps: usize,
}

/// Compute complex LU factorization with partial row pivoting for matrix A (m x n).
/// Produces P * A = L * U, where L is unit lower triangular and U is upper triangular.
pub fn pivoted_lu(
    a: &ComplexDenseMatrix,
    _span: Option<Span>,
) -> Result<ComplexLuDecomp, Diagnostic> {
    let m = a.rows;
    let n = a.cols;
    if m == 0 || n == 0 {
        return Ok(ComplexLuDecomp {
            l: ComplexDenseMatrix::empty(),
            u: ComplexDenseMatrix::empty(),
            p: DenseMatrix::empty(),
            p_vec: Vec::new(),
            num_swaps: 0,
        });
    }

    let k_max = m.min(n);
    let mut u_data = a.data.clone();
    let mut p_vec: Vec<usize> = (0..m).collect();
    let mut l_data = vec![Complex64::zero(); m * k_max];
    for i in 0..k_max {
        l_data[i * k_max + i] = Complex64::one();
    }
    let mut num_swaps = 0;

    for k in 0..k_max {
        // Find pivot: row r >= k with maximum modulus in column k of U
        let mut max_val = u_data[k * n + k].abs();
        let mut pivot_row = k;
        for r in (k + 1)..m {
            let val = u_data[r * n + k].abs();
            if val > max_val {
                max_val = val;
                pivot_row = r;
            }
        }

        if pivot_row != k {
            for j in 0..n {
                u_data.swap(k * n + j, pivot_row * n + j);
            }
            p_vec.swap(k, pivot_row);
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
                u_data[i * n + k] = Complex64::zero();
                for j in (k + 1)..n {
                    let term = factor * u_data[k * n + j];
                    u_data[i * n + j] -= term;
                }
            }
        }
    }

    let mut p_data = vec![0.0; m * m];
    for (i, &orig_row) in p_vec.iter().enumerate() {
        p_data[i * m + orig_row] = 1.0;
    }

    let u_rows = k_max;
    let mut final_u_data = vec![Complex64::zero(); u_rows * n];
    for r in 0..u_rows {
        for c in 0..n {
            final_u_data[r * n + c] = u_data[r * n + c];
        }
    }

    Ok(ComplexLuDecomp {
        l: ComplexDenseMatrix {
            rows: m,
            cols: k_max,
            data: l_data,
        },
        u: ComplexDenseMatrix {
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

/// Single-output `Y = lu(A)` for complex A.
pub fn lu_factor(
    a: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            "Single-output lu(A) requires a square matrix.",
            span,
        ));
    }
    let decomp = pivoted_lu(a, span)?;
    let n = a.rows;
    let mut y_data = vec![Complex64::zero(); n * n];
    for r in 0..n {
        for c in 0..n {
            if r > c {
                y_data[r * n + c] = decomp.l.data[r * n + c];
            } else {
                y_data[r * n + c] = decomp.u.data[r * n + c];
            }
        }
    }
    Ok(ComplexDenseMatrix {
        rows: n,
        cols: n,
        data: y_data,
    })
}

/// Two-output `[L, U] = lu(A)` for complex A.
pub fn lu_two_output(
    a: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<(ComplexDenseMatrix, ComplexDenseMatrix), Diagnostic> {
    let decomp = pivoted_lu(a, span)?;
    let m = decomp.l.rows;
    let k = decomp.l.cols;
    let mut l_perm = vec![Complex64::zero(); m * k];
    for (i, &orig_row) in decomp.p_vec.iter().enumerate() {
        for c in 0..k {
            l_perm[orig_row * k + c] = decomp.l.data[i * k + c];
        }
    }
    Ok((
        ComplexDenseMatrix {
            rows: m,
            cols: k,
            data: l_perm,
        },
        decomp.u,
    ))
}

/// Three-output `[L, U, P] = lu(A)` for complex A.
pub fn lu_three_output(
    a: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<(ComplexDenseMatrix, ComplexDenseMatrix, DenseMatrix), Diagnostic> {
    let decomp = pivoted_lu(a, span)?;
    Ok((decomp.l, decomp.u, decomp.p))
}

/// Complex determinant `det(A)`.
pub fn det(a: &ComplexDenseMatrix, span: Option<Span>) -> Result<Complex64, Diagnostic> {
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
        return Ok(Complex64::one());
    }
    if n == 1 {
        return Ok(a.data[0]);
    }
    let decomp = pivoted_lu(a, span)?;
    let mut d = if decomp.num_swaps % 2 == 1 {
        Complex64::new(-1.0, 0.0)
    } else {
        Complex64::new(1.0, 0.0)
    };
    for i in 0..n {
        d *= decomp.u.data[i * n + i];
    }
    Ok(d)
}

/// Complex matrix trace `trace(A)`.
pub fn trace(a: &ComplexDenseMatrix, _span: Option<Span>) -> Result<Complex64, Diagnostic> {
    let d = a.rows.min(a.cols);
    if d == 0 {
        return Ok(Complex64::zero());
    }
    let mut sum = Complex64::zero();
    for i in 0..d {
        sum += a.data[i * a.cols + i];
    }
    Ok(sum)
}

/// Complex matrix inverse `inv(A)`.
pub fn inv(a: &ComplexDenseMatrix, span: Option<Span>) -> Result<ComplexDenseMatrix, Diagnostic> {
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
        return Ok(ComplexDenseMatrix::empty());
    }
    if n == 1 {
        let val = a.data[0];
        if val.is_zero() {
            return Err(Diagnostic::error(
                "Matrix is singular to working precision.",
                span,
            ));
        }
        return Ok(ComplexDenseMatrix::scalar(Complex64::one() / val));
    }

    let decomp = pivoted_lu(a, span)?;
    let max_abs = decomp
        .u
        .data
        .iter()
        .fold(0.0f64, |acc, z| acc.max(z.abs()))
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

    let mut x_data = vec![Complex64::zero(); n * n];
    let mut y = vec![Complex64::zero(); n];
    let mut col_sol = vec![Complex64::zero(); n];

    for col in 0..n {
        for i in 0..n {
            let b_val = if decomp.p_vec[i] == col {
                Complex64::one()
            } else {
                Complex64::zero()
            };
            let mut sum = Complex64::zero();
            for (j, yj) in y.iter().enumerate().take(i) {
                sum += decomp.l.data[i * n + j] * *yj;
            }
            y[i] = b_val - sum;
        }

        for i in (0..n).rev() {
            let mut sum = Complex64::zero();
            for (j, value) in col_sol.iter().enumerate().skip(i + 1) {
                sum += decomp.u.data[i * n + j] * *value;
            }
            col_sol[i] = (y[i] - sum) / decomp.u.data[i * n + i];
        }

        for r in 0..n {
            x_data[r * n + col] = col_sol[r];
        }
    }

    Ok(ComplexDenseMatrix {
        rows: n,
        cols: n,
        data: x_data,
    })
}

/// Complex left matrix division `X = A \ B` (`mldivide`).
pub fn mldivide(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| x / s).collect(),
        });
    }

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

    if m == n {
        let decomp = pivoted_lu(a, span)?;
        let max_abs = decomp
            .u
            .data
            .iter()
            .fold(0.0f64, |acc, z| acc.max(z.abs()))
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

        let mut x_data = vec![Complex64::zero(); n * k];
        let mut y = vec![Complex64::zero(); n];
        let mut x_col = vec![Complex64::zero(); n];

        for c in 0..k {
            for i in 0..n {
                let orig_row = decomp.p_vec[i];
                let b_val = b.data[orig_row * k + c];
                let mut sum = Complex64::zero();
                for (j, yj) in y.iter().enumerate().take(i) {
                    sum += decomp.l.data[i * n + j] * *yj;
                }
                y[i] = b_val - sum;
            }

            for i in (0..n).rev() {
                let mut sum = Complex64::zero();
                for (j, value) in x_col.iter().enumerate().skip(i + 1) {
                    sum += decomp.u.data[i * n + j] * *value;
                }
                x_col[i] = (y[i] - sum) / decomp.u.data[i * n + i];
            }

            for r in 0..n {
                x_data[r * k + c] = x_col[r];
            }
        }

        return Ok(ComplexDenseMatrix {
            rows: n,
            cols: k,
            data: x_data,
        });
    }

    if m > n {
        return qr_least_squares(a, b, span);
    }

    // Underdetermined (m < n): minimum norm solution via pseudoinverse
    let p_inv = crate::svd::complex_pinv(a, None, span)?;
    matmul(&p_inv, b, span)
}

/// Complex Householder QR least squares solver for overdetermined A * X = B (m > n).
fn qr_least_squares(
    a: &ComplexDenseMatrix,
    b: &ComplexDenseMatrix,
    _span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    let m = a.rows;
    let n = a.cols;
    let k = b.cols;

    let mut r = a.data.clone();
    let mut c = b.data.clone();

    for j in 0..n {
        let mut norm_sq = 0.0f64;
        for i in j..m {
            let val = r[i * n + j];
            norm_sq += val.abs_sq();
        }
        let norm_x = norm_sq.sqrt();
        if norm_x <= 1e-15 {
            continue;
        }

        let x0 = r[j * n + j];
        let alpha = if x0.abs() > 0.0 {
            -(x0 / Complex64::real(x0.abs())) * Complex64::real(norm_x)
        } else {
            Complex64::real(-norm_x)
        };
        let u1 = x0 - alpha;

        let mut v = Vec::with_capacity(m - j);
        v.push(Complex64::one());
        for i in (j + 1)..m {
            v.push(r[i * n + j] / u1);
        }

        let mut v_norm_sq = 1.0f64;
        for vi in &v[1..] {
            v_norm_sq += vi.abs_sq();
        }
        let tau = 2.0 / v_norm_sq;

        // Apply H = I - tau * v * v^H to R[j..m, j..n]
        for col in j..n {
            let mut dot = Complex64::zero();
            for i in 0..(m - j) {
                dot += v[i].conj() * r[(j + i) * n + col];
            }
            for i in 0..(m - j) {
                let delta = Complex64::real(tau) * dot * v[i];
                r[(j + i) * n + col] -= delta;
            }
        }

        // Apply H to C[j..m, 0..k]
        for col in 0..k {
            let mut dot = Complex64::zero();
            for i in 0..(m - j) {
                dot += v[i].conj() * c[(j + i) * k + col];
            }
            for i in 0..(m - j) {
                let delta = Complex64::real(tau) * dot * v[i];
                c[(j + i) * k + col] -= delta;
            }
        }
    }

    // Back solve R[0..n, 0..n] * X = C[0..n, 0..k]
    let mut x_data = vec![Complex64::zero(); n * k];
    for col in 0..k {
        for i in (0..n).rev() {
            let mut sum = Complex64::zero();
            for j in (i + 1)..n {
                sum += r[i * n + j] * x_data[j * k + col];
            }
            x_data[i * k + col] = (c[i * k + col] - sum) / r[i * n + i];
        }
    }

    Ok(ComplexDenseMatrix {
        rows: n,
        cols: k,
        data: x_data,
    })
}

/// Complex right matrix division `X = B / A` (`mrdivide`).
/// Satisfies X * A = B <=> A^H * X^H = B^H <=> X^H = A^H \ B^H <=> X = (A^H \ B^H)^H
pub fn mrdivide(
    b: &ComplexDenseMatrix,
    a: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<ComplexDenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(ComplexDenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| x / s).collect(),
        });
    }

    let a_h = ctranspose(a);
    let b_h = ctranspose(b);
    let x_h = mldivide(&a_h, &b_h, span)?;
    Ok(ctranspose(&x_h))
}

pub fn mldivide_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => Ok(Value::Matrix(crate::mldivide(lm, rm, span)?)),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(mldivide(&lc, &rc, span)?))
        }
    }
}

pub fn mrdivide_values(l: &Value, r: &Value, span: Option<Span>) -> Result<Value, Diagnostic> {
    match (l, r) {
        (Value::Matrix(lm), Value::Matrix(rm)) => Ok(Value::Matrix(crate::mrdivide(lm, rm, span)?)),
        _ => {
            let lc = l.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            let rc = r.to_complex_matrix().ok_or_else(|| {
                Diagnostic::error("Operands must be numeric matrices or scalars.", span)
            })?;
            Ok(Value::from_complex_or_simplified(mrdivide(&lc, &rc, span)?))
        }
    }
}

/// Complex dense eigenvalue decomposition.
///
/// The matrix is first reduced to upper Hessenberg form by unitary Givens
/// similarities, then shifted QR iterations (complex Wilkinson shift) drive
/// the active subdiagonal blocks to zero. Unlike a real-only QR iteration,
/// this path can represent complex-conjugate eigenvalues directly.
///
/// Returns `(D, V)` where `D` is a column vector of eigenvalues and the
/// columns of `V` are right eigenvectors computed by inverse iteration.
/// Eigenvalues are sorted by real part, then imaginary part, for determinism.
pub fn eig_complex(
    a: &ComplexDenseMatrix,
    span: Option<Span>,
) -> Result<(ComplexDenseMatrix, ComplexDenseMatrix), Diagnostic> {
    let n = a.rows;
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix must be square for eigenvalues: {}x{}",
                a.rows, a.cols
            ),
            span,
        ));
    }
    if a.data.iter().any(|z| z.is_nan() || z.is_infinite()) {
        return Err(Diagnostic::error(
            "eig rejects matrices containing NaN or Inf.",
            span,
        ));
    }
    if n == 0 {
        return Ok((ComplexDenseMatrix::empty(), ComplexDenseMatrix::empty()));
    }
    if n == 1 {
        return Ok((a.clone(), ComplexDenseMatrix::eye(1)));
    }

    let mut h = hessenberg_complex(a);
    let norm = a.data.iter().map(|z| z.abs()).fold(0.0f64, f64::max);
    let mut eigenvalues = Vec::with_capacity(n);
    let mut hi: isize = n as isize - 1;
    let mut iterations = 0usize;
    let max_iter = 200usize;

    while hi >= 0 {
        let hi_u = hi as usize;
        if hi_u == 0 {
            eigenvalues.push(h.data[0]);
            hi = -1;
            continue;
        }

        // Find the start of the unreduced trailing block. A subdiagonal is
        // negligible when it is small relative to the local diagonal scale
        // (with a floor at the matrix norm to stop round-off from blocking
        // deflation indefinitely).
        let mut l = hi_u;
        while l > 0 {
            let sub = h.data[l * n + (l - 1)].abs();
            let diag_scale = h.data[(l - 1) * n + (l - 1)].abs() + h.data[l * n + l].abs();
            let threshold = 1e-13 * diag_scale + 1e-15 * norm.max(1.0);
            if sub <= threshold {
                h.data[l * n + (l - 1)] = Complex64::zero();
                break;
            }
            l -= 1;
        }

        if l == hi_u {
            // 1x1 deflated block.
            eigenvalues.push(h.data[hi_u * n + hi_u]);
            hi -= 1;
        } else if l + 1 == hi_u {
            // 2x2 deflated block: solve its quadratic characteristic equation.
            let (lambda1, lambda2) = two_by_two_eigenvalues(&h, l, l + 1);
            eigenvalues.push(lambda1);
            eigenvalues.push(lambda2);
            hi -= 2;
        } else {
            if iterations >= max_iter {
                return Err(Diagnostic::error(
                    "Complex eigenvalue iteration did not converge to working precision.",
                    span,
                ));
            }
            qr_step_complex(&mut h, l, hi_u);
            iterations += 1;
        }
    }

    // Sort eigenvalues (real, then imaginary) before computing the matching
    // eigenvectors so column order is deterministic.
    eigenvalues.sort_by(|a, b| {
        a.re.partial_cmp(&b.re)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.im.partial_cmp(&b.im).unwrap_or(std::cmp::Ordering::Equal))
    });
    let eigenvectors = inverse_iteration_eigenvectors(a, &eigenvalues);
    // Eigenpair refinement (Rayleigh-quotient polish + one extra inverse
    // iteration per eigenpair, accept-only-if-better) so the returned
    // eigenpairs satisfy A*V = V*D at backward-stable accuracy rather than
    // the listing accuracy of the raw QR pass.
    let (eigenvalues, eigenvectors) = refine_complex_eigenpairs(a, eigenvalues, eigenvectors);

    Ok((
        ComplexDenseMatrix {
            rows: n,
            cols: 1,
            data: eigenvalues,
        },
        eigenvectors,
    ))
}

/// Reduce a complex matrix to upper Hessenberg form with unitary Givens
/// similarities. Returns `H` with the same eigenvalues as `A`.
fn hessenberg_complex(a: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    let n = a.rows;
    let mut h = a.clone();
    for k in 0..n.saturating_sub(2) {
        for i in (k + 2)..n {
            let x = h.data[(k + 1) * n + k];
            let y = h.data[i * n + k];
            if y.abs() < 1e-300 {
                continue;
            }
            let r = x.abs().hypot(y.abs());
            if r < 1e-300 {
                continue;
            }
            let c = Complex64::new(x.re / r, -x.im / r);
            let s = Complex64::new(y.re / r, -y.im / r);
            apply_left_rotation(&mut h, k + 1, i, k, c, s);
            apply_right_rotation(&mut h, k + 1, i, c, s);
        }
    }
    // Enforce the Hessenberg shape exactly; the Givens step above creates
    // values that are zero in exact arithmetic but may retain round-off.
    for row in 0..n {
        for col in 0..row.saturating_sub(1) {
            h.data[row * n + col] = Complex64::zero();
        }
    }
    h
}

/// Apply the left factor of a complex Givens rotation to rows `k` and `i`,
/// starting at column `col_start`.
///
/// `c = conj(x)/r` and `s = conj(y)/r` are produced by `complex_givens`, so
/// the rotation maps `[x, y]^T` to `[r, 0]^T`.
fn apply_left_rotation(
    m: &mut ComplexDenseMatrix,
    k: usize,
    i: usize,
    col_start: usize,
    c: Complex64,
    s: Complex64,
) {
    let n = m.cols;
    for j in col_start..n {
        let a = m.data[k * n + j];
        let b = m.data[i * n + j];
        m.data[k * n + j] = c * a + s * b;
        m.data[i * n + j] = -s.conj() * a + c.conj() * b;
    }
}

/// Apply the right factor `G^H` of a complex Givens rotation to columns `k`
/// and `i`.
fn apply_right_rotation(
    m: &mut ComplexDenseMatrix,
    k: usize,
    i: usize,
    c: Complex64,
    s: Complex64,
) {
    let n = m.rows;
    let cols = m.cols;
    for row in 0..n {
        let a = m.data[row * cols + k];
        let b = m.data[row * cols + i];
        m.data[row * cols + k] = c.conj() * a + s.conj() * b;
        m.data[row * cols + i] = -s * a + c * b;
    }
}

/// Complex Givens coefficients that zero `y` against `x`.
fn complex_givens(x: Complex64, y: Complex64) -> (Complex64, Complex64) {
    let r = x.abs().hypot(y.abs());
    if r < 1e-300 {
        (Complex64::one(), Complex64::zero())
    } else {
        (
            Complex64::new(x.re / r, -x.im / r),
            Complex64::new(y.re / r, -y.im / r),
        )
    }
}

/// Eigenvalues of the 2x2 diagonal block at rows/columns `i`, `j`.
fn two_by_two_eigenvalues(h: &ComplexDenseMatrix, i: usize, j: usize) -> (Complex64, Complex64) {
    let n = h.cols;
    let a = h.data[i * n + i];
    let b = h.data[i * n + j];
    let c = h.data[j * n + i];
    let d = h.data[j * n + j];
    let half = Complex64::real(0.5) * (a + d);
    let det = a * d - b * c;
    let discriminant = half * half - det;
    let root = discriminant.sqrt();
    (half + root, half - root)
}

/// One implicit shifted QR step on the active Hessenberg block `l..=hi`.
fn qr_step_complex(h: &mut ComplexDenseMatrix, l: usize, hi: usize) {
    let n = h.cols;
    let (shift_a, shift_b) = two_by_two_eigenvalues(h, hi - 1, hi);
    let trailing = h.data[hi * n + hi];
    // Wilkinson shift: the trailing-block eigenvalue closest to H(hi,hi).
    let mut shift = if (shift_a - trailing).abs() <= (shift_b - trailing).abs() {
        shift_a
    } else {
        shift_b
    };

    // A defective trailing block (repeated eigenvalue equal to H(hi,hi)) gives
    // a zero classical Wilkinson shift. Equal-modulus spectra such as the
    // companion matrix of x^4+1 then cycle without converging. Use a
    // deterministic exceptional shift to move off that cycle.
    let norm = h.data.iter().map(|z| z.abs()).fold(0.0f64, f64::max);
    let scale = norm.max(1.0);
    let defective =
        (shift_a - shift_b).abs() <= 1e-12 * scale && (shift - trailing).abs() <= 1e-12 * scale;
    if defective {
        shift = trailing + Complex64::new(0.6, 0.8) * Complex64::real(scale);
    }

    let mut x = h.data[l * n + l] - shift;
    let mut y = h.data[(l + 1) * n + l];

    for k in l..=hi.saturating_sub(1) {
        let (c, s) = complex_givens(x, y);
        let col_start = if k == l { l } else { k - 1 };
        apply_left_rotation(h, k, k + 1, col_start, c, s);
        apply_right_rotation(h, k, k + 1, c, s);
        if k + 1 < hi {
            x = h.data[(k + 1) * n + k];
            y = h.data[(k + 2) * n + k];
        }
    }
}

/// Compute right eigenvectors by inverse iteration with a small diagonal
/// shift. The shift keeps `A - lambda I` nonsingular; the resulting vector
/// residual is on the order of the shift rather than the eigenvalue error.
fn inverse_iteration_eigenvectors(
    a: &ComplexDenseMatrix,
    eigenvalues: &[Complex64],
) -> ComplexDenseMatrix {
    let n = a.rows;
    let norm = a.data.iter().map(|z| z.abs()).fold(0.0f64, f64::max);
    let base_shift = 1e-10 * norm.max(1.0);
    let mut result = vec![Complex64::zero(); n * n];

    for (col, &lambda) in eigenvalues.iter().enumerate() {
        let mut x: Vec<Complex64> = (0..n)
            .map(|i| {
                let fi = i as f64;
                Complex64::new(1.0 + 0.25 * fi, 0.1 * fi)
            })
            .collect();
        let mut delta = base_shift;
        let mut converged = false;

        for _attempt in 0..4 {
            let mut shifted = a.clone();
            for i in 0..n {
                shifted.data[i * n + i] -= lambda + Complex64::real(delta);
            }
            let rhs = ComplexDenseMatrix {
                rows: n,
                cols: 1,
                data: x.clone(),
            };
            match mldivide(&shifted, &rhs, None) {
                Ok(solution) => {
                    let max = solution.data.iter().map(|z| z.abs()).fold(0.0, f64::max);
                    if max > 1e-300 {
                        x = solution
                            .data
                            .iter()
                            .map(|z| *z / Complex64::real(max))
                            .collect();
                        converged = true;
                        break;
                    }
                    delta *= 100.0;
                }
                Err(_) => {
                    delta *= 100.0;
                }
            }
        }

        if converged {
            // A few more inverse iterations at the same shift to stabilize
            // the direction after the first solve.
            for _ in 0..3 {
                let mut shifted = a.clone();
                for i in 0..n {
                    shifted.data[i * n + i] -= lambda + Complex64::real(delta);
                }
                let rhs = ComplexDenseMatrix {
                    rows: n,
                    cols: 1,
                    data: x.clone(),
                };
                if let Ok(solution) = mldivide(&shifted, &rhs, None) {
                    let max = solution.data.iter().map(|z| z.abs()).fold(0.0, f64::max);
                    if max > 1e-300 {
                        x = solution
                            .data
                            .iter()
                            .map(|z| *z / Complex64::real(max))
                            .collect();
                    }
                }
            }
        } else {
            // Degenerate fallback: a coordinate vector still gives the caller
            // a well-formed matrix rather than a silently transposed result.
            x = vec![Complex64::zero(); n];
            x[col % n] = Complex64::one();
        }

        for row in 0..n {
            result[row * n + col] = x[row];
        }
    }

    ComplexDenseMatrix {
        rows: n,
        cols: n,
        data: result,
    }
}

/// Row-major complex matrix-vector product used by the eigenpair refinement.
fn complex_matvec(a: &ComplexDenseMatrix, x: &[Complex64]) -> Vec<Complex64> {
    (0..a.rows)
        .map(|i| {
            let mut acc = Complex64::new(0.0, 0.0);
            for k in 0..a.cols {
                acc = acc + a.data[i * a.cols + k] * x[k];
            }
            acc
        })
        .collect()
}


/// 2-norm of the eigenvector-equation residual ||A*x - lambda*x||_2.
fn eigenpair_residual_norm(a: &ComplexDenseMatrix, x: &[Complex64], lambda: Complex64) -> f64 {
    let ax = complex_matvec(a, x);
    ax.iter()
        .zip(x.iter())
        .map(|(axi, xi)| {
            let r = *axi - lambda * *xi;
            r.re * r.re + r.im * r.im
        })
        .sum::<f64>()
        .sqrt()
}

/// Refine eigenvectors whose residual is not yet backward-stable-grade.
///
/// The eigenvalues are deliberately left exactly as produced by the QR /
/// inverse-iteration pass: changing them by even a few ulps perturbs
/// accumulated identities such as the determinant of an ill-conditioned
/// companion matrix.  For each eigenvector above the gate, extra
/// inverse-iteration solves are accepted only when they strictly lower
/// ||A*x - lambda*x||_2 at the original lambda.  A refined vector can
/// therefore improve the A*V = V*D residual without moving the spectrum.
/// Pairs at or below the gate are left untouched.
fn refine_complex_eigenpairs(
    a: &ComplexDenseMatrix,
    eigenvalues: Vec<Complex64>,
    mut eigenvectors: ComplexDenseMatrix,
) -> (Vec<Complex64>, ComplexDenseMatrix) {
    let n = a.rows;
    if n == 0 {
        return (eigenvalues, eigenvectors);
    }
    let norm = a.data.iter().map(|z| z.abs()).fold(0.0f64, f64::max);
    let base_shift = 1e-10 * norm.max(1.0);
    let frob_a = a
        .data
        .iter()
        .map(|z| z.re * z.re + z.im * z.im)
        .sum::<f64>()
        .sqrt();
    let gate = 0.25 * f64::EPSILON * (n as f64) * frob_a.max(1.0);

    for col in 0..n {
        let lambda = eigenvalues[col];
        let x0: Vec<Complex64> = (0..n).map(|i| eigenvectors.data[i * n + col]).collect();
        if x0.iter().all(|z| z.abs() == 0.0) || eigenpair_residual_norm(a, &x0, lambda) <= gate
        {
            continue;
        }

        let mut best_x = x0;
        let mut best_res = eigenpair_residual_norm(a, &best_x, lambda);
        for _round in 0..2 {
            let mut shifted = a.clone();
            for i in 0..n {
                shifted.data[i * n + i] -= lambda + Complex64::real(base_shift);
            }
            let rhs = ComplexDenseMatrix {
                rows: n,
                cols: 1,
                data: best_x.clone(),
            };
            match mldivide(&shifted, &rhs, None) {
                Ok(sol) => {
                    let max = sol.data.iter().map(|z| z.abs()).fold(0.0, f64::max);
                    if max <= 1e-300 {
                        break;
                    }
                    let w: Vec<Complex64> =
                        sol.data.iter().map(|z| *z / Complex64::real(max)).collect();
                    let res_w = eigenpair_residual_norm(a, &w, lambda);
                    if res_w < best_res {
                        best_res = res_w;
                        best_x = w;
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        for row in 0..n {
            eigenvectors.data[row * n + col] = best_x[row];
        }
    }
    (eigenvalues, eigenvectors)
}
