//! Diagnostic and assertion builtins: error, warning, assert.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::Value;
use rustlab_parse::Span;

/// `error(msg)` or `error(id, msg, ...)` raises a structured MATLAB error.
pub fn error_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:error", "error requires at least 1 input."));
    }
    let (id, msg) = if args.len() == 1 {
        let text = value_to_text(&args[0]);
        ("MATLAB:error".to_string(), text)
    } else {
        let id = value_to_text(&args[0]);
        let msg = value_to_text(&args[1]);
        (id, msg)
    };
    Err(EvalError::UserError {
        identifier: id,
        message: msg,
        span: Span::default(),
    })
}

/// `warning(msg)` or `warning(id, msg)` emits a diagnostic warning to stderr without halting.
pub fn warning_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Ok(Vec::new());
    }
    let msg = if args.len() == 1 {
        value_to_text(&args[0])
    } else {
        format!("{}: {}", value_to_text(&args[0]), value_to_text(&args[1]))
    };
    eprintln!("Warning: {}", msg);
    Ok(Vec::new())
}

/// `assert(cond, ...)` asserts truthiness of `cond`.
pub fn assert_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:assert:minrhs",
            "assert requires at least 1 input.",
        ));
    }
    let is_true = match &args[0] {
        Value::Logical(l) => !l.is_empty() && l.as_slice().iter().all(|&b| b),
        Value::Double(d) => !d.is_empty() && d.as_slice().iter().all(|&x| x != 0.0),
        _ => false,
    };
    if is_true {
        return Ok(Vec::new());
    }
    let (id, msg) = if args.len() == 1 {
        (
            "MATLAB:assertion:failed".to_string(),
            "Assertion failed.".to_string(),
        )
    } else if args.len() == 2 {
        (
            "MATLAB:assertion:failed".to_string(),
            value_to_text(&args[1]),
        )
    } else {
        (value_to_text(&args[1]), value_to_text(&args[2]))
    };
    Err(EvalError::UserError {
        identifier: id,
        message: msg,
        span: Span::default(),
    })
}

fn value_to_text(v: &Value) -> String {
    match v {
        Value::Char(c) => c.as_slice().iter().collect(),
        Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
        other => format!("{other}"),
    }
}
