//! Numerical methods: root finding, polynomials, numerical integration, differentiation, interpolation.

use crate::helpers::user_err;
use crate::linalg::svd::eig_fn;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

fn extract_f64_slice(val: &Value) -> Result<Vec<f64>, EvalError> {
    match val {
        Value::Double(d) => Ok(d.as_slice().to_vec()),
        Value::Single(s) => Ok(s.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int32(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        Value::UInt32(u) => Ok(u.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int64(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        Value::UInt64(u) => Ok(u.as_slice().iter().map(|&x| x as f64).collect()),
        _ => Err(user_err(
            "RUSTLAB:invalidType",
            "Expected numeric vector input.",
        )),
    }
}

/// Finds roots of polynomial p: roots(p).
pub fn roots_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "roots requires at least 1 input.",
        ));
    }
    let raw_coeffs = extract_f64_slice(&args[0])?;
    if raw_coeffs.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    // Strip leading zeros
    let first_nonzero = raw_coeffs.iter().position(|&c| c.abs() > 1e-15);
    let coeffs = match first_nonzero {
        Some(idx) => &raw_coeffs[idx..],
        None => return Ok(vec![Value::Double(DenseArray::empty())]),
    };

    let n = coeffs.len() - 1; // Degree of polynomial
    if n == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    let c0 = coeffs[0];

    // Degree 1: c0*x + c1 = 0 => x = -c1/c0
    if n == 1 {
        let r = -coeffs[1] / c0;
        let arr = DenseArray::from_vec_unchecked(vec![1, 1], vec![r]);
        return Ok(vec![Value::Double(arr)]);
    }

    // Degree 2: quadratic formula
    if n == 2 {
        let a = c0;
        let b = coeffs[1];
        let c = coeffs[2];
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let s = disc.sqrt();
            let r1 = (-b + s) / (2.0 * a);
            let r2 = (-b - s) / (2.0 * a);
            let arr = DenseArray::from_vec_unchecked(vec![2, 1], vec![r1, r2]);
            return Ok(vec![Value::Double(arr)]);
        } else {
            let re = -b / (2.0 * a);
            let im = (-disc).sqrt() / (2.0 * a);
            let r1 = num_complex::Complex64::new(re, im);
            let r2 = num_complex::Complex64::new(re, -im);
            let arr = DenseArray::from_vec_unchecked(vec![2, 1], vec![r1, r2]);
            return Ok(vec![Value::ComplexDouble(arr)]);
        }
    }

    // Degree >= 3: construct companion matrix C of size n x n
    // Column-major storage: element (row, col) is at index row + col * n
    let mut comp = vec![0.0; n * n];
    for col in 0..n {
        comp[col * n] = -coeffs[col + 1] / c0;
    }
    for i in 1..n {
        comp[i + (i - 1) * n] = 1.0;
    }

    let comp_da = DenseArray::from_vec_unchecked(vec![n, n], comp);
    let eig_res = eig_fn(&[Value::Double(comp_da)], 1)?;
    Ok(eig_res)
}

/// Evaluates polynomial at points x: y = polyval(p, x).
pub fn polyval_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "polyval requires at least 2 inputs (p, x).",
        ));
    }
    let coeffs = extract_f64_slice(&args[0])?;
    if coeffs.is_empty() {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }

    match &args[1] {
        Value::Double(x_da) => {
            let mut y_vals = Vec::with_capacity(x_da.numel());
            for &x in x_da.as_slice() {
                let mut acc = coeffs[0];
                for &c in &coeffs[1..] {
                    acc = acc * x + c;
                }
                y_vals.push(acc);
            }
            let res_da = DenseArray::from_vec_unchecked(x_da.shape().to_vec(), y_vals);
            Ok(vec![Value::Double(res_da)])
        }
        Value::Single(x_da) => {
            let mut y_vals = Vec::with_capacity(x_da.numel());
            for &x in x_da.as_slice() {
                let xf = x as f64;
                let mut acc = coeffs[0];
                for &c in &coeffs[1..] {
                    acc = acc * xf + c;
                }
                y_vals.push(acc as f32);
            }
            let res_da = DenseArray::from_vec_unchecked(x_da.shape().to_vec(), y_vals);
            Ok(vec![Value::Single(res_da)])
        }
        other => {
            let x_vec = extract_f64_slice(other)?;
            let mut y_vals = Vec::with_capacity(x_vec.len());
            for &x in &x_vec {
                let mut acc = coeffs[0];
                for &c in &coeffs[1..] {
                    acc = acc * x + c;
                }
                y_vals.push(acc);
            }
            let res_da = DenseArray::from_vec_unchecked(vec![1, y_vals.len()], y_vals);
            Ok(vec![Value::Double(res_da)])
        }
    }
}

/// Least-squares polynomial fit: p = polyfit(x, y, n).
pub fn polyfit_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "polyfit requires at least 3 inputs (x, y, n).",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let y = extract_f64_slice(&args[1])?;
    let deg = match &args[2] {
        Value::Double(d) if d.numel() == 1 => d.as_slice()[0] as usize,
        Value::Int32(i) if i.numel() == 1 => i.as_slice()[0] as usize,
        Value::UInt32(u) if u.numel() == 1 => u.as_slice()[0] as usize,
        _ => {
            return Err(user_err(
                "RUSTLAB:polyfit:degNotScalar",
                "Degree n must be a positive integer scalar.",
            ))
        }
    };

    if x.len() != y.len() {
        return Err(user_err(
            "RUSTLAB:polyfit:XYSizeMismatch",
            "X and Y vectors must have the same length.",
        ));
    }
    let m = x.len();
    if m < deg + 1 {
        return Err(user_err(
            "RUSTLAB:polyfit:PolyNotUnique",
            "Polynomial degree must be less than the number of data points.",
        ));
    }

    let p_len = deg + 1;
    // Build Vandermonde matrix V of size m x p_len: V[i, j] = x[i]^(deg - j)
    // Column-major: element (row i, col j) is at i + j * m
    let mut v_mat = vec![0.0; m * p_len];
    for i in 0..m {
        let mut cur = 1.0;
        // From lowest degree (j = deg, power 0) to highest (j = 0, power deg)
        for j in (0..p_len).rev() {
            v_mat[i + j * m] = cur;
            cur *= x[i];
        }
    }

    // Solve least-squares min ||V * p - y||_2 via Householder QR decomposition
    // (avoids normal equations V^T * V to preserve conditioning).
    let mut a_work = v_mat;
    let mut rhs_work = y.to_vec();

    for k in 0..p_len {
        let mut sum_sq = 0.0;
        for i in k..m {
            let val = a_work[i + k * m];
            sum_sq += val * val;
        }
        let norm_x = sum_sq.sqrt();
        if norm_x < 1e-15 {
            return Err(user_err(
                "RUSTLAB:polyfit:RepeatedPointsOrCollinear",
                "Matrix is close to singular or points are collinear.",
            ));
        }

        let x0 = a_work[k + k * m];
        let alpha = if x0 >= 0.0 { -norm_x } else { norm_x };
        let v0 = x0 - alpha;
        let tau = (alpha - x0) / alpha;

        let len = m - k;
        let mut v = vec![0.0; len];
        v[0] = 1.0;
        for i in (k + 1)..m {
            v[i - k] = a_work[i + k * m] / v0;
        }

        a_work[k + k * m] = alpha;
        for i in (k + 1)..m {
            a_work[i + k * m] = 0.0;
        }

        for j in (k + 1)..p_len {
            let mut dot = 0.0;
            for idx in 0..len {
                dot += v[idx] * a_work[(k + idx) + j * m];
            }
            for idx in 0..len {
                a_work[(k + idx) + j * m] -= tau * dot * v[idx];
            }
        }

        let mut dot_y = 0.0;
        for idx in 0..len {
            dot_y += v[idx] * rhs_work[k + idx];
        }
        for idx in 0..len {
            rhs_work[k + idx] -= tau * dot_y * v[idx];
        }
    }

    // Back substitution on upper triangular system R * p = Q^T * y
    let mut p_sol = vec![0.0; p_len];
    for k in (0..p_len).rev() {
        let mut sum = rhs_work[k];
        for c in (k + 1)..p_len {
            sum -= a_work[k + c * m] * p_sol[c];
        }
        let diag = a_work[k + k * m];
        if diag.abs() < 1e-15 {
            return Err(user_err(
                "RUSTLAB:polyfit:RepeatedPointsOrCollinear",
                "Matrix is close to singular or points are collinear.",
            ));
        }
        p_sol[k] = sum / diag;
    }

    let res_da = DenseArray::from_vec_unchecked(vec![1, p_len], p_sol);
    Ok(vec![Value::Double(res_da)])
}

/// Trapezoidal numerical integration: z = trapz(y) or trapz(x, y).
pub fn trapz_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "trapz requires at least 1 input.",
        ));
    }
    let (x, y) = if args.len() >= 2 {
        (extract_f64_slice(&args[0])?, extract_f64_slice(&args[1])?)
    } else {
        let y = extract_f64_slice(&args[0])?;
        let x = (1..=y.len()).map(|i| i as f64).collect();
        (x, y)
    };

    if x.len() != y.len() {
        return Err(user_err(
            "RUSTLAB:trapz:lengthMismatch",
            "X and Y vectors must have the same length.",
        ));
    }
    if y.len() < 2 {
        return Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![0.0],
        ))]);
    }

    let mut sum = 0.0;
    for i in 0..(y.len() - 1) {
        let dx = x[i + 1] - x[i];
        sum += 0.5 * (y[i] + y[i + 1]) * dx;
    }

    let res_da = DenseArray::from_vec_unchecked(vec![1, 1], vec![sum]);
    Ok(vec![Value::Double(res_da)])
}

/// Cumulative trapezoidal numerical integration: z = cumtrapz(y) or cumtrapz(x, y).
pub fn cumtrapz_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "cumtrapz requires at least 1 input.",
        ));
    }
    let (x, y) = if args.len() >= 2 {
        (extract_f64_slice(&args[0])?, extract_f64_slice(&args[1])?)
    } else {
        let y = extract_f64_slice(&args[0])?;
        let x = (1..=y.len()).map(|i| i as f64).collect();
        (x, y)
    };

    if x.len() != y.len() {
        return Err(user_err(
            "RUSTLAB:cumtrapz:lengthMismatch",
            "X and Y vectors must have the same length.",
        ));
    }

    let mut z = Vec::with_capacity(y.len());
    let mut acc = 0.0;
    z.push(0.0);
    for i in 0..(y.len().saturating_sub(1)) {
        let dx = x[i + 1] - x[i];
        acc += 0.5 * (y[i] + y[i + 1]) * dx;
        z.push(acc);
    }

    let res_da = DenseArray::from_vec_unchecked(vec![1, z.len()], z);
    Ok(vec![Value::Double(res_da)])
}

/// Numerical gradient: g = gradient(f) or gradient(f, h).
pub fn gradient_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "gradient requires at least 1 input.",
        ));
    }
    let f = extract_f64_slice(&args[0])?;
    let h = if args.len() >= 2 {
        extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0)
    } else {
        1.0
    };

    if h <= 0.0 {
        return Err(user_err(
            "RUSTLAB:gradient:invalidSpacing",
            "Spacing h must be positive.",
        ));
    }

    let n = f.len();
    if n == 0 {
        return Ok(vec![Value::Double(DenseArray::empty())]);
    }
    if n == 1 {
        return Ok(vec![Value::Double(DenseArray::from_vec_unchecked(
            vec![1, 1],
            vec![0.0],
        ))]);
    }

    let mut g = Vec::with_capacity(n);
    // Forward difference at start
    g.push((f[1] - f[0]) / h);

    // Central difference in interior
    for i in 1..(n - 1) {
        g.push((f[i + 1] - f[i - 1]) / (2.0 * h));
    }

    // Backward difference at end
    g.push((f[n - 1] - f[n - 2]) / h);

    let res_da = DenseArray::from_vec_unchecked(vec![1, n], g);
    Ok(vec![Value::Double(res_da)])
}

/// 1-D interpolation: yq = interp1(x, y, xq, method).
pub fn interp1_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "interp1 requires at least 3 inputs (x, y, xq).",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let y = extract_f64_slice(&args[1])?;
    let xq = extract_f64_slice(&args[2])?;
    let method = if args.len() >= 4 {
        match &args[3] {
            Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
            Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
            _ => "linear".to_string(),
        }
    } else {
        "linear".to_string()
    };

    if x.len() != y.len() {
        return Err(user_err(
            "RUSTLAB:interp1:XYLengthMismatch",
            "X and Y vectors must have the same length.",
        ));
    }
    if x.len() < 2 {
        return Err(user_err(
            "RUSTLAB:interp1:NotEnoughPoints",
            "There should be at least two data points.",
        ));
    }

    let mut yq = Vec::with_capacity(xq.len());
    let n = x.len();

    for &q in &xq {
        if q < x[0] || q > x[n - 1] {
            // Extrapolation produces NaN by default in MATLAB
            yq.push(f64::NAN);
            continue;
        }

        // Binary search for bracket
        let mut low = 0;
        let mut high = n - 1;
        while high - low > 1 {
            let mid = (low + high) / 2;
            if x[mid] <= q {
                low = mid;
            } else {
                high = mid;
            }
        }

        if method.eq_ignore_ascii_case("nearest") {
            if (q - x[low]).abs() <= (x[high] - q).abs() {
                yq.push(y[low]);
            } else {
                yq.push(y[high]);
            }
        } else {
            // Linear interpolation
            let dx = x[high] - x[low];
            let t = if dx > 0.0 { (q - x[low]) / dx } else { 0.0 };
            yq.push(y[low] + t * (y[high] - y[low]));
        }
    }

    let res_da = DenseArray::from_vec_unchecked(vec![1, yq.len()], yq);
    Ok(vec![Value::Double(res_da)])
}
