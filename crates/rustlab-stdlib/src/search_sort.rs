//! Sorting and searching operations: sort, find, unique.

use crate::helpers::{extract_f64_array, first_non_singleton_dim, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

pub fn sort_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "sort requires at least 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let dim = if args.len() >= 2 {
        let d = args[1].as_scalar_f64().map_err(crate::helpers::math_err)? as isize;
        if d <= 0 {
            return Err(user_err(
                "MATLAB:sort:invalidDim",
                "Dimension argument must be a positive integer scalar.",
            ));
        }
        d as usize
    } else {
        first_non_singleton_dim(da.shape())
    };

    let in_shape = da.shape();
    let max_rank = in_shape.len().max(dim);
    let mut exp_shape = in_shape.to_vec();
    while exp_shape.len() < max_rank {
        exp_shape.push(1);
    }

    let fiber_len = exp_shape[dim - 1];
    let mut sorted_data = da.as_slice().to_vec();
    let mut idx_data = vec![0.0; da.numel()];

    if fiber_len <= 1 {
        for v in idx_data.iter_mut() {
            *v = 1.0;
        }
        let s_arr = da.clone();
        let i_arr =
            DenseArray::from_vec(in_shape.to_vec(), idx_data).map_err(crate::helpers::math_err)?;
        if nargout > 1 {
            return Ok(vec![Value::Double(s_arr), Value::Double(i_arr)]);
        } else {
            return Ok(vec![Value::Double(s_arr)]);
        }
    }

    let mut base_shape = exp_shape.clone();
    base_shape[dim - 1] = 1;
    let total_fibers: usize = base_shape.iter().product();

    for f_idx in 0..total_fibers {
        let base_coords = rustlab_math::offset_to_coords(f_idx, &base_shape);
        let mut curr_coords = base_coords.clone();
        let mut pairs: Vec<(f64, usize, usize)> = Vec::with_capacity(fiber_len);

        for k in 0..fiber_len {
            curr_coords[dim - 1] = k;
            let offset = rustlab_math::coords_to_offset(&curr_coords, da.strides());
            pairs.push((da.as_slice()[offset], k + 1, offset));
        }

        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        for (k, &(val, idx, _)) in pairs.iter().enumerate() {
            curr_coords[dim - 1] = k;
            let dest_offset = rustlab_math::coords_to_offset(&curr_coords, da.strides());
            sorted_data[dest_offset] = val;
            idx_data[dest_offset] = idx as f64;
        }
    }

    let s_arr =
        DenseArray::from_vec(in_shape.to_vec(), sorted_data).map_err(crate::helpers::math_err)?;
    if nargout > 1 {
        let i_arr =
            DenseArray::from_vec(in_shape.to_vec(), idx_data).map_err(crate::helpers::math_err)?;
        Ok(vec![Value::Double(s_arr), Value::Double(i_arr)])
    } else {
        Ok(vec![Value::Double(s_arr)])
    }
}

pub fn find_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "find requires 1 input."));
    }

    let mut k_limit: Option<usize> = None;
    let mut find_last = false;

    if args.len() >= 2 {
        if let Ok(s) = crate::helpers::extract_string(&args[1]) {
            if s.eq_ignore_ascii_case("last") {
                find_last = true;
                k_limit = Some(1);
            } else if s.eq_ignore_ascii_case("first") {
                find_last = false;
                k_limit = Some(1);
            }
        } else {
            let k_val = args[1].as_scalar_f64().map_err(crate::helpers::math_err)? as isize;
            k_limit = Some(k_val.max(0) as usize);
        }
    }

    if args.len() >= 3 {
        let s = crate::helpers::extract_string(&args[2])?;
        if s.eq_ignore_ascii_case("last") {
            find_last = true;
        } else if s.eq_ignore_ascii_case("first") {
            find_last = false;
        }
    }

    let (linear_indices, row_indices, col_indices, is_row) = match &args[0] {
        Value::Sparse(s) => {
            let (rows, cols, _) = s.find_triplets();
            let mut l_idx = Vec::with_capacity(rows.len());
            let mut r_idx = Vec::with_capacity(rows.len());
            let mut c_idx = Vec::with_capacity(rows.len());
            for (&r, &c) in rows.iter().zip(cols.iter()) {
                l_idx.push((r - 1.0) + (c - 1.0) * (s.rows() as f64) + 1.0);
                r_idx.push(r);
                c_idx.push(c);
            }
            (l_idx, r_idx, c_idx, s.rows() == 1)
        }
        other => {
            let da = extract_f64_array(other)?;
            let total = da.numel();
            let shp = da.shape();
            let (_m, _) = (shp[0], shp.get(1).copied().unwrap_or(1));
            let mut l_idx = Vec::new();
            let mut r_idx = Vec::new();
            let mut c_idx = Vec::new();

            for offset in 0..total {
                let val = da.as_slice()[offset];
                if val != 0.0 && !val.is_nan() {
                    let coords = rustlab_math::offset_to_coords(offset, shp);
                    l_idx.push((offset + 1) as f64);
                    r_idx.push((coords[0] + 1) as f64);
                    c_idx.push((coords.get(1).copied().unwrap_or(0) + 1) as f64);
                }
            }
            (
                l_idx,
                r_idx,
                c_idx,
                shp.first() == Some(&1) && shp.len() == 2,
            )
        }
    };

    let total_found = linear_indices.len();
    let (sel_linear, sel_rows, sel_cols) = if let Some(k) = k_limit {
        let count = k.min(total_found);
        if find_last {
            let start = total_found.saturating_sub(count);
            (
                linear_indices[start..].to_vec(),
                row_indices[start..].to_vec(),
                col_indices[start..].to_vec(),
            )
        } else {
            (
                linear_indices[..count].to_vec(),
                row_indices[..count].to_vec(),
                col_indices[..count].to_vec(),
            )
        }
    } else {
        (linear_indices, row_indices, col_indices)
    };

    if nargout >= 2 {
        let count = sel_rows.len();
        let r_arr = DenseArray::from_vec_unchecked(vec![count, 1], sel_rows);
        let c_arr = DenseArray::from_vec_unchecked(vec![count, 1], sel_cols);
        Ok(vec![Value::Double(r_arr), Value::Double(c_arr)])
    } else {
        let count = sel_linear.len();
        let shape = if is_row {
            vec![1, count]
        } else {
            vec![count, 1]
        };
        let arr = DenseArray::from_vec_unchecked(shape, sel_linear);
        Ok(vec![Value::Double(arr)])
    }
}

pub fn unique_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "unique requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let mut vals: Vec<f64> = da.as_slice().to_vec();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    vals.dedup_by(|a, b| (*a - *b).abs() < 1e-15);

    let count = vals.len();
    let arr = DenseArray::from_vec_unchecked(vec![count, 1], vals);
    Ok(vec![Value::Double(arr)])
}
