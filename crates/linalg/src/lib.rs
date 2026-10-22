use matlab_diagnostics::{Diagnostic, Span};
use matlab_values::DenseMatrix;

pub mod arrays;

/// Matrix multiplication `A * B`
pub fn matmul(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    // Scalar broadcasting for *
    if a.is_scalar() {
        let s = a.data[0];
        let data = b.data.iter().map(|&x| s * x).collect();
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data,
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        let data = a.data.iter().map(|&x| x * s).collect();
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data,
        });
    }

    if a.cols != b.rows {
        return Err(Diagnostic::error(
            format!(
                "Incorrect dimensions for matrix multiplication. Check that the number of columns in the first matrix ({}) matches the number of rows in the second matrix ({}). To perform elementwise multiplication, use '.*'.",
                a.cols, b.rows
            ),
            span,
        ).with_hint("Ensure inner dimensions match: [M x K] * [K x N] = [M x N]"));
    }

    let m = a.rows;
    let k_dim = a.cols;
    let n = b.cols;
    let mut c_data = vec![0.0; m * n];

    for i in 0..m {
        for k in 0..k_dim {
            let a_ik = a.data[i * k_dim + k];
            for j in 0..n {
                c_data[i * n + j] += a_ik * b.data[k * n + j];
            }
        }
    }

    Ok(DenseMatrix {
        rows: m,
        cols: n,
        data: c_data,
    })
}

/// Element-wise multiplication `A .* B`
pub fn elementwise_mul(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| s * x).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x * s).collect(),
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
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Addition `A + B`
pub fn add(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| s + x).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x + s).collect(),
        });
    }

    if a.rows != b.rows || a.cols != b.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix dimensions must agree for addition: {}x{} vs {}x{}",
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
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Subtraction `A - B`
pub fn sub(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| s - x).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x - s).collect(),
        });
    }

    if a.rows != b.rows || a.cols != b.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix dimensions must agree for subtraction: {}x{} vs {}x{}",
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
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Matrix transposition `A.'`
pub fn transpose(a: &DenseMatrix) -> DenseMatrix {
    let mut data = vec![0.0; a.rows * a.cols];
    for r in 0..a.rows {
        for c in 0..a.cols {
            data[c * a.rows + r] = a.data[r * a.cols + c];
        }
    }
    DenseMatrix {
        rows: a.cols,
        cols: a.rows,
        data,
    }
}

/// Unary negation `-A`
pub fn negate(a: &DenseMatrix) -> DenseMatrix {
    DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data: a.data.iter().map(|&x| -x).collect(),
    }
}

/// Generic unary element-wise map.
pub fn map_unary(a: &DenseMatrix, f: impl Fn(f64) -> f64) -> DenseMatrix {
    DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data: a.data.iter().map(|&x| f(x)).collect(),
    }
}

/// Column-wise reduction helper used by `sum`/`max`/`min` (MATLAB default: reduce along columns).
/// Returns a 1-by-cols matrix (or the same scalar).
pub fn reduce_cols(a: &DenseMatrix, init: f64, fold: impl Fn(f64, f64) -> f64) -> DenseMatrix {
    if a.is_scalar() {
        return a.clone();
    }
    let mut data = Vec::with_capacity(a.cols);
    for c in 0..a.cols {
        let mut acc = init;
        for r in 0..a.rows {
            acc = fold(acc, a.data[r * a.cols + c]);
        }
        data.push(acc);
    }
    DenseMatrix {
        rows: 1,
        cols: a.cols,
        data,
    }
}

/// Element-wise division `A ./ B`. Zero division follows IEEE-754 semantics (Inf/NaN),
/// which matches MATLAB's behavior for double arrays.
pub fn elementwise_div(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| s / x).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x / s).collect(),
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
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Element-wise power `A .^ B`.
pub fn elementwise_pow(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() && b.is_scalar() {
        return Ok(DenseMatrix::scalar(a.data[0].powf(b.data[0])));
    }
    if a.is_scalar() {
        let s = a.data[0];
        return Ok(DenseMatrix {
            rows: b.rows,
            cols: b.cols,
            data: b.data.iter().map(|&x| s.powf(x)).collect(),
        });
    }
    if b.is_scalar() {
        let s = b.data[0];
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x.powf(s)).collect(),
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
        .map(|(&x, &y)| x.powf(y))
        .collect();
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Element-wise comparison producing a matrix of 0/1 (logical values are represented
/// as doubles in this milestone; see docs/compatibility-matrix.md).
pub fn compare(
    op: CompareOp,
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    let select = move |x: f64, y: f64| -> f64 {
        let r = match op {
            CompareOp::Eq => x == y,
            CompareOp::Ne => x != y,
            CompareOp::Lt => x < y,
            CompareOp::Le => x <= y,
            CompareOp::Gt => x > y,
            CompareOp::Ge => x >= y,
        };
        if r {
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
            data: b.data.iter().map(|&y| select(s, y)).collect(),
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
        .map(|(&x, &y)| select(x, y))
        .collect();
    Ok(DenseMatrix {
        rows: a.rows,
        cols: a.cols,
        data,
    })
}

/// Matrix inverse via Gauss-Jordan elimination with partial pivoting.
fn matrix_inv(b: &DenseMatrix, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    if b.rows != b.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix must be square to be inverted: {}x{}",
                b.rows, b.cols
            ),
            span,
        ));
    }
    let n = b.rows;
    let mut m = vec![0.0; n * 2 * n];
    for r in 0..n {
        let a_row = b.cols * r;
        m[r * 2 * n..r * 2 * n + n].copy_from_slice(&b.data[a_row..a_row + n]);
        for k in 0..n {
            m[r * 2 * n + n + k] = if k == r { 1.0 } else { 0.0 };
        }
    }

    for col in 0..n {
        // Partial pivot: pick largest magnitude in the column
        let mut pivot_row = col;
        let mut pivot_val = m[col * 2 * n + col].abs();
        for r in (col + 1)..n {
            let cand = m[r * 2 * n + col].abs();
            if cand > pivot_val {
                pivot_val = cand;
                pivot_row = r;
            }
        }
        if pivot_val < 1e-15 {
            return Err(Diagnostic::error(
                "Matrix is singular to working precision.",
                span,
            ));
        }
        if pivot_row != col {
            for j in 0..(2 * n) {
                m.swap(col * 2 * n + j, pivot_row * 2 * n + j);
            }
        }
        let pivot = m[col * 2 * n + col];
        for j in 0..2 * n {
            m[col * 2 * n + j] /= pivot;
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let factor = m[r * 2 * n + col];
            for j in 0..2 * n {
                m[r * 2 * n + j] -= factor * m[col * 2 * n + j];
            }
        }
    }

    let mut data = vec![0.0; n * n];
    for r in 0..n {
        for c in 0..n {
            data[r * n + c] = m[r * 2 * n + n + c];
        }
    }
    Ok(DenseMatrix {
        rows: n,
        cols: n,
        data,
    })
}

/// Matrix right division `A / B`: `A / s` scales; `A / B` computes `A * inv(B)`.
pub fn mrdivide(
    a: &DenseMatrix,
    b: &DenseMatrix,
    span: Option<Span>,
) -> Result<DenseMatrix, Diagnostic> {
    if b.is_scalar() {
        let s = b.data[0];
        if s == 0.0 {
            return Err(Diagnostic::error("Division by zero.", span));
        }
        return Ok(DenseMatrix {
            rows: a.rows,
            cols: a.cols,
            data: a.data.iter().map(|&x| x / s).collect(),
        });
    }
    let b_inv = matrix_inv(b, span)?;
    matmul(a, &b_inv, span)
}

/// Matrix power `A ^ p`: scalar exponent on scalars, integer non-negative exponent
/// on square matrices via repeated multiplication.
pub fn matrix_pow(a: &DenseMatrix, p: f64, span: Option<Span>) -> Result<DenseMatrix, Diagnostic> {
    if a.is_scalar() {
        return Ok(DenseMatrix::scalar(a.data[0].powf(p)));
    }
    if a.rows != a.cols {
        return Err(Diagnostic::error(
            format!(
                "Matrix dimensions must agree for matrix power: {}x{} is not square",
                a.rows, a.cols
            ),
            span,
        ));
    }
    if p.fract() != 0.0 {
        return Err(Diagnostic::error(
            "Matrix exponent must be an integer. Use '.^' for element-wise power.",
            span,
        ));
    }
    if p < 0.0 {
        let inv = matrix_inv(a, span)?;
        return matrix_pow(&inv, -p, span);
    }
    let mut result = DenseMatrix::eye(a.rows);
    let mut power = a.clone();
    let mut remaining = p as u32;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = matmul(&result, &power, None)?;
        }
        power = matmul(&power, &power, None)?;
        remaining >>= 1;
    }
    Ok(result)
}
