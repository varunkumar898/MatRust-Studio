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
            let m = coerce_matrix_pub(v, Some(span))?;
            Ok(vec![
                Value::Matrix(DenseMatrix::scalar(m.rows as f64)),
                Value::Matrix(DenseMatrix::scalar(m.cols as f64)),
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
        _ => {
            let val = call_builtin(name, args, env, span, io)?;
            Ok(vec![val])
        }
    }
}
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
        "abs" | "sqrt" | "floor" | "ceil" | "round" | "fix" | "exp" | "log" | "log2" | "log10"
        | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh"
        | "deg2rad" | "rad2deg" | "sind" | "cosd" | "tand" => {
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
                "round" => |x| x.round(),
                "fix" => |x| x.trunc(),
                "exp" => f64::exp,
                "log" => f64::ln,
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
                "sind" => |x| (x * std::f64::consts::PI / 180.0).sin(),
                "cosd" => |x| (x * std::f64::consts::PI / 180.0).cos(),
                "tand" => |x| (x * std::f64::consts::PI / 180.0).tan(),
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
        "nthroot" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("nthroot expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&x, &n)| x.powf(1.0 / n))
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "gcd" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("gcd expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| {
                    let mut a = a.abs() as i64;
                    let mut b = b.abs() as i64;
                    while b != 0 {
                        let t = b;
                        b = a % b;
                        a = t;
                    }
                    a as f64
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "lcm" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("lcm expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| {
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
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
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
                    if x < 0.0 {
                        f64::NAN
                    } else {
                        let n = x as u64;
                        let mut r = 1u64;
                        for i in 2..=n {
                            r = r.saturating_mul(i);
                        }
                        r as f64
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
            let n = coerce_matrix_pub(v1, span_op)?.data[0] as i64;
            let k = coerce_matrix_pub(v2, span_op)?.data[0] as i64;
            if n < 0 || k < 0 || k > n {
                return Err(Diagnostic::error("nchoosek: invalid n or k.", span_op));
            }
            let mut result = 1i64;
            for i in 1..=k {
                result = result * (n - k + i) / i;
            }
            Ok(Value::Matrix(DenseMatrix::scalar(result as f64)))
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
                    let n = x as i64;
                    if n < 2 {
                        0.0
                    } else if n == 2 {
                        1.0
                    } else if n % 2 == 0 {
                        0.0
                    } else {
                        let mut is_p = true;
                        let mut i = 3;
                        while i * i <= n {
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
            let n = m.data[0] as usize;
            if n < 2 {
                return Ok(Value::Matrix(DenseMatrix {
                    rows: 0,
                    cols: 0,
                    data: Vec::new(),
                }));
            }
            let mut sieve = vec![true; n + 1];
            sieve[0] = false;
            sieve[1] = false;
            for i in 2..=n {
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
            let n = m.data[0] as i64;
            if n < 0 {
                return Err(Diagnostic::error(
                    "factor: input must be a non-negative integer.",
                    span_op,
                ));
            }
            if n <= 1 {
                if n == 1 {
                    return Ok(Value::Matrix(DenseMatrix {
                        rows: 1,
                        cols: 1,
                        data: vec![1.0],
                    }));
                }
                return Ok(Value::Matrix(DenseMatrix {
                    rows: 0,
                    cols: 0,
                    data: Vec::new(),
                }));
            }
            let mut factors = Vec::new();
            let mut remaining = n;
            let mut d = 2i64;
            while d * d <= remaining {
                while remaining % d == 0 {
                    factors.push(d as f64);
                    remaining /= d;
                }
                d += if d == 2 { 1 } else { 2 };
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
        "bitand" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("bitand expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| ((a as i64) & (b as i64)) as f64)
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "bitor" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("bitor expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| ((a as i64) | (b as i64)) as f64)
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "bitxor" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("bitxor expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| ((a as i64) ^ (b as i64)) as f64)
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
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
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &k)| {
                    let shift = k as i64;
                    if shift >= 0 {
                        ((a as i64) << shift) as f64
                    } else {
                        ((a as i64) >> (-shift)) as f64
                    }
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "asind" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("asind expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(la::map_unary(&m, |x| {
                x.asin() * 180.0 / std::f64::consts::PI
            })))
        }
        "acosd" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("acosd expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(la::map_unary(&m, |x| {
                x.acos() * 180.0 / std::f64::consts::PI
            })))
        }
        "atand" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("atand expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            Ok(Value::Matrix(la::map_unary(&m, |x| {
                x.atan() * 180.0 / std::f64::consts::PI
            })))
        }
        "dec2bin" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "dec2bin expects 1 or 2 arguments.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let n = if args.len() == 2 {
                eval_expr_pub(env, &args[1], io)?
                    .as_matrix()
                    .and_then(|m| m.as_scalar())
                    .map(|x| x as usize)
            } else {
                None
            };
            let val = m.data[0] as i64;
            let mut s = if val < 0 {
                format!("{:b}", val as u32)
            } else {
                format!("{:b}", val as u64)
            };
            if let Some(width) = n {
                if s.len() < width {
                    s = format!("{:0>width$}", s, width = width);
                }
            }
            Ok(Value::String(s))
        }
        "bin2dec" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("bin2dec expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let s = match v {
                Value::String(s) => s,
                Value::Matrix(m) => m.data.iter().map(|&x| x as u8 as char).collect(),
                _ => {
                    return Err(Diagnostic::error(
                        "bin2dec expects a character vector.",
                        span_op,
                    ))
                }
            };
            let val = i64::from_str_radix(s.trim(), 2)
                .map_err(|_| Diagnostic::error("bin2dec: invalid binary string.", span_op))?;
            Ok(Value::Matrix(DenseMatrix::scalar(val as f64)))
        }
        "dec2hex" => {
            if args.is_empty() || args.len() > 2 {
                return Err(Diagnostic::error(
                    "dec2hex expects 1 or 2 arguments.",
                    span_op,
                ));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let m = coerce_matrix_pub(v, span_op)?;
            let n = if args.len() == 2 {
                eval_expr_pub(env, &args[1], io)?
                    .as_matrix()
                    .and_then(|m| m.as_scalar())
                    .map(|x| x as usize)
            } else {
                None
            };
            let val = m.data[0] as i64;
            let mut s = if val < 0 {
                format!("{:X}", val as u32)
            } else {
                format!("{:X}", val as u64)
            };
            if let Some(width) = n {
                if s.len() < width {
                    s = format!("{:0>width$}", s, width = width);
                }
            }
            Ok(Value::String(s))
        }
        "hex2dec" => {
            if args.len() != 1 {
                return Err(Diagnostic::error("hex2dec expects 1 argument.", span_op));
            }
            let v = eval_expr_pub(env, &args[0], io)?;
            let s = match v {
                Value::String(s) => s,
                Value::Matrix(m) => m.data.iter().map(|&x| x as u8 as char).collect(),
                _ => {
                    return Err(Diagnostic::error(
                        "hex2dec expects a character vector.",
                        span_op,
                    ))
                }
            };
            let val = i64::from_str_radix(s.trim().trim_start_matches("0x"), 16)
                .map_err(|_| Diagnostic::error("hex2dec: invalid hexadecimal string.", span_op))?;
            Ok(Value::Matrix(DenseMatrix::scalar(val as f64)))
        }
        "atan2" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("atan2 expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&y, &x)| y.atan2(x))
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
        }
        "hypot" => {
            if args.len() != 2 {
                return Err(Diagnostic::error("hypot expects 2 arguments.", span_op));
            }
            let v1 = eval_expr_pub(env, &args[0], io)?;
            let v2 = eval_expr_pub(env, &args[1], io)?;
            let m1 = coerce_matrix_pub(v1, span_op)?;
            let m2 = coerce_matrix_pub(v2, span_op)?;
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| a.hypot(b))
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
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
            let data = m1
                .data
                .iter()
                .zip(m2.data.iter())
                .map(|(&a, &b)| {
                    if name == "mod" {
                        a - b * (a / b).floor()
                    } else {
                        a - b * (a / b).trunc()
                    }
                })
                .collect();
            Ok(Value::Matrix(DenseMatrix {
                rows: m1.rows,
                cols: m1.cols,
                data,
            }))
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
