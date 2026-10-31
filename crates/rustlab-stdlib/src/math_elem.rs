//! Elementwise mathematical functions: abs, sqrt, exp, log, trig, rounding, etc.

use crate::helpers::{extract_f64_array, math_err, user_err};
use num_complex::Complex64;
use rustlab_eval::EvalError;
use rustlab_math::Value;

fn apply_f64_unary<F>(args: &[Value], f: F) -> Result<Vec<Value>, EvalError>
where
    F: Fn(f64) -> f64,
{
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => {
            let res = da.map(|&x| f(x));
            Ok(vec![Value::Double(res)])
        }
        Value::ComplexDouble(da) => {
            let res = da.map(|c| Complex64::new(f(c.re), c.im));
            Ok(vec![Value::ComplexDouble(res)])
        }
        v => {
            let da = extract_f64_array(v)?;
            let res = da.map(|&x| f(x));
            Ok(vec![Value::Double(res)])
        }
    }
}

pub fn abs_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Double(da.map(|&x| x.abs()))]),
        Value::Single(da) => Ok(vec![Value::Single(da.map(|&x| x.abs()))]),
        Value::ComplexDouble(da) => Ok(vec![Value::Double(da.map(|c| c.norm()))]),
        Value::Int8(da) => Ok(vec![Value::Int8(da.map(|&x| {
            if x == i8::MIN {
                i8::MAX
            } else {
                x.abs()
            }
        }))]),
        Value::Int16(da) => Ok(vec![Value::Int16(da.map(|&x| {
            if x == i16::MIN {
                i16::MAX
            } else {
                x.abs()
            }
        }))]),
        Value::Int32(da) => Ok(vec![Value::Int32(da.map(|&x| {
            if x == i32::MIN {
                i32::MAX
            } else {
                x.abs()
            }
        }))]),
        Value::Int64(da) => Ok(vec![Value::Int64(da.map(|&x| {
            if x == i64::MIN {
                i64::MAX
            } else {
                x.abs()
            }
        }))]),
        Value::UInt8(_) | Value::UInt16(_) | Value::UInt32(_) | Value::UInt64(_) => {
            Ok(vec![args[0].clone()])
        }
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da.map(|&x| x.abs()))])
        }
    }
}

pub fn sign_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Double(da.map(|&x| {
            if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                0.0
            }
        }))]),
        Value::ComplexDouble(da) => Ok(vec![Value::ComplexDouble(da.map(|c| {
            let n = c.norm();
            if n == 0.0 {
                Complex64::new(0.0, 0.0)
            } else {
                c / n
            }
        }))]),
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da.map(|&x| {
                if x > 0.0 {
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }))])
        }
    }
}

pub fn sqrt_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => {
            let has_neg = da.as_slice().iter().any(|&x| x < 0.0);
            if has_neg {
                let comp = da.map(|&x| Complex64::new(x, 0.0).sqrt());
                Ok(vec![Value::ComplexDouble(comp)])
            } else {
                Ok(vec![Value::Double(da.map(|&x| x.sqrt()))])
            }
        }
        Value::ComplexDouble(da) => Ok(vec![Value::ComplexDouble(da.map(|c| c.sqrt()))]),
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da.map(|&x| x.sqrt()))])
        }
    }
}

pub fn exp_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Double(da.map(|&x| x.exp()))]),
        Value::ComplexDouble(da) => Ok(vec![Value::ComplexDouble(da.map(|c| c.exp()))]),
        v => apply_f64_unary(std::slice::from_ref(v), |x| x.exp()),
    }
}

pub fn log_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "Not enough input arguments."));
    }
    match &args[0] {
        Value::Double(da) => {
            let has_neg = da.as_slice().iter().any(|&x| x < 0.0);
            if has_neg {
                let comp = da.map(|&x| Complex64::new(x, 0.0).ln());
                Ok(vec![Value::ComplexDouble(comp)])
            } else {
                Ok(vec![Value::Double(da.map(|&x| x.ln()))])
            }
        }
        Value::ComplexDouble(da) => Ok(vec![Value::ComplexDouble(da.map(|c| c.ln()))]),
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da.map(|&x| x.ln()))])
        }
    }
}

pub fn log10_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.log10())
}

pub fn log2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.log2())
}

pub fn sin_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.sin())
}

pub fn cos_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.cos())
}

pub fn tan_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.tan())
}

pub fn asin_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.asin())
}

pub fn acos_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.acos())
}

pub fn atan_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.atan())
}

pub fn atan2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "atan2 requires y and x coordinates.",
        ));
    }
    let da_y = extract_f64_array(&args[0])?;
    let da_x = extract_f64_array(&args[1])?;
    let res =
        rustlab_math::elementwise_binary(&da_y, &da_x, |&y, &x| y.atan2(x)).map_err(math_err)?;
    Ok(vec![Value::Double(res)])
}

pub fn round_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.round())
}

pub fn floor_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.floor())
}

pub fn ceil_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.ceil())
}

pub fn fix_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    apply_f64_unary(args, |x| x.trunc())
}

pub fn mod_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "mod requires 2 inputs."));
    }
    let da_x = extract_f64_array(&args[0])?;
    let da_y = extract_f64_array(&args[1])?;
    let res = rustlab_math::elementwise_binary(&da_x, &da_y, |&x, &y| {
        if y == 0.0 {
            x
        } else {
            x - y * (x / y).floor()
        }
    })
    .map_err(math_err)?;
    Ok(vec![Value::Double(res)])
}

pub fn rem_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "rem requires 2 inputs."));
    }
    let da_x = extract_f64_array(&args[0])?;
    let da_y = extract_f64_array(&args[1])?;
    let res = rustlab_math::elementwise_binary(&da_x, &da_y, |&x, &y| {
        if y == 0.0 {
            f64::NAN
        } else {
            x - y * (x / y).trunc()
        }
    })
    .map_err(math_err)?;
    Ok(vec![Value::Double(res)])
}

/// `isnan(A)` returns a logical array with true where elements are NaN.
pub fn isnan_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "isnan requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Logical(da.map(|&x| x.is_nan()))]),
        Value::ComplexDouble(da) => Ok(vec![Value::Logical(
            da.map(|c| c.re.is_nan() || c.im.is_nan()),
        )]),
        v => {
            let shp = v.shape().to_vec();
            let count: usize = shp.iter().product();
            Ok(vec![Value::Logical(
                rustlab_math::DenseArray::from_vec_unchecked(shp, vec![false; count]),
            )])
        }
    }
}

/// `isinf(A)` returns a logical array with true where elements are +/-Inf.
pub fn isinf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "isinf requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Logical(da.map(|&x| x.is_infinite()))]),
        Value::ComplexDouble(da) => Ok(vec![Value::Logical(
            da.map(|c| c.re.is_infinite() || c.im.is_infinite()),
        )]),
        v => {
            let shp = v.shape().to_vec();
            let count: usize = shp.iter().product();
            Ok(vec![Value::Logical(
                rustlab_math::DenseArray::from_vec_unchecked(shp, vec![false; count]),
            )])
        }
    }
}

/// `isfinite(A)` returns a logical array with true where elements are finite.
pub fn isfinite_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "isfinite requires at least 1 input.",
        ));
    }
    match &args[0] {
        Value::Double(da) => Ok(vec![Value::Logical(da.map(|&x| x.is_finite()))]),
        Value::ComplexDouble(da) => Ok(vec![Value::Logical(
            da.map(|c| c.re.is_finite() && c.im.is_finite()),
        )]),
        v => {
            let shp = v.shape().to_vec();
            let count: usize = shp.iter().product();
            Ok(vec![Value::Logical(
                rustlab_math::DenseArray::from_vec_unchecked(shp, vec![true; count]),
            )])
        }
    }
}
