//! Reductions and summary statistics: sum, prod, mean, std, var, min, max, any, all, diff.
//!
//! Implements strict MATLAB reduction semantics:
//! - N-dimensional reductions along arbitrary dimension `dim >= 1`.
//! - MathWorks MATLAB documentation parity:
//!   - `sum`, `prod`, `cumsum` on integer classes preserve the integer class and use saturating arithmetic.
//!   - `mean` always returns `double` even for integer inputs.

use crate::helpers::{extract_f64_array, first_non_singleton_dim, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{coords_to_offset, normalize_shape, offset_to_coords, DenseArray, Value};

/// Helper to reduce an N-D tensor along dimension `dim` (1-based).
pub fn reduce_nd<T, R, F>(da: &DenseArray<T>, dim: usize, mut op: F) -> DenseArray<R>
where
    T: Clone,
    R: Clone + Default + PartialEq + std::fmt::Debug,
    F: FnMut(&[T]) -> R,
{
    let in_shape = da.shape();
    let max_rank = in_shape.len().max(dim);
    let mut exp_shape = in_shape.to_vec();
    while exp_shape.len() < max_rank {
        exp_shape.push(1);
    }

    let mut out_shape = exp_shape.clone();
    out_shape[dim - 1] = 1;
    let norm_out_shape = normalize_shape(&out_shape);
    let total_out: usize = norm_out_shape.iter().product();

    let fiber_len = exp_shape[dim - 1];
    let mut out_data = Vec::with_capacity(total_out);
    let mut fiber_buf = Vec::with_capacity(fiber_len);

    for out_offset in 0..total_out {
        let out_coords = offset_to_coords(out_offset, &norm_out_shape);
        fiber_buf.clear();
        for k in 0..fiber_len {
            let mut in_coords = vec![0; exp_shape.len()];
            for (i, &c) in out_coords.iter().enumerate() {
                if i < in_coords.len() {
                    in_coords[i] = c;
                }
            }
            in_coords[dim - 1] = k;
            let in_offset = coords_to_offset(&in_coords, da.strides());
            fiber_buf.push(da.as_slice()[in_offset].clone());
        }
        out_data.push(op(&fiber_buf));
    }

    DenseArray::from_vec_unchecked(norm_out_shape, out_data)
}

/// Helper for cumulative operations (cumsum, cumprod) along dimension `dim` (1-based).
pub fn cum_nd<T, F>(da: &DenseArray<T>, dim: usize, mut op: F) -> DenseArray<T>
where
    T: Clone + Default + PartialEq + std::fmt::Debug,
    F: FnMut(T, &T) -> T,
{
    let in_shape = da.shape();
    let max_rank = in_shape.len().max(dim);
    let mut exp_shape = in_shape.to_vec();
    while exp_shape.len() < max_rank {
        exp_shape.push(1);
    }

    let fiber_len = exp_shape[dim - 1];
    if fiber_len <= 1 {
        return da.clone();
    }

    let mut out_data = da.as_slice().to_vec();

    // Reduced shape where dim has size 1
    let mut base_shape = exp_shape.clone();
    base_shape[dim - 1] = 1;
    let total_fibers: usize = base_shape.iter().product();

    for f_idx in 0..total_fibers {
        let base_coords = offset_to_coords(f_idx, &base_shape);
        let mut curr_coords = base_coords.clone();

        // First element
        curr_coords[dim - 1] = 0;
        let first_offset = coords_to_offset(&curr_coords, da.strides());
        let mut acc = da.as_slice()[first_offset].clone();
        out_data[first_offset] = acc.clone();

        for k in 1..fiber_len {
            curr_coords[dim - 1] = k;
            let in_offset = coords_to_offset(&curr_coords, da.strides());
            acc = op(acc, &da.as_slice()[in_offset]);
            out_data[in_offset] = acc.clone();
        }
    }

    DenseArray::from_vec_unchecked(in_shape.to_vec(), out_data)
}

fn as_string(v: &Value) -> Option<String> {
    match v {
        Value::Char(ca) => Some(ca.as_slice().iter().collect()),
        Value::String(sa) => Some(sa.as_slice().first().cloned().unwrap_or_default()),
        _ => None,
    }
}

fn parse_dim_and_nan_flag(args: &[Value], default_dim: usize) -> Result<(usize, bool), EvalError> {
    let mut dim = default_dim;
    let mut omitnan = false;

    for arg in &args[1..] {
        if let Some(s) = as_string(arg) {
            if s.eq_ignore_ascii_case("omitnan") {
                omitnan = true;
            } else if s.eq_ignore_ascii_case("includenan") {
                omitnan = false;
            } else {
                return Err(user_err(
                    "MATLAB:data:badOption",
                    &format!("Unsupported option '{}'", s),
                ));
            }
        } else {
            dim = arg.as_scalar_f64().map_err(math_err)? as usize;
            if dim == 0 {
                return Err(user_err(
                    "MATLAB:badsubscript",
                    "Dimension argument must be a positive integer scalar.",
                ));
            }
        }
    }
    Ok((dim, omitnan))
}

/// `sum(A [, dim] [, 'omitnan'])` calculates sum along dimension `dim`.
/// Preserves integer class with saturating addition.
pub fn sum_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "sum requires at least 1 input."));
    }
    let val = &args[0];
    if val.is_empty() {
        return Ok(vec![Value::scalar(0.0)]);
    }

    let (dim, omitnan) = parse_dim_and_nan_flag(args, first_non_singleton_dim(val.shape()))?;

    match val {
        Value::Int8(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0i8, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::Int8(res)])
        }
        Value::Int16(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0i16, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::Int16(res)])
        }
        Value::Int32(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0i32, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::Int32(res)])
        }
        Value::Int64(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0i64, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::Int64(res)])
        }
        Value::UInt8(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0u8, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::UInt8(res)])
        }
        Value::UInt16(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0u16, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::UInt16(res)])
        }
        Value::UInt32(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0u32, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::UInt32(res)])
        }
        Value::UInt64(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(0u64, |acc, &x| acc.saturating_add(x))
            });
            Ok(vec![Value::UInt64(res)])
        }
        Value::Single(a) => {
            let res = reduce_nd(a, dim, |slice| {
                if omitnan {
                    slice.iter().filter(|x| !x.is_nan()).sum::<f32>()
                } else {
                    slice.iter().sum::<f32>()
                }
            });
            Ok(vec![Value::Single(res)])
        }
        Value::Sparse(s) => {
            let da = s.to_dense();
            let res = reduce_nd(&da, dim, |slice| {
                if omitnan {
                    slice.iter().filter(|x| !x.is_nan()).sum::<f64>()
                } else {
                    slice.iter().sum::<f64>()
                }
            });
            Ok(vec![Value::Double(res)])
        }
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            let res = reduce_nd(&da, dim, |slice| {
                if omitnan {
                    slice.iter().filter(|x| !x.is_nan()).sum::<f64>()
                } else {
                    slice.iter().sum::<f64>()
                }
            });
            Ok(vec![Value::Double(res)])
        }
    }
}

/// `prod(A [, dim])` calculates product along dimension `dim`.
/// Preserves integer class with saturating multiplication.
pub fn prod_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "prod requires at least 1 input."));
    }
    let val = &args[0];
    if val.is_empty() {
        return Ok(vec![Value::scalar(1.0)]);
    }

    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(val.shape())
    };

    match val {
        Value::Int8(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1i8, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::Int8(res)])
        }
        Value::Int16(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1i16, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::Int16(res)])
        }
        Value::Int32(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1i32, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::Int32(res)])
        }
        Value::Int64(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1i64, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::Int64(res)])
        }
        Value::UInt8(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1u8, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::UInt8(res)])
        }
        Value::UInt16(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1u16, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::UInt16(res)])
        }
        Value::UInt32(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1u32, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::UInt32(res)])
        }
        Value::UInt64(a) => {
            let res = reduce_nd(a, dim, |slice| {
                slice.iter().fold(1u64, |acc, &x| acc.saturating_mul(x))
            });
            Ok(vec![Value::UInt64(res)])
        }
        Value::Single(a) => {
            let res = reduce_nd(a, dim, |slice| slice.iter().product::<f32>());
            Ok(vec![Value::Single(res)])
        }
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            let res = reduce_nd(&da, dim, |slice| slice.iter().product::<f64>());
            Ok(vec![Value::Double(res)])
        }
    }
}

/// `cumsum(A [, dim])` cumulative sum along dimension `dim`.
/// Preserves integer class with saturating addition.
pub fn cumsum_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "cumsum requires at least 1 input.",
        ));
    }
    let val = &args[0];
    if val.is_empty() {
        return Ok(vec![val.clone()]);
    }

    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(val.shape())
    };

    match val {
        Value::Int8(a) => Ok(vec![Value::Int8(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::Int16(a) => Ok(vec![Value::Int16(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::Int32(a) => Ok(vec![Value::Int32(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::Int64(a) => Ok(vec![Value::Int64(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::UInt8(a) => Ok(vec![Value::UInt8(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::UInt16(a) => Ok(vec![Value::UInt16(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::UInt32(a) => Ok(vec![Value::UInt32(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::UInt64(a) => Ok(vec![Value::UInt64(cum_nd(a, dim, |acc, &x| {
            acc.saturating_add(x)
        }))]),
        Value::Single(a) => Ok(vec![Value::Single(cum_nd(a, dim, |acc, &x| acc + x))]),
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            Ok(vec![Value::Double(cum_nd(&da, dim, |acc, &x| acc + x))])
        }
    }
}

/// `cumprod(A [, dim])` cumulative product along dimension `dim`.
pub fn cumprod_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "cumprod requires at least 1 input.",
        ));
    }
    let val = &args[0];
    if val.is_empty() {
        return Ok(vec![val.clone()]);
    }

    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(val.shape())
    };

    let da = val.to_double_array().map_err(math_err)?;
    Ok(vec![Value::Double(cum_nd(&da, dim, |acc, &x| acc * x))])
}

/// `diff(X [, dim])` differences along dimension `dim`.
pub fn diff_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "diff requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let shp = da.shape();
    if da.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(shp)
    };

    let dim_len = if dim <= shp.len() { shp[dim - 1] } else { 1 };
    if dim_len <= 1 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let mut out_shape = shp.to_vec();
    out_shape[dim - 1] = dim_len - 1;
    let norm_out = normalize_shape(&out_shape);
    let total_out: usize = norm_out.iter().product();
    let mut out_data = Vec::with_capacity(total_out);

    for out_offset in 0..total_out {
        let out_coords = offset_to_coords(out_offset, &norm_out);
        let c1 = out_coords.clone();
        let mut c2 = out_coords;
        c2[dim - 1] += 1;
        let off1 = coords_to_offset(&c1, da.strides());
        let off2 = coords_to_offset(&c2, da.strides());
        out_data.push(da.as_slice()[off2] - da.as_slice()[off1]);
    }

    let arr = DenseArray::from_vec(norm_out, out_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

/// `mean(A [, dim] [, 'omitnan'])` calculates mean along dimension `dim`.
/// According to MathWorks MATLAB Documentation on 'mean':
/// "mean(A) returns double floating-point numbers even if the input A is single or integer."
pub fn mean_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "mean requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }

    let (dim, omitnan) = parse_dim_and_nan_flag(args, first_non_singleton_dim(da.shape()))?;

    let res = reduce_nd(&da, dim, |slice| {
        if omitnan {
            let mut count = 0usize;
            let mut total = 0.0f64;
            for &x in slice {
                if !x.is_nan() {
                    total += x;
                    count += 1;
                }
            }
            if count == 0 {
                f64::NAN
            } else {
                total / count as f64
            }
        } else {
            let count = slice.len() as f64;
            let total: f64 = slice.iter().sum();
            total / count
        }
    });
    Ok(vec![Value::Double(res)])
}

/// `var(A [, flag, dim])` variance.
pub fn var_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "var requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let flag = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        0
    };
    let dim = if args.len() >= 3 {
        args[2].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(da.shape())
    };

    let res = reduce_nd(&da, dim, |slice| {
        let n = slice.len();
        if n <= 1 {
            return 0.0;
        }
        let mean = slice.iter().sum::<f64>() / (n as f64);
        let sum_sq_diff: f64 = slice.iter().map(|&x| (x - mean).powi(2)).sum();
        let divisor = if flag == 1 { n as f64 } else { (n - 1) as f64 };
        sum_sq_diff / divisor
    });
    Ok(vec![Value::Double(res)])
}

/// `std(A [, flag, dim])` standard deviation.
pub fn std_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let mut v = var_fn(args, 1)?;
    let val = v
        .pop()
        .ok_or_else(|| user_err("MATLAB:minrhs", "Not enough input arguments"))?;
    let da = extract_f64_array(&val)?;
    Ok(vec![Value::Double(da.map(|&x| x.sqrt()))])
}

/// `min(A [, dim] [, 'omitnan'])` minimum along dimension `dim`.
pub fn min_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "min requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let (dim, omitnan) = parse_dim_and_nan_flag(args, first_non_singleton_dim(da.shape()))?;

    let val_arr = reduce_nd(&da, dim, |slice| {
        if omitnan {
            let mut best = f64::NAN;
            for &x in slice {
                if !x.is_nan() && (best.is_nan() || x < best) {
                    best = x;
                }
            }
            best
        } else {
            let mut best = slice[0];
            for &x in &slice[1..] {
                if x < best || best.is_nan() {
                    best = x;
                }
            }
            best
        }
    });

    if nargout > 1 {
        let idx_arr = reduce_nd(&da, dim, |slice| {
            if omitnan {
                let mut best = f64::NAN;
                let mut best_idx = 1;
                for (i, &x) in slice.iter().enumerate() {
                    if !x.is_nan() && (best.is_nan() || x < best) {
                        best = x;
                        best_idx = i + 1;
                    }
                }
                best_idx as f64
            } else {
                let mut best = slice[0];
                let mut best_idx = 1;
                for (i, &x) in slice.iter().enumerate().skip(1) {
                    if x < best || best.is_nan() {
                        best = x;
                        best_idx = i + 1;
                    }
                }
                best_idx as f64
            }
        });
        Ok(vec![Value::Double(val_arr), Value::Double(idx_arr)])
    } else {
        Ok(vec![Value::Double(val_arr)])
    }
}

/// `max(A [, dim] [, 'omitnan'])` maximum along dimension `dim`.
pub fn max_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "max requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let (dim, omitnan) = parse_dim_and_nan_flag(args, first_non_singleton_dim(da.shape()))?;

    let val_arr = reduce_nd(&da, dim, |slice| {
        if omitnan {
            let mut best = f64::NAN;
            for &x in slice {
                if !x.is_nan() && (best.is_nan() || x > best) {
                    best = x;
                }
            }
            best
        } else {
            let mut best = slice[0];
            for &x in &slice[1..] {
                if x > best || best.is_nan() {
                    best = x;
                }
            }
            best
        }
    });

    if nargout > 1 {
        let idx_arr = reduce_nd(&da, dim, |slice| {
            if omitnan {
                let mut best = f64::NAN;
                let mut best_idx = 1;
                for (i, &x) in slice.iter().enumerate() {
                    if !x.is_nan() && (best.is_nan() || x > best) {
                        best = x;
                        best_idx = i + 1;
                    }
                }
                best_idx as f64
            } else {
                let mut best = slice[0];
                let mut best_idx = 1;
                for (i, &x) in slice.iter().enumerate().skip(1) {
                    if x > best || best.is_nan() {
                        best = x;
                        best_idx = i + 1;
                    }
                }
                best_idx as f64
            }
        });
        Ok(vec![Value::Double(val_arr), Value::Double(idx_arr)])
    } else {
        Ok(vec![Value::Double(val_arr)])
    }
}

/// `any(A [, dim])` true if any element is non-zero along dimension `dim`.
pub fn any_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "any requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(da.shape())
    };

    let res = reduce_nd(&da, dim, |slice| slice.iter().any(|&x| x != 0.0));
    Ok(vec![Value::Logical(res)])
}

/// `all(A [, dim])` true if all elements are non-zero along dimension `dim`.
pub fn all_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "all requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let dim = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as usize
    } else {
        first_non_singleton_dim(da.shape())
    };

    let res = reduce_nd(&da, dim, |slice| slice.iter().all(|&x| x != 0.0));
    Ok(vec![Value::Logical(res)])
}
