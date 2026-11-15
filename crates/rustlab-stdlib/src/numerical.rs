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

/// Numerical gradient: g = gradient(f) or [fx, fy] = gradient(z, hx, hy).
pub fn gradient_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "gradient requires at least 1 input.",
        ));
    }

    let shp = args[0].shape();
    // 2D matrix case
    if shp.len() == 2 && shp[0] > 1 && shp[1] > 1 {
        let (m, n) = (shp[0], shp[1]);
        let z = extract_f64_slice(&args[0])?;

        let (hx, hy) = if args.len() >= 3 {
            let hx = extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0);
            let hy = extract_f64_slice(&args[2])?.first().copied().unwrap_or(1.0);
            (hx, hy)
        } else if args.len() == 2 {
            let h = extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0);
            (h, h)
        } else {
            (1.0, 1.0)
        };

        if hx <= 0.0 || hy <= 0.0 {
            return Err(user_err(
                "RUSTLAB:gradient:invalidSpacing",
                "Spacing must be positive.",
            ));
        }

        // FX: gradient along columns (horizontal, across cols)
        let mut fx = vec![0.0; m * n];
        for r in 0..m {
            // First col
            fx[r] = (z[r + m] - z[r]) / hx;
            // Interior cols
            for c in 1..(n - 1) {
                fx[r + c * m] = (z[r + (c + 1) * m] - z[r + (c - 1) * m]) / (2.0 * hx);
            }
            // Last col
            fx[r + (n - 1) * m] = (z[r + (n - 1) * m] - z[r + (n - 2) * m]) / hx;
        }

        // FY: gradient along rows (vertical, down rows)
        let mut fy = vec![0.0; m * n];
        for c in 0..n {
            let col_off = c * m;
            // First row
            fy[col_off] = (z[1 + col_off] - z[col_off]) / hy;
            // Interior rows
            for r in 1..(m - 1) {
                fy[r + col_off] = (z[r + 1 + col_off] - z[r - 1 + col_off]) / (2.0 * hy);
            }
            // Last row
            fy[m - 1 + col_off] = (z[m - 1 + col_off] - z[m - 2 + col_off]) / hy;
        }

        let fx_arr = DenseArray::from_vec_unchecked(vec![m, n], fx);
        let fy_arr = DenseArray::from_vec_unchecked(vec![m, n], fy);

        if nargout >= 2 {
            return Ok(vec![Value::Double(fx_arr), Value::Double(fy_arr)]);
        } else {
            return Ok(vec![Value::Double(fx_arr)]);
        }
    }

    // 1D vector case
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

    let out_shp = if shp.len() == 2 && shp[0] == n {
        vec![n, 1]
    } else {
        vec![1, n]
    };
    let res_da = DenseArray::from_vec_unchecked(out_shp, g);
    Ok(vec![Value::Double(res_da)])
}

/// Discrete 5-point Laplacian: l = del2(u) or del2(u, h).
pub fn del2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "del2 requires at least 1 input."));
    }
    let shp = args[0].shape();
    let u = extract_f64_slice(&args[0])?;

    // 1D Vector case
    if shp.len() == 1 || (shp.len() == 2 && (shp[0] == 1 || shp[1] == 1)) {
        let len = u.len();
        if len < 3 {
            return Err(user_err(
                "RUSTLAB:del2:minSize",
                "Vector must have length >= 3.",
            ));
        }
        let h = if args.len() >= 2 {
            extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0)
        } else {
            1.0
        };
        let h2 = h * h;
        let mut out = vec![0.0; len];
        out[0] = 0.5 * (u[0] - 2.0 * u[1] + u[2]) / h2;
        for i in 1..(len - 1) {
            out[i] = 0.5 * (u[i + 1] - 2.0 * u[i] + u[i - 1]) / h2;
        }
        out[len - 1] = 0.5 * (u[len - 1] - 2.0 * u[len - 2] + u[len - 3]) / h2;
        let arr = DenseArray::from_vec_unchecked(shp.to_vec(), out);
        return Ok(vec![Value::Double(arr)]);
    }

    if shp.len() != 2 || shp[0] < 3 || shp[1] < 3 {
        return Err(user_err(
            "RUSTLAB:del2:minSize",
            "Matrix must be at least 3x3.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);
    let (hx, hy) = if args.len() >= 3 {
        let hx = extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0);
        let hy = extract_f64_slice(&args[2])?.first().copied().unwrap_or(1.0);
        (hx, hy)
    } else if args.len() >= 2 {
        let h = extract_f64_slice(&args[1])?.first().copied().unwrap_or(1.0);
        (h, h)
    } else {
        (1.0, 1.0)
    };
    let hx2 = hx * hx;
    let hy2 = hy * hy;

    let mut out = vec![0.0; m * n];
    for r in 0..m {
        for c in 0..n {
            let d2x = if c == 0 {
                (u[r] - 2.0 * u[r + m] + u[r + 2 * m]) / hx2
            } else if c == n - 1 {
                (u[r + (n - 1) * m] - 2.0 * u[r + (n - 2) * m] + u[r + (n - 3) * m]) / hx2
            } else {
                let u_c = u[r + c * m];
                (u[r + (c + 1) * m] - 2.0 * u_c + u[r + (c - 1) * m]) / hx2
            };

            let d2y = if r == 0 {
                (u[c * m] - 2.0 * u[1 + c * m] + u[2 + c * m]) / hy2
            } else if r == m - 1 {
                (u[(m - 1) + c * m] - 2.0 * u[(m - 2) + c * m] + u[(m - 3) + c * m]) / hy2
            } else {
                let u_c = u[r + c * m];
                (u[r + 1 + c * m] - 2.0 * u_c + u[r - 1 + c * m]) / hy2
            };

            out[r + c * m] = 0.25 * (d2x + d2y);
        }
    }

    let arr = DenseArray::from_vec_unchecked(vec![m, n], out);
    Ok(vec![Value::Double(arr)])
}

/// Helper to compute central differences along rows and columns for a 2D matrix.
pub fn partial_derivative_2d(
    z: &[f64],
    m: usize,
    n: usize,
    hx: f64,
    hy: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut dx = vec![0.0; m * n];
    let mut dy = vec![0.0; m * n];

    // Derivative along columns (x direction)
    for r in 0..m {
        if n == 1 {
            dx[r] = 0.0;
        } else if n == 2 {
            let diff = (z[r + m] - z[r]) / hx;
            dx[r] = diff;
            dx[r + m] = diff;
        } else {
            dx[r] = (z[r + m] - z[r]) / hx;
            for c in 1..(n - 1) {
                dx[r + c * m] = (z[r + (c + 1) * m] - z[r + (c - 1) * m]) / (2.0 * hx);
            }
            dx[r + (n - 1) * m] = (z[r + (n - 1) * m] - z[r + (n - 2) * m]) / hx;
        }
    }

    // Derivative along rows (y direction)
    for c in 0..n {
        if m == 1 {
            dy[c * m] = 0.0;
        } else if m == 2 {
            let diff = (z[1 + c * m] - z[c * m]) / hy;
            dy[c * m] = diff;
            dy[1 + c * m] = diff;
        } else {
            dy[c * m] = (z[1 + c * m] - z[c * m]) / hy;
            for r in 1..(m - 1) {
                dy[r + c * m] = (z[(r + 1) + c * m] - z[(r - 1) + c * m]) / (2.0 * hy);
            }
            dy[(m - 1) + c * m] = (z[(m - 1) + c * m] - z[(m - 2) + c * m]) / hy;
        }
    }

    (dx, dy)
}

/// Divergence of vector field: div = divergence(X, Y, U, V) or divergence(U, V).
pub fn divergence_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 && args.len() != 4 {
        return Err(user_err(
            "RUSTLAB:divergence:invalidNumInputs",
            "divergence requires 2 or 4 inputs: divergence(U, V) or divergence(X, Y, U, V).",
        ));
    }

    let (hx, hy, u_val, v_val) = if args.len() == 4 {
        let x_slice = extract_f64_slice(&args[0])?;
        let y_slice = extract_f64_slice(&args[1])?;
        let shp_x = args[0].shape();
        let hx = if shp_x.len() == 2 && shp_x[1] > 1 {
            (x_slice[shp_x[0]] - x_slice[0]).abs()
        } else if x_slice.len() > 1 {
            (x_slice[1] - x_slice[0]).abs()
        } else {
            1.0
        };
        let shp_y = args[1].shape();
        let hy = if (shp_y.len() == 2 && shp_y[0] > 1) || y_slice.len() > 1 {
            (y_slice[1] - y_slice[0]).abs()
        } else {
            1.0
        };
        (
            if hx > 1e-12 { hx } else { 1.0 },
            if hy > 1e-12 { hy } else { 1.0 },
            &args[2],
            &args[3],
        )
    } else {
        (1.0, 1.0, &args[0], &args[1])
    };

    let shp_u = u_val.shape();
    let shp_v = v_val.shape();
    if shp_u != shp_v || shp_u.len() != 2 {
        return Err(user_err(
            "RUSTLAB:divergence:sizeMismatch",
            "U and V must be 2D matrices of the same dimensions.",
        ));
    }

    let (m, n) = (shp_u[0], shp_u[1]);
    let u_slice = extract_f64_slice(u_val)?;
    let v_slice = extract_f64_slice(v_val)?;

    let (du_dx, _) = partial_derivative_2d(&u_slice, m, n, hx, hy);
    let (_, dv_dy) = partial_derivative_2d(&v_slice, m, n, hx, hy);

    let div: Vec<f64> = du_dx
        .iter()
        .zip(dv_dy.iter())
        .map(|(&du, &dv)| du + dv)
        .collect();

    let arr = DenseArray::from_vec_unchecked(vec![m, n], div);
    Ok(vec![Value::Double(arr)])
}

/// Curl and angular velocity of 2D vector field: [cav, vor] = curl(X, Y, U, V) or curl(U, V).
pub fn curl_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 && args.len() != 4 {
        return Err(user_err(
            "RUSTLAB:curl:invalidNumInputs",
            "curl requires 2 or 4 inputs: curl(U, V) or curl(X, Y, U, V).",
        ));
    }

    let (hx, hy, u_val, v_val) = if args.len() == 4 {
        let x_slice = extract_f64_slice(&args[0])?;
        let y_slice = extract_f64_slice(&args[1])?;
        let shp_x = args[0].shape();
        let hx = if shp_x.len() == 2 && shp_x[1] > 1 {
            (x_slice[shp_x[0]] - x_slice[0]).abs()
        } else if x_slice.len() > 1 {
            (x_slice[1] - x_slice[0]).abs()
        } else {
            1.0
        };
        let shp_y = args[1].shape();
        let hy = if (shp_y.len() == 2 && shp_y[0] > 1) || y_slice.len() > 1 {
            (y_slice[1] - y_slice[0]).abs()
        } else {
            1.0
        };
        (
            if hx > 1e-12 { hx } else { 1.0 },
            if hy > 1e-12 { hy } else { 1.0 },
            &args[2],
            &args[3],
        )
    } else {
        (1.0, 1.0, &args[0], &args[1])
    };

    let shp_u = u_val.shape();
    let shp_v = v_val.shape();
    if shp_u != shp_v || shp_u.len() != 2 {
        return Err(user_err(
            "RUSTLAB:curl:sizeMismatch",
            "U and V must be 2D matrices of the same dimensions.",
        ));
    }

    let (m, n) = (shp_u[0], shp_u[1]);
    let u_slice = extract_f64_slice(u_val)?;
    let v_slice = extract_f64_slice(v_val)?;

    let (_, du_dy) = partial_derivative_2d(&u_slice, m, n, hx, hy);
    let (dv_dx, _) = partial_derivative_2d(&v_slice, m, n, hx, hy);

    // vor = dv/dx - du/dy (vorticity)
    let vor: Vec<f64> = dv_dx
        .iter()
        .zip(du_dy.iter())
        .map(|(&vx, &uy)| vx - uy)
        .collect();

    // cav = 0.5 * vor (angular velocity)
    let cav: Vec<f64> = vor.iter().map(|&w| 0.5 * w).collect();

    let arr_cav = DenseArray::from_vec_unchecked(vec![m, n], cav);

    if nargout >= 2 {
        let arr_vor = DenseArray::from_vec_unchecked(vec![m, n], vor);
        Ok(vec![Value::Double(arr_cav), Value::Double(arr_vor)])
    } else {
        Ok(vec![Value::Double(arr_cav)])
    }
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

/// Cubic spline data interpolation: yy = spline(x, y, xx).
pub fn spline_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "spline requires at least 3 inputs (x, y, xx).",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let y = extract_f64_slice(&args[1])?;
    let xx = extract_f64_slice(&args[2])?;

    let n = x.len();
    if n < 2 || y.len() != n {
        return Err(user_err(
            "RUSTLAB:spline:mismatch",
            "x and y must have the same length >= 2.",
        ));
    }

    if n == 2 {
        let mut yy = Vec::with_capacity(xx.len());
        let dx = x[1] - x[0];
        for &q in &xx {
            let t = if dx.abs() > 1e-15 {
                (q - x[0]) / dx
            } else {
                0.0
            };
            yy.push(y[0] + t * (y[1] - y[0]));
        }
        let da = DenseArray::from_vec_unchecked(vec![1, yy.len()], yy);
        return Ok(vec![Value::Double(da)]);
    }

    let mut h = vec![0.0; n - 1];
    for i in 0..n - 1 {
        h[i] = x[i + 1] - x[i];
    }

    let m_size = n - 2;
    let mut sub = vec![0.0; m_size];
    let mut diag = vec![0.0; m_size];
    let mut sup = vec![0.0; m_size];
    let mut rhs = vec![0.0; m_size];

    for i in 0..m_size {
        let idx = i + 1;
        diag[i] = 2.0 * (h[idx - 1] + h[idx]);
        if i > 0 {
            sub[i] = h[idx - 1];
        }
        if i + 1 < m_size {
            sup[i] = h[idx];
        }
        let d1 = (y[idx + 1] - y[idx]) / h[idx];
        let d0 = (y[idx] - y[idx - 1]) / h[idx - 1];
        rhs[i] = 6.0 * (d1 - d0);
    }

    let mut c_prime = vec![0.0; m_size];
    let mut d_prime = vec![0.0; m_size];

    if m_size > 0 {
        c_prime[0] = sup[0] / diag[0];
        d_prime[0] = rhs[0] / diag[0];
        for i in 1..m_size {
            let denom = diag[i] - sub[i] * c_prime[i - 1];
            c_prime[i] = if i + 1 < m_size { sup[i] / denom } else { 0.0 };
            d_prime[i] = (rhs[i] - sub[i] * d_prime[i - 1]) / denom;
        }
    }

    let mut m_sol = vec![0.0; n];
    if m_size > 0 {
        m_sol[m_size] = d_prime[m_size - 1];
        for i in (0..m_size - 1).rev() {
            m_sol[i + 1] = d_prime[i] - c_prime[i] * m_sol[i + 2];
        }
    }

    let mut yy = Vec::with_capacity(xx.len());
    for &q in &xx {
        let k = if q <= x[0] {
            0
        } else if q >= x[n - 1] {
            n - 2
        } else {
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
            low
        };

        let hk = h[k];
        let dx_hi = x[k + 1] - q;
        let dx_lo = q - x[k];
        let term1 = (m_sol[k] * dx_hi.powi(3) + m_sol[k + 1] * dx_lo.powi(3)) / (6.0 * hk);
        let term2 = (y[k] - m_sol[k] * hk.powi(2) / 6.0) * (dx_hi / hk);
        let term3 = (y[k + 1] - m_sol[k + 1] * hk.powi(2) / 6.0) * (dx_lo / hk);
        yy.push(term1 + term2 + term3);
    }

    let da = DenseArray::from_vec_unchecked(vec![1, yy.len()], yy);
    Ok(vec![Value::Double(da)])
}

/// Piecewise Cubic Hermite Interpolating Polynomial: yy = pchip(x, y, xx).
pub fn pchip_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "pchip requires at least 3 inputs (x, y, xx).",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let y = extract_f64_slice(&args[1])?;
    let xx = extract_f64_slice(&args[2])?;

    let n = x.len();
    if n < 2 || y.len() != n {
        return Err(user_err(
            "RUSTLAB:pchip:mismatch",
            "x and y must have the same length >= 2.",
        ));
    }

    let mut h = vec![0.0; n - 1];
    let mut delta = vec![0.0; n - 1];
    for k in 0..n - 1 {
        h[k] = x[k + 1] - x[k];
        if h[k] <= 0.0 {
            return Err(user_err(
                "RUSTLAB:pchip:nonStrictlyIncreasing",
                "x data points must be strictly increasing.",
            ));
        }
        delta[k] = (y[k + 1] - y[k]) / h[k];
    }

    let mut d = vec![0.0; n];

    if n == 2 {
        d[0] = delta[0];
        d[1] = delta[0];
    } else {
        for k in 1..n - 1 {
            if delta[k - 1] * delta[k] <= 0.0 {
                d[k] = 0.0;
            } else {
                let w1 = 2.0 * h[k] + h[k - 1];
                let w2 = h[k] + 2.0 * h[k - 1];
                d[k] = (w1 + w2) / (w1 / delta[k - 1] + w2 / delta[k]);
            }
        }

        let d0 = ((2.0 * h[0] + h[1]) * delta[0] - h[0] * delta[1]) / (h[0] + h[1]);
        if d0 * delta[0] <= 0.0 {
            d[0] = 0.0;
        } else if delta[0] * delta[1] <= 0.0 && d0.abs() > 3.0 * delta[0].abs() {
            d[0] = 3.0 * delta[0];
        } else {
            d[0] = d0;
        }

        let dn = ((2.0 * h[n - 2] + h[n - 3]) * delta[n - 2] - h[n - 2] * delta[n - 3])
            / (h[n - 2] + h[n - 3]);
        if dn * delta[n - 2] <= 0.0 {
            d[n - 1] = 0.0;
        } else if delta[n - 2] * delta[n - 3] <= 0.0 && dn.abs() > 3.0 * delta[n - 2].abs() {
            d[n - 1] = 3.0 * delta[n - 2];
        } else {
            d[n - 1] = dn;
        }
    }

    let mut yy = Vec::with_capacity(xx.len());
    for &q in &xx {
        let k = if q <= x[0] {
            0
        } else if q >= x[n - 1] {
            n - 2
        } else {
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
            low
        };

        let hk = h[k];
        let t = (q - x[k]) / hk;
        let t2 = t * t;
        let t3 = t2 * t;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        let val = h00 * y[k] + h10 * hk * d[k] + h01 * y[k + 1] + h11 * hk * d[k + 1];
        yy.push(val);
    }

    let da = DenseArray::from_vec_unchecked(vec![1, yy.len()], yy);
    Ok(vec![Value::Double(da)])
}
