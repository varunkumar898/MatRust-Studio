//! Built-in functions.
//!
//! Builtin calls receive unevaluated argument ASTs, but they are evaluated
//! first in `call_builtin` for simplicity. Variables shadow builtins at runtime.
//!
//! Graphics builtins (`plot`, `figure`, ...) mutate the session's
//! `GraphicsState` through `ExecIo` and return an empty matrix; they produce
//! no console output. The desktop app renders figures from the state
//! snapshot returned in `RunResult::figures`.

use matlab_diagnostics::{Diagnostic, Span};
use matlab_linalg as la;
use matlab_syntax::ast::Expr;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix, Value};

use crate::graphics::{vector_from_matrix, PlotItem};
use crate::interp::{coerce_matrix_pub, eval_expr_pub, Env, ExecIo};

/// Multi-output builtin dispatch. Returns a vector of output values.
pub fn call_builtin_multi(
    name: &str,
    args: &[Expr],
    env: &Env,
    span: Span,
    io: &mut ExecIo<'_>,
    nargout: usize,
) -> Result<Vec<Value>, Diagnostic> {
    match name {
        "meshgrid" => {
            let mut matrices = Vec::with_capacity(args.len());
            for arg in args {
                matrices.push(crate::arrayfn::matrix_arg(arg, env, io, span, "meshgrid")?);
            }
            let outputs = la::arrays::meshgrid(&matrices)?;
            Ok(outputs.into_iter().map(Value::Matrix).collect())
        }
        "ndgrid" => {
            let mut matrices = Vec::with_capacity(args.len());
            for arg in args {
                matrices.push(crate::arrayfn::matrix_arg(arg, env, io, span, "ndgrid")?);
            }
            let outputs = la::arrays::ndgrid(&matrices)?;
            Ok(outputs.into_iter().map(Value::Matrix).collect())
        }
        "size" if nargout > 1 => {
            let v = eval_expr_pub(env, &args[0], io)?;
            let (rows, cols) = match &v {
                Value::Matrix(m) => (m.rows, m.cols),
                Value::ComplexMatrix(c) => (c.rows, c.cols),
                _ => {
                    let m = coerce_matrix_pub(v, Some(span))?;
                    (m.rows, m.cols)
                }
            };
            Ok(vec![
                Value::Matrix(DenseMatrix::scalar(rows as f64)),
                Value::Matrix(DenseMatrix::scalar(cols as f64)),
            ])
        }
        "find" if nargout > 1 => {
            let a = crate::arrayfn::matrix_arg(&args[0], env, io, span, "find")?;
            let (rows, cols, vals) = la::arrays::find_multi(&a);
            let mut outputs = vec![Value::Matrix(rows), Value::Matrix(cols)];
            if nargout >= 3 {
                outputs.push(Value::Matrix(vals));
            }
            Ok(outputs)
        }
        "min" if nargout > 1 => {
            let span_op = Some(span);
            if args.is_empty() || args.len() == 2 || args.len() > 3 {
                return Err(Diagnostic::error(
                    "min with 2 outputs expects 1 or 3 arguments: min(A) or min(A, [], dim).",
                    span_op,
                ));
            }
            let v0 = eval_expr_pub(env, &args[0], io)?;
            let m0 = coerce_matrix_pub(v0, span_op)?;
            if args.len() == 1 {
                let (vals, idxs) = la::stats::min_reduction(&m0, None);
                Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
            } else {
                let _ = eval_expr_pub(env, &args[1], io)?;
                let dim = parse_dim_arg(&args[2], env, io, span, "min")?;
                match dim {
                    DimArg::All => {
                        let (val, idx1) = la::stats::min_all_with_idx(&m0);
                        Ok(vec![
                            Value::Matrix(DenseMatrix::scalar(val)),
                            Value::Matrix(DenseMatrix::scalar(idx1 as f64)),
                        ])
                    }
                    DimArg::Dim(d) => {
                        let (vals, idxs) = la::stats::min_reduction(&m0, Some(d));
                        Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
                    }
                    DimArg::Default => {
                        let (vals, idxs) = la::stats::min_reduction(&m0, None);
                        Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
                    }
                }
            }
        }
        "max" if nargout > 1 => {
            let span_op = Some(span);
            if args.is_empty() || args.len() == 2 || args.len() > 3 {
                return Err(Diagnostic::error(
                    "max with 2 outputs expects 1 or 3 arguments: max(A) or max(A, [], dim).",
                    span_op,
                ));
            }
            let v0 = eval_expr_pub(env, &args[0], io)?;
            let m0 = coerce_matrix_pub(v0, span_op)?;
            if args.len() == 1 {
                let (vals, idxs) = la::stats::max_reduction(&m0, None);
                Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
            } else {
                let _ = eval_expr_pub(env, &args[1], io)?;
                let dim = parse_dim_arg(&args[2], env, io, span, "max")?;
                match dim {
                    DimArg::All => {
                        let (val, idx1) = la::stats::max_all_with_idx(&m0);
                        Ok(vec![
                            Value::Matrix(DenseMatrix::scalar(val)),
                            Value::Matrix(DenseMatrix::scalar(idx1 as f64)),
                        ])
                    }
                    DimArg::Dim(d) => {
                        let (vals, idxs) = la::stats::max_reduction(&m0, Some(d));
                        Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
                    }
                    DimArg::Default => {
                        let (vals, idxs) = la::stats::max_reduction(&m0, None);
                        Ok(vec![Value::Matrix(vals), Value::Matrix(idxs)])
                    }
                }
            }
        }
        "lu" if nargout > 1 => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "lu expects 1 input argument.",
                    Some(span),
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    if nargout == 2 {
                        let (l, u) = la::complex::lu_two_output(&c, Some(span))?;
                        Ok(vec![Value::ComplexMatrix(l), Value::ComplexMatrix(u)])
                    } else {
                        let (l, u, p) = la::complex::lu_three_output(&c, Some(span))?;
                        Ok(vec![
                            Value::ComplexMatrix(l),
                            Value::ComplexMatrix(u),
                            Value::Matrix(p),
                        ])
                    }
                }
                _ => {
                    let a = coerce_matrix_pub(v, Some(span))?;
                    if nargout == 2 {
                        let (l, u) = la::lu_two_output(&a, Some(span))?;
                        Ok(vec![Value::Matrix(l), Value::Matrix(u)])
                    } else {
                        let (l, u, p) = la::lu_three_output(&a, Some(span))?;
                        Ok(vec![Value::Matrix(l), Value::Matrix(u), Value::Matrix(p)])
                    }
                }
            }
        }
        "svd" if nargout > 1 => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "svd expects 1 input argument.",
                    Some(span),
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let (u, s, v_mat) = la::complex_svd_full(&c, Some(span))?;
                    Ok(vec![
                        Value::ComplexMatrix(u),
                        Value::Matrix(s),
                        Value::ComplexMatrix(v_mat),
                    ])
                }
                _ => {
                    let a = coerce_matrix_pub(v, Some(span))?;
                    let (u, s, v_mat) = la::svd_full(&a, Some(span))?;
                    Ok(vec![
                        Value::Matrix(u),
                        Value::Matrix(s),
                        Value::Matrix(v_mat),
                    ])
                }
            }
        }
        _ => {
            if let Some(func_stmt) = env.function_table.get(name).cloned() {
                return crate::interp::exec_user_function_multi(env, func_stmt, args, span, io);
            }
            let val = call_builtin(name, args, env, span, io)?;
            Ok(vec![val])
        }
    }
}
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Matrix(m) => m.format_matlab(),
        Value::ComplexMatrix(c) => c.format_matlab(),
        Value::String(s) => s.clone(),
        Value::Boolean(b) => {
            if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
    }
}

fn scalar_arg(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<f64, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            format!("{} expects 1 argument.", name),
            Some(span),
        ));
    }
    let v = eval_expr_pub(env, &args[0], io)?;
    let m = coerce_matrix_pub(v, Some(span))?;
    m.as_scalar().ok_or_else(|| {
        Diagnostic::error(format!("{} expects a scalar argument.", name), Some(span))
    })
}

/// Names that resolve as bare identifiers when not shadowed by a variable.
pub fn is_known_constant(name: &str) -> bool {
    matches!(name, "pi" | "inf" | "Inf" | "i" | "j")
}

fn sind_deg(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let k = (x / 90.0).round();
    if (x - k * 90.0).abs() < 1e-12 {
        let ki = k as i64;
        match ki.rem_euclid(4) {
            0 => 0.0,
            1 => 1.0,
            2 => 0.0,
            3 => -1.0,
            _ => unreachable!(),
        }
    } else {
        x.to_radians().sin()
    }
}

fn cosd_deg(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let k = (x / 90.0).round();
    if (x - k * 90.0).abs() < 1e-12 {
        let ki = k as i64;
        match ki.rem_euclid(4) {
            0 => 1.0,
            1 => 0.0,
            2 => -1.0,
            3 => 0.0,
            _ => unreachable!(),
        }
    } else {
        x.to_radians().cos()
    }
}

fn tand_deg(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let k = (x / 90.0).round();
    if (x - k * 90.0).abs() < 1e-12 {
        let ki = k as i64;
        match ki.rem_euclid(4) {
            0 | 2 => 0.0,
            1 => f64::INFINITY,
            3 => f64::NEG_INFINITY,
            _ => unreachable!(),
        }
    } else {
        x.to_radians().tan()
    }
}

fn asind_deg(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == 1.0 {
        90.0
    } else if x == -1.0 {
        -90.0
    } else if x == 0.0 {
        0.0
    } else if !(-1.0..=1.0).contains(&x) {
        f64::NAN
    } else {
        x.asin().to_degrees()
    }
}

fn acosd_deg(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == 1.0 {
        0.0
    } else if x == 0.0 {
        90.0
    } else if x == -1.0 {
        180.0
    } else if !(-1.0..=1.0).contains(&x) {
        f64::NAN
    } else {
        x.acos().to_degrees()
    }
}

fn atand_deg(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == 0.0 {
        0.0
    } else if x == 1.0 {
        45.0
    } else if x == -1.0 {
        -45.0
    } else if x == f64::INFINITY {
        90.0
    } else if x == f64::NEG_INFINITY {
        -90.0
    } else {
        x.atan().to_degrees()
    }
}

fn safe_bitshift(a: i64, k: i64, max_bits: Option<u32>) -> f64 {
    let shifted = if k >= 64 {
        0i64
    } else if k > 0 {
        a.checked_shl(k as u32).unwrap_or(0)
    } else if k <= -64 {
        if a >= 0 {
            0i64
        } else {
            -1i64
        }
    } else {
        a >> (-k as u32)
    };
    if let Some(bits) = max_bits {
        if bits < 64 {
            let mask = ((1u64 << bits) - 1) as i64;
            (shifted & mask) as f64
        } else {
            shifted as f64
        }
    } else {
        shifted as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimArg {
    Default,
    Dim(usize),
    All,
}

pub fn parse_dim_arg(
    arg: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<DimArg, Diagnostic> {
    let span_op = Some(span);
    let v = eval_expr_pub(env, arg, io)?;
    match v {
        Value::String(s) => {
            if s.eq_ignore_ascii_case("all") {
                Ok(DimArg::All)
            } else {
                Err(Diagnostic::error(
                    format!("{}: unrecognized dimension option '{}'.", name, s),
                    span_op,
                ))
            }
        }
        Value::Matrix(m) => {
            if let Some(val) = m.as_scalar() {
                if !val.is_finite() || val.fract() != 0.0 || val < 1.0 {
                    return Err(Diagnostic::error(
                        format!("{}: dimension must be a positive integer.", name),
                        span_op,
                    ));
                }
                Ok(DimArg::Dim(val as usize))
            } else if m.data.is_empty() {
                Ok(DimArg::Default)
            } else {
                Err(Diagnostic::error(
                    format!("{}: dimension must be a positive integer scalar.", name),
                    span_op,
                ))
            }
        }
        _ => Err(Diagnostic::error(
            format!(
                "{}: dimension must be a positive integer scalar or 'all'.",
                name
            ),
            span_op,
        )),
    }
}

pub fn parse_w_from_value(v: &Value, span: Option<Span>, name: &str) -> Result<usize, Diagnostic> {
    match v {
        Value::Matrix(m) => {
            if m.data.is_empty() {
                Ok(0)
            } else if let Some(val) = m.as_scalar() {
                if val == 0.0 {
                    Ok(0)
                } else if val == 1.0 {
                    Ok(1)
                } else {
                    Err(Diagnostic::error(
                        format!("{}: normalization flag must be 0 or 1.", name),
                        span,
                    ))
                }
            } else {
                Err(Diagnostic::error(
                    format!("{}: normalization flag must be a scalar 0 or 1.", name),
                    span,
                ))
            }
        }
        _ => Err(Diagnostic::error(
            format!("{}: normalization flag must be 0 or 1.", name),
            span,
        )),
    }
}

fn parse_dec_args(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<(DenseMatrix, Option<usize>), Diagnostic> {
    let span_op = Some(span);
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            format!("{} expects 1 or 2 arguments.", name),
            span_op,
        ));
    }
    let v0 = eval_expr_pub(env, &args[0], io)?;
    let m0 = coerce_matrix_pub(v0, span_op)?;
    let width = if args.len() == 2 {
        let v1 = eval_expr_pub(env, &args[1], io)?;
        let m1 = coerce_matrix_pub(v1, span_op)?;
        let w = m1.as_scalar().ok_or_else(|| {
            Diagnostic::error(
                format!("{} expects a scalar second argument.", name),
                span_op,
            )
        })?;
        if !w.is_finite() || w.fract() != 0.0 || w < 0.0 {
            return Err(Diagnostic::error(
                format!("{}: min digits must be a non-negative integer.", name),
                span_op,
            ));
        }
        Some(w as usize)
    } else {
        None
    };
    Ok((m0, width))
}

fn extract_string_or_chars(
    v: &Value,
    span: Option<Span>,
    name: &str,
) -> Result<String, Diagnostic> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Matrix(m) => {
            if m.rows <= 1 {
                let mut s = String::with_capacity(m.data.len());
                for &b in &m.data {
                    if (0.0..=255.0).contains(&b) && b.fract() == 0.0 {
                        s.push(b as u8 as char);
                    } else {
                        return Err(Diagnostic::error(
                            format!("{}: invalid character code.", name),
                            span,
                        ));
                    }
                }
                Ok(s)
            } else {
                Err(Diagnostic::error(
                    format!("{} expects a character vector or string.", name),
                    span,
                ))
            }
        }
        _ => Err(Diagnostic::error(
            format!("{} expects a character vector or string.", name),
            span,
        )),
    }
}

fn parse_norm_arg(
    arg: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<la::NormType, Diagnostic> {
    let v = eval_expr_pub(env, arg, io)?;
    match v {
        Value::String(s) => {
            let lower = s.to_ascii_lowercase();
            if lower == "fro" {
                Ok(la::NormType::Fro)
            } else if lower == "inf" {
                Ok(la::NormType::Inf)
            } else {
                Err(Diagnostic::error(
                    format!(
                        "Unrecognized norm type '{}'. Supported string norms are 'fro' and 'inf'.",
                        s
                    ),
                    Some(span),
                ))
            }
        }
        Value::Matrix(m) => {
            let val = m.as_scalar().ok_or_else(|| {
                Diagnostic::error("Norm type must be a scalar number or 'fro'.", Some(span))
            })?;
            if val == 1.0 {
                Ok(la::NormType::One)
            } else if val == 2.0 {
                Ok(la::NormType::Two)
            } else if val.is_infinite() && val > 0.0 {
                Ok(la::NormType::Inf)
            } else if val.is_infinite() && val < 0.0 {
                Ok(la::NormType::NegInf)
            } else {
                Ok(la::NormType::P(val))
            }
        }
        _ => Err(Diagnostic::error(
            "Second argument to norm must be a number or 'fro'.",
            Some(span),
        )),
    }
}

/// Dispatch a builtin call. `io.stdout` receives any side-effecting text;
/// `io.graphics` receives figure mutations.
pub fn call_builtin(
    name: &str,
    args: &[Expr],
    env: &Env,
    span: Span,
    io: &mut ExecIo<'_>,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    match name {
        "disp" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("disp expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let s = format_value(&v);
            io.stdout.push(s);
            Ok(Value::Matrix(DenseMatrix::empty()))
        }
        "zeros" | "ones" => {
            let (r, c) = dims_args(args, env, io, span, name)?;
            let m = if name == "zeros" {
                DenseMatrix::zeros(r, c)
            } else {
                DenseMatrix::ones(r, c)
            };
            Ok(Value::Matrix(m))
        }
        "eye" => {
            let n = scalar_arg(args, env, io, span, "eye")?;
            if n < 0.0 || n.fract() != 0.0 {
                return Err(Diagnostic::error(
                    "eye expects a non-negative integer dimension.",
                    span_op,
                ));
            }
            Ok(Value::Matrix(DenseMatrix::eye(n as usize)))
        }
        "lu" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("lu expects 1 argument: lu(A).", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let y = la::complex::lu_factor(&c, span_op)?;
                    Ok(Value::ComplexMatrix(y))
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let y = la::lu_factor(&a, span_op)?;
                    Ok(Value::Matrix(y))
                }
            }
        }
        "det" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "det expects 1 argument: det(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let d = la::complex::det(&c, span_op)?;
                    if d.is_real() {
                        Ok(Value::Matrix(DenseMatrix::scalar(d.re)))
                    } else {
                        Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(d)))
                    }
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let d = la::det(&a, span_op)?;
                    Ok(Value::Matrix(DenseMatrix::scalar(d)))
                }
            }
        }
        "inv" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "inv expects 1 argument: inv(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let inv_m = la::complex::inv(&c, span_op)?;
                    Ok(Value::from_complex_or_simplified(inv_m))
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let inv_m = la::inv(&a, span_op)?;
                    Ok(Value::Matrix(inv_m))
                }
            }
        }
        "trace" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "trace expects 1 argument: trace(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let tr = la::complex::trace(&c, span_op)?;
                    if tr.is_real() {
                        Ok(Value::Matrix(DenseMatrix::scalar(tr.re)))
                    } else {
                        Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(tr)))
                    }
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let tr = la::trace(&a, span_op)?;
                    Ok(Value::Matrix(DenseMatrix::scalar(tr)))
                }
            }
        }
        "svd" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "svd expects 1 argument: svd(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let s = la::complex_svd_values(&c, span_op)?;
                    Ok(Value::Matrix(s))
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let s = la::svd_values(&a, span_op)?;
                    Ok(Value::Matrix(s))
                }
            }
        }
        "rank" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "rank expects 1 or 2 arguments: rank(A) or rank(A, tol).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let tol = if args.len() == 2 {
                Some(scalar_arg(&args[1..2], env, io, span, "rank")?)
            } else {
                None
            };
            let r = match v {
                Value::ComplexMatrix(c) => la::complex_rank(&c, tol, span_op)?,
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    la::rank(&a, tol, span_op)?
                }
            };
            Ok(Value::Matrix(DenseMatrix::scalar(r as f64)))
        }
        "pinv" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "pinv expects 1 or 2 arguments: pinv(A) or pinv(A, tol).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let tol = if args.len() == 2 {
                Some(scalar_arg(&args[1..2], env, io, span, "pinv")?)
            } else {
                None
            };
            match v {
                Value::ComplexMatrix(c) => {
                    let p = la::complex_pinv(&c, tol, span_op)?;
                    Ok(Value::from_complex_or_simplified(p))
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let p = la::pinv(&a, tol, span_op)?;
                    Ok(Value::Matrix(p))
                }
            }
        }
        "eig" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "eig expects 1 argument: eig(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let (d, _) = la::complex::eig_complex(&c, span_op)?;
                    Ok(Value::ComplexMatrix(d))
                }
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    let (d, _) = la::decomposition::eig(&a, span_op)?;
                    Ok(Value::Matrix(d))
                }
            }
        }
        "prctile" => {
            if args.is_empty() || args.len() > 3 {
                return Err(Diagnostic::error(
                    "prctile expects 2 or 3 arguments.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let a = coerce_matrix_pub(v, span_op)?;
            let pct_vec: Option<Vec<f64>> = if args.len() >= 2 {
                let ev = eval_expr_pub(env, &args[1], io)?;
                match ev {
                    Value::Matrix(m) => {
                        if m.as_scalar().is_some() {
                            None
                        } else {
                            Some(m.data.clone())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            let pct = if args.len() >= 2 {
                let ev = eval_expr_pub(env, &args[1], io)?;
                match ev {
                    Value::Matrix(m) => {
                        if let Some(s) = m.as_scalar() {
                            s
                        } else if !m.data.is_empty() {
                            m.data[0]
                        } else {
                            50.0
                        }
                    }
                    _ => scalar_arg(&args[1..2], env, io, span, "prctile")?,
                }
            } else {
                50.0
            };
            if pct < 0.0 || pct > 100.0 {
                return Err(Diagnostic::error(
                    format!("prctile requires percentiles between 0 and 100; got {}.", pct),
                    span_op,
                ));
            }
            if let Some(vec_pct) = &pct_vec {
                for &pv in vec_pct {
                    if pv < 0.0 || pv > 100.0 {
                        return Err(Diagnostic::error(
                            format!("prctile requires percentiles between 0 and 100; got {}.", pv),
                            span_op,
                        ));
                    }
                }
            }
            let d = if args.len() > 2 {
                let ev = eval_expr_pub(env, &args[2], io)?;
                match ev {
                    Value::Matrix(m) => Some(m.as_scalar().map(|x| x as usize).unwrap_or(1)),
                    _ => Some(scalar_arg(&args[2..3], env, io, span, "prctile")? as usize),
                }
            } else {
                None
            };
            let res = if let Some(vec_pct) = pct_vec {
                la::stats::prctile_multi(&a, &vec_pct, d)
            } else {
                la::stats::prctile(&a, pct, d)
            };
            Ok(Value::Matrix(res))
        }
        "quantile" => {
            if args.is_empty() || args.len() > 3 {
                return Err(Diagnostic::error(
                    "quantile expects 2 or 3 arguments.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let p = if args.len() >= 2 {
                let ev = eval_expr_pub(env, &args[1], io)?;
                match ev {
                    Value::Matrix(m) => {
                        if let Some(s) = m.as_scalar() {
                            s
                        } else if !m.data.is_empty() {
                            m.data[0]
                        } else {
                            0.5
                        }
                    }
                    _ => scalar_arg(&args[1..2], env, io, span, "quantile")?,
                }
            } else {
                0.5
            };
            if p < 0.0 || (p > 1.0 && p.fract() != 0.0) {
                return Err(Diagnostic::error(
                    format!("quantile requires probabilities between 0 and 1 (or integer count > 1); got {}.", p),
                    span_op,
                ));
            }
            let d = if args.len() > 2 {
                let ev = eval_expr_pub(env, &args[2], io)?;
                match ev {
                    Value::Matrix(m) => Some(m.as_scalar().map(|x| x as usize).unwrap_or(1)),
                    _ => Some(scalar_arg(&args[2..3], env, io, span, "quantile")? as usize),
                }
            } else {
                None
            };
            let a = coerce_matrix_pub(v, span_op)?;
            let p_vec: Option<Vec<f64>> = if args.len() >= 2 {
                let ev = eval_expr_pub(env, &args[1], io)?;
                match ev {
                    Value::Matrix(m) => {
                        if m.as_scalar().is_some() {
                            None
                        } else {
                            Some(m.data.clone())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            let res = if p_vec.is_some() {
                // Vector form: probabilities already supplied.
                la::stats::quantile_multi(&a, &p_vec.unwrap(), d)
            } else {
                // Scalar probability argument.
                // Integer-count form: scalar n > 1 produces n quantiles.
                let p_val = p;
                if p_val > 1.0 && p_val.fract() == 0.0 {
                    let n = p_val as usize;
                    let gen: Vec<f64> = (1..=n)
                        .map(|k| k as f64 / (n as f64 + 1.0))
                        .collect();
                    la::stats::quantile_multi(&a, &gen, d)
                } else {
                    la::stats::quantile_matrix(&a, p_val, d)
                }
            };
            Ok(Value::Matrix(res))
        }
        "cov" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error("cov expects 1 or 2 arguments.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let a = coerce_matrix_pub(v, span_op)?;
            let (w, b_opt) = if args.len() > 1 {
                let v2 = eval_expr_pub(env, &args[1], io)?;
                let m2 = coerce_matrix_pub(v2, span_op)?;
                if m2.rows == 1 && m2.cols == 1 {
                    (m2.data[0] as usize, None)
                } else {
                    (0, Some(m2))
                }
            } else {
                (0, None)
            };
            let res = la::stats::cov_matrix(&a, w, b_opt.as_ref());
            Ok(Value::Matrix(res))
        }
        "corrcoef" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "corrcoef expects 1 or 2 arguments.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let a = coerce_matrix_pub(v, span_op)?;
            let res = la::stats::corrcoef_matrix(&a);
            Ok(Value::Matrix(res))
        }
        "norm" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "norm expects 1 or 2 arguments: norm(A) or norm(A, p).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let norm_type = if args.len() == 1 {
                la::NormType::Two
            } else {
                parse_norm_arg(&args[1], env, io, span)?
            };
            let res = match v {
                Value::ComplexMatrix(c) => la::complex_norm(&c, &norm_type, span_op)?,
                _ => {
                    let a = coerce_matrix_pub(v, span_op)?;
                    la::norm(&a, &norm_type, span_op)?
                }
            };
            Ok(Value::Matrix(DenseMatrix::scalar(res)))
        }
        "mldivide" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    "mldivide expects 2 arguments: mldivide(A, B).",
                    span_op,
                ));
            }
            let a = eval_expr_pub(env, &args[0], io)?;
            let b = eval_expr_pub(env, &args[1], io)?;
            la::mldivide_values(&a, &b, span_op)
        }
        "mrdivide" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    "mrdivide expects 2 arguments: mrdivide(A, B).",
                    span_op,
                ));
            }
            let a = eval_expr_pub(env, &args[0], io)?;
            let b = eval_expr_pub(env, &args[1], io)?;
            la::mrdivide_values(&a, &b, span_op)
        }
        "ldivide" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    "ldivide expects 2 arguments: ldivide(A, B).",
                    span_op,
                ));
            }
            let a = eval_expr_pub(env, &args[0], io)?;
            let b = eval_expr_pub(env, &args[1], io)?;
            la::dotdiv_values(&b, &a, span_op)
        }
        "rdivide" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    "rdivide expects 2 arguments: rdivide(A, B).",
                    span_op,
                ));
            }
            let a = eval_expr_pub(env, &args[0], io)?;
            let b = eval_expr_pub(env, &args[1], io)?;
            la::dotdiv_values(&a, &b, span_op)
        }
        "linspace" => crate::arrayfn::linspace(args, env, io, span),
        "diag" => crate::arrayfn::diag(args, env, io, span),
        "repmat" => crate::arrayfn::repmat(args, env, io, span),
        "reshape" => crate::arrayfn::reshape(args, env, io, span),
        "transpose" => crate::arrayfn::transpose(args, env, io, span),
        "flip" => crate::arrayfn::flip(args, env, io, span),
        "fliplr" => crate::arrayfn::fliplr(args, env, io, span),
        "flipud" => crate::arrayfn::flipud(args, env, io, span),
        "rot90" => crate::arrayfn::rot90(args, env, io, span),
        "circshift" => crate::arrayfn::circshift(args, env, io, span),
        "triu" => crate::arrayfn::triu(args, env, io, span),
        "tril" => crate::arrayfn::tril(args, env, io, span),
        "ndims" => crate::arrayfn::ndims(args, env, io, span),
        "isempty" => crate::arrayfn::isempty(args, env, io, span),
        "logspace" => crate::arrayfn::logspace(args, env, io, span),
        "magic" => crate::arrayfn::magic(args, env, io, span),
        "vertcat" => crate::arrayfn::vertcat(args, env, io, span),
        "horzcat" => crate::arrayfn::horzcat(args, env, io, span),
        "cat" => crate::arrayfn::cat(args, env, io, span),
        "blkdiag" => crate::arrayfn::blkdiag(args, env, io, span),
        "rand" => crate::arrayfn::rand(args, env, io, span),
        "randn" => crate::arrayfn::randn(args, env, io, span),
        "randi" => crate::arrayfn::randi(args, env, io, span),
        "randperm" => crate::arrayfn::randperm(args, env, io, span),
        "rng" => crate::arrayfn::rng(args, env, io, span),
        "sort" => crate::arrayfn::sort(args, env, io, span),
        "sortrows" => crate::arrayfn::sortrows(args, env, io, span),
        "unique" => crate::arrayfn::unique(args, env, io, span),
        "ismember" => crate::arrayfn::ismember(args, env, io, span),
        "setdiff" => crate::arrayfn::setdiff(args, env, io, span),
        "union" => crate::arrayfn::union(args, env, io, span),
        "intersect" => crate::arrayfn::intersect(args, env, io, span),
        "find" => crate::arrayfn::find(args, env, io, span),
        "any" => crate::arrayfn::any(args, env, io, span),
        "all" => crate::arrayfn::all(args, env, io, span),
        "cumsum" => crate::arrayfn::cumsum(args, env, io, span),
        "cumprod" => crate::arrayfn::cumprod(args, env, io, span),
        "diff" => crate::arrayfn::diff(args, env, io, span),
        "meshgrid" => {
            let mut matrices = Vec::with_capacity(args.len());
            for arg in args {
                matrices.push(crate::arrayfn::matrix_arg(arg, env, io, span, "meshgrid")?);
            }
            let outputs = la::arrays::meshgrid(&matrices)?;
            if outputs.is_empty() {
                Ok(Value::Matrix(DenseMatrix::empty()))
            } else {
                Ok(Value::Matrix(outputs[0].clone()))
            }
        }
        "ndgrid" => {
            let mut matrices = Vec::with_capacity(args.len());
            for arg in args {
                matrices.push(crate::arrayfn::matrix_arg(arg, env, io, span, "ndgrid")?);
            }
            let outputs = la::arrays::ndgrid(&matrices)?;
            if outputs.is_empty() {
                Ok(Value::Matrix(DenseMatrix::empty()))
            } else {
                Ok(Value::Matrix(outputs[0].clone()))
            }
        }
        "size" => {
            if args.is_empty() {
                return Err(Diagnostic::error(
                    "size expects at least 1 argument.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let (rows, cols) = match &v {
                Value::Matrix(m) => (m.rows, m.cols),
                Value::ComplexMatrix(c) => (c.rows, c.cols),
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    (m.rows, m.cols)
                }
            };
            if args.len() == 1 {
                let d = DenseMatrix::from_rows(vec![vec![rows as f64, cols as f64]], span_op)?;
                Ok(Value::Matrix(d))
            } else if args.len() == 2 {
                let k = scalar_expr(&args[1], env, io, span, "size")?;
                if k != 1.0 && k != 2.0 {
                    return Err(Diagnostic::error(
                        "Second argument of size must be 1 or 2.",
                        span_op,
                    ));
                }
                let d = if k == 1.0 { rows } else { cols };
                Ok(Value::Matrix(DenseMatrix::scalar(d as f64)))
            } else {
                Err(Diagnostic::error("size expects 1 or 2 arguments.", span_op))
            }
        }
        "length" => {
            let v = eval_expr_pub(env, args_first(args, "length", span)?, io)?;
            let (rows, cols) = match &v {
                Value::Matrix(m) => (m.rows, m.cols),
                Value::ComplexMatrix(c) => (c.rows, c.cols),
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    (m.rows, m.cols)
                }
            };
            Ok(Value::Matrix(DenseMatrix::scalar(rows.max(cols) as f64)))
        }
        "numel" => {
            let v = eval_expr_pub(env, args_first(args, "numel", span)?, io)?;
            let (rows, cols) = match &v {
                Value::Matrix(m) => (m.rows, m.cols),
                Value::ComplexMatrix(c) => (c.rows, c.cols),
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    (m.rows, m.cols)
                }
            };
            Ok(Value::Matrix(DenseMatrix::scalar((rows * cols) as f64)))
        }
        "complex" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "complex expects 1 or 2 arguments: complex(re) or complex(re, im).",
                    span_op,
                ));
            }
            let re_v = eval_expr_pub(env, &args[0], io)?;
            if args.len() == 1 {
                match re_v {
                    Value::ComplexMatrix(c) => Ok(Value::ComplexMatrix(c)),
                    Value::Matrix(m) => Ok(Value::ComplexMatrix(ComplexDenseMatrix::from_real(&m))),
                    Value::Boolean(b) => Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
                        Complex64::real(if b { 1.0 } else { 0.0 }),
                    ))),
                    Value::String(_) => Err(Diagnostic::error(
                        "Conversion to double from char is not possible.",
                        span_op,
                    )),
                }
            } else {
                let re_m = coerce_matrix_pub(re_v, span_op)?;
                let im_v = eval_expr_pub(env, &args[1], io)?;
                let im_m = coerce_matrix_pub(im_v, span_op)?;
                Ok(Value::ComplexMatrix(ComplexDenseMatrix::from_parts(
                    &re_m, &im_m, span_op,
                )?))
            }
        }
        "real" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "real expects 1 argument: real(Z).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => Ok(Value::Matrix(c.real_part())),
                Value::Matrix(m) => Ok(Value::Matrix(m)),
                Value::Boolean(b) => Ok(Value::Matrix(DenseMatrix::scalar(if b {
                    1.0
                } else {
                    0.0
                }))),
                Value::String(_) => Err(Diagnostic::error(
                    "Conversion to double from char is not possible.",
                    span_op,
                )),
            }
        }
        "imag" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "imag expects 1 argument: imag(Z).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => Ok(Value::Matrix(c.imag_part())),
                Value::Matrix(m) => Ok(Value::Matrix(DenseMatrix::zeros(m.rows, m.cols))),
                Value::Boolean(_) => Ok(Value::Matrix(DenseMatrix::scalar(0.0))),
                Value::String(_) => Err(Diagnostic::error(
                    "Conversion to double from char is not possible.",
                    span_op,
                )),
            }
        }
        "conj" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "conj expects 1 argument: conj(Z).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.conj()).collect();
                    Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                Value::Matrix(m) => Ok(Value::Matrix(m)),
                Value::Boolean(b) => Ok(Value::Matrix(DenseMatrix::scalar(if b {
                    1.0
                } else {
                    0.0
                }))),
                Value::String(_) => Err(Diagnostic::error(
                    "Conversion to double from char is not possible.",
                    span_op,
                )),
            }
        }
        "angle" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "angle expects 1 argument: angle(Z).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.arg()).collect();
                    Ok(Value::Matrix(DenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                Value::Matrix(m) => {
                    let data = m.data.iter().map(|&x| Complex64::real(x).arg()).collect();
                    Ok(Value::Matrix(DenseMatrix {
                        rows: m.rows,
                        cols: m.cols,
                        data,
                    }))
                }
                Value::Boolean(b) => {
                    let x = if b { 1.0 } else { 0.0 };
                    Ok(Value::Matrix(DenseMatrix::scalar(Complex64::real(x).arg())))
                }
                Value::String(_) => Err(Diagnostic::error(
                    "Conversion to double from char is not possible.",
                    span_op,
                )),
            }
        }
        "isreal" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    "isreal expects 1 argument: isreal(A).",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(_) => Ok(Value::Boolean(false)),
                _ => Ok(Value::Boolean(true)),
            }
        }
        "abs" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("abs expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.abs()).collect();
                    Ok(Value::Matrix(DenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    let data = m.data.into_iter().map(f64::abs).collect();
                    Ok(Value::Matrix(DenseMatrix {
                        rows: m.rows,
                        cols: m.cols,
                        data,
                    }))
                }
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("sqrt expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.sqrt()).collect();
                    Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    let has_neg = m.data.iter().any(|&x| x < 0.0);
                    if has_neg {
                        let data = m.data.iter().map(|&x| Complex64::real(x).sqrt()).collect();
                        Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                            rows: m.rows,
                            cols: m.cols,
                            data,
                        }))
                    } else {
                        let data = m.data.into_iter().map(f64::sqrt).collect();
                        Ok(Value::Matrix(DenseMatrix {
                            rows: m.rows,
                            cols: m.cols,
                            data,
                        }))
                    }
                }
            }
        }
        "exp" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("exp expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.exp()).collect();
                    Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    let data = m.data.into_iter().map(f64::exp).collect();
                    Ok(Value::Matrix(DenseMatrix {
                        rows: m.rows,
                        cols: m.cols,
                        data,
                    }))
                }
            }
        }
        "log" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("log expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            match v {
                Value::ComplexMatrix(c) => {
                    let data = c.data.iter().map(|z| z.ln()).collect();
                    Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                        rows: c.rows,
                        cols: c.cols,
                        data,
                    }))
                }
                _ => {
                    let m = coerce_matrix_pub(v, span_op)?;
                    let has_neg = m.data.iter().any(|&x| x < 0.0);
                    if has_neg {
                        let data = m.data.iter().map(|&x| Complex64::real(x).ln()).collect();
                        Ok(Value::ComplexMatrix(ComplexDenseMatrix {
                            rows: m.rows,
                            cols: m.cols,
                            data,
                        }))
                    } else {
                        let data = m.data.into_iter().map(f64::ln).collect();
                        Ok(Value::Matrix(DenseMatrix {
                            rows: m.rows,
                            cols: m.cols,
                            data,
                        }))
                    }
                }
            }
        }
        "floor" | "ceil" | "round" | "fix" | "log2" | "log10" | "sin" | "cos" | "tan" | "asin"
        | "acos" | "atan" | "sinh" | "cosh" | "tanh" | "deg2rad" | "rad2deg" | "sind" | "cosd"
        | "tand" | "asind" | "acosd" | "atand" | "sign" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    format!("{} expects 1 argument.", name),
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let f: fn(f64) -> f64 = match name {
                "floor" => f64::floor,
                "ceil" => f64::ceil,
                "round" => |x| x.round(),
                "fix" => |x| x.trunc(),
                "log2" => f64::log2,
                "log10" => f64::log10,
                "sin" => f64::sin,
                "cos" => f64::cos,
                "tan" => f64::tan,
                "asin" => f64::asin,
                "acos" => f64::acos,
                "atan" => f64::atan,
                "sinh" => f64::sinh,
                "cosh" => f64::cosh,
                "tanh" => f64::tanh,
                "deg2rad" => |x| x * std::f64::consts::PI / 180.0,
                "rad2deg" => |x| x * 180.0 / std::f64::consts::PI,
                "sind" => sind_deg,
                "cosd" => cosd_deg,
                "tand" => tand_deg,
                "asind" => asind_deg,
                "acosd" => acosd_deg,
                "atand" => atand_deg,
                "sign" => |x| {
                    if x.is_nan() {
                        f64::NAN
                    } else if x > 0.0 {
                        1.0
                    } else if x < 0.0 {
                        -1.0
                    } else {
                        0.0
                    }
                },
                _ => unreachable!(),
            };
            Ok(Value::Matrix(la::map_unary(&m, f)))
        }
        "mean" | "median" | "mode" | "range" | "iqr" | "prod" | "sum" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    format!("{} expects 1 or 2 arguments.", name),
                    span_op,
                ));
            }
            let v0 = eval_expr_pub(env, &args[0], io)?;
            let m0 = coerce_matrix_pub(v0, span_op)?;
            let dim = if args.len() == 2 {
                parse_dim_arg(&args[1], env, io, span, name)?
            } else {
                DimArg::Default
            };
            let res = match dim {
                DimArg::All => {
                    let scalar_val = match name {
                        "mean" => la::stats::mean_slice(&m0.data),
                        "median" => la::stats::median_slice(&m0.data),
                        "mode" => la::stats::mode_slice(&m0.data),
                        "range" => la::stats::range_slice(&m0.data),
                        "iqr" => la::stats::iqr_slice(&m0.data),
                        "prod" => la::stats::prod_slice(&m0.data),
                        "sum" => la::stats::sum_slice(&m0.data),
                        _ => unreachable!(),
                    };
                    DenseMatrix::scalar(scalar_val)
                }
                DimArg::Dim(d) => match name {
                    "mean" => la::stats::mean(&m0, Some(d)),
                    "median" => la::stats::median(&m0, Some(d)),
                    "mode" => la::stats::mode(&m0, Some(d)),
                    "range" => la::stats::range(&m0, Some(d)),
                    "iqr" => la::stats::iqr(&m0, Some(d)),
                    "prod" => la::stats::prod(&m0, Some(d)),
                    "sum" => la::stats::sum(&m0, Some(d)),
                    _ => unreachable!(),
                },
                DimArg::Default => match name {
                    "mean" => la::stats::mean(&m0, None),
                    "median" => la::stats::median(&m0, None),
                    "mode" => la::stats::mode(&m0, None),
                    "range" => la::stats::range(&m0, None),
                    "iqr" => la::stats::iqr(&m0, None),
                    "prod" => la::stats::prod(&m0, None),
                    "sum" => la::stats::sum(&m0, None),
                    _ => unreachable!(),
                },
            };
            Ok(Value::Matrix(res))
        }
        "std" | "var" => {
            if args.is_empty() || args.len() > 3 {
                return Err(Diagnostic::error(
                    format!("{} expects 1, 2, or 3 arguments.", name),
                    span_op,
                ));
            }
            let v0 = eval_expr_pub(env, &args[0], io)?;
            let m0 = coerce_matrix_pub(v0, span_op)?;
            let w = if args.len() >= 2 {
                let vw = eval_expr_pub(env, &args[1], io)?;
                parse_w_from_value(&vw, span_op, name)?
            } else {
                0
            };
            let dim = if args.len() == 3 {
                parse_dim_arg(&args[2], env, io, span, name)?
            } else {
                DimArg::Default
            };
            let res = match dim {
                DimArg::All => {
                    let scalar_val = if name == "std" {
                        la::stats::std_slice(&m0.data, w)
                    } else {
                        la::stats::var_slice(&m0.data, w)
                    };
                    DenseMatrix::scalar(scalar_val)
                }
                DimArg::Dim(d) => {
                    if name == "std" {
                        la::stats::std(&m0, w, Some(d))
                    } else {
                        la::stats::var(&m0, w, Some(d))
                    }
                }
                DimArg::Default => {
                    if name == "std" {
                        la::stats::std(&m0, w, None)
                    } else {
                        la::stats::var(&m0, w, None)
                    }
                }
            };
            Ok(Value::Matrix(res))
        }
        "min" | "max" => {
            if args.is_empty() || args.len() > 3 {
                return Err(Diagnostic::error(
                    format!("{} expects 1, 2, or 3 arguments.", name),
                    span_op,
                ));
            }
            let v0 = eval_expr_pub(env, &args[0], io)?;
            let m0 = coerce_matrix_pub(v0, span_op)?;
            if args.len() == 1 {
                let (vals, _) = if name == "min" {
                    la::stats::min_reduction(&m0, None)
                } else {
                    la::stats::max_reduction(&m0, None)
                };
                Ok(Value::Matrix(vals))
            } else if args.len() == 2 {
                let v1 = eval_expr_pub(env, &args[1], io)?;
                let m1 = coerce_matrix_pub(v1, span_op)?;
                let res = if name == "min" {
                    la::stats::min_elementwise(&m0, &m1, span_op)?
                } else {
                    la::stats::max_elementwise(&m0, &m1, span_op)?
                };
                Ok(Value::Matrix(res))
            } else {
                let _ = eval_expr_pub(env, &args[1], io)?;
                let dim = parse_dim_arg(&args[2], env, io, span, name)?;
                let res = match dim {
                    DimArg::All => {
                        let (val, _) = if name == "min" {
                            la::stats::min_all_with_idx(&m0)
                        } else {
                            la::stats::max_all_with_idx(&m0)
                        };
                        DenseMatrix::scalar(val)
                    }
                    DimArg::Dim(d) => {
                        let (vals, _) = if name == "min" {
                            la::stats::min_reduction(&m0, Some(d))
                        } else {
                            la::stats::max_reduction(&m0, Some(d))
                        };
                        vals
                    }
                    DimArg::Default => {
                        let (vals, _) = if name == "min" {
                            la::stats::min_reduction(&m0, None)
                        } else {
                            la::stats::max_reduction(&m0, None)
                        };
                        vals
                    }
                };
                Ok(Value::Matrix(res))
            }
        }
        "nthroot" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("nthroot expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary(&m1, &m2, span_op, |x, n| {
                if x.is_nan() || n.is_nan() {
                    Ok(f64::NAN)
                } else if n == 0.0 {
                    Err(Diagnostic::error(
                        "nthroot: root must be non-zero.",
                        span_op,
                    ))
                } else if x < 0.0 {
                    if n.fract() == 0.0 && (n as i64) % 2 != 0 {
                        Ok(-((-x).powf(1.0 / n)))
                    } else {
                        Err(Diagnostic::error(
                            "If X is negative, N must be an odd integer.",
                            span_op,
                        ))
                    }
                } else {
                    Ok(x.powf(1.0 / n))
                }
            })?;
            Ok(Value::Matrix(res))
        }
        "gcd" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("gcd expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary_simple(&m1, &m2, span_op, |a, b| {
                let mut a = a.abs() as i64;
                let mut b = b.abs() as i64;
                while b != 0 {
                    let t = b;
                    b = a % b;
                    a = t;
                }
                a as f64
            })?;
            Ok(Value::Matrix(res))
        }
        "lcm" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("lcm expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary_simple(&m1, &m2, span_op, |a, b| {
                let a = a.abs() as i64;
                let b = b.abs() as i64;
                if a == 0 || b == 0 {
                    0.0
                } else {
                    let mut g = a;
                    let mut bb = b;
                    while bb != 0 {
                        let t = bb;
                        bb = g % bb;
                        g = t;
                    }
                    (a / g * b) as f64
                }
            })?;
            Ok(Value::Matrix(res))
        }
        "factorial" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("factorial expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let data = m
                .data
                .iter()
                .map(|&x| {
                    if x.is_nan() || x < 0.0 || x.fract() != 0.0 {
                        f64::NAN
                    } else {
                        let n = x as u64;
                        if n > 170 {
                            f64::INFINITY
                        } else {
                            let mut r = 1.0f64;
                            for i in 2..=n {
                                r *= i as f64;
                            }
                            r
                        }
                    }
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m.rows,
                cols: m.cols,
                data,
            }))
        }
        "nchoosek" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("nchoosek expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let n_val = m1.as_scalar().ok_or_else(|| {
                Diagnostic::error("nchoosek: first argument must be a scalar.", span_op)
            })?;
            let k_val = m2.as_scalar().ok_or_else(|| {
                Diagnostic::error("nchoosek: second argument must be a scalar.", span_op)
            })?;
            if !n_val.is_finite()
                || !k_val.is_finite()
                || n_val.fract() != 0.0
                || k_val.fract() != 0.0
            {
                return Err(Diagnostic::error(
                    "nchoosek: arguments must be non-negative integers.",
                    span_op,
                ));
            }
            let n = n_val as i64;
            let k = k_val as i64;
            if n < 0 || k < 0 || k > n {
                return Err(Diagnostic::error(
                    "nchoosek: invalid n or k (k must be between 0 and n).",
                    span_op,
                ));
            }
            if k == 0 || k == n {
                return Ok(Value::Matrix(DenseMatrix::scalar(1.0)));
            }
            let k_eff = (k as u64).min((n - k) as u64);
            let mut result = 1.0f64;
            for i in 1..=k_eff {
                result = result * (n as f64 - k_eff as f64 + i as f64) / (i as f64);
            }
            Ok(Value::Matrix(DenseMatrix::scalar(result.round())))
        }
        "isprime" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("isprime expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let data = m
                .data
                .iter()
                .map(|&x| {
                    if x < 2.0 || x.fract() != 0.0 || !x.is_finite() {
                        0.0
                    } else {
                        let n = x as i64;
                        if n == 2 {
                            1.0
                        } else if n % 2 == 0 {
                            0.0
                        } else {
                            let mut is_p = true;
                            let mut i = 3i64;
                            while i.saturating_mul(i) <= n {
                                if n % i == 0 {
                                    is_p = false;
                                    break;
                                }
                                i += 2;
                            }
                            if is_p {
                                1.0
                            } else {
                                0.0
                            }
                        }
                    }
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m.rows,
                cols: m.cols,
                data,
            }))
        }
        "primes" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("primes expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let val = m
                .as_scalar()
                .ok_or_else(|| Diagnostic::error("primes expects a scalar argument.", span_op))?;
            if !val.is_finite() || val < 2.0 {
                return Ok(Value::Matrix(DenseMatrix {
                    rows: 1,
                    cols: 0,
                    data: Vec::new(),
                }));
            }
            let n = val as usize;
            let mut sieve = vec![true; n + 1];
            sieve[0] = false;
            sieve[1] = false;
            let limit = (n as f64).sqrt() as usize;
            for i in 2..=limit {
                if sieve[i] {
                    let mut j = i * i;
                    while j <= n {
                        sieve[j] = false;
                        j += i;
                    }
                }
            }
            let primes: Vec<f64> = (2..=n).filter(|&i| sieve[i]).map(|i| i as f64).collect();
            let count = primes.len();
            Ok(Value::Matrix(DenseMatrix {
                rows: 1,
                cols: count,
                data: primes,
            }))
        }
        "factor" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("factor expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let val = m
                .as_scalar()
                .ok_or_else(|| Diagnostic::error("factor expects a scalar argument.", span_op))?;
            if !val.is_finite() {
                return Err(Diagnostic::error("factor: input must be finite.", span_op));
            }
            if val.fract() != 0.0 {
                return Err(Diagnostic::error(
                    "factor: input must be an integer.",
                    span_op,
                ));
            }
            if val <= 0.0 {
                return Err(Diagnostic::error(
                    "factor: input must be a positive integer.",
                    span_op,
                ));
            }
            if val > 9007199254740991.0 {
                return Err(Diagnostic::error(
                    "factor: input exceeds maximum supported integer 2^53 - 1.",
                    span_op,
                ));
            }
            if val == 1.0 {
                return Ok(Value::Matrix(DenseMatrix {
                    rows: 1,
                    cols: 1,
                    data: vec![1.0],
                }));
            }
            let mut factors = Vec::new();
            let mut remaining = val as u64;
            while remaining % 2 == 0 {
                factors.push(2.0);
                remaining /= 2;
            }
            let mut d = 3u64;
            while d.saturating_mul(d) <= remaining {
                while remaining % d == 0 {
                    factors.push(d as f64);
                    remaining /= d;
                }
                d += 2;
            }
            if remaining > 1 {
                factors.push(remaining as f64);
            }
            let count = factors.len();
            Ok(Value::Matrix(DenseMatrix {
                rows: 1,
                cols: count,
                data: factors,
            }))
        }
        "bitand" | "bitor" | "bitxor" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    format!("{} expects 2 arguments.", name),
                    span_op,
                ));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary(&m1, &m2, span_op, |a, b| {
                if !a.is_finite() || !b.is_finite() || a.fract() != 0.0 || b.fract() != 0.0 {
                    return Err(Diagnostic::error(
                        format!("{}: inputs must be integers.", name),
                        span_op,
                    ));
                }
                let ai = a as i64;
                let bi = b as i64;
                let r = match name {
                    "bitand" => ai & bi,
                    "bitor" => ai | bi,
                    "bitxor" => ai ^ bi,
                    _ => unreachable!(),
                };
                Ok(r as f64)
            })?;
            Ok(Value::Matrix(res))
        }
        "bitshift" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(Diagnostic::error(
                    "bitshift expects 2 or 3 arguments.",
                    span_op,
                ));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let max_bits: Option<u32> = if args.len() == 3 {
                let v3 = eval_expr_pub(env, &args[2], io)?;
                let m3 = coerce_matrix_pub(v3, span_op)?;
                let n_val = m3.as_scalar().ok_or_else(|| {
                    Diagnostic::error(
                        "bitshift: third argument must be a scalar integer.",
                        span_op,
                    )
                })?;
                if !n_val.is_finite() || n_val.fract() != 0.0 || n_val < 1.0 || n_val > 64.0 {
                    return Err(Diagnostic::error(
                        "bitshift: maximum bits N must be an integer between 1 and 64.",
                        span_op,
                    ));
                }
                Some(n_val as u32)
            } else {
                None
            };
            let res = la::map_binary(&m1, &m2, span_op, |a, k| {
                if !a.is_finite() || !k.is_finite() || a.fract() != 0.0 || k.fract() != 0.0 {
                    return Err(Diagnostic::error(
                        "bitshift: inputs must be integers.",
                        span_op,
                    ));
                }
                Ok(safe_bitshift(a as i64, k as i64, max_bits))
            })?;
            Ok(Value::Matrix(res))
        }
        "dec2bin" => {
            let (m0, width) = parse_dec_args(args, env, io, span, "dec2bin")?;
            if m0.data.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let mut formatted = Vec::with_capacity(m0.data.len());
            let mut max_len = width.unwrap_or(0);
            for &raw_val in &m0.data {
                if !raw_val.is_finite() {
                    return Err(Diagnostic::error("dec2bin: input must be finite.", span_op));
                }
                let val = raw_val.floor();
                let (is_neg, s) = if val < 0.0 {
                    if val < -2147483648.0 {
                        return Err(Diagnostic::error(
                            "dec2bin: negative value exceeds 32-bit range.",
                            span_op,
                        ));
                    }
                    (true, format!("{:b}", (val as i32) as u32))
                } else {
                    if val > 9007199254740991.0 {
                        return Err(Diagnostic::error(
                            "dec2bin: value exceeds maximum supported integer 2^53 - 1.",
                            span_op,
                        ));
                    }
                    (false, format!("{:b}", val as u64))
                };
                max_len = max_len.max(s.len());
                formatted.push((is_neg, s));
            }
            let rows: Vec<String> = formatted
                .into_iter()
                .map(|(is_neg, s)| {
                    if s.len() < max_len {
                        if is_neg {
                            format!("{:1>width$}", s, width = max_len)
                        } else {
                            format!("{:0>width$}", s, width = max_len)
                        }
                    } else {
                        s
                    }
                })
                .collect();
            let result = if m0.rows == 1 && m0.cols == 1 {
                rows.into_iter().next().unwrap()
            } else {
                rows.join("\n")
            };
            Ok(Value::String(result))
        }
        "bin2dec" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("bin2dec expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let s = extract_string_or_chars(&v, span_op, "bin2dec")?;
            let lines: Vec<&str> = s.lines().collect();
            if lines.is_empty() {
                return Err(Diagnostic::error(
                    "bin2dec: input must be a non-empty binary string.",
                    span_op,
                ));
            }
            let mut results = Vec::with_capacity(lines.len());
            for line in lines {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return Err(Diagnostic::error(
                        "bin2dec: input must be a non-empty binary string.",
                        span_op,
                    ));
                }
                for c in trimmed.chars() {
                    if c != '0' && c != '1' {
                        return Err(Diagnostic::error(
                            "bin2dec: invalid binary string.",
                            span_op,
                        ));
                    }
                }
                let unpadded = trimmed.trim_start_matches('0');
                if unpadded.len() > 53 {
                    return Err(Diagnostic::error(
                        "bin2dec: binary string exceeds maximum precision of 53 bits.",
                        span_op,
                    ));
                }
                let val = if unpadded.is_empty() {
                    0u64
                } else {
                    u64::from_str_radix(unpadded, 2).map_err(|_| {
                        Diagnostic::error("bin2dec: invalid binary string.", span_op)
                    })?
                };
                results.push(val as f64);
            }
            if results.len() == 1 {
                Ok(Value::Matrix(DenseMatrix::scalar(results[0])))
            } else {
                let rows = results.len();
                Ok(Value::Matrix(DenseMatrix {
                    rows,
                    cols: 1,
                    data: results,
                }))
            }
        }
        "dec2hex" => {
            let (m0, width) = parse_dec_args(args, env, io, span, "dec2hex")?;
            if m0.data.is_empty() {
                return Ok(Value::String(String::new()));
            }
            let mut formatted = Vec::with_capacity(m0.data.len());
            let mut max_len = width.unwrap_or(0);
            for &raw_val in &m0.data {
                if !raw_val.is_finite() {
                    return Err(Diagnostic::error("dec2hex: input must be finite.", span_op));
                }
                let val = raw_val.floor();
                let (is_neg, s) = if val < 0.0 {
                    if val < -2147483648.0 {
                        return Err(Diagnostic::error(
                            "dec2hex: negative value exceeds 32-bit range.",
                            span_op,
                        ));
                    }
                    (true, format!("{:X}", (val as i32) as u32))
                } else {
                    if val > 9007199254740991.0 {
                        return Err(Diagnostic::error(
                            "dec2hex: value exceeds maximum supported integer 2^53 - 1.",
                            span_op,
                        ));
                    }
                    (false, format!("{:X}", val as u64))
                };
                max_len = max_len.max(s.len());
                formatted.push((is_neg, s));
            }
            let rows: Vec<String> = formatted
                .into_iter()
                .map(|(is_neg, s)| {
                    if s.len() < max_len {
                        if is_neg {
                            format!("{:F>width$}", s, width = max_len)
                        } else {
                            format!("{:0>width$}", s, width = max_len)
                        }
                    } else {
                        s
                    }
                })
                .collect();
            let result = if m0.rows == 1 && m0.cols == 1 {
                rows.into_iter().next().unwrap()
            } else {
                rows.join("\n")
            };
            Ok(Value::String(result))
        }
        "hex2dec" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("hex2dec expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let s = extract_string_or_chars(&v, span_op, "hex2dec")?;
            let lines: Vec<&str> = s.lines().collect();
            if lines.is_empty() {
                return Err(Diagnostic::error(
                    "hex2dec: input must be a non-empty hexadecimal string.",
                    span_op,
                ));
            }
            let mut results = Vec::with_capacity(lines.len());
            for line in lines {
                let trimmed = line.trim();
                let clean = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                    &trimmed[2..]
                } else {
                    trimmed
                };
                if clean.is_empty() {
                    return Err(Diagnostic::error(
                        "hex2dec: input must be a non-empty hexadecimal string.",
                        span_op,
                    ));
                }
                for c in clean.chars() {
                    if !c.is_ascii_hexdigit() {
                        return Err(Diagnostic::error(
                            "hex2dec: invalid hexadecimal string.",
                            span_op,
                        ));
                    }
                }
                let unpadded = clean.trim_start_matches('0');
                let val = if unpadded.is_empty() {
                    0u64
                } else {
                    if unpadded.len() > 16 {
                        return Err(Diagnostic::error(
                            "hex2dec: hexadecimal string exceeds maximum precision of 52 bits.",
                            span_op,
                        ));
                    }
                    let parsed = u64::from_str_radix(unpadded, 16).map_err(|_| {
                        Diagnostic::error("hex2dec: invalid hexadecimal string.", span_op)
                    })?;
                    if parsed > 9007199254740991 {
                        return Err(Diagnostic::error(
                            "hex2dec: hexadecimal string exceeds maximum precision of 52 bits.",
                            span_op,
                        ));
                    }
                    parsed
                };
                results.push(val as f64);
            }
            if results.len() == 1 {
                Ok(Value::Matrix(DenseMatrix::scalar(results[0])))
            } else {
                let rows = results.len();
                Ok(Value::Matrix(DenseMatrix {
                    rows,
                    cols: 1,
                    data: results,
                }))
            }
        }
        "atan2" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("atan2 expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary_simple(&m1, &m2, span_op, |y, x| y.atan2(x))?;
            Ok(Value::Matrix(res))
        }
        "hypot" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("hypot expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let res = la::map_binary_simple(&m1, &m2, span_op, |a, b| a.hypot(b))?;
            Ok(Value::Matrix(res))
        }
        "mod" | "rem" => {
            if args.len() != 2 {
                return Err(Diagnostic::error(
                    format!("{} expects 2 arguments.", name),
                    span_op,
                ));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let is_mod = name == "mod";
            let res = la::map_binary_simple(&m1, &m2, span_op, |a, b| {
                if a.is_nan() || b.is_nan() {
                    f64::NAN
                } else if b == 0.0 {
                    if is_mod {
                        a
                    } else {
                        f64::NAN
                    }
                } else if is_mod {
                    a - b * (a / b).floor()
                } else {
                    a - b * (a / b).trunc()
                }
            })?;
            Ok(Value::Matrix(res))
        }

        "pi" => {
            if !args.is_empty() {
                return Err(Diagnostic::error("pi takes no arguments.", span_op));
            }
            Ok(Value::Matrix(DenseMatrix::scalar(std::f64::consts::PI)))
        }
        "inf" | "Inf" => {
            if !args.is_empty() {
                return Err(Diagnostic::error("Inf takes no arguments.", span_op));
            }
            Ok(Value::Matrix(DenseMatrix::scalar(f64::INFINITY)))
        }
        "i" | "j" => {
            if !args.is_empty() {
                return Err(Diagnostic::error(
                    format!("{} takes no arguments.", name),
                    span_op,
                ));
            }
            Ok(Value::ComplexMatrix(ComplexDenseMatrix::scalar(
                Complex64::new(0.0, 1.0),
            )))
        }
        // ---- Graphics -------------------------------------------------
        "figure" => graphics_figure(args, env, io, span),
        "title" | "xlabel" | "ylabel" => graphics_label(name, args, env, io, span),
        "legend" => graphics_legend(args, env, io, span),
        "grid" => graphics_grid(args, env, io, span),
        "clf" => graphics_clf(args, io, span),
        "close" => graphics_close(args, env, io, span),
        "plot" | "scatter" => graphics_xy(name, args, env, io, span),
        "bar" => graphics_bar(args, env, io, span),
        "histogram" => graphics_histogram(name, args, env, io, span),

        "csvread" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(Diagnostic::error("csvread expects 1 or 2 arguments: csvread(filename) or csvread(filename, range).", span_op));
            }
            let v_path = eval_expr_pub(env, &args[0], io)?;
            let path_str = match v_path {
                Value::String(s) => s,
                _ => {
                    let m = coerce_matrix_pub(v_path, span_op)?;
                    if let Some(s) = m.as_scalar() {
                        s.to_string()
                    } else {
                        return Err(Diagnostic::error("csvread: file path must be a string or scalar.", span_op));
                    }
                }
            };
            let content = std::fs::read_to_string(&path_str).map_err(|e| Diagnostic::error(format!("csvread: cannot read '{}' ({}).", path_str, e), span_op))?;
            let mut rows: Vec<Vec<f64>> = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('%') {
                    continue;
                }
                // Skip variable-name header lines produced by `save` (single alphabetic word, no commas).
                if !line.contains(',') && line.split_whitespace().next().map_or(false, |s| s.chars().all(|c| c.is_alphabetic() || c == '_')) {
                    continue;
                }
                let mut row: Vec<f64> = Vec::new();
                for token in line.split(',') {
                    let token = token.trim();
                    if token.is_empty() {
                        return Err(Diagnostic::error("csvread: empty cell.", span_op));
                    }
                    if token.eq_ignore_ascii_case("NaN") || token.eq_ignore_ascii_case("nan") {
                        row.push(f64::NAN);
                    } else if token.eq_ignore_ascii_case("Inf") || token.eq_ignore_ascii_case("inf") {
                        row.push(f64::INFINITY);
                    } else if token.eq_ignore_ascii_case("-Inf") || token.eq_ignore_ascii_case("-inf") {
                        row.push(f64::NEG_INFINITY);
                    } else {
                        let val: f64 = token.parse().map_err(|_| Diagnostic::error(format!("csvread: invalid number '{}'", token), span_op))?;
                        row.push(val);
                    }
                }
                if !row.is_empty() {
                    rows.push(row);
                }
            }
            if rows.is_empty() {
                return Ok(Value::Matrix(DenseMatrix::empty()));
            }
            let ncols = rows[0].len();
            for (i, r) in rows.iter().enumerate() {
                if r.len() != ncols {
                    return Err(Diagnostic::error(format!("csvread: row {} has {} columns, expected {}.", i + 1, r.len(), ncols), span_op));
                }
            }
            let flat: Vec<f64> = rows.iter().flatten().copied().collect();
            Ok(Value::Matrix(DenseMatrix { rows: rows.len(), cols: ncols, data: flat }))
        }
        "csvwrite" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("csvwrite expects 2 arguments: csvwrite(filename, A).", span_op));
            }
            let v_path = eval_expr_pub(env, &args[0], io)?;
            let path_str = match v_path {
                Value::String(s) => s,
                _ => {
                    let m = coerce_matrix_pub(v_path, span_op)?;
                    if let Some(s) = m.as_scalar() {
                        s.to_string()
                    } else {
                        return Err(Diagnostic::error("csvwrite: file path must be a string or scalar.", span_op));
                    }
                }
            };
            let a = coerce_matrix_pub(eval_expr_pub(env, &args[1], io)?, span_op)?;
            use std::fs::File;
            use std::io::Write;
            let mut f = File::create(&path_str).map_err(|e| Diagnostic::error(format!("csvwrite: cannot create '{}' ({}).", path_str, e), span_op))?;
            for r in 1..=a.rows {
                let mut line = String::new();
                for c in 1..=a.cols {
                    if c > 1 {
                        line.push(',');
                    }
                    let v = a.get_2d(r, c, span_op)?;
                    if v.is_nan() {
                        line.push_str("NaN");
                    } else if v.is_infinite() {
                        line.push_str("Inf");
                    } else if v == f64::NEG_INFINITY {
                        line.push_str("-Inf");
                    } else {
                        line.push_str(&format!("{}", v));
                    }
                }
                writeln!(f, "{}", line).map_err(|e| Diagnostic::error(format!("csvwrite: write error ({}).", e), span_op))?;
            }
            Ok(Value::Matrix(DenseMatrix::empty()))
        }
        "load" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("load expects 1 argument: load(filename).", span_op));
            }
            let v_path = eval_expr_pub(env, &args[0], io)?;
            let path_str = match v_path {
                Value::String(s) => s,
                _ => {
                    let m = coerce_matrix_pub(v_path, span_op)?;
                    if let Some(s) = m.as_scalar() {
                        s.to_string()
                    } else {
                        return Err(Diagnostic::error("load: file path must be a string or scalar.", span_op));
                    }
                }
            };
            // Minimal persistence: read CSV and set variable `data` (or named variable in second arg if any)
            let content = std::fs::read_to_string(&path_str).map_err(|e| Diagnostic::error(format!("load: cannot read '{}' ({}).", path_str, e), span_op))?;
            let mut rows: Vec<Vec<f64>> = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('%') {
                    continue;
                }
                // Skip variable-name header lines produced by `save` (single alphabetic word, no commas).
                if !line.contains(',') && line.split_whitespace().next().map_or(false, |s| s.chars().all(|c| c.is_alphabetic() || c == '_')) {
                    continue;
                }
                let mut row: Vec<f64> = Vec::new();
                for token in line.split(',') {
                    let token = token.trim();
                    if token.is_empty() {
                        return Err(Diagnostic::error("load: empty cell.", span_op));
                    }
                    if token.eq_ignore_ascii_case("NaN") || token.eq_ignore_ascii_case("nan") {
                        row.push(f64::NAN);
                    } else if token.eq_ignore_ascii_case("Inf") || token.eq_ignore_ascii_case("inf") {
                        row.push(f64::INFINITY);
                    } else if token.eq_ignore_ascii_case("-Inf") || token.eq_ignore_ascii_case("-inf") {
                        row.push(f64::NEG_INFINITY);
                    } else {
                        let val: f64 = token.parse().map_err(|_| Diagnostic::error(format!("load: invalid number '{}'", token), span_op))?;
                        row.push(val);
                    }
                }
                if !row.is_empty() {
                    rows.push(row);
                }
            }
            if rows.is_empty() {
                return Ok(Value::Matrix(DenseMatrix::empty()));
            } else {
                let ncols = rows[0].len();
                for (i, r) in rows.iter().enumerate() {
                    if r.len() != ncols {
                        return Err(Diagnostic::error(format!("load: row {} has {} columns, expected {}.", i + 1, r.len(), ncols), span_op));
                    }
                }
                let flat: Vec<f64> = rows.iter().flatten().copied().collect();
                return Ok(Value::Matrix(DenseMatrix { rows: rows.len(), cols: ncols, data: flat }));
            }
        }
        "save" => {
            if args.len() != 1 && args.len() != 2 {
                return Err(Diagnostic::error("save expects 1 or 2 arguments: save(filename) or save(filename, varname).", span_op));
            }
            let v_path = eval_expr_pub(env, &args[0], io)?;
            let path_str = match v_path {
                Value::String(s) => s,
                _ => {
                    let m = coerce_matrix_pub(v_path, span_op)?;
                    if let Some(s) = m.as_scalar() {
                        s.to_string()
                    } else {
                        return Err(Diagnostic::error("save: file path must be a string or scalar.", span_op));
                    }
                }
            };
            let mut f = std::fs::File::create(&path_str).map_err(|e| Diagnostic::error(format!("save: cannot create '{}' ({}).", path_str, e), span_op))?;
            use std::io::Write;
            for name in env.variable_names() {
                if let Some(val) = env.get(name) {
                    if let Value::Matrix(m) = val {
                        writeln!(f, "{}", name).map_err(|e| Diagnostic::error(format!("save: write error ({}).", e), span_op))?;
                        for r in 1..=m.rows {
                            let mut line = String::new();
                            for c in 1..=m.cols {
                                if c > 1 {
                                    line.push(',');
                                }
                                let v = m.get_2d(r, c, span_op)?;
                                if v.is_nan() {
                                    line.push_str("NaN");
                                } else if v.is_infinite() {
                                    line.push_str("Inf");
                                } else if v == f64::NEG_INFINITY {
                                    line.push_str("-Inf");
                                } else {
                                    line.push_str(&format!("{}", v));
                                }
                            }
                            writeln!(f, "{}", line).map_err(|e| Diagnostic::error(format!("save: write error ({}).", e), span_op))?;
                        }
                    }
                }
            }
            Ok(Value::Matrix(DenseMatrix::empty()))
        }
        other => Err(Diagnostic::error(
            format!("Unrecognized function or variable '{}'.", other),
            span_op,
        )),
    }
}

/// Evaluate one argument as a numeric matrix.
fn numeric_matrix_arg(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<DenseMatrix, Diagnostic> {
    let v = eval_expr_pub(env, expr, io)?;
    coerce_matrix_pub(v, Some(span))
}

fn next_series_label(io: &mut ExecIo<'_>) -> String {
    let figure = io.graphics.current_or_new();
    format!("data {}", figure.items.len() + 1)
}

fn empty_result() -> Value {
    Value::Matrix(DenseMatrix::empty())
}

fn graphics_figure(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    match args.len() {
        0 => {
            io.graphics.current_or_new();
        }
        1 => {
            let n = scalar_arg(args, env, io, span, "figure")?;
            if n < 1.0 || n.fract() != 0.0 {
                return Err(Diagnostic::error(
                    "figure: handle must be a positive integer.",
                    span_op,
                ));
            }
            io.graphics.select_handle(n as usize);
        }
        _ => {
            return Err(Diagnostic::error(
                "figure expects 0 or 1 arguments.",
                span_op,
            ));
        }
    }
    Ok(empty_result())
}

fn graphics_label(
    name: &str,
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    if args.len() != 1 {
        return Err(Diagnostic::error(
            format!("{} expects 1 argument.", name),
            span_op,
        ));
    }
    match eval_expr_pub(env, &args[0], io)? {
        Value::String(s) => {
            let figure = io.graphics.current_or_new();
            match name {
                "title" => figure.title = Some(s),
                "xlabel" => figure.xlabel = Some(s),
                "ylabel" => figure.ylabel = Some(s),
                _ => unreachable!(),
            }
            Ok(empty_result())
        }
        _ => Err(Diagnostic::error(
            format!(
                "{} expects a character vector, e.g. {}('text').",
                name, name
            ),
            span_op,
        )),
    }
}

fn graphics_legend(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    let mut labels = Vec::with_capacity(args.len());
    for arg in args {
        match eval_expr_pub(env, arg, io)? {
            Value::String(s) => labels.push(s),
            _ => {
                return Err(Diagnostic::error(
                    "legend: arguments must be character vectors.",
                    span_op,
                ));
            }
        }
    }
    let figure = io.graphics.current_or_new();
    if labels.len() > figure.items.len() {
        return Err(Diagnostic::error(
            format!(
                "legend: {} labels for {} plotted series.",
                labels.len(),
                figure.items.len()
            ),
            span_op,
        ));
    }
    for (i, item) in figure.items.iter_mut().enumerate() {
        if i < labels.len() {
            item.set_label(&labels[i]);
        }
    }
    figure.legend_shown = true;
    Ok(empty_result())
}

fn graphics_grid(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    if args.len() != 1 {
        return Err(Diagnostic::error("grid expects 1 argument.", span_op));
    }
    let on = match eval_expr_pub(env, &args[0], io)? {
        Value::String(s) => {
            let lowered = s.to_lowercase();
            match lowered.as_str() {
                "on" => true,
                "off" => false,
                _ => {
                    return Err(Diagnostic::error(
                        "grid: argument must be 'on' or 'off'.",
                        span_op,
                    ));
                }
            }
        }
        _ => {
            return Err(Diagnostic::error(
                "grid: argument must be 'on' or 'off'.",
                span_op,
            ));
        }
    };
    io.graphics.current_or_new().grid = on;
    Ok(empty_result())
}

fn graphics_clf(args: &[Expr], io: &mut ExecIo<'_>, span: Span) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    if !args.is_empty() {
        return Err(Diagnostic::error("clf expects no arguments.", span_op));
    }
    let figure = io.graphics.current_or_new();
    let handle = figure.handle;
    *figure = crate::graphics::FigureState::fresh(handle);
    Ok(empty_result())
}

fn graphics_close(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    match args.len() {
        0 => {
            io.graphics.close_current();
        }
        1 => match eval_expr_pub(env, &args[0], io)? {
            Value::String(s) => {
                if s.eq_ignore_ascii_case("all") {
                    io.graphics.clear_all();
                } else {
                    return Err(Diagnostic::error(
                        "close: argument must be a figure handle or 'all'.",
                        span_op,
                    ));
                }
            }
            v => {
                let m = coerce_matrix_pub(v, span_op)?;
                let n = m.as_scalar().ok_or_else(|| {
                    Diagnostic::error("close: argument must be a figure handle or 'all'.", span_op)
                })?;
                if n < 1.0 || n.fract() != 0.0 {
                    return Err(Diagnostic::error(
                        "close: figure handle must be a positive integer.",
                        span_op,
                    ));
                }
                io.graphics.remove_handle(n as usize);
            }
        },
        _ => {
            return Err(Diagnostic::error(
                "close expects 0 or 1 arguments.",
                span_op,
            ));
        }
    }
    Ok(empty_result())
}

fn graphics_xy(
    name: &str,
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    let (x, y) = match args.len() {
        1 => {
            let m = numeric_matrix_arg(&args[0], env, io, span)?;
            let y = vector_from_matrix(&m, name)?;
            let x: Vec<f64> = (1..=y.len()).map(|i| i as f64).collect();
            (x, y)
        }
        2 => {
            let x = vector_from_matrix(&numeric_matrix_arg(&args[0], env, io, span)?, name)?;
            let y = vector_from_matrix(&numeric_matrix_arg(&args[1], env, io, span)?, name)?;
            if x.len() != y.len() {
                return Err(Diagnostic::error(
                    format!("{}: Vectors must be the same length.", name),
                    span_op,
                ));
            }
            (x, y)
        }
        _ => {
            return Err(Diagnostic::error(
                format!("{} expects 1 or 2 arguments.", name),
                span_op,
            ));
        }
    };
    let label = next_series_label(io);
    let figure = io.graphics.current_or_new();
    figure.items.push(if name == "plot" {
        PlotItem::Line { label, x, y }
    } else {
        PlotItem::Scatter { label, x, y }
    });
    Ok(empty_result())
}

fn graphics_bar(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    let (positions, values) = match args.len() {
        1 => {
            let values = vector_from_matrix(&numeric_matrix_arg(&args[0], env, io, span)?, "bar")?;
            let positions: Vec<f64> = (1..=values.len()).map(|i| i as f64).collect();
            (positions, values)
        }
        2 => {
            let x = vector_from_matrix(&numeric_matrix_arg(&args[0], env, io, span)?, "bar")?;
            let y = vector_from_matrix(&numeric_matrix_arg(&args[1], env, io, span)?, "bar")?;
            if x.len() != y.len() {
                return Err(Diagnostic::error(
                    "bar: Vectors must be the same length.",
                    span_op,
                ));
            }
            (x, y)
        }
        _ => {
            return Err(Diagnostic::error("bar expects 1 or 2 arguments.", span_op));
        }
    };
    let label = next_series_label(io);
    io.graphics.current_or_new().items.push(PlotItem::Bar {
        label,
        positions,
        values,
    });
    Ok(empty_result())
}

fn graphics_histogram(
    name: &str,
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    if args.is_empty() || args.len() > 2 {
        return Err(Diagnostic::error(
            format!("{} expects 1 or 2 arguments.", name),
            span_op,
        ));
    }
    let values = vector_from_matrix(&numeric_matrix_arg(&args[0], env, io, span)?, name)?;
    let bins = if args.len() == 2 {
        let b = scalar_arg(&args[1..], env, io, span, name)?;
        if b < 1.0 || b.fract() != 0.0 {
            return Err(Diagnostic::error(
                "histogram: bin count must be a positive integer.",
                span_op,
            ));
        }
        b as usize
    } else {
        10
    };
    let label = next_series_label(io);
    io.graphics
        .current_or_new()
        .items
        .push(PlotItem::Histogram {
            label,
            values,
            bins,
        });
    Ok(empty_result())
}

// ---- Argument helpers ---------------------------------------------------

fn args_first<'a>(args: &'a [Expr], name: &str, span: Span) -> Result<&'a Expr, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            format!("{} expects 1 argument.", name),
            Some(span),
        ));
    }
    Ok(&args[0])
}

fn dims_args(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<(usize, usize), Diagnostic> {
    if args.len() == 1 {
        let n = scalar_arg(args, env, io, span, name)?;
        if n < 0.0 || n.fract() != 0.0 {
            return Err(Diagnostic::error(
                format!("{} expects non-negative integer dimensions.", name),
                Some(span),
            ));
        }
        let n = n as usize;
        return Ok((n, n));
    }
    if args.len() != 2 {
        return Err(Diagnostic::error(
            format!("{} expects 1 or 2 arguments.", name),
            Some(span),
        ));
    }
    let r = scalar_arg(&args[..1], env, io, span, name)?;
    let c = scalar_arg(&args[1..], env, io, span, name)?;
    if r < 0.0 || r.fract() != 0.0 || c < 0.0 || c.fract() != 0.0 {
        return Err(Diagnostic::error(
            format!("{} expects non-negative integer dimensions.", name),
            Some(span),
        ));
    }
    Ok((r as usize, c as usize))
}

fn scalar_expr(
    expr: &Expr,
    env: &Env,
    io: &mut ExecIo<'_>,
    span: Span,
    name: &str,
) -> Result<f64, Diagnostic> {
    let v = eval_expr_pub(env, expr, io)?;
    let m = coerce_matrix_pub(v, Some(span))?;
    m.as_scalar().ok_or_else(|| {
        Diagnostic::error(format!("{} expects a scalar argument.", name), Some(span))
    })
}
/// Reusable argument extraction: returns scalar if scalar matrix, else matrix data.
#[allow(dead_code)]
fn numeric_arg(
    args: &[Expr],
    env: &Env,
    io: &mut ExecIo,
    span: Span,
    name: &str,
    arg_idx: usize,
) -> Result<(f64, Option<Vec<f64>>), Diagnostic> {
    if args.len() <= arg_idx {
        return Err(Diagnostic::error(
            format!("{} expects argument at position {}.", name, arg_idx + 1),
            Some(span),
        ));
    }
    let v = eval_expr_pub(env, &args[arg_idx], io)?;
    match v {
        Value::Matrix(m) => {
            if let Some(s) = m.as_scalar() {
                Ok((s, None))
            } else {
                Ok((f64::NAN, Some(m.data.clone())))
            }
        }
        Value::ComplexMatrix(_) => Err(Diagnostic::error(
            format!("{}: complex arrays not supported for this argument.", name),
            Some(span),
        )),
        Value::String(_) => Err(Diagnostic::error(
            format!("{} expects a numeric argument.", name),
            Some(span),
        )),
        Value::Boolean(_) => Err(Diagnostic::error(
            format!("{} expects a numeric argument.", name),
            Some(span),
        )),
    }
}

