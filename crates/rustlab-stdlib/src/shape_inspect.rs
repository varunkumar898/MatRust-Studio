//! Array shape inspection and manipulation: size, length, numel, reshape, repmat, diag, etc.

use crate::helpers::{extract_f64_array, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

pub fn size(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "size requires at least 1 input."));
    }
    let shp = args[0].shape();

    if args.len() >= 2 {
        // size(A, dim)
        let dim = args[1].as_scalar_f64().map_err(math_err)? as usize;
        let d = if dim == 0 {
            return Err(user_err("MATLAB:size:badDim", "Dimension must be >= 1."));
        } else if dim <= shp.len() {
            shp[dim - 1]
        } else {
            1
        };
        return Ok(vec![Value::scalar(d as f64)]);
    }

    if nargout > 1 {
        // [d1, d2, ...] = size(A)
        let mut outputs = Vec::with_capacity(nargout);
        for i in 0..nargout {
            let dim_sz = if i < shp.len() {
                shp[i]
            } else if i == nargout - 1 && shp.len() > nargout {
                // If fewer outputs than dimensions, last output absorbs remaining trailing dims product
                shp[i..].iter().product()
            } else {
                1
            };
            outputs.push(Value::scalar(dim_sz as f64));
        }
        return Ok(outputs);
    }

    // Default: returns 1 x K row vector of dimensions
    let data: Vec<f64> = shp.iter().map(|&d| d as f64).collect();
    let arr = DenseArray::row_vector(data);
    Ok(vec![Value::Double(arr)])
}

pub fn length(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "length requires 1 input."));
    }
    let shp = args[0].shape();
    let max_dim = shp.iter().copied().max().unwrap_or(0);
    Ok(vec![Value::scalar(max_dim as f64)])
}

pub fn numel(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "numel requires 1 input."));
    }
    Ok(vec![Value::scalar(args[0].numel() as f64)])
}

pub fn ndims(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "ndims requires 1 input."));
    }
    Ok(vec![Value::scalar(args[0].shape().len() as f64)])
}

pub fn isempty(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isempty requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_empty())])
}

pub fn isscalar(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isscalar requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].is_scalar())])
}

pub fn isvector(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "isvector requires 1 input."));
    }
    let s = args[0].shape();
    let is_vec = s.len() == 2 && (s[0] == 1 || s[1] == 1);
    Ok(vec![Value::logical_scalar(is_vec)])
}

pub fn ismatrix(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "ismatrix requires 1 input."));
    }
    Ok(vec![Value::logical_scalar(args[0].shape().len() == 2)])
}

pub fn reshape(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "reshape requires array and dimensions.",
        ));
    }
    let da = extract_f64_array(&args[0])?;
    let new_dims = crate::helpers::parse_dims(&args[1..])?;
    let target_numel: usize = new_dims.iter().product();

    if target_numel != da.numel() {
        return Err(user_err(
            "MATLAB:getReshapeDims:notSameNumel",
            "To RESHAPE the number of elements must not change.",
        ));
    }

    let out = DenseArray::from_vec(new_dims, da.as_slice().to_vec()).map_err(math_err)?;
    Ok(vec![Value::Double(out)])
}

pub fn repmat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "repmat requires at least 1 input.",
        ));
    }
    let da = extract_f64_array(&args[0])?;
    let reps = crate::helpers::parse_dims(&args[1..])?;
    if reps.len() != 2 || da.ndims() != 2 {
        return Err(user_err(
            "MATLAB:repmat:unsupportedDims",
            "2D repmat currently supported.",
        ));
    }

    let (m_rep, n_rep) = (reps[0], reps[1]);
    let (m_orig, n_orig) = (da.shape()[0], da.shape()[1]);
    let (m_out, n_out) = (m_orig * m_rep, n_orig * n_rep);

    let mut out_data = vec![0.0; m_out * n_out];

    for c_rep in 0..n_rep {
        for r_rep in 0..m_rep {
            let row_offset = r_rep * m_orig;
            let col_offset = c_rep * n_orig;

            for c in 0..n_orig {
                for r in 0..m_orig {
                    let src_val = *da.get_subscript(&[r + 1, c + 1])?;
                    let dst_r = row_offset + r;
                    let dst_c = col_offset + c;
                    out_data[dst_r + dst_c * m_out] = src_val;
                }
            }
        }
    }

    let arr = DenseArray::from_vec(vec![m_out, n_out], out_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn squeeze(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "squeeze requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let orig_shape = da.shape();
    if orig_shape.len() <= 2 {
        return Ok(vec![Value::Double(da)]);
    }

    let mut new_shape: Vec<usize> = orig_shape.iter().copied().filter(|&dim| dim != 1).collect();
    if new_shape.is_empty() {
        new_shape = vec![1, 1];
    } else if new_shape.len() == 1 {
        new_shape.push(1);
    }

    let arr = DenseArray::from_vec(new_shape, da.as_slice().to_vec()).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn diag(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "diag requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let k = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as isize
    } else {
        0
    };

    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err("MATLAB:diag:invalidDims", "Input must be 2D."));
    }

    let (m, n) = (shp[0], shp[1]);
    let is_vec = m == 1 || n == 1;

    if is_vec {
        // Vector -> Diagonal matrix
        let vec_len = da.numel();
        let diag_size = (vec_len as isize + k.abs()) as usize;
        let mut mat_data = vec![0.0; diag_size * diag_size];

        for i in 0..vec_len {
            let (r, c) = if k >= 0 {
                (i, i + k as usize)
            } else {
                (i + (-k) as usize, i)
            };
            if r < diag_size && c < diag_size {
                let v = da.as_slice()[i];
                mat_data[r + c * diag_size] = v;
            }
        }
        let out = DenseArray::from_vec(vec![diag_size, diag_size], mat_data).map_err(math_err)?;
        Ok(vec![Value::Double(out)])
    } else {
        // Matrix -> Column vector of diagonal elements
        let mut diag_vals = Vec::new();
        let mut r: isize = if k < 0 { -k } else { 0 };
        let mut c: isize = if k > 0 { k } else { 0 };

        while (r as usize) < m && (c as usize) < n {
            let val = *da.get_subscript(&[r as usize + 1, c as usize + 1])?;
            diag_vals.push(val);
            r += 1;
            c += 1;
        }

        let out_len = diag_vals.len();
        let out = DenseArray::from_vec(vec![out_len, 1], diag_vals).map_err(math_err)?;
        Ok(vec![Value::Double(out)])
    }
}

pub fn fliplr(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "fliplr requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.ndims() != 2 {
        return Err(user_err("MATLAB:fliplr:not2D", "Input must be 2D matrix."));
    }
    let (m, n) = (da.shape()[0], da.shape()[1]);
    let mut out = vec![0.0; m * n];

    for c in 0..n {
        let rev_c = n - 1 - c;
        for r in 0..m {
            let val = *da.get_subscript(&[r + 1, rev_c + 1])?;
            out[r + c * m] = val;
        }
    }
    let arr = DenseArray::from_vec(vec![m, n], out).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn flipud(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "flipud requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.ndims() != 2 {
        return Err(user_err("MATLAB:flipud:not2D", "Input must be 2D matrix."));
    }
    let (m, n) = (da.shape()[0], da.shape()[1]);
    let mut out = vec![0.0; m * n];

    for c in 0..n {
        for r in 0..m {
            let rev_r = m - 1 - r;
            let val = *da.get_subscript(&[rev_r + 1, c + 1])?;
            out[r + c * m] = val;
        }
    }
    let arr = DenseArray::from_vec(vec![m, n], out).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

pub fn rot90(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "rot90 requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.ndims() != 2 {
        return Err(user_err("MATLAB:rot90:not2D", "Input must be 2D matrix."));
    }
    let k = if args.len() >= 2 {
        args[1].as_scalar_f64().map_err(math_err)? as isize
    } else {
        1
    };

    let k_mod = ((k % 4) + 4) % 4;
    let (m, n) = (da.shape()[0], da.shape()[1]);

    match k_mod {
        0 => Ok(vec![Value::Double(da)]),
        1 => {
            // 90 deg CCW: (r, c) -> (n - 1 - c, r) with output shape [n, m]
            let mut out = vec![0.0; m * n];
            for c in 0..n {
                for r in 0..m {
                    let val = *da.get_subscript(&[r + 1, c + 1])?;
                    let out_r = n - 1 - c;
                    let out_c = r;
                    out[out_r + out_c * n] = val;
                }
            }
            let arr = DenseArray::from_vec(vec![n, m], out).map_err(math_err)?;
            Ok(vec![Value::Double(arr)])
        }
        2 => {
            // 180 deg
            let mut out = vec![0.0; m * n];
            for c in 0..n {
                for r in 0..m {
                    let val = *da.get_subscript(&[r + 1, c + 1])?;
                    let out_r = m - 1 - r;
                    let out_c = n - 1 - c;
                    out[out_r + out_c * m] = val;
                }
            }
            let arr = DenseArray::from_vec(vec![m, n], out).map_err(math_err)?;
            Ok(vec![Value::Double(arr)])
        }
        3 => {
            // 270 deg CCW: (r, c) -> (c, m - 1 - r) with output shape [n, m]
            let mut out = vec![0.0; m * n];
            for c in 0..n {
                for r in 0..m {
                    let val = *da.get_subscript(&[r + 1, c + 1])?;
                    let out_r = c;
                    let out_c = m - 1 - r;
                    out[out_r + out_c * n] = val;
                }
            }
            let arr = DenseArray::from_vec(vec![n, m], out).map_err(math_err)?;
            Ok(vec![Value::Double(arr)])
        }
        _ => unreachable!(),
    }
}
