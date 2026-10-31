//! Signal processing builtins: digital filtering, window functions, correlation.

use crate::helpers::user_err;
use crate::signal::conv_fn;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};
use std::f64::consts::PI;

fn extract_f64_slice(val: &Value) -> Result<Vec<f64>, EvalError> {
    match val {
        Value::Double(d) => Ok(d.as_slice().to_vec()),
        Value::Single(s) => Ok(s.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int32(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        Value::UInt32(u) => Ok(u.as_slice().iter().map(|&x| x as f64).collect()),
        _ => Err(user_err("RUSTLAB:invalidType", "Expected numeric vector.")),
    }
}

/// Digital filter implementation (Direct Form II Transposed): y = filter(b, a, x).
pub fn filter_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "filter requires at least 3 inputs (b, a, x).",
        ));
    }
    let b = extract_f64_slice(&args[0])?;
    let a = extract_f64_slice(&args[1])?;
    let x = extract_f64_slice(&args[2])?;

    if b.is_empty() || a.is_empty() {
        return Err(user_err(
            "RUSTLAB:filter:emptyCoeffs",
            "Filter coefficients b and a must not be empty.",
        ));
    }
    let a0 = a[0];
    if a0.abs() < 1e-15 {
        return Err(user_err(
            "RUSTLAB:filter:zeroFirstDenCoeff",
            "First denominator coefficient a(1) cannot be zero.",
        ));
    }

    // Normalize b and a by a[0]
    let b_norm: Vec<f64> = b.iter().map(|&v| v / a0).collect();
    let a_norm: Vec<f64> = a.iter().map(|&v| v / a0).collect();

    let m = b_norm.len().max(a_norm.len());
    let mut b_pad = vec![0.0; m];
    let mut a_pad = vec![0.0; m];
    for (i, &v) in b_norm.iter().enumerate() {
        b_pad[i] = v;
    }
    for (i, &v) in a_norm.iter().enumerate() {
        a_pad[i] = v;
    }

    let n = x.len();
    let mut y = Vec::with_capacity(n);
    let mut state = vec![0.0; m.saturating_sub(1)];

    for &xi in &x {
        let yi = b_pad[0] * xi + state.first().copied().unwrap_or(0.0);
        y.push(yi);

        let state_len = state.len();
        for k in 0..state_len {
            let next_st = if k + 1 < state_len { state[k + 1] } else { 0.0 };
            state[k] = b_pad[k + 1] * xi + next_st - a_pad[k + 1] * yi;
        }
    }

    let res_da = DenseArray::from_vec_unchecked(vec![1, n], y);
    Ok(vec![Value::Double(res_da)])
}

/// Zero-phase digital filtering: y = filtfilt(b, a, x).
pub fn filtfilt_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "filtfilt requires at least 3 inputs (b, a, x).",
        ));
    }
    // Forward pass
    let mut forward_outs = filter_fn(args, nargout)?;
    let y1 = forward_outs.remove(0);
    let y1_slice = extract_f64_slice(&y1)?;

    // Reverse
    let mut y1_rev = y1_slice;
    y1_rev.reverse();
    let y1_rev_val = Value::Double(DenseArray::from_vec_unchecked(
        vec![1, y1_rev.len()],
        y1_rev,
    ));

    // Backward pass
    let backward_args = vec![args[0].clone(), args[1].clone(), y1_rev_val];
    let mut backward_outs = filter_fn(&backward_args, nargout)?;
    let y2 = backward_outs.remove(0);
    let mut y2_slice = extract_f64_slice(&y2)?;
    y2_slice.reverse();

    let res_da = DenseArray::from_vec_unchecked(vec![1, y2_slice.len()], y2_slice);
    Ok(vec![Value::Double(res_da)])
}

/// Hamming window: w = hamming(N).
pub fn hamming_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "hamming requires 1 input."));
    }
    let n = args[0].as_scalar_f64().map_err(|_| {
        user_err(
            "RUSTLAB:hamming:invalidN",
            "N must be a positive integer scalar.",
        )
    })? as usize;
    if n == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if n == 1 {
        return Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![1.0],
        ))]);
    }

    let mut w = Vec::with_capacity(n);
    let denom = (n - 1) as f64;
    for i in 0..n {
        let val = 0.54 - 0.46 * (2.0 * PI * i as f64 / denom).cos();
        w.push(val);
    }
    let da = DenseArray::from_vec_unchecked(vec![n, 1], w);
    Ok(vec![Value::Double(da)])
}

/// Hann window: w = hann(N).
pub fn hann_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "hann requires 1 input."));
    }
    let n = args[0].as_scalar_f64().map_err(|_| {
        user_err(
            "RUSTLAB:hann:invalidN",
            "N must be a positive integer scalar.",
        )
    })? as usize;
    if n == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if n == 1 {
        return Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![1.0],
        ))]);
    }

    let mut w = Vec::with_capacity(n);
    let denom = (n - 1) as f64;
    for i in 0..n {
        let val = 0.5 * (1.0 - (2.0 * PI * i as f64 / denom).cos());
        w.push(val);
    }
    let da = DenseArray::from_vec_unchecked(vec![n, 1], w);
    Ok(vec![Value::Double(da)])
}

/// Blackman window: w = blackman(N).
pub fn blackman_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "blackman requires 1 input."));
    }
    let n = args[0].as_scalar_f64().map_err(|_| {
        user_err(
            "RUSTLAB:blackman:invalidN",
            "N must be a positive integer scalar.",
        )
    })? as usize;
    if n == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if n == 1 {
        return Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![1.0],
        ))]);
    }

    let mut w = Vec::with_capacity(n);
    let denom = (n - 1) as f64;
    for i in 0..n {
        let th = 2.0 * PI * i as f64 / denom;
        let val = 0.42 - 0.5 * th.cos() + 0.08 * (2.0 * th).cos();
        w.push(val);
    }
    let da = DenseArray::from_vec_unchecked(vec![n, 1], w);
    Ok(vec![Value::Double(da)])
}

/// Cross-correlation: c = xcorr(x, y) or xcorr(x).
pub fn xcorr_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "xcorr requires at least 1 input.",
        ));
    }
    let (x, y) = if args.len() >= 2 {
        (args[0].clone(), args[1].clone())
    } else {
        (args[0].clone(), args[0].clone())
    };

    let mut y_slice = extract_f64_slice(&y)?;
    y_slice.reverse();
    let y_rev = Value::Double(DenseArray::from_vec_unchecked(
        vec![1, y_slice.len()],
        y_slice,
    ));

    conv_fn(&[x, y_rev], nargout)
}
