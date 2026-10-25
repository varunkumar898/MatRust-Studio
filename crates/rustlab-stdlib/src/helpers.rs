//! Internal helpers for parsing MATLAB arguments and validating shapes.

use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, MathError, Value};
use rustlab_parse::Span;

/// Helper to wrap MathError with a default span.
pub fn math_err(error: MathError) -> EvalError {
    EvalError::Math {
        error,
        span: Span::default(),
    }
}

/// Helper to construct a standard MATLAB error.
pub fn user_err(identifier: &str, message: &str) -> EvalError {
    EvalError::UserError {
        identifier: identifier.to_string(),
        message: message.to_string(),
        span: Span::default(),
    }
}

/// Parses dimension arguments:
/// e.g. `(3)` -> `[3, 3]`, `(3, 4)` -> `[3, 4]`, `([3, 4])` -> `[3, 4]`
pub fn parse_dims(args: &[Value]) -> Result<Vec<usize>, EvalError> {
    if args.is_empty() {
        return Ok(vec![1, 1]);
    }

    if args.len() == 1 {
        let v = &args[0];
        if v.is_scalar() {
            let n = v.as_scalar_f64().map_err(math_err)? as isize;
            let sz = n.max(0) as usize;
            return Ok(vec![sz, sz]);
        } else if let Ok(arr) = v.to_double_array() {
            let dims: Vec<usize> = arr
                .as_slice()
                .iter()
                .map(|&d| (d as isize).max(0) as usize)
                .collect();
            if dims.len() == 1 {
                return Ok(vec![dims[0], dims[0]]);
            }
            return Ok(dims);
        }
    }

    let mut dims = Vec::with_capacity(args.len());
    for a in args {
        let n = a.as_scalar_f64().map_err(math_err)? as isize;
        dims.push(n.max(0) as usize);
    }
    if dims.len() == 1 {
        dims.push(dims[0]);
    }
    Ok(dims)
}

/// Extracts a DenseArray<f64> from a Value, returning an error if incompatible.
pub fn extract_f64_array(val: &Value) -> Result<DenseArray<f64>, EvalError> {
    val.to_double_array().map_err(math_err)
}

/// Finds the first non-singleton dimension (1-based index) of a shape.
pub fn first_non_singleton_dim(shape: &[usize]) -> usize {
    for (i, &s) in shape.iter().enumerate() {
        if s > 1 {
            return i + 1;
        }
    }
    1
}

/// Extracts a string from a Char row vector or String scalar.
pub fn extract_string(val: &Value) -> Result<String, EvalError> {
    match val {
        Value::Char(c) => Ok(c.as_slice().iter().collect()),
        Value::String(s) => Ok(s.as_slice().first().cloned().unwrap_or_default()),
        _ => Err(user_err(
            "MATLAB:invalidType",
            "Expected character vector or string scalar.",
        )),
    }
}
