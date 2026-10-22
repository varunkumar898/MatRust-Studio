//! Built-in functions for milestone 1.
//!
//! Builtin calls receive unevaluated argument ASTs so functions like `size(A, 1)`
//! can inspect index expressions cheaply, but they are evaluated first in
//! `call_builtin` for simplicity. Variables shadow builtins at runtime.

use matlab_diagnostics::{Diagnostic, Span};
use matlab_linalg as la;
use matlab_syntax::ast::Expr;
use matlab_values::{DenseMatrix, Value};

use crate::interp::{coerce_matrix_pub, eval_expr_pub, Env};

/// Format a value the way MATLAB prints it in the command window.
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Matrix(m) => m.format_matlab(),
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
    stdout: &mut Vec<String>,
    span: Span,
    name: &str,
) -> Result<f64, Diagnostic> {
    if args.len() != 1 {
        return Err(Diagnostic::error(
            format!("{} expects 1 argument.", name),
            Some(span),
        ));
    }
    let v = eval_expr_pub(env, &args[0], stdout)?;
    let m = coerce_matrix_pub(v, Some(span))?;
    m.as_scalar().ok_or_else(|| {
        Diagnostic::error(format!("{} expects a scalar argument.", name), Some(span))
    })
}

/// Names that resolve as bare identifiers when not shadowed by a variable.
pub fn is_known_constant(name: &str) -> bool {
    matches!(name, "pi")
}

/// Dispatch a builtin call. `stdout` receives any side-effecting text.
pub fn call_builtin(
    name: &str,
    args: &[Expr],
    env: &Env,
    span: Span,
    stdout: &mut Vec<String>,
) -> Result<Value, Diagnostic> {
    let span_op = Some(span);
    match name {
        "disp" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("disp expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], stdout)?;
            let s = match &v {
                Value::Matrix(m) => m.format_matlab(),
                Value::String(s) => s.clone(),
                Value::Boolean(b) => {
                    if *b {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
            };
            stdout.push(s);
            Ok(Value::Matrix(DenseMatrix::empty()))
        }
        "zeros" | "ones" => {
            let (r, c) = dims_args(args, env, stdout, span, name)?;
            let m = if name == "zeros" {
                DenseMatrix::zeros(r, c)
            } else {
                DenseMatrix::ones(r, c)
            };
            Ok(Value::Matrix(m))
        }
        "eye" => {
            let n = scalar_arg(args, env, stdout, span, "eye")?;
            if n < 0.0 || n.fract() != 0.0 {
                return Err(Diagnostic::error(
                    "eye expects a non-negative integer dimension.",
                    span_op,
                ));
            }
            Ok(Value::Matrix(DenseMatrix::eye(n as usize)))
        }
        "size" => {
            if args.is_empty() {
                return Err(Diagnostic::error(
                    "size expects at least 1 argument.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            if args.len() == 1 {
                let d = DenseMatrix::from_rows(vec![vec![m.rows as f64, m.cols as f64]], span_op)?;
                Ok(Value::Matrix(d))
            } else if args.len() == 2 {
                let k = scalar_expr(&args[1], env, stdout, span, "size")?;
                if k != 1.0 && k != 2.0 {
                    return Err(Diagnostic::error(
                        "Second argument of size must be 1 or 2.",
                        span_op,
                    ));
                }
                let d = if k == 1.0 { m.rows } else { m.cols };
                Ok(Value::Matrix(DenseMatrix::scalar(d as f64)))
            } else {
                Err(Diagnostic::error("size expects 1 or 2 arguments.", span_op))
            }
        }
        "length" => {
            let v = eval_expr_pub(env, args_first(args, "length", span)?, stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(
                DenseMatrix::scalar(m.rows.max(m.cols) as f64),
            ))
        }
        "numel" => {
            let v = eval_expr_pub(env, args_first(args, "numel", span)?, stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(DenseMatrix::scalar((m.rows * m.cols) as f64)))
        }
        "abs" | "sqrt" | "floor" | "ceil" | "round" | "exp" | "log" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    format!("{} expects 1 argument.", name),
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let f: fn(f64) -> f64 = match name {
                "abs" => f64::abs,
                "sqrt" => f64::sqrt,
                "floor" => f64::floor,
                "ceil" => f64::ceil,
                "round" => |x| x.round(), // MATLAB round: half away from zero for .5; note limitation
                "exp" => f64::exp,
                "log" => f64::ln,
                _ => unreachable!(),
            };
            Ok(Value::Matrix(la::map_unary(&m, f)))
        }
        "sum" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("sum expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(la::reduce_cols(&m, 0.0, |a, b| a + b)))
        }
        "max" | "min" => {
            if args.len() != 1 {
                return Err(Diagnostic::error(
                    format!("{} expects 1 argument.", name),
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], stdout)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let init = if name == "max" {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            Ok(Value::Matrix(la::reduce_cols(&m, init, |a, b| {
                if name == "max" {
                    a.max(b)
                } else {
                    a.min(b)
                }
            })))
        }
        "pi" => {
            if !args.is_empty() {
                return Err(Diagnostic::error("pi takes no arguments.", span_op));
            }
            Ok(Value::Matrix(DenseMatrix::scalar(std::f64::consts::PI)))
        }
        other => Err(Diagnostic::error(
            format!("Unrecognized function or variable '{}'.", other),
            span_op,
        )),
    }
}

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
    stdout: &mut Vec<String>,
    span: Span,
    name: &str,
) -> Result<(usize, usize), Diagnostic> {
    if args.len() == 1 {
        let n = scalar_arg(args, env, stdout, span, name)?;
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
    let r = scalar_arg(&args[..1], env, stdout, span, name)? as usize;
    let c = scalar_arg(&args[1..], env, stdout, span, name)? as usize;
    Ok((r, c))
}

fn scalar_expr(
    expr: &Expr,
    env: &Env,
    stdout: &mut Vec<String>,
    span: Span,
    name: &str,
) -> Result<f64, Diagnostic> {
    let v = eval_expr_pub(env, expr, stdout)?;
    let m = coerce_matrix_pub(v, Some(span))?;
    m.as_scalar().ok_or_else(|| {
        Diagnostic::error(format!("{} expects a scalar argument.", name), Some(span))
    })
}
