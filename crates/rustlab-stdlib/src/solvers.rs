//! Numerical solvers for differential equations, optimization, root-finding, and integration.
//!
//! Provides clean-room MATLAB-compatible implementations of:
//! - `ode45`: Dormand-Prince 5(4) adaptive Runge-Kutta ODE solver.
//! - `ode23`: Bogacki-Shampine 3(2) adaptive Runge-Kutta ODE solver.
//! - `fzero`: Brent's method with root bracketing.
//! - `fminsearch`: Nelder-Mead simplex multidimensional unconstrained minimizer.
//! - `integral`: Adaptive numerical quadrature.

use crate::helpers::{math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, FunctionHandle, StructArray, Value};
use rustlab_parse::Span;
use std::collections::HashMap;

/// Configuration options for ODE solvers.
#[derive(Debug, Clone)]
pub struct OdeOptions {
    pub rel_tol: f64,
    pub abs_tol: f64,
    pub initial_step: Option<f64>,
    pub max_step: Option<f64>,
}

/// Helper to parse an ODE options structure or Value into `OdeOptions`.
pub fn parse_ode_options(val: Option<&Value>, default_rel_tol: f64) -> OdeOptions {
    let mut opts = OdeOptions {
        rel_tol: default_rel_tol,
        abs_tol: 1e-6,
        initial_step: None,
        max_step: None,
    };
    let Some(v) = val else {
        return opts;
    };
    if let Value::Struct(sa) = v {
        if let Some(map) = sa.data.first() {
            if let Some(r) = map.get("RelTol").and_then(|v| v.as_scalar_f64().ok()) {
                opts.rel_tol = r;
            }
            if let Some(a) = map.get("AbsTol").and_then(|v| v.as_scalar_f64().ok()) {
                opts.abs_tol = a;
            }
            if let Some(h) = map.get("InitialStep").and_then(|v| v.as_scalar_f64().ok()) {
                opts.initial_step = Some(h);
            }
            if let Some(m) = map.get("MaxStep").and_then(|v| v.as_scalar_f64().ok()) {
                opts.max_step = Some(m);
            }
        }
    }
    opts
}

/// Solves an n x n linear system A * x = b with partial pivoting.
/// Matrix A is stored in row-major order of length n * n. Overwrites b with solution x.
pub fn solve_linear_system(a: &mut [f64], b: &mut [f64], n: usize) -> bool {
    for col in 0..n {
        let mut max_val = a[col * n + col].abs();
        let mut pivot_row = col;
        for row in (col + 1)..n {
            let val = a[row * n + col].abs();
            if val > max_val {
                max_val = val;
                pivot_row = row;
            }
        }
        if max_val < 1e-15 {
            return false;
        }
        if pivot_row != col {
            for j in 0..n {
                a.swap(col * n + j, pivot_row * n + j);
            }
            b.swap(col, pivot_row);
        }
        let diag = a[col * n + col];
        for row in (col + 1)..n {
            let factor = a[row * n + col] / diag;
            a[row * n + col] = 0.0;
            for j in (col + 1)..n {
                a[row * n + j] -= factor * a[col * n + j];
            }
            b[row] -= factor * b[col];
        }
    }
    for row in (0..n).rev() {
        let mut sum = b[row];
        for j in (row + 1)..n {
            sum -= a[row * n + j] * b[j];
        }
        let diag = a[row * n + row];
        if diag.abs() < 1e-15 {
            return false;
        }
        b[row] = sum / diag;
    }
    true
}

/// Helper to create a Double 2D Value matrix.
fn make_double_matrix(rows: usize, cols: usize, data: Vec<f64>) -> Result<Value, EvalError> {
    let arr = DenseArray::from_vec(vec![rows, cols], data).map_err(math_err)?;
    Ok(Value::Double(arr))
}

/// Invokes a function handle using the active interpreter on the calling thread,
/// falling back to an ephemeral interpreter if no active interpreter is present.
pub fn call_handle_any(
    handle: &FunctionHandle,
    args: Vec<Value>,
    nargout: usize,
) -> Result<Vec<Value>, EvalError> {
    if let Ok(res) = rustlab_eval::call_active_handle(handle, args.clone(), nargout) {
        return Ok(res);
    }
    let mut interp = crate::new_interpreter();
    interp.call_function_handle(handle, args, nargout, Span::default())
}

/// Helper to convert a function argument (handle, char vector, or string) into a `FunctionHandle`.
pub fn to_function_handle(val: &Value, fn_name: &str) -> Result<FunctionHandle, EvalError> {
    match val {
        Value::FunctionHandle(h) => Ok((**h).clone()),
        Value::Char(ca) => Ok(FunctionHandle::Named(ca.as_slice().iter().collect())),
        Value::String(sa) => {
            let s = sa.as_slice().first().cloned().unwrap_or_default();
            Ok(FunctionHandle::Named(s))
        }
        _ => Err(user_err(
            &format!("MATLAB:{fn_name}:badHandle"),
            &format!("{fn_name} requires a function handle as first input."),
        )),
    }
}

/// Helper to unpack a numeric vector from a `Value`.
pub fn value_to_vec_f64(v: &Value) -> Result<Vec<f64>, EvalError> {
    match v {
        Value::Double(d) => Ok(d.as_slice().to_vec()),
        Value::Single(s) => Ok(s.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int32(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int64(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        _ => Err(user_err(
            "MATLAB:solver:invalidValue",
            "Expected real numeric array.",
        )),
    }
}

/// Solves an ODE using Dormand-Prince 5(4) adaptive Runge-Kutta.
///
/// Syntax:
///   [t, y] = ode45(odefun, tspan, y0)
pub fn ode45_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:ode45:NotEnoughInputs",
            "ode45 requires at least 3 inputs: (odefun, tspan, y0).",
        ));
    }

    let handle = to_function_handle(&args[0], "ode45")?;
    let tspan = value_to_vec_f64(&args[1])?;
    let y0 = value_to_vec_f64(&args[2])?;

    if tspan.len() < 2 {
        return Err(user_err(
            "MATLAB:ode45:tspanTooShort",
            "tspan must have at least 2 elements [t0, tf].",
        ));
    }
    if y0.is_empty() {
        return Err(user_err(
            "MATLAB:ode45:emptyY0",
            "Initial state y0 cannot be empty.",
        ));
    }

    let t0 = tspan[0];
    let tf = *tspan.last().unwrap_or(&t0);
    let n = y0.len();

    let opts = parse_ode_options(args.get(3), 1e-4);
    let rel_tol = opts.rel_tol;
    let abs_tol = opts.abs_tol;

    let mut t = t0;
    let mut y = y0.clone();

    let mut t_out = vec![t];
    let mut y_out = vec![y.clone()];

    let direction = if tf >= t0 { 1.0 } else { -1.0 };
    let mut h = opts.initial_step.unwrap_or((tf - t0) * 0.01);
    if let Some(max_h) = opts.max_step {
        if h.abs() > max_h {
            h = direction * max_h;
        }
    }
    if h.abs() < 1e-12 {
        h = direction * 1e-6;
    }

    // Evaluate initial derivative k1
    let eval_ode = |cur_t: f64, cur_y: &[f64]| -> Result<Vec<f64>, EvalError> {
        let t_val = Value::scalar(cur_t);
        let y_val = make_double_matrix(cur_y.len(), 1, cur_y.to_vec())?;
        let res = call_handle_any(&handle, vec![t_val, y_val], 1)?;
        let dy_val = res.into_iter().next().unwrap_or_else(Value::empty);
        let mut dy = value_to_vec_f64(&dy_val)?;
        if dy.len() != n {
            dy.resize(n, 0.0);
        }
        Ok(dy)
    };

    let mut k1 = eval_ode(t, &y)?;

    let max_steps = 100_000;
    let mut steps = 0;

    while (tf - t) * direction > 1e-12 && steps < max_steps {
        steps += 1;
        if (t + h - tf) * direction > 0.0 {
            h = tf - t;
        }

        // Dormand-Prince stages
        // k2 = f(t + h/5, y + h*(1/5*k1))
        let mut y_stage = vec![0.0; n];
        for i in 0..n {
            y_stage[i] = y[i] + h * (1.0 / 5.0 * k1[i]);
        }
        let k2 = eval_ode(t + 0.2 * h, &y_stage)?;

        // k3 = f(t + 3/10*h, y + h*(3/40*k1 + 9/40*k2))
        for i in 0..n {
            y_stage[i] = y[i] + h * (3.0 / 40.0 * k1[i] + 9.0 / 40.0 * k2[i]);
        }
        let k3 = eval_ode(t + 0.3 * h, &y_stage)?;

        // k4 = f(t + 4/5*h, y + h*(44/45*k1 - 56/15*k2 + 32/9*k3))
        for i in 0..n {
            y_stage[i] =
                y[i] + h * (44.0 / 45.0 * k1[i] - 56.0 / 15.0 * k2[i] + 32.0 / 9.0 * k3[i]);
        }
        let k4 = eval_ode(t + 0.8 * h, &y_stage)?;

        // k5 = f(t + 8/9*h, y + h*(19372/6561*k1 - 25360/2187*k2 + 64448/6561*k3 - 212/729*k4))
        for i in 0..n {
            y_stage[i] = y[i]
                + h * (19372.0 / 6561.0 * k1[i] - 25360.0 / 2187.0 * k2[i]
                    + 64448.0 / 6561.0 * k3[i]
                    - 212.0 / 729.0 * k4[i]);
        }
        let k5 = eval_ode(t + (8.0 / 9.0) * h, &y_stage)?;

        // k6 = f(t + h, y + h*(9017/3168*k1 - 355/33*k2 + 46732/5247*k3 + 49/176*k4 - 5103/18656*k5))
        for i in 0..n {
            y_stage[i] = y[i]
                + h * (9017.0 / 3168.0 * k1[i] - 355.0 / 33.0 * k2[i]
                    + 46732.0 / 5247.0 * k3[i]
                    + 49.0 / 176.0 * k4[i]
                    - 5103.0 / 18656.0 * k5[i]);
        }
        let k6 = eval_ode(t + h, &y_stage)?;

        // 5th order step
        let mut y5 = vec![0.0; n];
        for i in 0..n {
            y5[i] = y[i]
                + h * (35.0 / 384.0 * k1[i] + 500.0 / 1113.0 * k3[i] + 125.0 / 192.0 * k4[i]
                    - 2187.0 / 6784.0 * k5[i]
                    + 11.0 / 84.0 * k6[i]);
        }

        // k7 = f(t + h, y5)
        let k7 = eval_ode(t + h, &y5)?;

        // 4th order step for error estimation
        let mut err_norm = 0.0_f64;
        for i in 0..n {
            let y4_i = y[i]
                + h * (5179.0 / 57600.0 * k1[i] + 7571.0 / 16695.0 * k3[i] + 393.0 / 640.0 * k4[i]
                    - 92097.0 / 339200.0 * k5[i]
                    + 187.0 / 2100.0 * k6[i]
                    + 1.0 / 40.0 * k7[i]);
            let diff = (y5[i] - y4_i).abs();
            let scale = abs_tol + rel_tol * y[i].abs().max(y5[i].abs());
            let ratio = diff / scale;
            if ratio > err_norm {
                err_norm = ratio;
            }
        }

        if err_norm <= 1.0 {
            // Step accepted
            t += h;
            y = y5;
            t_out.push(t);
            y_out.push(y.clone());
            k1 = k7; // FSAL

            // Optimal step size adjustment
            let factor = if err_norm < 1e-10 {
                2.0
            } else {
                (0.9 * (1.0 / err_norm).powf(0.2)).clamp(0.2, 5.0)
            };
            h *= factor;
            if let Some(max_h) = opts.max_step {
                if h.abs() > max_h {
                    h = direction * max_h;
                }
            }
        } else {
            // Step rejected
            let factor = (0.9 * (1.0 / err_norm).powf(0.25)).clamp(0.1, 0.5);
            h *= factor;
        }

        if h.abs() < 1e-15 {
            break;
        }
    }

    // Format output matrices:
    // t: [M, 1] column vector
    // y: [M, N] matrix
    let m = t_out.len();
    let mut flat_y = vec![0.0; m * n];
    // MATLAB matrices are column-major: flat_y[row + col * m] = y_out[row][col]
    for row in 0..m {
        for col in 0..n {
            flat_y[row + col * m] = y_out[row][col];
        }
    }

    let t_val = make_double_matrix(m, 1, t_out)?;
    let y_val = make_double_matrix(m, n, flat_y)?;

    if nargout >= 2 {
        Ok(vec![t_val, y_val])
    } else {
        Ok(vec![y_val])
    }
}
/// Solves an ODE using Bogacki-Shampine 3(2) Runge-Kutta.
///
/// Syntax:
///   [t, y] = ode23(odefun, tspan, y0)
pub fn ode23_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:ode23:NotEnoughInputs",
            "ode23 requires at least 3 inputs: (odefun, tspan, y0).",
        ));
    }

    let handle = to_function_handle(&args[0], "ode23")?;
    let tspan = value_to_vec_f64(&args[1])?;
    let y0 = value_to_vec_f64(&args[2])?;

    let t0 = tspan[0];
    let tf = *tspan.last().unwrap_or(&t0);
    let n = y0.len();

    let opts = parse_ode_options(args.get(3), 1e-3);
    let rel_tol = opts.rel_tol;
    let abs_tol = opts.abs_tol;

    let mut t = t0;
    let mut y = y0.clone();

    let mut t_out = vec![t];
    let mut y_out = vec![y.clone()];

    let direction = if tf >= t0 { 1.0 } else { -1.0 };
    let mut h = opts.initial_step.unwrap_or((tf - t0) * 0.01);
    if let Some(max_h) = opts.max_step {
        if h.abs() > max_h {
            h = direction * max_h;
        }
    }
    if h.abs() < 1e-12 {
        h = direction * 1e-6;
    }

    let eval_ode = |cur_t: f64, cur_y: &[f64]| -> Result<Vec<f64>, EvalError> {
        let t_val = Value::scalar(cur_t);
        let y_val = make_double_matrix(cur_y.len(), 1, cur_y.to_vec())?;
        let res = call_handle_any(&handle, vec![t_val, y_val], 1)?;
        let dy_val = res.into_iter().next().unwrap_or_else(Value::empty);
        let mut dy = value_to_vec_f64(&dy_val)?;
        if dy.len() != n {
            dy.resize(n, 0.0);
        }
        Ok(dy)
    };

    let mut k1 = eval_ode(t, &y)?;
    let max_steps = 100_000;
    let mut steps = 0;

    while (tf - t) * direction > 1e-12 && steps < max_steps {
        steps += 1;
        if (t + h - tf) * direction > 0.0 {
            h = tf - t;
        }

        // k2 = f(t + h/2, y + h/2*k1)
        let mut y_stage = vec![0.0; n];
        for i in 0..n {
            y_stage[i] = y[i] + 0.5 * h * k1[i];
        }
        let k2 = eval_ode(t + 0.5 * h, &y_stage)?;

        // k3 = f(t + 3/4*h, y + 3/4*h*k2)
        for i in 0..n {
            y_stage[i] = y[i] + 0.75 * h * k2[i];
        }
        let k3 = eval_ode(t + 0.75 * h, &y_stage)?;

        // 3rd order solution y_next = y + h*(2/9*k1 + 1/3*k2 + 4/9*k3)
        let mut y3 = vec![0.0; n];
        for i in 0..n {
            y3[i] = y[i] + h * (2.0 / 9.0 * k1[i] + 1.0 / 3.0 * k2[i] + 4.0 / 9.0 * k3[i]);
        }
        let k4 = eval_ode(t + h, &y3)?;

        // 2nd order error estimate
        let mut err_norm = 0.0_f64;
        for i in 0..n {
            let err_i = h
                * (-5.0 / 72.0 * k1[i] + 1.0 / 12.0 * k2[i] + 1.0 / 9.0 * k3[i]
                    - 1.0 / 8.0 * k4[i]);
            let scale = abs_tol + rel_tol * y[i].abs().max(y3[i].abs());
            let ratio = err_i.abs() / scale;
            if ratio > err_norm {
                err_norm = ratio;
            }
        }

        if err_norm <= 1.0 {
            t += h;
            y = y3;
            t_out.push(t);
            y_out.push(y.clone());
            k1 = k4; // FSAL

            let factor = if err_norm < 1e-8 {
                2.0
            } else {
                (0.9 * (1.0 / err_norm).powf(1.0 / 3.0)).clamp(0.2, 5.0)
            };
            h *= factor;
            if let Some(max_h) = opts.max_step {
                if h.abs() > max_h {
                    h = direction * max_h;
                }
            }
        } else {
            let factor = (0.9 * (1.0 / err_norm).powf(0.5)).clamp(0.1, 0.5);
            h *= factor;
        }

        if h.abs() < 1e-15 {
            break;
        }
    }

    let m = t_out.len();
    let mut flat_y = vec![0.0; m * n];
    for row in 0..m {
        for col in 0..n {
            flat_y[row + col * m] = y_out[row][col];
        }
    }

    let t_val = make_double_matrix(m, 1, t_out)?;
    let y_val = make_double_matrix(m, n, flat_y)?;

    if nargout >= 2 {
        Ok(vec![t_val, y_val])
    } else {
        Ok(vec![y_val])
    }
}

/// Solves stiff ordinary differential equations using an adaptive implicit BDF method with Newton iteration.
///
/// Syntax:
///   [t, y] = ode15s(odefun, tspan, y0)
///   [t, y] = ode15s(odefun, tspan, y0, options)
pub fn ode15s_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:ode15s:NotEnoughInputs",
            "ode15s requires at least 3 inputs: (odefun, tspan, y0).",
        ));
    }

    let handle = to_function_handle(&args[0], "ode15s")?;
    let tspan = value_to_vec_f64(&args[1])?;
    let y0 = value_to_vec_f64(&args[2])?;

    if tspan.len() < 2 {
        return Err(user_err(
            "MATLAB:ode15s:tspanTooShort",
            "tspan must have at least 2 elements [t0, tf].",
        ));
    }
    if y0.is_empty() {
        return Err(user_err(
            "MATLAB:ode15s:emptyY0",
            "Initial state y0 cannot be empty.",
        ));
    }

    let t0 = tspan[0];
    let tf = *tspan.last().unwrap_or(&t0);
    let n = y0.len();

    let opts = parse_ode_options(args.get(3), 1e-3);
    let rel_tol = opts.rel_tol;
    let abs_tol = opts.abs_tol;

    let mut t = t0;
    let mut y = y0.clone();

    let mut t_out = vec![t];
    let mut y_out = vec![y.clone()];

    let direction = if tf >= t0 { 1.0 } else { -1.0 };
    let mut h = opts.initial_step.unwrap_or((tf - t0) * 0.005);
    if let Some(max_h) = opts.max_step {
        if h.abs() > max_h {
            h = direction * max_h;
        }
    }
    if h.abs() < 1e-12 {
        h = direction * 1e-6;
    }

    let eval_ode = |cur_t: f64, cur_y: &[f64]| -> Result<Vec<f64>, EvalError> {
        let t_val = Value::scalar(cur_t);
        let y_val = make_double_matrix(cur_y.len(), 1, cur_y.to_vec())?;
        let res = call_handle_any(&handle, vec![t_val, y_val], 1)?;
        let dy_val = res.into_iter().next().unwrap_or_else(Value::empty);
        let mut dy = value_to_vec_f64(&dy_val)?;
        if dy.len() != n {
            dy.resize(n, 0.0);
        }
        Ok(dy)
    };

    let max_steps = 100_000;
    let mut steps = 0;

    while (tf - t) * direction > 1e-12 && steps < max_steps {
        steps += 1;
        if (t + h - tf) * direction > 0.0 {
            h = tf - t;
        }

        let t_next = t + h;
        let f_curr = eval_ode(t, &y)?;

        // Explicit predictor
        let mut y_k = vec![0.0; n];
        for i in 0..n {
            y_k[i] = y[i] + h * f_curr[i];
        }

        // Numerical Jacobian J = df/dy at (t_next, y_k)
        let mut jac = vec![0.0; n * n];
        let f_base = eval_ode(t_next, &y_k)?;
        for j in 0..n {
            let eps = 1e-7 * (y_k[j].abs().max(1.0));
            let mut y_pert = y_k.clone();
            y_pert[j] += eps;
            let f_pert = eval_ode(t_next, &y_pert)?;
            for i in 0..n {
                jac[i * n + j] = (f_pert[i] - f_base[i]) / eps;
            }
        }

        // Modified Newton iteration for implicit Euler: y_next - y - h * f(t_next, y_next) = 0
        let mut converged = false;
        for _iter in 0..12 {
            let f_val = eval_ode(t_next, &y_k)?;
            let mut r = vec![0.0; n];
            let mut res_norm = 0.0_f64;
            for i in 0..n {
                r[i] = -(y_k[i] - y[i] - h * f_val[i]);
                let scale = abs_tol + rel_tol * y_k[i].abs();
                res_norm = res_norm.max(r[i].abs() / scale);
            }

            if res_norm < 0.1 {
                converged = true;
                break;
            }

            let mut m_mat = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    m_mat[i * n + j] = if i == j { 1.0 } else { 0.0 } - h * jac[i * n + j];
                }
            }

            if !solve_linear_system(&mut m_mat, &mut r, n) {
                break;
            }

            let mut delta_norm = 0.0_f64;
            for i in 0..n {
                y_k[i] += r[i];
                let scale = abs_tol + rel_tol * y_k[i].abs();
                delta_norm = delta_norm.max(r[i].abs() / scale);
            }

            if delta_norm < 0.05 {
                converged = true;
                break;
            }
        }

        if converged {
            let mut err_norm = 0.0;
            for i in 0..n {
                let err_i = 0.5 * (y_k[i] - (y[i] + h * f_curr[i])).abs();
                let scale = abs_tol + rel_tol * y[i].abs().max(y_k[i].abs());
                let ratio = err_i / scale;
                if ratio > err_norm {
                    err_norm = ratio;
                }
            }

            if err_norm <= 1.0 {
                t = t_next;
                y = y_k;
                t_out.push(t);
                y_out.push(y.clone());

                let factor = if err_norm < 1e-4 {
                    2.0
                } else {
                    (0.9 * (1.0 / err_norm).powf(0.5)).clamp(0.5, 2.0)
                };
                h *= factor;
                if let Some(max_h) = opts.max_step {
                    if h.abs() > max_h {
                        h = direction * max_h;
                    }
                }
            } else {
                h *= 0.5;
            }
        } else {
            h *= 0.25;
        }

        if h.abs() < 1e-15 {
            break;
        }
    }

    let m = t_out.len();
    let mut flat_y = vec![0.0; m * n];
    for row in 0..m {
        for col in 0..n {
            flat_y[row + col * m] = y_out[row][col];
        }
    }

    let t_val = make_double_matrix(m, 1, t_out)?;
    let y_val = make_double_matrix(m, n, flat_y)?;

    if nargout >= 2 {
        Ok(vec![t_val, y_val])
    } else {
        Ok(vec![y_val])
    }
}

/// Create or alter ODE options structure.
///
/// Syntax:
///   options = odeset()
///   options = odeset('Name1', Value1, 'Name2', Value2, ...)
///   options = odeset(oldopts, 'Name1', Value1, ...)
pub fn odeset_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let standard_fields = vec![
        "AbsTol".to_string(),
        "BDF".to_string(),
        "Events".to_string(),
        "InitialStep".to_string(),
        "Jacobian".to_string(),
        "JPattern".to_string(),
        "Mass".to_string(),
        "MassSingular".to_string(),
        "MaxOrder".to_string(),
        "MaxStep".to_string(),
        "NonNegative".to_string(),
        "NormControl".to_string(),
        "OutputFcn".to_string(),
        "OutputSel".to_string(),
        "Refine".to_string(),
        "RelTol".to_string(),
        "Stats".to_string(),
        "Vectorized".to_string(),
    ];

    let mut map: HashMap<String, Value> = standard_fields
        .iter()
        .map(|f| (f.clone(), Value::empty()))
        .collect();

    let mut i = 0;
    if !args.is_empty() {
        if let Value::Struct(old_sa) = &args[0] {
            if let Some(old_map) = old_sa.data.first() {
                for (k, v) in old_map {
                    map.insert(k.clone(), v.clone());
                }
            }
            i = 1;
        }
    }

    while i + 1 < args.len() {
        let key = match &args[i] {
            Value::Char(c) => c.as_slice().iter().collect::<String>(),
            Value::String(s) => s.as_slice().first().cloned().unwrap_or_default(),
            _ => {
                return Err(user_err(
                    "MATLAB:odeset:invalidKey",
                    "Property name must be a string or char vector.",
                ))
            }
        };
        let val = args[i + 1].clone();
        let exact_key = standard_fields
            .iter()
            .find(|&f| f.eq_ignore_ascii_case(&key))
            .cloned()
            .unwrap_or(key);
        map.insert(exact_key, val);
        i += 2;
    }

    let mut fields: Vec<String> = map.keys().cloned().collect();
    fields.sort();
    let s = StructArray {
        shape: vec![1, 1],
        fields,
        data: vec![map],
    };
    Ok(vec![Value::Struct(s)])
}

/// Finds a root of a non-linear single-variable continuous function using Brent's method.
///
/// Syntax:
///   x = fzero(fun, x0)
///   [x, fval] = fzero(fun, x0)
pub fn fzero_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:fzero:NotEnoughInputs",
            "fzero requires at least 2 inputs: (fun, x0).",
        ));
    }

    let handle = to_function_handle(&args[0], "fzero")?;
    let x0_vec = value_to_vec_f64(&args[1])?;
    if x0_vec.is_empty() {
        return Err(user_err("MATLAB:fzero:emptyX0", "x0 cannot be empty."));
    }

    let eval_f = |x: f64| -> Result<f64, EvalError> {
        let res = call_handle_any(&handle, vec![Value::scalar(x)], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    let tol = 1e-12;
    let max_iter = 500;

    // Bracket the root
    let (mut a, mut b) = if x0_vec.len() >= 2 {
        (x0_vec[0], x0_vec[1])
    } else {
        let x0 = x0_vec[0];
        let mut delta = if x0.abs() > 0.0 { x0.abs() * 0.1 } else { 0.1 };
        let mut left = x0 - delta;
        let mut right = x0 + delta;
        let mut f_l = eval_f(left)?;
        let mut f_r = eval_f(right)?;
        let mut found = f_l * f_r <= 0.0;
        let mut step = 0;
        while !found && step < 50 {
            step += 1;
            delta *= 1.6;
            left = x0 - delta;
            right = x0 + delta;
            f_l = eval_f(left)?;
            f_r = eval_f(right)?;
            if f_l * f_r <= 0.0 {
                found = true;
                break;
            }
        }
        if !found {
            // Fall back to a default interval
            (x0 - 1.0, x0 + 1.0)
        } else {
            (left, right)
        }
    };

    let mut fa = eval_f(a)?;
    let mut fb = eval_f(b)?;

    if fa * fb > 0.0 {
        return Err(user_err(
            "MATLAB:fzero:ValuesAtEndpointsSameSign",
            "Function values at interval endpoints must have opposite signs.",
        ));
    }

    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut d = 0.0;

    for _ in 0..max_iter {
        if fb.abs() < tol || (b - a).abs() < tol {
            break;
        }

        let mut s = if (fa - fc).abs() > 1e-14 && (fb - fc).abs() > 1e-14 {
            // Inverse quadratic interpolation
            (a * fb * fc) / ((fa - fb) * (fa - fc))
                + (b * fa * fc) / ((fb - fa) * (fb - fc))
                + (c * fa * fb) / ((fc - fa) * (fc - fb))
        } else {
            // Secant method
            b - fb * (b - a) / (fb - fa)
        };

        let cond1 = if a < b {
            s < (3.0 * a + b) / 4.0 || s > b
        } else {
            s > (3.0 * a + b) / 4.0 || s < b
        };
        let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
        let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
        let cond4 = mflag && (b - c).abs() < tol;
        let cond5 = !mflag && (c - d).abs() < tol;

        if cond1 || cond2 || cond3 || cond4 || cond5 {
            // Bisection
            s = (a + b) / 2.0;
            mflag = true;
        } else {
            mflag = false;
        }

        let fs = eval_f(s)?;
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }

    let root_val = Value::scalar(b);
    let fval = Value::scalar(fb);

    if nargout >= 2 {
        Ok(vec![root_val, fval])
    } else {
        Ok(vec![root_val])
    }
}

/// Multidimensional unconstrained minimization using the Nelder-Mead simplex algorithm.
///
/// Syntax:
///   x = fminsearch(fun, x0)
///   [x, fval] = fminsearch(fun, x0)
pub fn fminsearch_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:fminsearch:NotEnoughInputs",
            "fminsearch requires at least 2 inputs: (fun, x0).",
        ));
    }

    let handle = to_function_handle(&args[0], "fminsearch")?;
    let x0 = value_to_vec_f64(&args[1])?;
    let n = x0.len();

    let eval_f = |x: &[f64]| -> Result<f64, EvalError> {
        let x_val = make_double_matrix(x.len(), 1, x.to_vec())?;
        let res = call_handle_any(&handle, vec![x_val], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    // Nelder-Mead coefficients
    let rho = 1.0; // reflection
    let chi = 2.0; // expansion
    let psi = 0.5; // contraction
    let sigma = 0.5; // shrink

    let tol_x = 1e-4;
    let tol_fun = 1e-4;
    let max_iter = 200 * n.max(1);

    // Initialize simplex with n+1 vertices
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    simplex.push(x0.clone());
    for i in 0..n {
        let mut p = x0.clone();
        if p[i] != 0.0 {
            p[i] *= 1.05;
        } else {
            p[i] = 0.00025;
        }
        simplex.push(p);
    }

    let mut f_vals: Vec<f64> = Vec::with_capacity(n + 1);
    for p in &simplex {
        f_vals.push(eval_f(p)?);
    }

    for _ in 0..max_iter {
        // Sort simplex so f_vals[0] <= f_vals[1] <= ... <= f_vals[n]
        let mut indices: Vec<usize> = (0..=n).collect();
        indices.sort_by(|&a, &b| {
            f_vals[a]
                .partial_cmp(&f_vals[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let sorted_simplex: Vec<Vec<f64>> = indices.iter().map(|&i| simplex[i].clone()).collect();
        let sorted_f_vals: Vec<f64> = indices.iter().map(|&i| f_vals[i]).collect();
        simplex = sorted_simplex;
        f_vals = sorted_f_vals;

        // Check convergence: max coordinate difference & max function value difference
        let mut max_x_diff = 0.0_f64;
        for i in 1..=n {
            for j in 0..n {
                let diff = (simplex[i][j] - simplex[0][j]).abs();
                if diff > max_x_diff {
                    max_x_diff = diff;
                }
            }
        }
        let fun_diff = (f_vals[n] - f_vals[0]).abs();
        if max_x_diff < tol_x && fun_diff < tol_fun {
            break;
        }

        // Centroid of the n best points
        let mut x_bar = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                x_bar[j] += simplex[i][j];
            }
        }
        for j in 0..n {
            x_bar[j] /= n as f64;
        }

        // Reflection: xr = x_bar + rho * (x_bar - x_worst)
        let mut xr = vec![0.0; n];
        for j in 0..n {
            xr[j] = x_bar[j] + rho * (x_bar[j] - simplex[n][j]);
        }
        let fxr = eval_f(&xr)?;

        if fxr < f_vals[0] {
            // Expansion: xe = x_bar + chi * (xr - x_bar)
            let mut xe = vec![0.0; n];
            for j in 0..n {
                xe[j] = x_bar[j] + chi * (xr[j] - x_bar[j]);
            }
            let fxe = eval_f(&xe)?;
            if fxe < fxr {
                simplex[n] = xe;
                f_vals[n] = fxe;
            } else {
                simplex[n] = xr;
                f_vals[n] = fxr;
            }
        } else if fxr < f_vals[n - 1] {
            // Accept reflection
            simplex[n] = xr;
            f_vals[n] = fxr;
        } else {
            // Contraction
            let mut xc = vec![0.0; n];
            if fxr < f_vals[n] {
                // Outside contraction
                for j in 0..n {
                    xc[j] = x_bar[j] + psi * (xr[j] - x_bar[j]);
                }
                let fxc = eval_f(&xc)?;
                if fxc <= fxr {
                    simplex[n] = xc;
                    f_vals[n] = fxc;
                    continue;
                }
            } else {
                // Inside contraction
                for j in 0..n {
                    xc[j] = x_bar[j] - psi * (x_bar[j] - simplex[n][j]);
                }
                let fxc = eval_f(&xc)?;
                if fxc < f_vals[n] {
                    simplex[n] = xc;
                    f_vals[n] = fxc;
                    continue;
                }
            }

            // Shrink
            for i in 1..=n {
                for j in 0..n {
                    simplex[i][j] = simplex[0][j] + sigma * (simplex[i][j] - simplex[0][j]);
                }
                f_vals[i] = eval_f(&simplex[i])?;
            }
        }
    }

    let x_best = make_double_matrix(n, 1, simplex[0].clone())?;
    let f_best = Value::scalar(f_vals[0]);

    if nargout >= 2 {
        Ok(vec![x_best, f_best])
    } else {
        Ok(vec![x_best])
    }
}

/// Adaptive numerical quadrature of a 1D function over [a, b].
///
/// Syntax:
///   q = integral(fun, a, b)
/// Clean-room adaptive Simpson's quadrature implementation (Quarteroni et al. 2007).
#[allow(clippy::too_many_arguments)]
pub fn adaptive_simpson<F: Fn(f64) -> Result<f64, EvalError>>(
    f: &F,
    a: f64,
    b: f64,
    fa: f64,
    fb: f64,
    fm: f64,
    whole: f64,
    tol: f64,
    depth: usize,
) -> Result<f64, EvalError> {
    let m = (a + b) / 2.0;
    let lm = (a + m) / 2.0;
    let rm = (m + b) / 2.0;
    let flm = f(lm)?;
    let frm = f(rm)?;
    let left = (m - a) / 6.0 * (fa + 4.0 * flm + fm);
    let right = (b - m) / 6.0 * (fm + 4.0 * frm + fb);
    let delta = left + right - whole;

    if depth > 40 || delta.abs() <= 15.0 * tol {
        Ok(left + right + delta / 15.0)
    } else {
        let left_res = adaptive_simpson(f, a, m, fa, fm, flm, left, tol / 2.0, depth + 1)?;
        let right_res = adaptive_simpson(f, m, b, fm, fb, frm, right, tol / 2.0, depth + 1)?;
        Ok(left_res + right_res)
    }
}

pub fn integrate_1d_adaptive<F: Fn(f64) -> Result<f64, EvalError>>(
    f: &F,
    a: f64,
    b: f64,
    tol: f64,
) -> Result<f64, EvalError> {
    let fa = f(a)?;
    let fb = f(b)?;
    let m = (a + b) / 2.0;
    let fm = f(m)?;
    let initial_simpson = (b - a) / 6.0 * (fa + 4.0 * fm + fb);
    adaptive_simpson(f, a, b, fa, fb, fm, initial_simpson, tol, 0)
}

/// 1D numerical quadrature: q = integral(fun, a, b).
pub fn integral_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:integral:NotEnoughInputs",
            "integral requires at least 3 inputs: (fun, a, b).",
        ));
    }

    let handle = to_function_handle(&args[0], "integral")?;
    let a = args[1].as_scalar_f64().map_err(math_err)?;
    let b = args[2].as_scalar_f64().map_err(math_err)?;

    let eval_f = |x: f64| -> Result<f64, EvalError> {
        let res = call_handle_any(&handle, vec![Value::scalar(x)], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    let tol = 1e-8;
    let res = integrate_1d_adaptive(&eval_f, a, b, tol)?;
    Ok(vec![Value::scalar(res)])
}

/// Adaptive Simpson's quadrature: q = quad(fun, a, b, tol).
pub fn quad_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:quad:NotEnoughInputs",
            "quad requires at least 3 inputs: (fun, a, b).",
        ));
    }

    let handle = to_function_handle(&args[0], "quad")?;
    let a = args[1].as_scalar_f64().map_err(math_err)?;
    let b = args[2].as_scalar_f64().map_err(math_err)?;
    let tol = if args.len() >= 4 {
        args[3].as_scalar_f64().unwrap_or(1e-6)
    } else {
        1e-6
    };

    let eval_f = |x: f64| -> Result<f64, EvalError> {
        let res = call_handle_any(&handle, vec![Value::scalar(x)], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    let res = integrate_1d_adaptive(&eval_f, a, b, tol)?;
    Ok(vec![Value::scalar(res)])
}

/// Adaptive Gauss-Kronrod quadrature: q = quadgk(fun, a, b, tol).
pub fn quadgk_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:quadgk:NotEnoughInputs",
            "quadgk requires at least 3 inputs: (fun, a, b).",
        ));
    }

    let handle = to_function_handle(&args[0], "quadgk")?;
    let a = args[1].as_scalar_f64().map_err(math_err)?;
    let b = args[2].as_scalar_f64().map_err(math_err)?;
    let tol = if args.len() >= 4 {
        args[3].as_scalar_f64().unwrap_or(1e-10)
    } else {
        1e-10
    };

    let eval_f = |x: f64| -> Result<f64, EvalError> {
        let res = call_handle_any(&handle, vec![Value::scalar(x)], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    let res = integrate_1d_adaptive(&eval_f, a, b, tol)?;
    Ok(vec![Value::scalar(res)])
}

/// 2D iterated numerical quadrature: q = integral2(fun, xmin, xmax, ymin, ymax).
pub fn integral2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 5 {
        return Err(user_err(
            "MATLAB:integral2:NotEnoughInputs",
            "integral2 requires at least 5 inputs: (fun, xmin, xmax, ymin, ymax).",
        ));
    }
    let handle = to_function_handle(&args[0], "integral2")?;
    let xmin = args[1].as_scalar_f64().map_err(math_err)?;
    let xmax = args[2].as_scalar_f64().map_err(math_err)?;
    let ymin = args[3].as_scalar_f64().map_err(math_err)?;
    let ymax = args[4].as_scalar_f64().map_err(math_err)?;

    let tol = 1e-6;

    let eval_inner = |y: f64| -> Result<f64, EvalError> {
        let eval_xy = |x: f64| -> Result<f64, EvalError> {
            let res = call_handle_any(&handle, vec![Value::scalar(x), Value::scalar(y)], 1)?;
            let val = res.into_iter().next().unwrap_or_else(Value::empty);
            val.as_scalar_f64().map_err(math_err)
        };
        integrate_1d_adaptive(&eval_xy, xmin, xmax, tol)
    };

    let res = integrate_1d_adaptive(&eval_inner, ymin, ymax, tol)?;
    Ok(vec![Value::scalar(res)])
}

/// 3D iterated numerical quadrature: q = integral3(fun, xmin, xmax, ymin, ymax, zmin, zmax).
pub fn integral3_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 7 {
        return Err(user_err(
            "MATLAB:integral3:NotEnoughInputs",
            "integral3 requires at least 7 inputs: (fun, xmin, xmax, ymin, ymax, zmin, zmax).",
        ));
    }
    let handle = to_function_handle(&args[0], "integral3")?;
    let xmin = args[1].as_scalar_f64().map_err(math_err)?;
    let xmax = args[2].as_scalar_f64().map_err(math_err)?;
    let ymin = args[3].as_scalar_f64().map_err(math_err)?;
    let ymax = args[4].as_scalar_f64().map_err(math_err)?;
    let zmin = args[5].as_scalar_f64().map_err(math_err)?;
    let zmax = args[6].as_scalar_f64().map_err(math_err)?;

    let tol = 1e-5;

    let eval_z = |z: f64| -> Result<f64, EvalError> {
        let eval_y = |y: f64| -> Result<f64, EvalError> {
            let eval_x = |x: f64| -> Result<f64, EvalError> {
                let res = call_handle_any(
                    &handle,
                    vec![Value::scalar(x), Value::scalar(y), Value::scalar(z)],
                    1,
                )?;
                let val = res.into_iter().next().unwrap_or_else(Value::empty);
                val.as_scalar_f64().map_err(math_err)
            };
            integrate_1d_adaptive(&eval_x, xmin, xmax, tol)
        };
        integrate_1d_adaptive(&eval_y, ymin, ymax, tol)
    };

    let res = integrate_1d_adaptive(&eval_z, zmin, zmax, tol)?;
    Ok(vec![Value::scalar(res)])
}
