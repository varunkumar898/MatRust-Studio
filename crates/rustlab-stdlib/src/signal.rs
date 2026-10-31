//! Signal processing, transforms, and complex arithmetic: fft, ifft, fft2, ifft2, fftshift, conv, real, imag, angle.

pub mod radix2;

use crate::helpers::{extract_f64_array, math_err, user_err};
use num_complex::Complex64;
use rustlab_eval::EvalError;
use rustlab_math::{DenseArray, Value};
use std::f64::consts::PI;

/// Computes the 1D Discrete Fourier Transform using `rustfft` for arbitrary lengths.
pub fn fft_vector(input: &[Complex64], inverse: bool) -> Vec<Complex64> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }
    let mut planner = rustfft::FftPlanner::new();
    let mut buffer = input.to_vec();
    if inverse {
        let ifft = planner.plan_fft_inverse(n);
        ifft.process(&mut buffer);
        let scale = 1.0 / (n as f64);
        for x in &mut buffer {
            *x *= scale;
        }
    } else {
        let fft = planner.plan_fft_forward(n);
        fft.process(&mut buffer);
    }
    buffer
}

fn extract_complex_array(val: &Value) -> Result<DenseArray<Complex64>, EvalError> {
    match val {
        Value::ComplexDouble(da) => Ok(da.clone()),
        Value::Double(da) => {
            let c: Vec<Complex64> = da
                .as_slice()
                .iter()
                .map(|&x| Complex64::new(x, 0.0))
                .collect();
            DenseArray::from_vec(da.shape().to_vec(), c).map_err(math_err)
        }
        v => {
            let da = extract_f64_array(v)?;
            let c: Vec<Complex64> = da
                .as_slice()
                .iter()
                .map(|&x| Complex64::new(x, 0.0))
                .collect();
            DenseArray::from_vec(da.shape().to_vec(), c).map_err(math_err)
        }
    }
}

/// Computes the 1D Discrete Fourier Transform: fft(X).
pub fn fft_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "fft requires 1 input."));
    }
    let da = extract_complex_array(&args[0])?;
    let shp = da.shape();

    if shp.len() == 2 && (shp[0] == 1 || shp[1] == 1) {
        // Vector case
        let res = fft_vector(da.as_slice(), false);
        let arr = DenseArray::from_vec_unchecked(shp.to_vec(), res);
        Ok(vec![Value::ComplexDouble(arr)])
    } else if shp.len() == 2 {
        // 2D matrix case: apply FFT down each column (length M)
        let (m, n) = (shp[0], shp[1]);
        let mut out = vec![Complex64::new(0.0, 0.0); m * n];
        for c in 0..n {
            let mut col_buf = Vec::with_capacity(m);
            for r in 0..m {
                col_buf.push(*da.get_subscript(&[r + 1, c + 1])?);
            }
            let res = fft_vector(&col_buf, false);
            for r in 0..m {
                out[r + c * m] = res[r];
            }
        }
        let arr = DenseArray::from_vec_unchecked(vec![m, n], out);
        Ok(vec![Value::ComplexDouble(arr)])
    } else {
        let res = fft_vector(da.as_slice(), false);
        let arr = DenseArray::from_vec_unchecked(shp.to_vec(), res);
        Ok(vec![Value::ComplexDouble(arr)])
    }
}

/// Computes the 1D Inverse Discrete Fourier Transform: ifft(X).
pub fn ifft_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "ifft requires 1 input."));
    }
    let da = extract_complex_array(&args[0])?;
    let shp = da.shape();

    if shp.len() == 2 && (shp[0] == 1 || shp[1] == 1) {
        let res = fft_vector(da.as_slice(), true);
        let arr = DenseArray::from_vec_unchecked(shp.to_vec(), res);
        Ok(vec![Value::ComplexDouble(arr)])
    } else if shp.len() == 2 {
        let (m, n) = (shp[0], shp[1]);
        let mut out = vec![Complex64::new(0.0, 0.0); m * n];
        for c in 0..n {
            let mut col_buf = Vec::with_capacity(m);
            for r in 0..m {
                col_buf.push(*da.get_subscript(&[r + 1, c + 1])?);
            }
            let res = fft_vector(&col_buf, true);
            for r in 0..m {
                out[r + c * m] = res[r];
            }
        }
        let arr = DenseArray::from_vec_unchecked(vec![m, n], out);
        Ok(vec![Value::ComplexDouble(arr)])
    } else {
        let res = fft_vector(da.as_slice(), true);
        let arr = DenseArray::from_vec_unchecked(shp.to_vec(), res);
        Ok(vec![Value::ComplexDouble(arr)])
    }
}

/// Computes the 2D Discrete Fourier Transform: fft2(X).
pub fn fft2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "fft2 requires 1 input."));
    }
    let da = extract_complex_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:fft2:matrixRequired",
            "fft2 requires a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);

    // Step 1: 1D FFT along columns
    let mut temp = vec![Complex64::new(0.0, 0.0); m * n];
    for c in 0..n {
        let mut col_buf = Vec::with_capacity(m);
        for r in 0..m {
            col_buf.push(*da.get_subscript(&[r + 1, c + 1])?);
        }
        let res = fft_vector(&col_buf, false);
        for r in 0..m {
            temp[r + c * m] = res[r];
        }
    }

    // Step 2: 1D FFT along rows
    let mut out = vec![Complex64::new(0.0, 0.0); m * n];
    for r in 0..m {
        let mut row_buf = Vec::with_capacity(n);
        for c in 0..n {
            row_buf.push(temp[r + c * m]);
        }
        let res = fft_vector(&row_buf, false);
        for c in 0..n {
            out[r + c * m] = res[c];
        }
    }

    let arr = DenseArray::from_vec_unchecked(vec![m, n], out);
    Ok(vec![Value::ComplexDouble(arr)])
}

/// Computes the 2D Inverse Discrete Fourier Transform: ifft2(X).
pub fn ifft2_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "ifft2 requires 1 input."));
    }
    let da = extract_complex_array(&args[0])?;
    let shp = da.shape();
    if shp.len() != 2 {
        return Err(user_err(
            "MATLAB:ifft2:matrixRequired",
            "ifft2 requires a 2D matrix.",
        ));
    }
    let (m, n) = (shp[0], shp[1]);

    // Step 1: 1D IFFT along columns
    let mut temp = vec![Complex64::new(0.0, 0.0); m * n];
    for c in 0..n {
        let mut col_buf = Vec::with_capacity(m);
        for r in 0..m {
            col_buf.push(*da.get_subscript(&[r + 1, c + 1])?);
        }
        let res = fft_vector(&col_buf, true);
        for r in 0..m {
            temp[r + c * m] = res[r];
        }
    }

    // Step 2: 1D IFFT along rows
    let mut out = vec![Complex64::new(0.0, 0.0); m * n];
    for r in 0..m {
        let mut row_buf = Vec::with_capacity(n);
        for c in 0..n {
            row_buf.push(temp[r + c * m]);
        }
        let res = fft_vector(&row_buf, true);
        for c in 0..n {
            out[r + c * m] = res[c];
        }
    }

    let arr = DenseArray::from_vec_unchecked(vec![m, n], out);
    Ok(vec![Value::ComplexDouble(arr)])
}

pub fn fftshift_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "fftshift requires 1 input."));
    }
    let da = extract_f64_array(&args[0])?;
    let n = da.numel();
    let mid = n.div_ceil(2);
    let mut out = Vec::with_capacity(n);
    let s = da.as_slice();
    out.extend_from_slice(&s[mid..]);
    out.extend_from_slice(&s[..mid]);

    let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), out);
    Ok(vec![Value::Double(arr)])
}

pub fn conv_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err("MATLAB:minrhs", "conv requires 2 inputs."));
    }
    let u = extract_f64_array(&args[0])?;
    let v = extract_f64_array(&args[1])?;

    let u_s = u.as_slice();
    let v_s = v.as_slice();
    let n = u_s.len();
    let m = v_s.len();

    if n == 0 || m == 0 {
        let arr = DenseArray::from_vec_unchecked(vec![1, 0], Vec::new());
        return Ok(vec![Value::Double(arr)]);
    }

    let out_len = n + m - 1;
    let mut w = vec![0.0; out_len];

    for i in 0..n {
        for j in 0..m {
            w[i + j] += u_s[i] * v_s[j];
        }
    }

    let is_row = args[0].shape().first() == Some(&1);
    let shape = if is_row {
        vec![1, out_len]
    } else {
        vec![out_len, 1]
    };

    let arr = DenseArray::from_vec_unchecked(shape, w);
    Ok(vec![Value::Double(arr)])
}

pub fn real_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "real requires 1 input."));
    }
    match &args[0] {
        Value::ComplexDouble(da) => {
            let re: Vec<f64> = da.as_slice().iter().map(|c| c.re).collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), re);
            Ok(vec![Value::Double(arr)])
        }
        Value::Double(_) => Ok(vec![args[0].clone()]),
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da)])
        }
    }
}

pub fn imag_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "imag requires 1 input."));
    }
    match &args[0] {
        Value::ComplexDouble(da) => {
            let im: Vec<f64> = da.as_slice().iter().map(|c| c.im).collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), im);
            Ok(vec![Value::Double(arr)])
        }
        Value::Double(da) => {
            let zeros = vec![0.0; da.numel()];
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), zeros);
            Ok(vec![Value::Double(arr)])
        }
        v => {
            let da = extract_f64_array(v)?;
            let zeros = vec![0.0; da.numel()];
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), zeros);
            Ok(vec![Value::Double(arr)])
        }
    }
}

pub fn angle_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "angle requires 1 input."));
    }
    match &args[0] {
        Value::ComplexDouble(da) => {
            let ang: Vec<f64> = da.as_slice().iter().map(|c| c.arg()).collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), ang);
            Ok(vec![Value::Double(arr)])
        }
        Value::Double(da) => {
            let ang: Vec<f64> = da
                .as_slice()
                .iter()
                .map(|&x| if x < 0.0 { PI } else { 0.0 })
                .collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), ang);
            Ok(vec![Value::Double(arr)])
        }
        v => {
            let da = extract_f64_array(v)?;
            let ang: Vec<f64> = da
                .as_slice()
                .iter()
                .map(|&x| if x < 0.0 { PI } else { 0.0 })
                .collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), ang);
            Ok(vec![Value::Double(arr)])
        }
    }
}

pub fn conj_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err("MATLAB:minrhs", "conj requires 1 input."));
    }
    match &args[0] {
        Value::ComplexDouble(da) => {
            let conj: Vec<Complex64> = da.as_slice().iter().map(|c| c.conj()).collect();
            let arr = DenseArray::from_vec_unchecked(da.shape().to_vec(), conj);
            Ok(vec![Value::ComplexDouble(arr)])
        }
        Value::Double(_) => Ok(vec![args[0].clone()]),
        v => {
            let da = extract_f64_array(v)?;
            Ok(vec![Value::Double(da)])
        }
    }
}
