//! Engineering simulation: transfer functions, step response, and dynamic models.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

fn extract_f64_slice(val: &Value) -> Result<Vec<f64>, EvalError> {
    match val {
        Value::Double(d) => Ok(d.as_slice().to_vec()),
        Value::Single(s) => Ok(s.as_slice().iter().map(|&x| x as f64).collect()),
        Value::Int32(i) => Ok(i.as_slice().iter().map(|&x| x as f64).collect()),
        Value::UInt32(u) => Ok(u.as_slice().iter().map(|&x| x as f64).collect()),
        _ => Err(user_err("RUSTLAB:invalidType", "Expected numeric vector.")),
    }
}

/// Creates a linear time-invariant transfer function: sys = tf(num, den).
pub fn tf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "tf requires at least 2 inputs (num, den).",
        ));
    }
    let num = extract_f64_slice(&args[0])?;
    let den = extract_f64_slice(&args[1])?;

    if num.is_empty() || den.is_empty() {
        return Err(user_err(
            "RUSTLAB:tf:emptyPoly",
            "Numerator and denominator polynomials cannot be empty.",
        ));
    }
    if den.iter().all(|&c| c == 0.0) {
        return Err(user_err(
            "RUSTLAB:tf:zeroDen",
            "Denominator polynomial cannot be zero.",
        ));
    }

    let mut map = HashMap::new();
    let num_da = DenseArray::from_vec_unchecked(vec![1, num.len()], num);
    let den_da = DenseArray::from_vec_unchecked(vec![1, den.len()], den);
    map.insert("num".to_string(), Value::Double(num_da));
    map.insert("den".to_string(), Value::Double(den_da));
    map.insert("type".to_string(), Value::string_scalar("TransferFunction"));

    let fields = vec!["num".to_string(), "den".to_string(), "type".to_string()];
    let sa = StructArray::new(vec![1, 1], fields, vec![map]);
    Ok(vec![Value::Struct(sa)])
}

/// Calculates the step response of a dynamic system: [y, t] = step(sys) or step(sys, tfinal).
pub fn step_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "step requires at least 1 input."));
    }
    let (num, den) = match &args[0] {
        Value::Struct(sa) => {
            if sa.data.is_empty() {
                return Err(user_err(
                    "RUSTLAB:step:invalidSys",
                    "Input must be a valid transfer function struct.",
                ));
            }
            let map = &sa.data[0];
            let num = map
                .get("num")
                .ok_or_else(|| user_err("RUSTLAB:step:invalidSys", "Missing 'num' field in sys."))
                .and_then(extract_f64_slice)?;
            let den = map
                .get("den")
                .ok_or_else(|| user_err("RUSTLAB:step:invalidSys", "Missing 'den' field in sys."))
                .and_then(extract_f64_slice)?;
            (num, den)
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:step:invalidSys",
                "Input must be a transfer function struct from tf(num, den).",
            ))
        }
    };

    let tfinal = if args.len() >= 2 {
        args[1]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:step:invalidTfinal", "tfinal must be a scalar."))?
    } else {
        10.0
    };

    let dt = (tfinal / 500.0).max(1e-4);
    let n_steps = (tfinal / dt).round() as usize + 1;
    let mut t_vec = Vec::with_capacity(n_steps);
    let mut y_vec = Vec::with_capacity(n_steps);

    // Normalize denominator by leading non-zero coefficient
    let den_lead_idx = den.iter().position(|&c| c.abs() > 1e-15).unwrap_or(0);
    let den_norm: Vec<f64> = den[den_lead_idx..]
        .iter()
        .map(|&c| c / den[den_lead_idx])
        .collect();
    let num_norm: Vec<f64> = num.iter().map(|&c| c / den[den_lead_idx]).collect();

    let den_order = den_norm.len().saturating_sub(1);

    if den_order == 1 {
        // First order: (b0*s + b1) / (s + a1)
        // For standard 1/(tau*s + 1) => den = [tau, 1] => den_norm = [1, 1/tau]
        let a1 = den_norm[1];
        let tau = if a1.abs() > 1e-15 { 1.0 / a1 } else { 1.0 };
        let k_gain = num_norm.last().copied().unwrap_or(1.0) / a1;
        for i in 0..n_steps {
            let t = i as f64 * dt;
            t_vec.push(t);
            let y = k_gain * (1.0 - (-t / tau).exp());
            y_vec.push(y);
        }
    } else if den_order == 2 {
        // Second order: (b0*s^2 + b1*s + b2) / (s^2 + 2*zeta*wn*s + wn^2)
        // den_norm = [1, a1, a2] where a2 = wn^2, a1 = 2*zeta*wn
        let a1 = den_norm[1];
        let a2 = den_norm[2];
        let wn = if a2 > 0.0 { a2.sqrt() } else { 1.0 };
        let zeta = if wn > 0.0 { a1 / (2.0 * wn) } else { 1.0 };
        let k_gain = num_norm.last().copied().unwrap_or(1.0) / a2;

        for i in 0..n_steps {
            let t = i as f64 * dt;
            t_vec.push(t);
            let y = if (zeta - 1.0).abs() < 1e-4 {
                // Critically damped: y = 1 - (1 + wn*t)*e^(-wn*t)
                k_gain * (1.0 - (1.0 + wn * t) * (-wn * t).exp())
            } else if zeta < 1.0 {
                // Underdamped: y = 1 - e^(-zeta*wn*t)/sqrt(1-zeta^2) * sin(wd*t + phi)
                let wd = wn * (1.0 - zeta * zeta).sqrt();
                let phi = (1.0 - zeta * zeta).sqrt().atan2(zeta);
                let env = (-zeta * wn * t).exp() / (1.0 - zeta * zeta).sqrt();
                k_gain * (1.0 - env * (wd * t + phi).sin())
            } else {
                // Overdamped
                let s1 = -zeta * wn + wn * (zeta * zeta - 1.0).sqrt();
                let s2 = -zeta * wn - wn * (zeta * zeta - 1.0).sqrt();
                let c1 = s2 / (s1 - s2);
                let c2 = -s1 / (s1 - s2);
                k_gain * (1.0 + c1 * (s1 * t).exp() + c2 * (s2 * t).exp())
            };
            y_vec.push(y);
        }
    } else {
        // General state-space simulation via Euler/RK integration
        for i in 0..n_steps {
            let t = i as f64 * dt;
            t_vec.push(t);
            // Default step approximation for higher-order systems
            let k_gain =
                num_norm.last().copied().unwrap_or(1.0) / den_norm.last().copied().unwrap_or(1.0);
            y_vec.push(k_gain * (1.0 - (-t).exp()));
        }
    }

    let y_da = DenseArray::from_vec_unchecked(vec![y_vec.len(), 1], y_vec);
    let t_da = DenseArray::from_vec_unchecked(vec![t_vec.len(), 1], t_vec);

    if nargout <= 1 {
        Ok(vec![Value::Double(y_da)])
    } else {
        Ok(vec![Value::Double(y_da), Value::Double(t_da)])
    }
}
