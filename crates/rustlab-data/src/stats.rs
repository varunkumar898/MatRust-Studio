//! B2.1 statistics builtins.
//!
//! Clean-room implementations over the column-major `DenseArray<f64>`:
//! median, mode, prctile/quantile, cov, corrcoef, histcounts, movmean,
//! movmedian, cummax, cummin, zscore, sortrows, and unique with ia/ic.
//!
//! Percentile interpolation follows the linear rule documented in the B2
//! design note (`h = (N-1)q + 1`); the exact MathWorks method is marked
//! unverified there. All reductions default to the first non-singleton
//! dimension and accept an explicit `dim`.

use crate::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{coords_to_offset, normalize_shape, offset_to_coords, DenseArray, Value};

fn get_da(v: &Value) -> Result<DenseArray<f64>, EvalError> {
    v.to_double_array()
        .map_err(|e| user_err("RUSTLAB:data:badInput", &format!("{e}")))
}

fn as_string(v: &Value) -> Option<String> {
    match v {
        Value::Char(ca) => Some(ca.as_slice().iter().collect()),
        Value::String(sa) => Some(sa.as_slice().first().cloned().unwrap_or_default()),
        _ => None,
    }
}

fn parse_dim_value(v: &Value) -> Result<usize, EvalError> {
    let x = v.as_scalar_f64().map_err(|e| {
        user_err(
            "RUSTLAB:data:badDim",
            &format!("Dimension argument must be a positive integer scalar: {e}"),
        )
    })?;
    if !x.is_finite() || x < 1.0 || x.fract() != 0.0 {
        return Err(user_err(
            "RUSTLAB:data:badDim",
            "Dimension argument must be a positive integer scalar.",
        ));
    }
    Ok(x as usize)
}

fn first_non_singleton_dim(shape: &[usize]) -> usize {
    for (i, &s) in shape.iter().enumerate() {
        if s > 1 {
            return i + 1;
        }
    }
    1
}

fn dim_and_omit(
    args: &[Value],
    start: usize,
    default_dim: usize,
) -> Result<(usize, bool), EvalError> {
    let mut dim = default_dim;
    let mut omit = false;
    for v in &args[start..] {
        if let Some(s) = as_string(v) {
            if s.eq_ignore_ascii_case("omitnan") {
                omit = true;
            } else {
                return Err(user_err(
                    "RUSTLAB:data:badOption",
                    &format!("Unsupported option '{s}'."),
                ));
            }
        } else {
            dim = parse_dim_value(v)?;
        }
    }
    Ok((dim, omit))
}

fn reduce_f64<F>(da: &DenseArray<f64>, dim: usize, mut op: F) -> DenseArray<f64>
where
    F: FnMut(&[f64]) -> f64,
{
    let in_shape = da.shape();
    let max_rank = in_shape.len().max(dim);
    let mut exp_shape = in_shape.to_vec();
    while exp_shape.len() < max_rank {
        exp_shape.push(1);
    }
    let mut out_shape = exp_shape.clone();
    out_shape[dim - 1] = 1;
    let norm_out = normalize_shape(&out_shape);
    let total_out: usize = norm_out.iter().product();
    let fiber_len = exp_shape[dim - 1];
    let mut out_data = Vec::with_capacity(total_out);
    let mut fiber = Vec::with_capacity(fiber_len);

    for out_offset in 0..total_out {
        let out_coords = offset_to_coords(out_offset, &norm_out);
        fiber.clear();
        for k in 0..fiber_len {
            let mut coords = vec![0usize; exp_shape.len()];
            for (i, &c) in out_coords.iter().enumerate() {
                if i < coords.len() {
                    coords[i] = c;
                }
            }
            coords[dim - 1] = k;
            let off = coords_to_offset(&coords, da.strides());
            fiber.push(da.as_slice()[off]);
        }
        out_data.push(op(&fiber));
    }
    DenseArray::from_vec_unchecked(norm_out, out_data)
}

fn map_fibers<F>(da: &DenseArray<f64>, dim: usize, mut op: F) -> DenseArray<f64>
where
    F: FnMut(&[f64]) -> Vec<f64>,
{
    let in_shape = da.shape();
    // Fast path: 1D contiguous column or row vector along its active dimension
    if in_shape.len() == 2 && ((in_shape[1] == 1 && dim == 1) || (in_shape[0] == 1 && dim == 2)) {
        let out = op(da.as_slice());
        return DenseArray::from_vec_unchecked(in_shape.to_vec(), out);
    }
    let max_rank = in_shape.len().max(dim);
    let mut exp_shape = in_shape.to_vec();
    while exp_shape.len() < max_rank {
        exp_shape.push(1);
    }
    let fiber_len = exp_shape[dim - 1];
    let mut base_shape = exp_shape.clone();
    base_shape[dim - 1] = 1;
    let total_fibers: usize = base_shape.iter().product();
    let mut out = da.as_slice().to_vec();

    for f_idx in 0..total_fibers {
        let base_coords = offset_to_coords(f_idx, &base_shape);
        let mut offsets = Vec::with_capacity(fiber_len);
        let mut vals = Vec::with_capacity(fiber_len);
        for k in 0..fiber_len {
            let mut coords = base_coords.clone();
            coords[dim - 1] = k;
            let off = coords_to_offset(&coords, da.strides());
            offsets.push(off);
            vals.push(da.as_slice()[off]);
        }
        let res = op(&vals);
        for (k, &off) in offsets.iter().enumerate() {
            out[off] = res.get(k).copied().unwrap_or(f64::NAN);
        }
    }
    DenseArray::from_vec_unchecked(da.shape().to_vec(), out)
}

fn median_slice(slice: &[f64], omitnan: bool) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if !omitnan && slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    let mut v: Vec<f64> = slice.iter().copied().filter(|x| !x.is_nan()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn mode_slice(slice: &[f64], omitnan: bool) -> f64 {
    if slice.is_empty() {
        return f64::NAN;
    }
    if !omitnan && slice.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    let mut v: Vec<f64> = slice.iter().copied().filter(|x| !x.is_nan()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let mut best = v[0];
    let mut best_count = 0usize;
    let mut i = 0usize;
    while i < v.len() {
        let mut j = i + 1;
        while j < v.len() && (v[j] == v[i] || (v[j].is_nan() && v[i].is_nan())) {
            j += 1;
        }
        let count = j - i;
        if count > best_count || (count == best_count && v[i] < best) {
            best = v[i];
            best_count = count;
        }
        i = j;
    }
    best
}

fn percentile_interp(sorted: &[f64], q: f64, method: Option<&str>) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    // MATLAB rule (B2.1.1): piecewise linear through (i - 0.5)/n, clamped to ends.
    // That is, position pos = q * n + 0.5.
    // Optional 'method' can select 'numpy' (Method 7: pos = (n - 1) * q + 1).
    let is_numpy = method.is_some_and(|m| m.eq_ignore_ascii_case("numpy"));
    let pos = if is_numpy {
        (n - 1) as f64 * q + 1.0
    } else {
        q * n as f64 + 0.5
    };

    if pos <= 1.0 {
        return sorted[0];
    }
    if pos >= n as f64 {
        return sorted[n - 1];
    }
    let lo = pos.floor() as usize;
    let frac = pos - lo as f64;
    sorted[lo - 1] + frac * (sorted[lo] - sorted[lo - 1])
}

/// `median(X [, dim] [, 'omitnan'])`
pub fn median(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let first = args
        .first()
        .ok_or_else(|| user_err("MATLAB:minrhs", "median requires at least 1 input."))?;
    let da = get_da(first)?;
    if da.is_empty() {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, omit) = dim_and_omit(args, 1, default_dim)?;
    let res = reduce_f64(&da, dim, |slice| median_slice(slice, omit));
    Ok(vec![Value::Double(res)])
}

/// `mode(X [, dim] [, 'omitnan'])` — smallest value on ties.
pub fn mode(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let first = args
        .first()
        .ok_or_else(|| user_err("MATLAB:minrhs", "mode requires at least 1 input."))?;
    let da = get_da(first)?;
    if da.is_empty() {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, omit) = dim_and_omit(args, 1, default_dim)?;
    let res = reduce_f64(&da, dim, |slice| mode_slice(slice, omit));
    Ok(vec![Value::Double(res)])
}

fn percentile_impl(args: &[Value], divide_by_100: bool) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "prctile/quantile requires at least 2 inputs.",
        ));
    }
    let x = get_da(&args[0])?;
    if x.is_empty() {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }
    let p = get_da(&args[1])?;

    // Parse optional method parameter ('method', 'numpy'/'matlab' or 'numpy'/'matlab')
    let mut method: Option<String> = None;
    let mut arg_idx = 2;
    while arg_idx < args.len() {
        if let Some(s) = as_string(&args[arg_idx]) {
            if s.eq_ignore_ascii_case("method") && arg_idx + 1 < args.len() {
                if let Some(m) = as_string(&args[arg_idx + 1]) {
                    method = Some(m);
                    arg_idx += 2;
                    continue;
                }
            } else if s.eq_ignore_ascii_case("numpy") || s.eq_ignore_ascii_case("matlab") {
                method = Some(s);
            }
        }
        arg_idx += 1;
    }

    let mut vals: Vec<f64> = x.as_slice().to_vec();
    if vals.iter().any(|v| v.is_nan()) {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }
    vals.sort_by(f64::total_cmp);
    let mut out = Vec::with_capacity(p.numel());
    for &pv in p.as_slice() {
        if !pv.is_finite() {
            return Err(user_err(
                "RUSTLAB:data:badInput",
                "Percentile/quantile values must be finite.",
            ));
        }
        let q = if divide_by_100 {
            if !(0.0..=100.0).contains(&pv) {
                return Err(user_err(
                    "RUSTLAB:data:badInput",
                    "Percentile values must be in [0, 100].",
                ));
            }
            pv / 100.0
        } else {
            if !(0.0..=1.0).contains(&pv) {
                return Err(user_err(
                    "RUSTLAB:data:badInput",
                    "Quantile values must be in [0, 1].",
                ));
            }
            pv
        };
        out.push(percentile_interp(&vals, q, method.as_deref()));
    }
    if p.is_scalar() {
        Ok(vec![Value::scalar(out[0])])
    } else {
        Ok(vec![Value::Double(DenseArray::row_vector(out))])
    }
}

/// `prctile(X, p)` with linear interpolation.
pub fn prctile(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    percentile_impl(args, true)
}

/// `quantile(X, q)` with linear interpolation.
pub fn quantile(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    percentile_impl(args, false)
}

fn variance_of(vals: &[f64], flag: usize, omitnan: bool) -> f64 {
    let filtered: Vec<f64> = if omitnan {
        vals.iter().copied().filter(|x| !x.is_nan()).collect()
    } else if vals.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    } else {
        vals.to_vec()
    };
    let n = filtered.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return 0.0;
    }
    let mean = filtered.iter().sum::<f64>() / n as f64;
    let ss: f64 = filtered.iter().map(|x| (x - mean) * (x - mean)).sum();
    let denom = if flag == 1 { n as f64 } else { (n - 1) as f64 };
    ss / denom
}

/// Covariance matrix (columns are variables) or scalar for vector inputs.
fn covariance(da: &DenseArray<f64>) -> Result<Value, EvalError> {
    let shape = da.shape();
    if shape.len() != 2 {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "cov requires a 2-D numeric array.",
        ));
    }
    let (m, n) = (shape[0], shape[1]);
    if m == 0 || n == 0 {
        return Ok(Value::Double(DenseArray::empty()));
    }
    if m == 1 || n == 1 {
        let vals = da.as_slice();
        let n = vals.len();
        if n <= 1 {
            return Ok(Value::scalar(f64::NAN));
        }
        let mean = vals.iter().sum::<f64>() / n as f64;
        let ss: f64 = vals.iter().map(|x| (x - mean) * (x - mean)).sum();
        return Ok(Value::scalar(ss / (n - 1) as f64));
    }
    let mut means = vec![0.0; n];
    for (c, mean_val) in means.iter_mut().enumerate() {
        let mut s = 0.0;
        for r in 0..m {
            s += da.as_slice()[r + c * m];
        }
        *mean_val = s / m as f64;
    }
    let mut cov = vec![0.0; n * n];
    for i in 0..n {
        for j in i..n {
            let mut s = 0.0;
            for r in 0..m {
                let a = da.as_slice()[r + i * m] - means[i];
                let b = da.as_slice()[r + j * m] - means[j];
                s += a * b;
            }
            let val = s / (m - 1) as f64;
            cov[i + j * n] = val;
            cov[j + i * n] = val;
        }
    }
    Ok(Value::Double(DenseArray::from_vec_unchecked(
        vec![n, n],
        cov,
    )))
}

/// `cov(X)` / `cov(X, Y)`
pub fn cov(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "cov requires at least 1 input."));
    }
    if args.len() >= 2 {
        let x = get_da(&args[0])?;
        let y = get_da(&args[1])?;
        if x.numel() != y.numel() {
            return Err(user_err(
                "RUSTLAB:data:badInput",
                "cov(X,Y) requires X and Y to have the same number of elements.",
            ));
        }
        let n = x.numel();
        if n <= 1 {
            return Ok(vec![Value::scalar(f64::NAN)]);
        }
        let mx = x.as_slice().iter().sum::<f64>() / n as f64;
        let my = y.as_slice().iter().sum::<f64>() / n as f64;
        let mut s = 0.0;
        for i in 0..n {
            s += (x.as_slice()[i] - mx) * (y.as_slice()[i] - my);
        }
        return Ok(vec![Value::scalar(s / (n - 1) as f64)]);
    }
    Ok(vec![covariance(&get_da(&args[0])?)?])
}

/// `corrcoef(X)` / `corrcoef(X, Y)`
pub fn corrcoef(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "corrcoef requires at least 1 input.",
        ));
    }
    if args.len() >= 2 {
        let c = cov(args, 1)?;
        let v = c[0].as_scalar_f64().unwrap_or(f64::NAN);
        return Ok(vec![Value::scalar(if v > 0.0 { 1.0 } else { f64::NAN })]);
    }
    let da = get_da(&args[0])?;
    let shape = da.shape();
    if shape.len() != 2 || shape[0] == 0 || shape[1] == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if shape[0] == 1 || shape[1] == 1 {
        let vals = da.as_slice();
        let n = vals.len();
        if n <= 1 {
            return Ok(vec![Value::scalar(f64::NAN)]);
        }
        let mean = vals.iter().sum::<f64>() / n as f64;
        let ss: f64 = vals.iter().map(|x| (x - mean) * (x - mean)).sum();
        return Ok(vec![Value::scalar(if ss > 0.0 { 1.0 } else { f64::NAN })]);
    }
    let cov_val = covariance(&da)?;
    let cov_arr = match cov_val {
        Value::Double(a) => a,
        _ => {
            return Err(user_err(
                "RUSTLAB:data:badInput",
                "cov returned non-numeric",
            ))
        }
    };
    let n = shape[1];
    let mut r = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                r[i + j * n] = 1.0;
            } else {
                let di = cov_arr.as_slice()[i + i * n];
                let dj = cov_arr.as_slice()[j + j * n];
                let denom = (di * dj).sqrt();
                r[i + j * n] = if denom > 0.0 {
                    cov_arr.as_slice()[i + j * n] / denom
                } else {
                    f64::NAN
                };
            }
        }
    }
    Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
        vec![n, n],
        r,
    ))])
}

/// `histcounts(X [, edges])`
pub fn histcounts(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "histcounts requires at least 1 input.",
        ));
    }
    let x = get_da(&args[0])?;
    let vals: Vec<f64> = x
        .as_slice()
        .iter()
        .copied()
        .filter(|v| !v.is_nan())
        .collect();
    let edges: Vec<f64> = if args.len() >= 2 {
        let e = get_da(&args[1])?;
        e.as_slice().to_vec()
    } else {
        if vals.is_empty() {
            return Ok(vec![Value::Double(DenseArray::row_vector(vec![0.0]))]);
        }
        let min = vals.iter().copied().fold(f64::INFINITY, f64::min);
        let max = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bins = (vals.len() as f64).sqrt().ceil().max(1.0) as usize;
        if min == max {
            vec![min - 0.5, max + 0.5]
        } else {
            let step = (max - min) / bins as f64;
            (0..=bins).map(|i| min + step * i as f64).collect()
        }
    };
    if edges.len() < 2 {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "histcounts edges must contain at least two values.",
        ));
    }
    if edges
        .windows(2)
        .any(|w| w[0] >= w[1] || w[0].is_nan() || w[1].is_nan())
    {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "histcounts edges must be strictly increasing.",
        ));
    }
    let mut counts = vec![0.0; edges.len() - 1];
    let last = edges.len() - 2;
    for &v in &vals {
        if v < edges[0] || v > *edges.last().unwrap_or(&edges[0]) {
            continue;
        }
        for j in 0..edges.len() - 1 {
            let in_bin = if j == last {
                v >= edges[j] && v <= edges[j + 1]
            } else {
                v >= edges[j] && v < edges[j + 1]
            };
            if in_bin {
                counts[j] += 1.0;
                break;
            }
        }
    }
    Ok(vec![Value::Double(DenseArray::row_vector(counts))])
}

fn parse_window(v: &Value) -> Result<usize, EvalError> {
    let x = v.as_scalar_f64().map_err(|e| {
        user_err(
            "RUSTLAB:data:badInput",
            &format!("Window length must be a positive integer scalar: {e}"),
        )
    })?;
    if !x.is_finite() || x < 1.0 || x.fract() != 0.0 {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "Window length must be a positive integer scalar.",
        ));
    }
    Ok(x as usize)
}

fn window_bounds(i: usize, len: usize, k: usize) -> (usize, usize) {
    let left = (k - 1) / 2;
    let right = k.saturating_sub(1) - left;
    (i.saturating_sub(left), (i + right).min(len - 1))
}

#[inline]
fn compute_window(w: &[f64], omit: bool) -> f64 {
    if omit {
        let mut sum = 0.0f64;
        let mut count = 0usize;
        for &x in w {
            if !x.is_nan() {
                sum += x;
                count += 1;
            }
        }
        if count == 0 {
            f64::NAN
        } else {
            sum / count as f64
        }
    } else {
        let mut sum = 0.0f64;
        for &x in w {
            if x.is_nan() {
                return f64::NAN;
            }
            sum += x;
        }
        sum / w.len() as f64
    }
}

fn movmean_slice(vals: &[f64], k: usize, omit: bool) -> Vec<f64> {
    let len = vals.len();
    if len == 0 {
        return Vec::new();
    }
    if k <= 1 {
        if omit {
            return vals
                .iter()
                .map(|&x| if x.is_nan() { f64::NAN } else { x })
                .collect();
        } else {
            return vals.to_vec();
        }
    }

    let left = (k - 1) / 2;
    let right = k.saturating_sub(1) - left;
    let mut out = vec![0.0f64; len];

    if len < k {
        for (i, slot) in out.iter_mut().enumerate().take(len) {
            let a = i.saturating_sub(left);
            let b = (i + right).min(len - 1);
            *slot = compute_window(&vals[a..=b], omit);
        }
        return out;
    }

    // 1. Left boundary: i < left
    for (i, slot) in out.iter_mut().enumerate().take(left) {
        let b = (i + right).min(len - 1);
        *slot = compute_window(&vals[0..=b], omit);
    }

    // 2. Steady-state interior: left <= i && i + right < len
    let inv_k = 1.0 / (k as f64);
    let mut running_sum: f64 = 0.0;
    let mut window_finite = true;

    // Initialize running sum for window at i = left (vals[0..k])
    for &x in &vals[0..k] {
        if !x.is_finite() {
            window_finite = false;
        }
        running_sum += x;
    }

    if window_finite {
        out[left] = running_sum * inv_k;
    } else {
        out[left] = compute_window(&vals[0..k], omit);
    }

    let steady_end = len - right;
    for i in (left + 1)..steady_end {
        let val_out = vals[i - left - 1];
        let val_in = vals[i + right];

        if window_finite && val_in.is_finite() && val_out.is_finite() {
            // Periodic exact recompute every 1024 steps to prevent floating point drift
            if (i - left) & 1023 == 0 {
                let mut sum = 0.0f64;
                for &x in &vals[(i - left)..=(i + right)] {
                    sum += x;
                }
                running_sum = sum;
            } else {
                running_sum += val_in - val_out;
            }
            out[i] = running_sum * inv_k;
        } else {
            // Window contains or touches non-finite value (NaN or Inf):
            // Fall back to exact per-window compute
            out[i] = compute_window(&vals[(i - left)..=(i + right)], omit);

            // Check if window has returned to fully finite
            let mut all_finite = true;
            let mut sum = 0.0f64;
            for &x in &vals[(i - left)..=(i + right)] {
                if !x.is_finite() {
                    all_finite = false;
                    break;
                }
                sum += x;
            }
            if all_finite {
                running_sum = sum;
                window_finite = true;
            } else {
                window_finite = false;
            }
        }
    }

    // 3. Right boundary: i >= steady_end
    for (i, slot) in out.iter_mut().enumerate().take(len).skip(steady_end) {
        let a = i.saturating_sub(left);
        *slot = compute_window(&vals[a..len], omit);
    }

    out
}

/// `movmean(X, k)`
pub fn movmean(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "movmean requires at least 2 inputs.",
        ));
    }
    let da = get_da(&args[0])?;
    let k = parse_window(&args[1])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(da)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, omit) = dim_and_omit(args, 2, default_dim)?;
    let res = map_fibers(&da, dim, |vals| movmean_slice(vals, k, omit));
    Ok(vec![Value::Double(res)])
}

/// `movmedian(X, k)`
pub fn movmedian(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "movmedian requires at least 2 inputs.",
        ));
    }
    let da = get_da(&args[0])?;
    let k = parse_window(&args[1])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(da)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, omit) = dim_and_omit(args, 2, default_dim)?;
    let res = map_fibers(&da, dim, |vals| {
        let len = vals.len();
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let (a, b) = window_bounds(i, len, k);
            out.push(median_slice(&vals[a..=b], omit));
        }
        out
    });
    Ok(vec![Value::Double(res)])
}

fn cumulative_slice(vals: &[f64], want_max: bool) -> Vec<f64> {
    let len = vals.len();
    if len == 0 {
        return Vec::new();
    }
    let mut out = vec![0.0f64; len];
    let mut current = vals[0];
    if current.is_nan() {
        out.fill(f64::NAN);
        return out;
    }
    out[0] = current;
    let mut idx = 1;

    if want_max {
        while idx < len {
            let x = vals[idx];
            if x.is_nan() {
                break;
            }
            if x > current {
                current = x;
            }
            out[idx] = current;
            idx += 1;
        }
    } else {
        while idx < len {
            let x = vals[idx];
            if x.is_nan() {
                break;
            }
            if x < current {
                current = x;
            }
            out[idx] = current;
            idx += 1;
        }
    }

    while idx < len {
        out[idx] = f64::NAN;
        idx += 1;
    }

    out
}

fn cumulative(args: &[Value], want_max: bool) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "cummax/cummin requires at least 1 input.",
        ));
    }
    let da = get_da(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(da)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, _omit) = dim_and_omit(args, 1, default_dim)?;
    let res = map_fibers(&da, dim, |vals| cumulative_slice(vals, want_max));
    Ok(vec![Value::Double(res)])
}

/// `cummax(X [, dim])`
pub fn cummax(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    cumulative(args, true)
}

/// `cummin(X [, dim])`
pub fn cummin(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    cumulative(args, false)
}

fn zscore_slice(vals: &[f64]) -> Vec<f64> {
    let n = vals.len();
    if n <= 1 {
        return vec![f64::NAN; n];
    }

    // Pass 1: compute mean with 4 accumulators for instruction-level parallelism
    let (chunks, rem) = vals.as_chunks::<4>();
    let mut s0 = 0.0f64;
    let mut s1 = 0.0f64;
    let mut s2 = 0.0f64;
    let mut s3 = 0.0f64;
    for c in chunks {
        s0 += c[0];
        s1 += c[1];
        s2 += c[2];
        s3 += c[3];
    }
    let mut sum = (s0 + s1) + (s2 + s3);
    for &x in rem {
        sum += x;
    }
    let mean = sum / n as f64;

    // Pass 2: compute sum of squared differences from mean (two-pass)
    let (chunks, rem) = vals.as_chunks::<4>();
    let mut ss0 = 0.0f64;
    let mut ss1 = 0.0f64;
    let mut ss2 = 0.0f64;
    let mut ss3 = 0.0f64;
    for c in chunks {
        let d0 = c[0] - mean;
        let d1 = c[1] - mean;
        let d2 = c[2] - mean;
        let d3 = c[3] - mean;
        ss0 += d0 * d0;
        ss1 += d1 * d1;
        ss2 += d2 * d2;
        ss3 += d3 * d3;
    }
    let mut ss = (ss0 + ss1) + (ss2 + ss3);
    for &x in rem {
        let d = x - mean;
        ss += d * d;
    }

    let sd = (ss / (n - 1) as f64).sqrt();
    if sd <= 0.0 || sd.is_nan() {
        return vec![f64::NAN; n];
    }
    let inv_sd = 1.0 / sd;

    // Pass 3: normalize directly into pre-allocated output
    let mut out = vec![0.0f64; n];
    let (in_chunks, in_rem) = vals.as_chunks::<4>();
    let (out_chunks, out_rem) = out.as_chunks_mut::<4>();

    for (ci, co) in in_chunks.iter().zip(out_chunks.iter_mut()) {
        co[0] = (ci[0] - mean) * inv_sd;
        co[1] = (ci[1] - mean) * inv_sd;
        co[2] = (ci[2] - mean) * inv_sd;
        co[3] = (ci[3] - mean) * inv_sd;
    }
    for (xi, xo) in in_rem.iter().zip(out_rem.iter_mut()) {
        *xo = (*xi - mean) * inv_sd;
    }

    out
}

/// `zscore(X [, dim])`
pub fn zscore(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "zscore requires at least 1 input.",
        ));
    }
    let da = get_da(&args[0])?;
    if da.is_empty() {
        return Ok(vec![Value::Double(da)]);
    }
    let default_dim = first_non_singleton_dim(da.shape());
    let (dim, _omit) = dim_and_omit(args, 1, default_dim)?;
    let res = map_fibers(&da, dim, zscore_slice);
    Ok(vec![Value::Double(res)])
}

/// `sortrows(A [, cols])`
pub fn sortrows(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "sortrows requires at least 1 input.",
        ));
    }
    let da = get_da(&args[0])?;
    let shape = da.shape();
    if shape.len() != 2 {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "sortrows requires a 2-D numeric array.",
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    if rows == 0 || cols == 0 {
        return Ok(vec![Value::Double(da)]);
    }
    let keys: Vec<(usize, bool)> = if args.len() >= 2 {
        let k = get_da(&args[1])?;
        let mut out = Vec::with_capacity(k.numel());
        for &c in k.as_slice() {
            if c == 0.0 || c.fract() != 0.0 {
                return Err(user_err(
                    "RUSTLAB:data:badInput",
                    "sortrows column indices must be nonzero integers.",
                ));
            }
            let idx = c.abs() as usize;
            if idx > cols {
                return Err(user_err(
                    "RUSTLAB:data:badInput",
                    "sortrows column index exceeds the number of columns.",
                ));
            }
            out.push((idx - 1, c > 0.0));
        }
        if out.is_empty() {
            (0..cols).map(|c| (c, true)).collect()
        } else {
            out
        }
    } else {
        (0..cols).map(|c| (c, true)).collect()
    };
    let mut order: Vec<usize> = (0..rows).collect();
    order.sort_by(|&ra, &rb| {
        for &(c, asc) in &keys {
            let a = da.as_slice()[ra + c * rows];
            let b = da.as_slice()[rb + c * rows];
            let ord = a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal);
            if ord != std::cmp::Ordering::Equal {
                return if asc { ord } else { ord.reverse() };
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut data = vec![0.0; rows * cols];
    for (new_r, &old_r) in order.iter().enumerate() {
        for c in 0..cols {
            data[new_r + c * rows] = da.as_slice()[old_r + c * rows];
        }
    }
    let sorted = Value::Double(DenseArray::from_vec_unchecked(vec![rows, cols], data));
    if nargout > 1 {
        let idx: Vec<f64> = order.iter().map(|&r| (r + 1) as f64).collect();
        Ok(vec![sorted, Value::Double(DenseArray::col_vector(idx))])
    } else {
        Ok(vec![sorted])
    }
}

fn eq_f64(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

fn f64_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    if eq_f64(a, b) {
        std::cmp::Ordering::Equal
    } else {
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// `unique(A)` / `[C, ia, ic] = unique(A)`
pub fn unique(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "unique requires at least 1 input.",
        ));
    }
    let da = get_da(&args[0])?;
    let shape = da.shape();
    if shape.len() != 2 {
        return Err(user_err(
            "RUSTLAB:data:badInput",
            "unique requires a 2-D numeric array.",
        ));
    }
    let is_vector = shape[0] == 1 || shape[1] == 1;
    if is_vector {
        let n = da.numel();
        let mut items: Vec<(f64, usize)> = da
            .as_slice()
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();
        items.sort_by(|a, b| f64_cmp(a.0, b.0));
        let mut vals: Vec<f64> = Vec::new();
        let mut ia: Vec<f64> = Vec::new();
        let mut group_of = vec![0usize; n];
        let mut i = 0usize;
        while i < items.len() {
            let mut j = i + 1;
            while j < items.len() && eq_f64(items[j].0, items[i].0) {
                j += 1;
            }
            let gid = vals.len();
            vals.push(items[i].0);
            let first = items[i..j].iter().map(|x| x.1).min().unwrap_or(0);
            ia.push((first + 1) as f64);
            for item in &items[i..j] {
                group_of[item.1] = gid;
            }
            i = j;
        }
        let c = if shape[0] == 1 {
            Value::Double(DenseArray::row_vector(vals))
        } else {
            Value::Double(DenseArray::col_vector(vals))
        };
        if nargout >= 3 {
            let ic: Vec<f64> = group_of.iter().map(|&g| (g + 1) as f64).collect();
            return Ok(vec![
                c,
                Value::Double(DenseArray::col_vector(ia)),
                Value::Double(DenseArray::col_vector(ic)),
            ]);
        }
        if nargout == 2 {
            return Ok(vec![c, Value::Double(DenseArray::col_vector(ia))]);
        }
        return Ok(vec![c]);
    }

    // Matrix input: unique rows, sorted lexicographically.
    let (rows, cols) = (shape[0], shape[1]);
    let mut order: Vec<usize> = (0..rows).collect();
    order.sort_by(|&a, &b| {
        for c in 0..cols {
            let ord = f64_cmp(da.as_slice()[a + c * rows], da.as_slice()[b + c * rows]);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut uniq_rows: Vec<Vec<f64>> = Vec::new();
    let mut ia: Vec<f64> = Vec::new();
    let mut group_of = vec![0usize; rows];
    let mut i = 0usize;
    while i < order.len() {
        let mut j = i + 1;
        while j < order.len() {
            let same = (0..cols).all(|c| {
                eq_f64(
                    da.as_slice()[order[i] + c * rows],
                    da.as_slice()[order[j] + c * rows],
                )
            });
            if same {
                j += 1;
            } else {
                break;
            }
        }
        let gid = uniq_rows.len();
        uniq_rows.push(
            (0..cols)
                .map(|c| da.as_slice()[order[i] + c * rows])
                .collect(),
        );
        let first = order[i..j].iter().copied().min().unwrap_or(0);
        ia.push((first + 1) as f64);
        for &r in &order[i..j] {
            group_of[r] = gid;
        }
        i = j;
    }
    let mut data = vec![0.0; uniq_rows.len() * cols];
    for (r, row) in uniq_rows.iter().enumerate() {
        for c in 0..cols {
            data[r + c * uniq_rows.len()] = row[c];
        }
    }
    let c = Value::Double(DenseArray::from_vec_unchecked(
        vec![uniq_rows.len(), cols],
        data,
    ));
    if nargout >= 3 {
        let ic: Vec<f64> = group_of.iter().map(|&g| (g + 1) as f64).collect();
        return Ok(vec![
            c,
            Value::Double(DenseArray::col_vector(ia)),
            Value::Double(DenseArray::col_vector(ic)),
        ]);
    }
    if nargout == 2 {
        return Ok(vec![c, Value::Double(DenseArray::col_vector(ia))]);
    }
    Ok(vec![c])
}

/// `var(X [, flag] [, dim] [, 'omitnan'])` variance.
pub fn var_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let first = args
        .first()
        .ok_or_else(|| user_err("MATLAB:minrhs", "var requires at least 1 input."))?;
    let da = get_da(first)?;
    if da.is_empty() && (da.shape() == [0, 0] || da.shape().is_empty()) {
        return Ok(vec![Value::scalar(f64::NAN)]);
    }

    let default_dim = first_non_singleton_dim(da.shape());
    let mut flag = 0usize;
    let mut dim = default_dim;
    let mut omit = false;
    let mut seen_flag = false;
    let mut seen_dim = false;

    for v in &args[1..] {
        if let Some(s) = as_string(v) {
            if s.eq_ignore_ascii_case("omitnan") {
                omit = true;
            } else if s.eq_ignore_ascii_case("includenan") {
                omit = false;
            } else {
                return Err(user_err(
                    "RUSTLAB:data:badOption",
                    &format!("Unsupported option '{s}'."),
                ));
            }
        } else {
            if !seen_flag {
                let fl = v.as_scalar_f64().map_err(|e| {
                    user_err(
                        "RUSTLAB:data:badFlag",
                        &format!("Normalization flag must be 0 or 1: {e}"),
                    )
                })?;
                if fl == 0.0 {
                    flag = 0;
                } else if fl == 1.0 {
                    flag = 1;
                } else {
                    return Err(user_err(
                        "RUSTLAB:data:badFlag",
                        "Normalization flag must be 0 or 1.",
                    ));
                }
                seen_flag = true;
            } else if !seen_dim {
                dim = parse_dim_value(v)?;
                seen_dim = true;
            } else {
                return Err(user_err("MATLAB:maxrhs", "Too many input arguments."));
            }
        }
    }

    let res = reduce_f64(&da, dim, |slice| variance_of(slice, flag, omit));
    Ok(vec![Value::Double(res)])
}

/// `std(X [, flag] [, dim] [, 'omitnan'])` standard deviation.
pub fn std_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let mut v = var_fn(args, 1)?;
    let val = v
        .pop()
        .ok_or_else(|| user_err("MATLAB:minrhs", "Not enough input arguments."))?;
    let da = get_da(&val)?;
    Ok(vec![Value::Double(da.map(|&x| x.sqrt()))])
}
