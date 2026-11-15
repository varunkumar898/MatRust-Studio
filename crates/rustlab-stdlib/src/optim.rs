//! Numerical optimization solvers and options management.
//!
//! Provides clean-room MATLAB-compatible implementations of:
//! - `optimset`: Creation and inspection of optimization options structures.
//! - `fminbnd`: 1D bounded minimization via Brent's method (golden-section + parabolic interpolation).
//! - `fminunc`: Multivariable unconstrained minimization via BFGS quasi-Newton method with Armijo line search.
//! - `fsolve`: System of nonlinear equations solver via Levenberg-Marquardt.
//! - `lsqnonlin`: Nonlinear least-squares solver via Levenberg-Marquardt.
//! - `lsqcurvefit`: Nonlinear curve fitting via Levenberg-Marquardt.
//! - `linprog`: Linear programming via the Two-Phase Simplex algorithm with Bland's anti-cycling rule.
//!
//! Citations:
//! - Brent, R. P. (1973). Algorithms for Minimization without Derivatives. Prentice-Hall.
//! - Nocedal, J., & Wright, S. J. (2006). Numerical Optimization. 2nd ed. Springer.
//! - Bertsimas, D., & Tsitsiklis, J. N. (1997). Introduction to Linear Optimization. Athena Scientific.

use crate::helpers::{math_err, user_err};
use crate::solvers::{call_handle_any, solve_linear_system, to_function_handle, value_to_vec_f64};
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

/// Helper to create a Double 2D Value matrix (column vector if cols == 1).
fn make_double_matrix(rows: usize, cols: usize, data: Vec<f64>) -> Result<Value, EvalError> {
    let arr = DenseArray::from_vec(vec![rows, cols], data).map_err(math_err)?;
    Ok(Value::Double(arr))
}

/// Helper to create a standard output structure for optimization routines.
fn make_output_struct(
    iterations: usize,
    func_count: usize,
    algorithm: &str,
    message: &str,
) -> Value {
    let mut map = HashMap::new();
    map.insert("iterations".to_string(), Value::scalar(iterations as f64));
    map.insert("funcCount".to_string(), Value::scalar(func_count as f64));
    map.insert(
        "algorithm".to_string(),
        Value::String(DenseArray::scalar(algorithm.to_string())),
    );
    map.insert(
        "message".to_string(),
        Value::String(DenseArray::scalar(message.to_string())),
    );

    let sa = StructArray {
        fields: vec![
            "iterations".to_string(),
            "funcCount".to_string(),
            "algorithm".to_string(),
            "message".to_string(),
        ],
        shape: vec![1, 1],
        data: vec![map],
    };
    Value::Struct(sa)
}

/// Creates or modifies an optimization options structure.
///
/// Syntax:
///   options = optimset()
///   options = optimset('solver_name')
///   options = optimset('param1', value1, 'param2', value2, ...)
pub fn optimset_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let mut map: HashMap<String, Value> = HashMap::new();
    let fields = vec![
        "Display",
        "MaxIter",
        "MaxFunEvals",
        "TolX",
        "TolFun",
        "TolCon",
        "Algorithm",
        "LargeScale",
        "Jacobian",
        "GradObj",
        "Hessian",
    ];

    for &f in &fields {
        map.insert(f.to_string(), Value::empty());
    }

    if args.is_empty() {
        let sa = StructArray {
            fields: fields.into_iter().map(String::from).collect(),
            shape: vec![1, 1],
            data: vec![map],
        };
        return Ok(vec![Value::Struct(sa)]);
    }

    let mut start_idx = 0;
    if args.len() % 2 == 1 {
        // First argument is solver name or base options struct
        if let Value::Struct(ref sa) = args[0] {
            if let Some(base_map) = sa.data.first() {
                for (k, v) in base_map {
                    map.insert(k.clone(), v.clone());
                }
            }
            start_idx = 1;
        } else {
            let solver_name = match &args[0] {
                Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
                Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
                _ => String::new(),
            };
            match solver_name.to_ascii_lowercase().as_str() {
                "fzero" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolX".to_string(), Value::scalar(1e-12));
                    map.insert("MaxIter".to_string(), Value::scalar(500.0));
                }
                "fminbnd" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolX".to_string(), Value::scalar(1e-4));
                    map.insert("MaxIter".to_string(), Value::scalar(500.0));
                    map.insert("MaxFunEvals".to_string(), Value::scalar(500.0));
                }
                "fminsearch" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolX".to_string(), Value::scalar(1e-4));
                    map.insert("TolFun".to_string(), Value::scalar(1e-4));
                    map.insert("MaxIter".to_string(), Value::scalar(400.0));
                    map.insert("MaxFunEvals".to_string(), Value::scalar(400.0));
                }
                "fminunc" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolX".to_string(), Value::scalar(1e-6));
                    map.insert("TolFun".to_string(), Value::scalar(1e-6));
                    map.insert("MaxIter".to_string(), Value::scalar(400.0));
                    map.insert(
                        "Algorithm".to_string(),
                        Value::String(DenseArray::scalar("quasi-newton".to_string())),
                    );
                }
                "fsolve" | "lsqnonlin" | "lsqcurvefit" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolX".to_string(), Value::scalar(1e-6));
                    map.insert("TolFun".to_string(), Value::scalar(1e-6));
                    map.insert("MaxIter".to_string(), Value::scalar(400.0));
                    map.insert(
                        "Algorithm".to_string(),
                        Value::String(DenseArray::scalar("levenberg-marquardt".to_string())),
                    );
                }
                "linprog" => {
                    map.insert(
                        "Display".to_string(),
                        Value::String(DenseArray::scalar("notify".to_string())),
                    );
                    map.insert("TolFun".to_string(), Value::scalar(1e-8));
                    map.insert(
                        "Algorithm".to_string(),
                        Value::String(DenseArray::scalar("simplex".to_string())),
                    );
                }
                _ => {}
            }
            start_idx = 1;
        }
    }

    let mut i = start_idx;
    while i + 1 < args.len() {
        let key = match &args[i] {
            Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
            Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
            _ => {
                return Err(user_err(
                    "MATLAB:optimset:invalidKey",
                    "Expected string parameter name in optimset.",
                ))
            }
        };
        // Normalize common keys case-insensitively
        let canonical_key = fields
            .iter()
            .find(|&&f| f.eq_ignore_ascii_case(&key))
            .map(|&f| f.to_string())
            .unwrap_or(key);

        map.insert(canonical_key, args[i + 1].clone());
        i += 2;
    }

    let all_keys: Vec<String> = map.keys().cloned().collect();
    let sa = StructArray {
        fields: all_keys,
        shape: vec![1, 1],
        data: vec![map],
    };
    Ok(vec![Value::Struct(sa)])
}

/// 1D bounded minimization using Brent's method (golden-section and parabolic interpolation).
///
/// Syntax:
///   x = fminbnd(fun, x1, x2)
///   [x, fval, exitflag, output] = fminbnd(fun, x1, x2, options)
pub fn fminbnd_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:fminbnd:NotEnoughInputs",
            "fminbnd requires at least 3 inputs: (fun, x1, x2).",
        ));
    }
    let handle = to_function_handle(&args[0], "fminbnd")?;
    let mut a = args[1].as_scalar_f64().map_err(math_err)?;
    let mut b = args[2].as_scalar_f64().map_err(math_err)?;
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }

    let tol = 1e-6;
    let max_iter = 500;
    let c = 0.5 * (3.0 - 5.0_f64.sqrt()); // golden section ratio: ~0.381966

    let mut func_count = 0;
    let eval_f = |x: f64, count: &mut usize| -> Result<f64, EvalError> {
        *count += 1;
        let res = call_handle_any(&handle, vec![Value::scalar(x)], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    let mut x = a + c * (b - a);
    let mut w = x;
    let mut v = w;
    let mut fx = eval_f(x, &mut func_count)?;
    let mut fw = fx;
    let mut fv = fx;
    let mut d: f64 = 0.0;
    let mut e: f64 = 0.0;
    let mut iter = 0;
    let mut exitflag = 1.0;

    for k in 0..max_iter {
        iter = k + 1;
        let m = 0.5 * (a + b);
        let tol1 = tol * x.abs() + 1e-10;
        let tol2 = 2.0 * tol1;

        if (x - m).abs() <= tol2 - 0.5 * (b - a) {
            break;
        }

        let mut p = 0.0;
        let mut q = 0.0;
        let mut r = 0.0;

        if e.abs() > tol1 {
            // Fit parabola
            r = (x - w) * (fx - fv);
            q = (x - v) * (fx - fw);
            p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);
            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }
            r = e;
            e = d;
        }

        if p.abs() < (0.5 * q * r).abs() && p > q * (a - x) && p < q * (b - x) {
            // Parabolic interpolation step
            d = p / q;
            let u = x + d;
            if (u - a) < tol2 || (b - u) < tol2 {
                d = if x < m { tol1 } else { -tol1 };
            }
        } else {
            // Golden section step
            e = if x < m { b - x } else { a - x };
            d = c * e;
        }

        let u = if d.abs() >= tol1 {
            x + d
        } else if d > 0.0 {
            x + tol1
        } else {
            x - tol1
        };

        let fu = eval_f(u, &mut func_count)?;

        if fu <= fx {
            if u >= x {
                a = x;
            } else {
                b = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || w == x {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || v == x || v == w {
                v = u;
                fv = fu;
            }
        }
    }

    if iter >= max_iter {
        exitflag = 0.0;
    }

    let mut out = vec![Value::scalar(x)];
    if nargout > 1 {
        out.push(Value::scalar(fx));
    }
    if nargout > 2 {
        out.push(Value::scalar(exitflag));
    }
    if nargout > 3 {
        let output = make_output_struct(
            iter,
            func_count,
            "golden-section/parabolic",
            "Optimization terminated.",
        );
        out.push(output);
    }
    Ok(out)
}

/// Multivariable unconstrained minimization via BFGS quasi-Newton method with Armijo backtracking line search.
///
/// Syntax:
///   x = fminunc(fun, x0)
///   [x, fval, exitflag, output, grad, hessian] = fminunc(fun, x0, options)
pub fn fminunc_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:fminunc:NotEnoughInputs",
            "fminunc requires at least 2 inputs: (fun, x0).",
        ));
    }
    let handle = to_function_handle(&args[0], "fminunc")?;
    let mut x = value_to_vec_f64(&args[1])?;
    let n = x.len();
    if n == 0 {
        return Err(user_err("MATLAB:fminunc:emptyX0", "x0 cannot be empty."));
    }

    let mut func_count = 0;
    let eval_f = |v: &[f64], count: &mut usize| -> Result<f64, EvalError> {
        *count += 1;
        let x_mat = make_double_matrix(v.len(), 1, v.to_vec())?;
        let res = call_handle_any(&handle, vec![x_mat], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        val.as_scalar_f64().map_err(math_err)
    };

    // Helper to evaluate numerical gradient via central differences
    let eval_grad = |v: &[f64], count: &mut usize| -> Result<Vec<f64>, EvalError> {
        let mut g = vec![0.0; n];
        let mut v_tmp = v.to_vec();
        for i in 0..n {
            let h = 1e-7 * v[i].abs().max(1.0);
            v_tmp[i] = v[i] + h;
            let f_plus = eval_f(&v_tmp, count)?;
            v_tmp[i] = v[i] - h;
            let f_minus = eval_f(&v_tmp, count)?;
            v_tmp[i] = v[i];
            g[i] = (f_plus - f_minus) / (2.0 * h);
        }
        Ok(g)
    };

    let tol_fun = 1e-6;
    let tol_x = 1e-6;
    let max_iter = 200 * n.max(1);

    // Inverse Hessian approximation H initialized to identity (n x n, row-major)
    let mut h_inv = vec![0.0; n * n];
    for i in 0..n {
        h_inv[i * n + i] = 1.0;
    }

    let mut fx = eval_f(&x, &mut func_count)?;
    let mut g = eval_grad(&x, &mut func_count)?;
    let mut iter = 0;
    let mut exitflag = 1.0;

    for k in 0..max_iter {
        iter = k + 1;
        // Check gradient convergence ||g||_inf <= tol_fun
        let g_norm = g.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if g_norm <= tol_fun {
            break;
        }

        // Search direction p = -H * g
        let mut p = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                p[i] -= h_inv[i * n + j] * g[j];
            }
        }

        // Verify descent direction p^T * g < 0
        let g_dot_p: f64 = g.iter().zip(&p).map(|(&gi, &pi)| gi * pi).sum();
        if g_dot_p >= 0.0 {
            // Reset H to identity and p = -g
            for i in 0..n {
                for j in 0..n {
                    h_inv[i * n + j] = if i == j { 1.0 } else { 0.0 };
                }
                p[i] = -g[i];
            }
        }
        let g_dot_p: f64 = g.iter().zip(&p).map(|(&gi, &pi)| gi * pi).sum();

        // Armijo backtracking line search
        let mut alpha = 1.0;
        let c1 = 1e-4;
        let mut x_new = vec![0.0; n];
        let mut fx_new = fx;

        for _ in 0..30 {
            for i in 0..n {
                x_new[i] = x[i] + alpha * p[i];
            }
            fx_new = eval_f(&x_new, &mut func_count)?;
            if fx_new <= fx + c1 * alpha * g_dot_p {
                break;
            }
            alpha *= 0.5;
            if alpha < 1e-12 {
                break;
            }
        }

        // Step s = x_new - x
        let mut s = vec![0.0; n];
        let mut s_norm = 0.0_f64;
        for i in 0..n {
            s[i] = x_new[i] - x[i];
            s_norm = s_norm.max(s[i].abs());
        }

        let g_new = eval_grad(&x_new, &mut func_count)?;
        let mut y = vec![0.0; n];
        for i in 0..n {
            y[i] = g_new[i] - g[i];
        }

        let y_dot_s: f64 = y.iter().zip(&s).map(|(&yi, &si)| yi * si).sum();

        // BFGS update of H_inv if curvature condition y^T * s > 0 holds
        if y_dot_s > 1e-12 {
            let rho = 1.0 / y_dot_s;
            // V = (I - rho * s * y^T)
            let mut v_mat = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    let delta = if i == j { 1.0 } else { 0.0 };
                    v_mat[i * n + j] = delta - rho * s[i] * y[j];
                }
            }
            // H_next = V * H * V^T + rho * s * s^T
            let mut tmp = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    for m in 0..n {
                        tmp[i * n + j] += v_mat[i * n + m] * h_inv[m * n + j];
                    }
                }
            }
            let mut h_next = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    for m in 0..n {
                        h_next[i * n + j] += tmp[i * n + m] * v_mat[j * n + m];
                    }
                    h_next[i * n + j] += rho * s[i] * s[j];
                }
            }
            h_inv = h_next;
        }

        x = x_new;
        fx = fx_new;
        g = g_new;

        if s_norm <= tol_x {
            break;
        }
    }

    if iter >= max_iter {
        exitflag = 0.0;
    }

    let x_out = make_double_matrix(n, 1, x)?;
    let mut out = vec![x_out];
    if nargout > 1 {
        out.push(Value::scalar(fx));
    }
    if nargout > 2 {
        out.push(Value::scalar(exitflag));
    }
    if nargout > 3 {
        let output = make_output_struct(
            iter,
            func_count,
            "quasi-newton BFGS",
            "Local minimum found.",
        );
        out.push(output);
    }
    if nargout > 4 {
        let g_out = make_double_matrix(n, 1, g)?;
        out.push(g_out);
    }
    if nargout > 5 {
        // Approximate Hessian from H_inv
        let mut h_mat = h_inv.clone();
        let mut b_eye = vec![0.0; n * n];
        for i in 0..n {
            b_eye[i * n + i] = 1.0;
        }
        let _ = solve_linear_system(&mut h_mat, &mut b_eye, n);
        let hess_out = make_double_matrix(n, n, b_eye)?;
        out.push(hess_out);
    }
    Ok(out)
}

/// Solves a system of nonlinear equations F(x) = 0 via Levenberg-Marquardt.
///
/// Syntax:
///   x = fsolve(fun, x0)
///   [x, fval, exitflag, output, jacobian] = fsolve(fun, x0, options)
pub fn fsolve_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:fsolve:NotEnoughInputs",
            "fsolve requires at least 2 inputs: (fun, x0).",
        ));
    }
    let handle = to_function_handle(&args[0], "fsolve")?;
    let mut x = value_to_vec_f64(&args[1])?;
    let n = x.len();
    if n == 0 {
        return Err(user_err("MATLAB:fsolve:emptyX0", "x0 cannot be empty."));
    }

    let mut func_count = 0;
    let eval_r = |v: &[f64], count: &mut usize| -> Result<Vec<f64>, EvalError> {
        *count += 1;
        let x_mat = make_double_matrix(v.len(), 1, v.to_vec())?;
        let res = call_handle_any(&handle, vec![x_mat], 1)?;
        let val = res.into_iter().next().unwrap_or_else(Value::empty);
        value_to_vec_f64(&val)
    };

    let tol_fun = 1e-6;
    let tol_x = 1e-6;
    let max_iter = 400;

    let mut r = eval_r(&x, &mut func_count)?;
    let m = r.len();
    let mut lambda = 1e-2;
    let mut iter = 0;
    let mut exitflag = 1.0;

    for k in 0..max_iter {
        iter = k + 1;
        // Check residual norm
        let r_max = r.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if r_max <= tol_fun {
            break;
        }

        // Numerical Jacobian J (m x n, row-major)
        let mut j_mat = vec![0.0; m * n];
        let mut x_tmp = x.clone();
        for col in 0..n {
            let h = 1e-7 * x[col].abs().max(1.0);
            x_tmp[col] = x[col] + h;
            let r_plus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] = x[col] - h;
            let r_minus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] = x[col];

            for row in 0..m {
                j_mat[row * n + col] = (r_plus[row] - r_minus[row]) / (2.0 * h);
            }
        }

        // Normal equations: (J^T * J + lambda * I) * delta = -J^T * r
        let mut a_mat = vec![0.0; n * n];
        let mut b_vec = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for row in 0..m {
                    sum += j_mat[row * n + i] * j_mat[row * n + j];
                }
                if i == j {
                    sum += lambda;
                }
                a_mat[i * n + j] = sum;
            }
            let mut rhs = 0.0;
            for row in 0..m {
                rhs -= j_mat[row * n + i] * r[row];
            }
            b_vec[i] = rhs;
        }

        let mut delta = b_vec.clone();
        let solved = solve_linear_system(&mut a_mat, &mut delta, n);
        if !solved {
            lambda *= 10.0;
            continue;
        }

        let mut x_cand = vec![0.0; n];
        for i in 0..n {
            x_cand[i] = x[i] + delta[i];
        }
        let r_cand = eval_r(&x_cand, &mut func_count)?;

        let cost_curr: f64 = r.iter().map(|v| v * v).sum();
        let cost_cand: f64 = r_cand.iter().map(|v| v * v).sum();

        if cost_cand < cost_curr {
            // Successful step
            x = x_cand;
            r = r_cand;
            lambda = (lambda * 0.2).max(1e-7);

            let delta_norm = delta.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
            if delta_norm <= tol_x {
                break;
            }
        } else {
            // Reject step, increase damping
            lambda = (lambda * 5.0).min(1e7);
        }
    }

    if iter >= max_iter {
        exitflag = 0.0;
    }

    let x_out = make_double_matrix(n, 1, x)?;
    let mut out = vec![x_out];
    if nargout > 1 {
        let fval_out = make_double_matrix(m, 1, r)?;
        out.push(fval_out);
    }
    if nargout > 2 {
        out.push(Value::scalar(exitflag));
    }
    if nargout > 3 {
        let output =
            make_output_struct(iter, func_count, "levenberg-marquardt", "Equation solved.");
        out.push(output);
    }
    if nargout > 4 {
        // Re-evaluate Jacobian at solution
        let mut j_mat = vec![0.0; m * n];
        let mut x_tmp = value_to_vec_f64(&out[0])?;
        for col in 0..n {
            let h = 1e-7 * x_tmp[col].abs().max(1.0);
            x_tmp[col] += h;
            let r_plus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] -= 2.0 * h;
            let r_minus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] += h;

            for row in 0..m {
                j_mat[row * n + col] = (r_plus[row] - r_minus[row]) / (2.0 * h);
            }
        }
        let j_out = make_double_matrix(m, n, j_mat)?;
        out.push(j_out);
    }
    Ok(out)
}

/// Solves nonlinear least-squares problem min sum(F_i(x)^2) via Levenberg-Marquardt.
///
/// Syntax:
///   x = lsqnonlin(fun, x0)
///   [x, resnorm, residual, exitflag, output] = lsqnonlin(fun, x0, lb, ub, options)
pub fn lsqnonlin_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:lsqnonlin:NotEnoughInputs",
            "lsqnonlin requires at least 2 inputs: (fun, x0).",
        ));
    }
    let res = fsolve_fn(args, 5)?;
    let x_out = res[0].clone();
    let r_out = res.get(1).cloned().unwrap_or_else(Value::empty);
    let r_vec = value_to_vec_f64(&r_out).unwrap_or_default();
    let resnorm: f64 = r_vec.iter().map(|&v| v * v).sum();
    let exitflag = res.get(2).cloned().unwrap_or_else(|| Value::scalar(1.0));
    let output = res.get(3).cloned().unwrap_or_else(Value::empty);

    let mut out = vec![x_out];
    if nargout > 1 {
        out.push(Value::scalar(resnorm));
    }
    if nargout > 2 {
        out.push(r_out);
    }
    if nargout > 3 {
        out.push(exitflag);
    }
    if nargout > 4 {
        out.push(output);
    }
    Ok(out)
}

/// Nonlinear curve fitting in least-squares sense: min sum((fun(x, xdata) - ydata)^2).
///
/// Syntax:
///   x = lsqcurvefit(fun, x0, xdata, ydata)
///   [x, resnorm, residual, exitflag, output] = lsqcurvefit(fun, x0, xdata, ydata, lb, ub, options)
pub fn lsqcurvefit_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 4 {
        return Err(user_err(
            "MATLAB:lsqcurvefit:NotEnoughInputs",
            "lsqcurvefit requires at least 4 inputs: (fun, x0, xdata, ydata).",
        ));
    }
    let handle = to_function_handle(&args[0], "lsqcurvefit")?;
    let mut x = value_to_vec_f64(&args[1])?;
    let xdata = args[2].clone();
    let ydata = value_to_vec_f64(&args[3])?;
    let n = x.len();
    let m = ydata.len();

    let mut func_count = 0;
    let eval_r = |v: &[f64], count: &mut usize| -> Result<Vec<f64>, EvalError> {
        *count += 1;
        let x_mat = make_double_matrix(v.len(), 1, v.to_vec())?;
        let res = call_handle_any(&handle, vec![x_mat, xdata.clone()], 1)?;
        let y_pred_val = res.into_iter().next().unwrap_or_else(Value::empty);
        let y_pred = value_to_vec_f64(&y_pred_val)?;
        let mut r = Vec::with_capacity(m);
        for i in 0..m {
            let pred = y_pred.get(i).copied().unwrap_or(0.0);
            r.push(pred - ydata[i]);
        }
        Ok(r)
    };

    let tol_fun = 1e-6;
    let max_iter = 400;

    let mut r = eval_r(&x, &mut func_count)?;
    let mut lambda = 1e-2;
    let mut iter = 0;
    let mut exitflag = 1.0;

    for k in 0..max_iter {
        iter = k + 1;
        let r_max = r.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if r_max <= tol_fun {
            break;
        }

        // Numerical Jacobian J (m x n)
        let mut j_mat = vec![0.0; m * n];
        let mut x_tmp = x.clone();
        for col in 0..n {
            let h = 1e-7 * x[col].abs().max(1.0);
            x_tmp[col] = x[col] + h;
            let r_plus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] = x[col] - h;
            let r_minus = eval_r(&x_tmp, &mut func_count)?;
            x_tmp[col] = x[col];

            for row in 0..m {
                j_mat[row * n + col] = (r_plus[row] - r_minus[row]) / (2.0 * h);
            }
        }

        let mut a_mat = vec![0.0; n * n];
        let mut b_vec = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for row in 0..m {
                    sum += j_mat[row * n + i] * j_mat[row * n + j];
                }
                if i == j {
                    sum += lambda;
                }
                a_mat[i * n + j] = sum;
            }
            let mut rhs = 0.0;
            for row in 0..m {
                rhs -= j_mat[row * n + i] * r[row];
            }
            b_vec[i] = rhs;
        }

        let mut delta = b_vec.clone();
        let solved = solve_linear_system(&mut a_mat, &mut delta, n);
        if !solved {
            lambda *= 10.0;
            continue;
        }

        let mut x_cand = vec![0.0; n];
        for i in 0..n {
            x_cand[i] = x[i] + delta[i];
        }
        let r_cand = eval_r(&x_cand, &mut func_count)?;

        let cost_curr: f64 = r.iter().map(|v| v * v).sum();
        let cost_cand: f64 = r_cand.iter().map(|v| v * v).sum();

        if cost_cand < cost_curr {
            x = x_cand;
            r = r_cand;
            lambda = (lambda * 0.2).max(1e-7);
        } else {
            lambda = (lambda * 5.0).min(1e7);
        }
    }

    if iter >= max_iter {
        exitflag = 0.0;
    }

    let resnorm: f64 = r.iter().map(|v| v * v).sum();
    let x_out = make_double_matrix(n, 1, x)?;
    let mut out = vec![x_out];
    if nargout > 1 {
        out.push(Value::scalar(resnorm));
    }
    if nargout > 2 {
        let r_out = make_double_matrix(m, 1, r)?;
        out.push(r_out);
    }
    if nargout > 3 {
        out.push(Value::scalar(exitflag));
    }
    if nargout > 4 {
        let output = make_output_struct(
            iter,
            func_count,
            "levenberg-marquardt",
            "Curve fit converged.",
        );
        out.push(output);
    }
    Ok(out)
}

/// Linear programming via the Two-Phase Simplex algorithm with Bland's rule.
///
/// Syntax:
///   [x, fval, exitflag, output] = linprog(f, A, b, Aeq, beq, lb, ub)
pub fn linprog_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:linprog:NotEnoughInputs",
            "linprog requires at least 1 input: (f).",
        ));
    }
    let c = value_to_vec_f64(&args[0])?;
    let n_vars = c.len();

    // Parse inequality constraints A * x <= b
    let (a_ineq, b_ineq, m_ineq) = if args.len() > 2 && !args[1].is_empty() && !args[2].is_empty() {
        let a_vals = value_to_vec_f64(&args[1])?;
        let b_vals = value_to_vec_f64(&args[2])?;
        let m = b_vals.len();
        (a_vals, b_vals, m)
    } else {
        (vec![], vec![], 0)
    };

    // Parse equality constraints Aeq * x == beq
    let (a_eq, b_eq, m_eq) = if args.len() > 4 && !args[3].is_empty() && !args[4].is_empty() {
        let a_vals = value_to_vec_f64(&args[3])?;
        let b_vals = value_to_vec_f64(&args[4])?;
        let m = b_vals.len();
        (a_vals, b_vals, m)
    } else {
        (vec![], vec![], 0)
    };

    // Lower bounds lb <= x
    let lb = if args.len() > 5 && !args[5].is_empty() {
        value_to_vec_f64(&args[5])?
    } else {
        vec![0.0; n_vars]
    };

    // Upper bounds x <= ub
    let ub = if args.len() > 6 && !args[6].is_empty() {
        Some(value_to_vec_f64(&args[6])?)
    } else {
        None
    };

    // Total rows for standard form:
    // m_ineq (from A*x <= b)
    // + m_eq (from Aeq*x == beq)
    // + extra inequality rows if ub is provided
    let m_ub = if let Some(ref u) = ub { u.len() } else { 0 };
    let m_total = m_ineq + m_eq + m_ub;

    // Standard form formulation with shift x_orig = x' + lb:
    // min c^T x' + c^T lb
    // subject to A' x' <= b'
    // Total variables: n_vars (x') + m_ineq (slack) + m_ub (slack) + artificial variables
    let num_slack = m_ineq + m_ub;
    let total_cols = n_vars + num_slack + m_total + 1; // last col is RHS b

    // Tableau: (m_total + 1) rows x total_cols columns
    // Row 0: Phase I objective
    // Rows 1..=m_total: constraints
    let mut tableau = vec![0.0; (m_total + 2) * total_cols];
    let row_len = total_cols;

    let mut current_row = 1;
    let mut slack_col = n_vars;
    let mut artif_col = n_vars + num_slack;

    // Populate inequality rows A * (x' + lb) <= b => A * x' + s = b - A * lb
    for i in 0..m_ineq {
        let mut rhs = b_ineq[i];
        for j in 0..n_vars {
            let a_ij = a_ineq[j * m_ineq + i];
            tableau[current_row * row_len + j] = a_ij;
            let lb_j = lb.get(j).copied().unwrap_or(0.0);
            rhs -= a_ij * lb_j;
        }
        tableau[current_row * row_len + slack_col] = 1.0;
        slack_col += 1;

        if rhs < 0.0 {
            // Flip signs so RHS >= 0
            for j in 0..total_cols {
                tableau[current_row * row_len + j] = -tableau[current_row * row_len + j];
            }
            rhs = -rhs;
            // Needs artificial variable because slack has -1
            tableau[current_row * row_len + artif_col] = 1.0;
            artif_col += 1;
        }
        tableau[current_row * row_len + total_cols - 1] = rhs;
        current_row += 1;
    }

    // Populate upper bounds x' <= ub - lb
    if let Some(ref u) = ub {
        for j in 0..n_vars {
            let lb_j = lb.get(j).copied().unwrap_or(0.0);
            let rhs = u[j] - lb_j;
            tableau[current_row * row_len + j] = 1.0;
            tableau[current_row * row_len + slack_col] = 1.0;
            slack_col += 1;
            tableau[current_row * row_len + total_cols - 1] = rhs;
            current_row += 1;
        }
    }

    // Populate equality constraints Aeq * (x' + lb) == beq
    for i in 0..m_eq {
        let mut rhs = b_eq[i];
        for j in 0..n_vars {
            let a_ij = a_eq[j * m_eq + i];
            tableau[current_row * row_len + j] = a_ij;
            let lb_j = lb.get(j).copied().unwrap_or(0.0);
            rhs -= a_ij * lb_j;
        }
        if rhs < 0.0 {
            for j in 0..total_cols {
                tableau[current_row * row_len + j] = -tableau[current_row * row_len + j];
            }
            rhs = -rhs;
        }
        tableau[current_row * row_len + artif_col] = 1.0;
        artif_col += 1;
        tableau[current_row * row_len + total_cols - 1] = rhs;
        current_row += 1;
    }

    let num_artif = artif_col - (n_vars + num_slack);
    let mut basis: Vec<usize> = Vec::with_capacity(m_total);

    // Initial basis assignment
    let mut cur_slack = n_vars;
    let mut cur_art = n_vars + num_slack;
    for r in 1..=m_total {
        let is_art = (n_vars + num_slack..artif_col).any(|c| tableau[r * row_len + c] == 1.0);
        if is_art {
            basis.push(cur_art);
            cur_art += 1;
        } else {
            basis.push(cur_slack);
            cur_slack += 1;
        }
    }

    // Phase I objective: min sum(artificials) => row 0 = - sum(artificial rows)
    if num_artif > 0 {
        for r in 1..=m_total {
            if basis[r - 1] >= n_vars + num_slack {
                for col in 0..total_cols {
                    tableau[col] -= tableau[r * row_len + col];
                }
            }
        }

        // Simplex iterations for Phase I
        for _ in 0..1000 {
            // Find pivot column by Bland's rule: smallest index with negative reduced cost
            let mut pivot_col = None;
            for col in 0..(total_cols - 1) {
                if tableau[col] < -1e-8 {
                    pivot_col = Some(col);
                    break;
                }
            }
            let Some(p_col) = pivot_col else {
                break; // Phase I optimal
            };

            // Minimum ratio test by Bland's rule
            let mut min_ratio = f64::INFINITY;
            let mut pivot_row = None;
            for r in 1..=m_total {
                let a_val = tableau[r * row_len + p_col];
                if a_val > 1e-9 {
                    let rhs = tableau[r * row_len + total_cols - 1];
                    let ratio = rhs / a_val;
                    if ratio < min_ratio - 1e-9 {
                        min_ratio = ratio;
                        pivot_row = Some(r);
                    }
                }
            }
            let Some(p_row) = pivot_row else {
                break;
            };

            // Pivot on (p_row, p_col)
            let pivot_val = tableau[p_row * row_len + p_col];
            for col in 0..total_cols {
                tableau[p_row * row_len + col] /= pivot_val;
            }
            for r in 0..=m_total {
                if r != p_row {
                    let factor = tableau[r * row_len + p_col];
                    if factor.abs() > 1e-15 {
                        for col in 0..total_cols {
                            tableau[r * row_len + col] -= factor * tableau[p_row * row_len + col];
                        }
                    }
                }
            }
            basis[p_row - 1] = p_col;
        }

        let phase1_cost = tableau[total_cols - 1];
        if phase1_cost.abs() > 1e-4 {
            // Infeasible
            return Ok(vec![Value::empty(), Value::empty(), Value::scalar(-2.0)]);
        }
    }

    // Phase II: Restore original objective c in row 0
    for col in 0..total_cols {
        tableau[col] = 0.0;
    }
    tableau[..n_vars].copy_from_slice(&c[..n_vars]);
    // Adjust row 0 for current basic variables
    for r in 1..=m_total {
        let basic_var = basis[r - 1];
        if basic_var < n_vars {
            let cost = c[basic_var];
            for col in 0..total_cols {
                tableau[col] -= cost * tableau[r * row_len + col];
            }
        }
    }

    // Simplex iterations for Phase II
    let mut exitflag = 1.0;
    for _ in 0..1000 {
        let mut pivot_col = None;
        for col in 0..(n_vars + num_slack) {
            if tableau[col] < -1e-8 {
                pivot_col = Some(col);
                break;
            }
        }
        let Some(p_col) = pivot_col else {
            break; // Optimal found
        };

        let mut min_ratio = f64::INFINITY;
        let mut pivot_row = None;
        for r in 1..=m_total {
            let a_val = tableau[r * row_len + p_col];
            if a_val > 1e-9 {
                let rhs = tableau[r * row_len + total_cols - 1];
                let ratio = rhs / a_val;
                if ratio < min_ratio - 1e-9 {
                    min_ratio = ratio;
                    pivot_row = Some(r);
                }
            }
        }
        let Some(p_row) = pivot_row else {
            // Unbounded
            exitflag = -3.0;
            break;
        };

        let pivot_val = tableau[p_row * row_len + p_col];
        for col in 0..total_cols {
            tableau[p_row * row_len + col] /= pivot_val;
        }
        for r in 0..=m_total {
            if r != p_row {
                let factor = tableau[r * row_len + p_col];
                if factor.abs() > 1e-15 {
                    for col in 0..total_cols {
                        tableau[r * row_len + col] -= factor * tableau[p_row * row_len + col];
                    }
                }
            }
        }
        basis[p_row - 1] = p_col;
    }

    // Extract solution x = x' + lb
    let mut x_sol = vec![0.0; n_vars];
    for r in 1..=m_total {
        let basic_var = basis[r - 1];
        if basic_var < n_vars {
            x_sol[basic_var] = tableau[r * row_len + total_cols - 1];
        }
    }
    for j in 0..n_vars {
        let lb_j = lb.get(j).copied().unwrap_or(0.0);
        x_sol[j] += lb_j;
    }

    let fval: f64 = c.iter().zip(&x_sol).map(|(&ci, &xi)| ci * xi).sum();

    let x_out = make_double_matrix(n_vars, 1, x_sol)?;
    let mut out = vec![x_out];
    if nargout > 1 {
        out.push(Value::scalar(fval));
    }
    if nargout > 2 {
        out.push(Value::scalar(exitflag));
    }
    if nargout > 3 {
        let output = make_output_struct(0, 0, "simplex", "Optimal solution found.");
        out.push(output);
    }
    Ok(out)
}
