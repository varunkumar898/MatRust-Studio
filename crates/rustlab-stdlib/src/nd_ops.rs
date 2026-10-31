//! N-dimensional array manipulation: cat, horzcat, vertcat, permute, meshgrid, ndgrid, repmat, reshape.

use crate::helpers::{extract_f64_array, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{
    compute_column_major_strides, coords_to_offset, normalize_shape, offset_to_coords, DenseArray,
    Value,
};

/// `cat(dim, A, B, ...)` concatenates arrays along dimension `dim`.
pub fn cat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "cat requires at least 2 inputs."));
    }
    let dim = args[0].as_scalar_f64().map_err(math_err)? as usize;
    if dim == 0 {
        return Err(user_err("MATLAB:cat:badDim", "Dimension must be >= 1."));
    }

    let items = &args[1..];
    // Filter out completely empty 0x0 matrices if there are non-empty ones
    let non_empty: Vec<&Value> = items.iter().filter(|v| !v.is_empty()).collect();
    if non_empty.is_empty() {
        return Ok(vec![Value::empty()]);
    }

    // Determine target concatenated shape
    let max_rank = non_empty
        .iter()
        .map(|v| v.shape().len())
        .max()
        .unwrap_or(2)
        .max(dim);
    let mut ref_shape = vec![1; max_rank];
    for &v in &non_empty {
        for (i, &d) in v.shape().iter().enumerate() {
            if i != dim - 1 {
                if ref_shape[i] == 1 && d != 1 {
                    ref_shape[i] = d;
                } else if d != 1 && d != ref_shape[i] {
                    return Err(user_err(
                        "MATLAB:catenate:dimensionMismatch",
                        "Dimensions of arrays being concatenated are not consistent.",
                    ));
                }
            }
        }
    }

    let mut total_dim_len = 0;
    for &v in &non_empty {
        let d = if dim <= v.shape().len() {
            v.shape()[dim - 1]
        } else {
            1
        };
        total_dim_len += d;
    }
    ref_shape[dim - 1] = total_dim_len;

    // Concatenate element data in column-major order
    let out_shape = normalize_shape(&ref_shape);
    let out_strides = compute_column_major_strides(&out_shape);
    let total_elements: usize = out_shape.iter().product();
    let mut out_data = vec![0.0; total_elements];

    let mut current_offset_in_dim = 0;
    for &v in &non_empty {
        let da = v.to_double_array().map_err(math_err)?;
        let v_shp = da.shape();
        let v_len_in_dim = if dim <= v_shp.len() {
            v_shp[dim - 1]
        } else {
            1
        };

        for linear_idx in 0..da.numel() {
            let coords = offset_to_coords(linear_idx, v_shp);
            let mut target_coords = vec![0; out_shape.len()];
            for (i, &c) in coords.iter().enumerate() {
                if i < target_coords.len() {
                    target_coords[i] = c;
                }
            }
            target_coords[dim - 1] += current_offset_in_dim;
            let out_idx = coords_to_offset(&target_coords, &out_strides);
            out_data[out_idx] = da.as_slice()[linear_idx];
        }

        current_offset_in_dim += v_len_in_dim;
    }

    let arr = DenseArray::from_vec(out_shape, out_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

/// `horzcat(A, B, ...)` horizontal concatenation (along dimension 2).
pub fn horzcat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let mut cat_args = Vec::with_capacity(args.len() + 1);
    cat_args.push(Value::scalar(2.0));
    cat_args.extend_from_slice(args);
    cat(&cat_args, 1)
}

/// `vertcat(A, B, ...)` vertical concatenation (along dimension 1).
pub fn vertcat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let mut cat_args = Vec::with_capacity(args.len() + 1);
    cat_args.push(Value::scalar(1.0));
    cat_args.extend_from_slice(args);
    cat(&cat_args, 1)
}

/// `permute(A, order)` rearranges array dimensions.
pub fn permute(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "permute requires 2 inputs."));
    }
    let da = extract_f64_array(&args[0])?;
    let order_arr = args[1].to_double_array().map_err(math_err)?;
    let order: Vec<usize> = order_arr
        .as_slice()
        .iter()
        .map(|&x| (x as usize).saturating_sub(1))
        .collect();

    let orig_shape = da.shape();
    let max_order = order.iter().copied().max().unwrap_or(0);
    let mut expanded_shape = orig_shape.to_vec();
    while expanded_shape.len() <= max_order {
        expanded_shape.push(1);
    }

    let mut new_shape = Vec::with_capacity(order.len());
    for &axis in &order {
        new_shape.push(expanded_shape.get(axis).copied().unwrap_or(1));
    }

    let norm_new_shape = normalize_shape(&new_shape);
    let new_strides = compute_column_major_strides(&norm_new_shape);
    let total_elements = da.numel();
    let mut new_data = vec![0.0; total_elements];

    for old_offset in 0..total_elements {
        let old_coords = offset_to_coords(old_offset, &expanded_shape);
        let mut new_coords = vec![0; norm_new_shape.len()];
        for (new_axis, &old_axis) in order.iter().enumerate() {
            if new_axis < new_coords.len() {
                new_coords[new_axis] = old_coords[old_axis];
            }
        }
        let new_offset = coords_to_offset(&new_coords, &new_strides);
        new_data[new_offset] = da.as_slice()[old_offset];
    }

    let arr = DenseArray::from_vec(norm_new_shape, new_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

/// `[X, Y] = meshgrid(x, y)` generates 2D / 3D coordinate grids.
pub fn meshgrid(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "meshgrid requires at least 1 input.",
        ));
    }
    let x_arr = args[0].to_double_array().map_err(math_err)?;
    let x_vec = x_arr.as_slice();
    let y_vec_owned: Vec<f64>;
    let y_vec = if args.len() >= 2 {
        y_vec_owned = args[1]
            .to_double_array()
            .map_err(math_err)?
            .as_slice()
            .to_vec();
        &y_vec_owned
    } else {
        x_vec
    };

    let nx = x_vec.len();
    let ny = y_vec.len();

    // X has shape [ny, nx]: rows repeat x
    let mut x_grid = vec![0.0; ny * nx];
    for j in 0..nx {
        for i in 0..ny {
            x_grid[i + j * ny] = x_vec[j];
        }
    }
    let val_x = Value::Double(DenseArray::from_vec(vec![ny, nx], x_grid).map_err(math_err)?);

    // Y has shape [ny, nx]: columns repeat y
    let mut y_grid = vec![0.0; ny * nx];
    for j in 0..nx {
        for i in 0..ny {
            y_grid[i + j * ny] = y_vec[i];
        }
    }
    let val_y = Value::Double(DenseArray::from_vec(vec![ny, nx], y_grid).map_err(math_err)?);

    if nargout <= 1 {
        Ok(vec![val_x])
    } else {
        Ok(vec![val_x, val_y])
    }
}

/// `[X1, X2, ...] = ndgrid(x1, x2, ...)` generates N-D coordinate grids.
pub fn ndgrid(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "ndgrid requires at least 1 input.",
        ));
    }

    let num_inputs = args.len();
    let mut vectors = Vec::with_capacity(num_inputs);
    for a in args {
        let da = a.to_double_array().map_err(math_err)?;
        vectors.push(da.as_slice().to_vec());
    }

    let mut shape: Vec<usize> = vectors.iter().map(|v| v.len()).collect();
    if shape.len() == 1 {
        shape.push(1);
    }
    let total_elements: usize = shape.iter().product();

    let count = nargout.max(1);
    let mut outputs = Vec::with_capacity(count);

    for k in 0..count {
        let mut grid = vec![0.0; total_elements];
        if k < vectors.len() {
            let ref_vec = &vectors[k];
            for offset in 0..total_elements {
                let coords = offset_to_coords(offset, &shape);
                grid[offset] = ref_vec[coords[k]];
            }
        }
        let arr = DenseArray::from_vec(shape.clone(), grid).map_err(math_err)?;
        outputs.push(Value::Double(arr));
    }

    Ok(outputs)
}

/// Enhanced `reshape(A, d1, d2, ...)` supporting `[]` (empty) for at most one dimension.
pub fn reshape(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "reshape requires array and dimensions.",
        ));
    }
    let in_val = &args[0];
    let total_numel = in_val.numel();

    // Parse requested dimensions with possible []
    let dim_args = &args[1..];
    let mut requested_dims = Vec::new();
    let mut empty_dim_idx = None;

    if dim_args.len() == 1 && !dim_args[0].is_empty() && !dim_args[0].is_scalar() {
        let d_arr = dim_args[0].to_double_array().map_err(math_err)?;
        for &d in d_arr.as_slice() {
            if d < 0.0 {
                return Err(user_err(
                    "MATLAB:checkdim:badSizeArg",
                    "Size arguments must be integer scalars or [].",
                ));
            }
            requested_dims.push(d as usize);
        }
    } else {
        for (idx, a) in dim_args.iter().enumerate() {
            if a.is_empty() {
                if empty_dim_idx.is_some() {
                    return Err(user_err(
                        "MATLAB:getReshapeDims:multipleNegative",
                        "Only one dimension can be unknown ([]).",
                    ));
                }
                empty_dim_idx = Some(idx);
                requested_dims.push(0);
            } else {
                let d = a.as_scalar_f64().map_err(math_err)?;
                if d < 0.0 {
                    return Err(user_err(
                        "MATLAB:checkdim:badSizeArg",
                        "Size arguments must be integer scalars or [].",
                    ));
                }
                requested_dims.push(d as usize);
            }
        }
    }

    if requested_dims.len() == 1 {
        requested_dims.push(1);
    }

    if let Some(idx) = empty_dim_idx {
        let known_product: usize = requested_dims
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, &d)| d)
            .product();
        if known_product == 0 || !total_numel.is_multiple_of(known_product) {
            return Err(user_err(
                "MATLAB:getReshapeDims:notSameNumel",
                "Dimensions must evaluate to the same number of elements.",
            ));
        }
        requested_dims[idx] = total_numel / known_product;
    }

    let target_numel: usize = requested_dims.iter().product();
    if target_numel != total_numel {
        return Err(user_err(
            "MATLAB:getReshapeDims:notSameNumel",
            "To RESHAPE the number of elements must not change.",
        ));
    }

    macro_rules! reshape_variant {
        ($arr:expr, $variant:ident) => {{
            let out =
                DenseArray::from_vec(requested_dims, $arr.as_slice().to_vec()).map_err(math_err)?;
            Ok(vec![Value::$variant(out)])
        }};
    }

    match in_val {
        Value::Double(da) => reshape_variant!(da, Double),
        Value::Single(da) => reshape_variant!(da, Single),
        Value::Int8(da) => reshape_variant!(da, Int8),
        Value::Int16(da) => reshape_variant!(da, Int16),
        Value::Int32(da) => reshape_variant!(da, Int32),
        Value::Int64(da) => reshape_variant!(da, Int64),
        Value::UInt8(da) => reshape_variant!(da, UInt8),
        Value::UInt16(da) => reshape_variant!(da, UInt16),
        Value::UInt32(da) => reshape_variant!(da, UInt32),
        Value::UInt64(da) => reshape_variant!(da, UInt64),
        Value::Logical(da) => reshape_variant!(da, Logical),
        Value::Char(da) => reshape_variant!(da, Char),
        Value::Cell(da) => reshape_variant!(da, Cell),
        other => {
            let da = other.to_double_array().map_err(math_err)?;
            reshape_variant!(da, Double)
        }
    }
}

/// Enhanced `repmat(A, r1, r2, ...)` supporting arbitrary N-D repetition.
pub fn repmat(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "repmat requires at least 1 input.",
        ));
    }
    let da = extract_f64_array(&args[0])?;
    let reps = crate::helpers::parse_dims(&args[1..])?;

    let in_shp = da.shape();
    let max_rank = in_shp.len().max(reps.len());
    let mut exp_in_shp = in_shp.to_vec();
    while exp_in_shp.len() < max_rank {
        exp_in_shp.push(1);
    }
    let mut exp_reps = reps.clone();
    while exp_reps.len() < max_rank {
        exp_reps.push(1);
    }

    let out_shp: Vec<usize> = exp_in_shp
        .iter()
        .zip(exp_reps.iter())
        .map(|(&d, &r)| d * r)
        .collect();

    let total_out: usize = out_shp.iter().product();
    let mut out_data = vec![0.0; total_out];

    for out_offset in 0..total_out {
        let out_coords = offset_to_coords(out_offset, &out_shp);
        let mut in_coords = vec![0; exp_in_shp.len()];
        for (i, &c) in out_coords.iter().enumerate() {
            in_coords[i] = c % exp_in_shp[i];
        }
        let in_offset = coords_to_offset(&in_coords, da.strides());
        out_data[out_offset] = da.as_slice()[in_offset];
    }

    let arr = DenseArray::from_vec(out_shp, out_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}

/// `ipermute(A, order)` inverse of permute.
pub fn ipermute(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "ipermute requires 2 inputs."));
    }
    let order_arr = args[1].to_double_array().map_err(math_err)?;
    let order_slice = order_arr.as_slice();
    let n = order_slice.len();
    let mut inv_order = vec![0.0; n];
    for (i, &p) in order_slice.iter().enumerate() {
        let idx = p as usize;
        if (1..=n).contains(&idx) {
            inv_order[idx - 1] = (i + 1) as f64;
        } else {
            return Err(user_err(
                "MATLAB:ipermute:badOrder",
                "Invalid permutation order.",
            ));
        }
    }
    let inv_order_val = Value::Double(DenseArray::row_vector(inv_order));
    permute(&[args[0].clone(), inv_order_val], nargout)
}

/// `circshift(A, k)` shifts array elements circularly.
pub fn circshift(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "circshift requires array and shift vector.",
        ));
    }
    let da = extract_f64_array(&args[0])?;
    let shifts_arr = args[1].to_double_array().map_err(math_err)?;
    let shifts = shifts_arr.as_slice();
    let shape = da.shape();
    let total_elements = da.numel();
    if total_elements == 0 {
        return Ok(vec![args[0].clone()]);
    }
    let mut out_data = vec![0.0; total_elements];
    let strides = da.strides();

    for offset in 0..total_elements {
        let coords = offset_to_coords(offset, shape);
        let mut new_coords = coords.clone();
        for (i, &dim_len) in shape.iter().enumerate() {
            if i < shifts.len() && dim_len > 0 {
                let shift = shifts[i] as isize;
                let c = coords[i] as isize;
                let rem = (c + shift).rem_euclid(dim_len as isize);
                new_coords[i] = rem as usize;
            }
        }
        let new_offset = coords_to_offset(&new_coords, strides);
        out_data[new_offset] = da.as_slice()[offset];
    }
    let arr = DenseArray::from_vec(shape.to_vec(), out_data).map_err(math_err)?;
    Ok(vec![Value::Double(arr)])
}
