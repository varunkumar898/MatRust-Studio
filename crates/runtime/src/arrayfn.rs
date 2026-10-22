//! Argument handling for the array-construction and array-manipulation
//! builtins.
//!
//! The numeric algorithms live in `matlab_linalg::arrays`; this module owns
//! MATLAB-style argument validation (counts, dimension shapes, integer
//! requirements) and maps them onto those algorithms.
//!
//! Scope limit: the runtime stores arrays as dense 2-D matrices
//! (`values::DenseMatrix`), so every function here is 2-D only. N-D
//! behaviour is *not* emulated; it is reported as an error where MATLAB
//! would accept it.

use matlab_diagnostics::{Diagnostic, Span};
use matlab_linalg as la;
use matlab_syntax::ast::Expr;
use matlab_values::{DenseMatrix, Value};

use crate::interp::{coerce_matrix_pub, eval_expr_pub, Env, ExecIo};
use crate::rng::Rng;

/// Evaluate an argument expression as a dense numeric matrix.
pub fn matrix_arg(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<DenseMatrix, Diagnostic> {
    let v = eval_expr_pub(env, expr, io)?;
    coerce_matrix_pub(v, Some(span)).map_err(|_| {
        Diagnostic::error(
            format!("{} expects a numeric array argument.", name),
            Some(span),
        )
    })
}

/// Evaluate an argument expression as a finite scalar number.
fn scalar_arg(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<f64, Diagnostic> {
    let m = matrix_arg(expr, env, io, span, name)?;
    m.as_scalar().ok_or_else(|| {
        Diagnostic::error(format!("{} expects a scalar argument.", name), Some(span))
    })
}

/// Evaluate a scalar argument that must be a non-negative integer.
fn dim_arg(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<usize, Diagnostic> {
    let n = scalar_arg(expr, env, io, span, name)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(Diagnostic::error(
            format!("{} expects non-negative integer dimensions.", name),
            Some(span),
        ));
    }
    Ok(n as usize)
}

/// Evaluate a scalar argument that must be an integer (may be negative).
fn int_arg(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<isize, Diagnostic> {
    let n = scalar_arg(expr, env, io, span, name)?;
    if !n.is_finite() || n.fract() != 0.0 {
        return Err(Diagnostic::error(
            format!("{} expects an integer argument.", name),
            Some(span),
        ));
    }
    Ok(n as isize)
}

/// `linspace(a, b, n)` — `n` evenly spaced points from `a` to `b`.
///
/// Returns a 1-by-n row vector. `n == 0` yields a 1-by-0 empty array and
/// `n == 1` yields the single point `a`.
pub fn linspace(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 3 {
        return Err(Diagnostic::error(
            "linspace expects 3 arguments (start, stop, count).",
            Some(span),
        ));
    }
    let a = scalar_arg(&args[0], env, io, span, "linspace")?;
    let b = scalar_arg(&args[1], env, io, span, "linspace")?;
    let n = dim_arg(&args[2], env, io, span, "linspace")?;
    let m = la::arrays::linspace(a, b, n, Some(span))?;
    Ok(Value::Matrix(m))
}

/// `diag(v)` / `diag(v, k)` — build a diagonal matrix from a vector, or
/// extract a diagonal from a matrix.
///
/// A vector input builds a matrix; a matrix input extracts. `k` selects the
/// offset diagonal (negative = below the main diagonal).
pub fn diag(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "diag expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "diag")?;
    let k = if args.len() == 2 {
        int_arg(&args[1], env, io, span, "diag")?
    } else {
        0
    };
    let m = la::arrays::diag(&a, k)?;
    Ok(Value::Matrix(m))
}

/// `repmat(A, m, n)` — tile `A` `m` times down and `n` times across.
pub fn repmat(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 3 {
        return Err(Diagnostic::error(
            "repmat expects 3 arguments (A, rows, cols).",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "repmat")?;
    let m = dim_arg(&args[1], env, io, span, "repmat")?;
    let n = dim_arg(&args[2], env, io, span, "repmat")?;
    let out = la::arrays::repmat(&a, m, n)?;
    Ok(Value::Matrix(out))
}

/// `ndims(A)` — number of dimensions.
///
/// The runtime stores 2-D arrays only, so this always reports 2.
pub fn ndims(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("ndims expects 1 argument.", Some(span)));
    }
    let v = eval_expr_pub(env, &args[0], io)?;
    match v {
        Value::Matrix(_) | Value::ComplexMatrix(_) | Value::Boolean(_) | Value::String(_) => {
            Ok(Value::Matrix(DenseMatrix::scalar(2.0)))
        }
    }
}

/// `isempty(A)` — 1 when `A` has no elements, 0 otherwise.
pub fn isempty(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("isempty expects 1 argument.", Some(span)));
    }
    let v = eval_expr_pub(env, &args[0], io)?;
    let empty = match &v {
        Value::Matrix(m) => m.rows == 0 || m.cols == 0 || m.data.is_empty(),
        Value::ComplexMatrix(c) => c.rows == 0 || c.cols == 0 || c.data.is_empty(),
        Value::String(s) => s.is_empty(),
        Value::Boolean(_) => false,
    };
    Ok(Value::Matrix(DenseMatrix::scalar(if empty {
        1.0
    } else {
        0.0
    })))
}

/// `reshape(A, m, n)` — same elements, new shape, column-major order.
pub fn reshape(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 3 {
        return Err(Diagnostic::error(
            "reshape expects 3 arguments (A, rows, cols).",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "reshape")?;
    let m = dim_arg(&args[1], env, io, span, "reshape")?;
    let n = dim_arg(&args[2], env, io, span, "reshape")?;
    if m * n != a.rows * a.cols {
        return Err(Diagnostic::error(
            "reshape: the number of elements must match the requested dimensions.",
            Some(span),
        ));
    }
    let out = la::arrays::reshape(&a, Some(m), Some(n))?;
    Ok(Value::Matrix(out))
}

/// `transpose(A)` — transpose without conjugation (values are real here).
pub fn transpose(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            "transpose expects 1 argument.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "transpose")?;
    Ok(Value::Matrix(la::transpose(&a)))
}

/// `flip(A, dim)` — reverse the order of elements along `dim`.
pub fn flip(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "flip expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "flip")?;
    let dim = if args.len() == 2 {
        int_arg(&args[1], env, io, span, "flip")?
    } else {
        1
    };
    if dim < 1 || dim > 2 {
        return Err(Diagnostic::error(
            "flip: dimension must be 1 or 2 in the 2-D array model.",
            Some(span),
        ));
    }
    let out = la::arrays::flip(&a, dim as usize, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `fliplr(A)` — reverse the columns of `A`.
pub fn fliplr(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("fliplr expects 1 argument.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "fliplr")?;
    let out = la::arrays::fliplr(&a)?;
    Ok(Value::Matrix(out))
}

/// `flipud(A)` — reverse the rows of `A`.
pub fn flipud(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("flipud expects 1 argument.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "flipud")?;
    let out = la::arrays::flipud(&a)?;
    Ok(Value::Matrix(out))
}

/// `rot90(A)` / `rot90(A, k)` — rotate `A` counter-clockwise `k` times.
pub fn rot90(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "rot90 expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "rot90")?;
    let k = if args.len() == 2 {
        int_arg(&args[1], env, io, span, "rot90")?
    } else {
        1
    };
    let out = la::arrays::rot90(&a, k)?;
    Ok(Value::Matrix(out))
}

/// `circshift(A, shift)` / `circshift(A, rowShift, colShift)` — circular
/// shift with wrap-around.
pub fn circshift(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 2 && args.len() != 3 {
        return Err(Diagnostic::error(
            "circshift expects 2 or 3 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "circshift")?;
    let shifts = if args.len() == 3 {
        vec![
            scalar_arg(&args[1], env, io, span, "circshift")?,
            scalar_arg(&args[2], env, io, span, "circshift")?,
        ]
    } else {
        let m = matrix_arg(&args[1], env, io, span, "circshift")?;
        if m.rows == 1 && m.cols == 2 {
            m.data.clone()
        } else if m.rows == 2 && m.cols == 1 {
            m.data.clone()
        } else {
            return Err(Diagnostic::error(
                "circshift: the shift argument must be a 2-element vector.",
                Some(span),
            ));
        }
    };
    let out = la::arrays::circshift(&a, &shifts, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `triu(A)` / `tril(A)` / `triu(A, k)` / `tril(A, k)` — upper or lower
/// triangular part, with optional diagonal offset `k`.
///
/// Rectangular input is accepted (MATLAB returns the triangular part of the
/// trapezoid); the dimensions of `A` are preserved.
pub fn triangular(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    upper: bool,
    name: &str,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            format!("{} expects 1 or 2 arguments.", name),
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, name)?;
    let k = if args.len() == 2 {
        int_arg(&args[1], env, io, span, name)?
    } else {
        0
    };
    let out = la::arrays::triangular(&a, upper, k, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `triu(A)` / `triu(A, k)` — upper triangular part.
pub fn triu(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    triangular(args, env, io, span, true, "triu")
}

/// `tril(A)` / `tril(A, k)` — lower triangular part.
pub fn tril(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    triangular(args, env, io, span, false, "tril")
}

/// `logspace(a, b, n)` — `n` logarithmically spaced points from `10^a` to `10^b`.
pub fn logspace(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 3 {
        return Err(Diagnostic::error(
            "logspace expects 3 arguments (start, stop, count).",
            Some(span),
        ));
    }
    let a = scalar_arg(&args[0], env, io, span, "logspace")?;
    let b = scalar_arg(&args[1], env, io, span, "logspace")?;
    let n = dim_arg(&args[2], env, io, span, "logspace")?;
    let m = la::arrays::logspace(a, b, n, Some(span))?;
    Ok(Value::Matrix(m))
}

/// `magic(n)` — magic square of order `n`.
pub fn magic(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            "magic expects 1 argument (order n).",
            Some(span),
        ));
    }
    let n = dim_arg(&args[0], env, io, span, "magic")?;
    let m = la::arrays::magic(n, Some(span))?;
    Ok(Value::Matrix(m))
}

/// `vertcat(A, B, ...)` — vertical concatenation (stack along rows).
pub fn vertcat(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() < 2 {
        return Err(Diagnostic::error(
            "vertcat expects at least 2 arguments.",
            Some(span),
        ));
    }
    let mut matrices = Vec::with_capacity(args.len());
    for arg in args {
        matrices.push(matrix_arg(arg, env, io, span, "vertcat")?);
    }
    let out = la::arrays::vertcat(&matrices, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `horzcat(A, B, ...)` — horizontal concatenation (place side by side).
pub fn horzcat(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() < 2 {
        return Err(Diagnostic::error(
            "horzcat expects at least 2 arguments.",
            Some(span),
        ));
    }
    let mut matrices = Vec::with_capacity(args.len());
    for arg in args {
        matrices.push(matrix_arg(arg, env, io, span, "horzcat")?);
    }
    let out = la::arrays::horzcat(&matrices, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `cat(dim, A, B, ...)` — concatenate along dimension `dim` (1 or 2).
pub fn cat(args: &[Expr], env: &Env, io: &mut ExecIo<'_>, span: Span) -> Result<Value, Diagnostic> {
    if args.len() < 3 {
        return Err(Diagnostic::error(
            "cat expects at least 3 arguments (dim, A, B, ...).",
            Some(span),
        ));
    }
    let dim = dim_arg(&args[0], env, io, span, "cat")?;
    if dim != 1 && dim != 2 {
        return Err(Diagnostic::error(
            "cat: dimension must be 1 or 2 in this runtime (2-D arrays).",
            Some(span),
        ));
    }
    let mut matrices = Vec::with_capacity(args.len() - 1);
    for arg in &args[1..] {
        matrices.push(matrix_arg(arg, env, io, span, "cat")?);
    }
    let out = la::arrays::cat(dim, &matrices, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `blkdiag(A, B, ...)` — block-diagonal matrix assembly.
pub fn blkdiag(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() {
        return Err(Diagnostic::error(
            "blkdiag expects at least 1 argument.",
            Some(span),
        ));
    }
    let mut matrices = Vec::with_capacity(args.len());
    for arg in args {
        matrices.push(matrix_arg(arg, env, io, span, "blkdiag")?);
    }
    let out = la::arrays::blkdiag(&matrices, Some(span))?;
    Ok(Value::Matrix(out))
}

/// `rand` / `rand(m,n)` / `rand([m n])` — uniform (0,1) random matrix.
pub fn rand(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let (r, c) = match args.len() {
        0 => (1, 1),
        1 => {
            let m = matrix_arg(&args[0], env, io, span, "rand")?;
            if m.rows == 1 && m.cols == 2 {
                let v = m.to_col_major();
                (v[0] as usize, v[1] as usize)
            } else {
                let n = m.as_scalar().ok_or_else(|| {
                    Diagnostic::error(
                        "rand: size argument must be a scalar or a 2-element vector.",
                        Some(span),
                    )
                })?;
                (n as usize, n as usize)
            }
        }
        2 => {
            let r = dim_arg(&args[0], env, io, span, "rand")?;
            let c = dim_arg(&args[1], env, io, span, "rand")?;
            (r, c)
        }
        _ => {
            return Err(Diagnostic::error(
                "rand expects 0, 1, or 2 arguments.",
                Some(span),
            ));
        }
    };
    let mut data = Vec::with_capacity(r * c);
    for _ in 0..r * c {
        data.push(io.rng.rand());
    }
    Ok(Value::Matrix(DenseMatrix {
        rows: r,
        cols: c,
        data,
    }))
}

/// `randn` / `randn(m,n)` / `randn([m n])` — standard normal random matrix.
pub fn randn(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let (r, c) = match args.len() {
        0 => (1, 1),
        1 => {
            let m = matrix_arg(&args[0], env, io, span, "randn")?;
            if m.rows == 1 && m.cols == 2 {
                let v = m.to_col_major();
                (v[0] as usize, v[1] as usize)
            } else {
                let n = m.as_scalar().ok_or_else(|| {
                    Diagnostic::error(
                        "randn: size argument must be a scalar or a 2-element vector.",
                        Some(span),
                    )
                })?;
                (n as usize, n as usize)
            }
        }
        2 => {
            let r = dim_arg(&args[0], env, io, span, "randn")?;
            let c = dim_arg(&args[1], env, io, span, "randn")?;
            (r, c)
        }
        _ => {
            return Err(Diagnostic::error(
                "randn expects 0, 1, or 2 arguments.",
                Some(span),
            ));
        }
    };
    let mut data = Vec::with_capacity(r * c);
    for _ in 0..r * c {
        data.push(io.rng.randn());
    }
    Ok(Value::Matrix(DenseMatrix {
        rows: r,
        cols: c,
        data,
    }))
}

/// `randi(max)` / `randi(max,m,n)` / `randi([lo hi],m,n)` / `randi(..., [m n])`
/// Uniform integer random matrix in the specified inclusive range.
pub fn randi(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 4 {
        return Err(Diagnostic::error(
            "randi expects 1 to 4 arguments.",
            Some(span),
        ));
    }
    // Parse range: first arg is either scalar max (lo=1) or [lo hi]
    let (lo, hi) = {
        let m = matrix_arg(&args[0], env, io, span, "randi")?;
        if m.rows == 1 && m.cols == 2 {
            let v = m.to_col_major();
            (v[0] as i64, v[1] as i64)
        } else {
            let max = m.as_scalar().ok_or_else(|| {
                Diagnostic::error(
                    "randi: first argument must be a scalar max or [lo hi] vector.",
                    Some(span),
                )
            })?;
            (1, max as i64)
        }
    };
    if lo > hi {
        return Err(Diagnostic::error("randi: lo must be <= hi.", Some(span)));
    }
    // Parse dimensions
    let (r, c) = match args.len() {
        1 => (1, 1),
        2 => {
            let m = matrix_arg(&args[1], env, io, span, "randi")?;
            if m.rows == 1 && m.cols == 2 {
                let v = m.to_col_major();
                (v[0] as usize, v[1] as usize)
            } else {
                let n = m.as_scalar().ok_or_else(|| {
                    Diagnostic::error(
                        "randi: size argument must be a scalar or a 2-element vector.",
                        Some(span),
                    )
                })?;
                (n as usize, n as usize)
            }
        }
        3 => {
            let r = dim_arg(&args[1], env, io, span, "randi")?;
            let c = dim_arg(&args[2], env, io, span, "randi")?;
            (r, c)
        }
        4 => {
            // randi(range, m, n, _) - ignore 4th arg (MATLAB compatibility)
            let r = dim_arg(&args[1], env, io, span, "randi")?;
            let c = dim_arg(&args[2], env, io, span, "randi")?;
            (r, c)
        }
        _ => unreachable!(),
    };
    let mut data = Vec::with_capacity(r * c);
    for _ in 0..r * c {
        data.push(io.rng.randi_inclusive(lo, hi) as f64);
    }
    Ok(Value::Matrix(DenseMatrix {
        rows: r,
        cols: c,
        data,
    }))
}

/// `randperm(n)` / `randperm(n, k)` — random permutation of 1..n (or first k elements).
pub fn randperm(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "randperm expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let n = dim_arg(&args[0], env, io, span, "randperm")?;
    let k = if args.len() == 2 {
        dim_arg(&args[1], env, io, span, "randperm")?
    } else {
        n
    };
    if k > n {
        return Err(Diagnostic::error(
            "randperm: k cannot exceed n.",
            Some(span),
        ));
    }
    // Fisher-Yates shuffle of 1..n, then take first k
    let mut perm: Vec<f64> = (1..=n).map(|i| i as f64).collect();
    for i in (1..n).rev() {
        let j = (io.rng.rand() * (i + 1) as f64).floor() as usize;
        perm.swap(i, j);
    }
    perm.truncate(k);
    Ok(Value::Matrix(DenseMatrix {
        rows: 1,
        cols: k,
        data: perm,
    }))
}

/// `rng(seed)` — set the random number generator seed for the current session.
pub fn rng(args: &[Expr], env: &Env, io: &mut ExecIo<'_>, span: Span) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            "rng expects 1 argument (seed).",
            Some(span),
        ));
    }
    let seed = scalar_arg(&args[0], env, io, span, "rng")?;
    if seed < 0.0 || seed.fract() != 0.0 || seed > u64::MAX as f64 {
        return Err(Diagnostic::error(
            "rng seed must be a non-negative integer.",
            Some(span),
        ));
    }
    *io.rng = Rng::new(seed as u64);
    Ok(Value::Matrix(DenseMatrix::empty()))
}

/// `sort(A)` / `sort(A, dim)` — sort along dimension 1 (columns) or 2 (rows).
pub fn sort(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 3 {
        return Err(Diagnostic::error(
            "sort expects 1 to 3 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "sort")?;
    // Parse optional direction ('ascend'/'descend') and dim.
    let mut dim = if a.rows == 1 && a.cols > 1 { 2 } else { 1 };
    let mut descending = false;
    for i in 1..args.len() {
        // Check if arg is a string
        let v = eval_expr_pub(env, &args[i], io)?;
        if let Value::String(s) = &v {
            let lower = s.to_lowercase();
            if lower == "descend" {
                descending = true;
            } else if lower == "ascend" {
                descending = false;
            } else {
                // Try as dim
                if let Some(m) = v.as_matrix() {
                    if let Some(d) = m.as_scalar() {
                        if d == 1.0 || d == 2.0 {
                            dim = d as usize;
                        }
                    }
                }
            }
        } else if let Some(m) = v.as_matrix() {
            if let Some(d) = m.as_scalar() {
                if d == 1.0 || d == 2.0 {
                    dim = d as usize;
                }
            }
        }
    }
    let out = la::arrays::sort(&a, dim, descending)?;
    Ok(Value::Matrix(out))
}

/// `sortrows(A)` / `sortrows(A, col)` — sort rows by column `col` (1-based).
pub fn sortrows(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "sortrows expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "sortrows")?;
    let cols = if args.len() == 2 {
        let v = eval_expr_pub(env, &args[1], io)?;
        if let Some(m) = v.as_matrix() {
            m.to_col_major()
                .iter()
                .map(|&x| x as usize)
                .collect::<Vec<_>>()
        } else {
            vec![0]
        }
    } else {
        vec![0]
    };
    let out = la::arrays::sortrows(&a, &cols)?;
    Ok(Value::Matrix(out))
}

/// `unique(A)` — sorted unique values (column vector).
pub fn unique(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("unique expects 1 argument.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "unique")?;
    let out = la::arrays::unique(&a)?;
    Ok(Value::Matrix(out))
}

/// `ismember(A, B)` — element-wise membership test (0/1 matrix).
pub fn ismember(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 2 {
        return Err(Diagnostic::error(
            "ismember expects 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "ismember")?;
    let b = matrix_arg(&args[1], env, io, span, "ismember")?;
    let out = la::arrays::ismember(&a, &b)?;
    Ok(Value::Matrix(out))
}

/// `setdiff(A, B)` — values in A not in B.
pub fn setdiff(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 2 {
        return Err(Diagnostic::error(
            "setdiff expects 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "setdiff")?;
    let b = matrix_arg(&args[1], env, io, span, "setdiff")?;
    let out = la::arrays::setdiff(&a, &b)?;
    Ok(Value::Matrix(out))
}

/// `union(A, B)` — sorted unique values in either A or B.
pub fn union(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 2 {
        return Err(Diagnostic::error("union expects 2 arguments.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "union")?;
    let b = matrix_arg(&args[1], env, io, span, "union")?;
    let out = la::arrays::union(&a, &b)?;
    Ok(Value::Matrix(out))
}

/// `intersect(A, B)` — sorted unique values in both A and B.
pub fn intersect(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 2 {
        return Err(Diagnostic::error(
            "intersect expects 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "intersect")?;
    let b = matrix_arg(&args[1], env, io, span, "intersect")?;
    let out = la::arrays::intersect(&a, &b)?;
    Ok(Value::Matrix(out))
}

/// `find(A)` / `find(A, n)` / `find(A, n, direction)` — indices of nonzero elements.
pub fn find(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 3 {
        return Err(Diagnostic::error(
            "find expects 1 to 3 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "find")?;
    let n = if args.len() >= 2 {
        int_arg(&args[1], env, io, span, "find")? as usize
    } else {
        0 // 0 means all
    };
    let direction = if args.len() >= 3 {
        let v = eval_expr_pub(env, &args[2], io)?;
        match v {
            Value::String(s) => s.to_lowercase(),
            _ => "first".to_string(),
        }
    } else {
        "first".to_string()
    };

    let mut indices = Vec::new();
    let cm = a.to_col_major();
    if direction == "last" && n > 0 {
        let all_idx: Vec<f64> = cm
            .iter()
            .enumerate()
            .filter(|(_, &val)| val != 0.0)
            .map(|(idx, _)| (idx + 1) as f64)
            .collect();
        let start = if n < all_idx.len() {
            all_idx.len() - n
        } else {
            0
        };
        indices = all_idx[start..].to_vec();
    } else {
        for (idx, &val) in cm.iter().enumerate() {
            if val != 0.0 {
                indices.push((idx + 1) as f64);
                if n > 0 && indices.len() >= n {
                    break;
                }
            }
        }
    }
    let n_out = indices.len();
    Ok(Value::Matrix(DenseMatrix {
        rows: n_out,
        cols: 1,
        data: indices,
    }))
}

/// `any(A)` / `any(A, dim)` — 1 if any element is nonzero along dim.
pub fn any(args: &[Expr], env: &Env, io: &mut ExecIo<'_>, span: Span) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "any expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "any")?;
    let dim = if args.len() == 2 {
        int_arg(&args[1], env, io, span, "any")? as usize
    } else {
        if a.rows == 1 {
            2
        } else {
            1
        }
    };
    let out = la::arrays::any_dim(&a, dim)?;
    Ok(Value::Matrix(out))
}

/// `all(A)` / `all(A, dim)` — 1 if all elements are nonzero along dim.
pub fn all(args: &[Expr], env: &Env, io: &mut ExecIo<'_>, span: Span) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            "all expects 1 or 2 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "all")?;
    let dim = if args.len() == 2 {
        int_arg(&args[1], env, io, span, "all")? as usize
    } else {
        if a.rows == 1 {
            2
        } else {
            1
        }
    };
    let out = la::arrays::all_dim(&a, dim)?;
    Ok(Value::Matrix(out))
}

/// `cumsum(A)` — cumulative sum.
pub fn cumsum(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("cumsum expects 1 argument.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "cumsum")?;
    let out = la::arrays::cumsum(&a)?;
    Ok(Value::Matrix(out))
}

/// `cumprod(A)` — cumulative product.
pub fn cumprod(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error("cumprod expects 1 argument.", Some(span)));
    }
    let a = matrix_arg(&args[0], env, io, span, "cumprod")?;
    let out = la::arrays::cumprod(&a)?;
    Ok(Value::Matrix(out))
}

/// `diff(A)` / `diff(A, n)` / `diff(A, n, dim)` — differences.
pub fn diff(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    if args.is_empty() || args.len() > 3 {
        return Err(Diagnostic::error(
            "diff expects 1 to 3 arguments.",
            Some(span),
        ));
    }
    let a = matrix_arg(&args[0], env, io, span, "diff")?;
    let n = if args.len() >= 2 {
        int_arg(&args[1], env, io, span, "diff")? as usize
    } else {
        1
    };
    let dim = if args.len() >= 3 {
        int_arg(&args[2], env, io, span, "diff")? as usize
    } else {
        1
    };
    if dim != 1 && dim != 2 {
        return Err(Diagnostic::error(
            "diff: dimension must be 1 or 2.",
            Some(span),
        ));
    }
    let out = la::arrays::diff(&a, n, dim)?;
    Ok(Value::Matrix(out))
}
