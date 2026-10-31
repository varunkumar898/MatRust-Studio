//! Robotics and kinematics builtins: rotations, homogeneous transforms, forward/inverse kinematics, trajectories.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};

fn extract_scalar_angle(val: &Value) -> Result<f64, EvalError> {
    val.as_scalar_f64().map_err(|_| {
        user_err(
            "RUSTLAB:robotics:invalidAngle",
            "Angle must be a scalar in radians.",
        )
    })
}

/// 3x3 rotation matrix around X-axis: R = rotx(theta).
pub fn rotx_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "rotx requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    // Column-major: column 0, column 1, column 2
    let data = vec![
        1.0, 0.0, 0.0, // Column 0
        0.0, c, s, // Column 1
        0.0, -s, c, // Column 2
    ];
    let da = DenseArray::from_vec_unchecked(vec![3, 3], data);
    Ok(vec![Value::Double(da)])
}

/// 3x3 rotation matrix around Y-axis: R = roty(theta).
pub fn roty_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "roty requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    let data = vec![
        c, 0.0, -s, // Column 0
        0.0, 1.0, 0.0, // Column 1
        s, 0.0, c, // Column 2
    ];
    let da = DenseArray::from_vec_unchecked(vec![3, 3], data);
    Ok(vec![Value::Double(da)])
}

/// 3x3 rotation matrix around Z-axis: R = rotz(theta).
pub fn rotz_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "rotz requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    let data = vec![
        c, s, 0.0, // Column 0
        -s, c, 0.0, // Column 1
        0.0, 0.0, 1.0, // Column 2
    ];
    let da = DenseArray::from_vec_unchecked(vec![3, 3], data);
    Ok(vec![Value::Double(da)])
}

/// 4x4 homogeneous transform rotation around X: T = trotx(theta).
pub fn trotx_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "trotx requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    let data = vec![
        1.0, 0.0, 0.0, 0.0, // Col 0
        0.0, c, s, 0.0, // Col 1
        0.0, -s, c, 0.0, // Col 2
        0.0, 0.0, 0.0, 1.0, // Col 3
    ];
    let da = DenseArray::from_vec_unchecked(vec![4, 4], data);
    Ok(vec![Value::Double(da)])
}

/// 4x4 homogeneous transform rotation around Y: T = troty(theta).
pub fn troty_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "troty requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    let data = vec![
        c, 0.0, -s, 0.0, // Col 0
        0.0, 1.0, 0.0, 0.0, // Col 1
        s, 0.0, c, 0.0, // Col 2
        0.0, 0.0, 0.0, 1.0, // Col 3
    ];
    let da = DenseArray::from_vec_unchecked(vec![4, 4], data);
    Ok(vec![Value::Double(da)])
}

/// 4x4 homogeneous transform rotation around Z: T = trotz(theta).
pub fn trotz_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "trotz requires 1 input (theta)."));
    }
    let th = extract_scalar_angle(&args[0])?;
    let c = th.cos();
    let s = th.sin();
    let data = vec![
        c, s, 0.0, 0.0, // Col 0
        -s, c, 0.0, 0.0, // Col 1
        0.0, 0.0, 1.0, 0.0, // Col 2
        0.0, 0.0, 0.0, 1.0, // Col 3
    ];
    let da = DenseArray::from_vec_unchecked(vec![4, 4], data);
    Ok(vec![Value::Double(da)])
}

/// 4x4 homogeneous translation matrix: T = transl(x, y, z).
pub fn transl_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "transl requires 3 inputs (x, y, z).",
        ));
    }
    let x = args[0]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:transl:invalid", "x must be scalar."))?;
    let y = args[1]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:transl:invalid", "y must be scalar."))?;
    let z = args[2]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:transl:invalid", "z must be scalar."))?;

    let data = vec![
        1.0, 0.0, 0.0, 0.0, // Col 0
        0.0, 1.0, 0.0, 0.0, // Col 1
        0.0, 0.0, 1.0, 0.0, // Col 2
        x, y, z, 1.0, // Col 3
    ];
    let da = DenseArray::from_vec_unchecked(vec![4, 4], data);
    Ok(vec![Value::Double(da)])
}

/// Forward kinematics for 2-link planar arm: [x, y] = fkine_2link(l1, l2, q1, q2).
pub fn fkine_2link_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 4 {
        return Err(user_err(
            "MATLAB:minrhs",
            "fkine_2link requires 4 inputs (l1, l2, q1, q2).",
        ));
    }
    let l1 = args[0]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:fkine:invalid", "l1 must be scalar."))?;
    let l2 = args[1]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:fkine:invalid", "l2 must be scalar."))?;
    let q1 = args[2]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:fkine:invalid", "q1 must be scalar."))?;
    let q2 = args[3]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:fkine:invalid", "q2 must be scalar."))?;

    let x = l1 * q1.cos() + l2 * (q1 + q2).cos();
    let y = l1 * q1.sin() + l2 * (q1 + q2).sin();

    if nargout <= 1 {
        let da = DenseArray::from_vec_unchecked(vec![1, 2], vec![x, y]);
        Ok(vec![Value::Double(da)])
    } else {
        Ok(vec![Value::scalar(x), Value::scalar(y)])
    }
}

/// Inverse kinematics for 2-link planar arm: [q1, q2] = ikine_2link(l1, l2, x, y).
pub fn ikine_2link_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 4 {
        return Err(user_err(
            "MATLAB:minrhs",
            "ikine_2link requires 4 inputs (l1, l2, x, y).",
        ));
    }
    let l1 = args[0]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:ikine:invalid", "l1 must be scalar."))?;
    let l2 = args[1]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:ikine:invalid", "l2 must be scalar."))?;
    let x = args[2]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:ikine:invalid", "x must be scalar."))?;
    let y = args[3]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:ikine:invalid", "y must be scalar."))?;

    let r_sq = x * x + y * y;
    let cos_q2 = (r_sq - l1 * l1 - l2 * l2) / (2.0 * l1 * l2);
    if cos_q2.abs() > 1.0 {
        return Err(user_err(
            "RUSTLAB:ikine:targetUnreachable",
            "Target position is outside robot workspace.",
        ));
    }

    let q2 = cos_q2.acos(); // Elbow-up configuration
    let q1 = y.atan2(x) - (l2 * q2.sin()).atan2(l1 + l2 * cos_q2);

    if nargout <= 1 {
        let da = DenseArray::from_vec_unchecked(vec![1, 2], vec![q1, q2]);
        Ok(vec![Value::Double(da)])
    } else {
        Ok(vec![Value::scalar(q1), Value::scalar(q2)])
    }
}

/// Quintic polynomial joint trajectory: q = jtraj(q0, q1, n).
pub fn jtraj_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 3 {
        return Err(user_err(
            "MATLAB:minrhs",
            "jtraj requires 3 inputs (q0, q1, n).",
        ));
    }
    let q0 = args[0]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:jtraj:invalid", "q0 must be scalar."))?;
    let q1 = args[1]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:jtraj:invalid", "q1 must be scalar."))?;
    let n = args[2]
        .as_scalar_f64()
        .map_err(|_| user_err("RUSTLAB:jtraj:invalid", "n must be positive integer."))?
        as usize;

    if n < 2 {
        let da = DenseArray::from_vec_unchecked(vec![1, 1], vec![q0]);
        return Ok(vec![Value::Double(da)]);
    }

    let mut q_traj = Vec::with_capacity(n);
    let delta = q1 - q0;
    for i in 0..n {
        let t = (i as f64) / ((n - 1) as f64);
        // Quintic blend s(t) = 10*t^3 - 15*t^4 + 6*t^5
        let s = 10.0 * t * t * t - 15.0 * t * t * t * t + 6.0 * t * t * t * t * t;
        q_traj.push(q0 + delta * s);
    }

    let da = DenseArray::from_vec_unchecked(vec![n, 1], q_traj);
    Ok(vec![Value::Double(da)])
}
