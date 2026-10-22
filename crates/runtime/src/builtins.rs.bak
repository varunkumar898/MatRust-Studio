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
use matlab_values::{DenseMatrix, Value};

use crate::graphics::{vector_from_matrix, PlotItem};
use crate::interp::{coerce_matrix_pub, eval_expr_pub, Env, ExecIo};

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
    matches!(name, "pi")
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
        "size" => {
            if args.is_empty() {
                return Err(Diagnostic::error(
                    "size expects at least 1 argument.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            if args.len() == 1 {
                let d = DenseMatrix::from_rows(vec![vec![m.rows as f64, m.cols as f64]], span_op)?;
                Ok(Value::Matrix(d))
            } else if args.len() == 2 {
                let k = scalar_expr(&args[1], env, io, span, "size")?;
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
            let v = eval_expr_pub(env, args_first(args, "length", span)?, io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(
                DenseMatrix::scalar(m.rows.max(m.cols) as f64),
            ))
        }
        "numel" => {
            let v = eval_expr_pub(env, args_first(args, "numel", span)?, io)?;
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
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let f: fn(f64) -> f64 = match name {
                "abs" => f64::abs,
                "sqrt" => f64::sqrt,
                "floor" => f64::floor,
                "ceil" => f64::ceil,
                "round" => |x| x.round(), // MATLAB round: half away from zero; noted limitation
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
            let v = eval_expr_pub(env, &args[0], io)?;
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
            let v = eval_expr_pub(env, &args[0], io)?;
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

        other => Err(Diagnostic::error(
            format!("Unrecognized function or variable '{}'.", other),
            span_op,
        )),
    }
}

// ---- Graphics helpers -------------------------------------------------

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
    let r = scalar_arg(&args[..1], env, io, span, name)? as usize;
    let c = scalar_arg(&args[1..], env, io, span, name)? as usize;
    Ok((r, c))
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
