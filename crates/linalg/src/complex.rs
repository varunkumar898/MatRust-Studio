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
            Ok(Value::ComplexMatrix(add(&lc, &rc, span)?))
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
            Ok(Value::ComplexMatrix(sub(&lc, &rc, span)?))
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
            Ok(Value::ComplexMatrix(matmul(&lc, &rc, span)?))
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
            Ok(Value::ComplexMatrix(elementwise_mul(&lc, &rc, span)?))
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
            Ok(Value::ComplexMatrix(elementwise_div(&lc, &rc, span)?))
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
                Ok(Value::ComplexMatrix(elementwise_pow(&lc, &rc, span)?))
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
            Ok(Value::ComplexMatrix(elementwise_pow(&lc, &rc, span)?))
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
