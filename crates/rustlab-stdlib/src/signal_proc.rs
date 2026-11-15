//! Signal processing builtins: digital filtering, window functions, correlation.

use crate::helpers::user_err;
use crate::signal::conv_fn;
use num_complex::Complex64;
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

/// Deconvolution and polynomial division: [q, r] = deconv(b, a).
pub fn deconv_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "deconv requires 2 inputs: (b, a).",
        ));
    }
    let b = extract_f64_slice(&args[0])?;
    let a = extract_f64_slice(&args[1])?;

    if a.is_empty() {
        return Err(user_err(
            "RUSTLAB:deconv:emptyA",
            "Second argument cannot be empty.",
        ));
    }
    if a[0].abs() < 1e-15 {
        return Err(user_err(
            "RUSTLAB:deconv:zeroLeadCoeff",
            "First element of second argument cannot be zero.",
        ));
    }

    let nb = b.len();
    let na = a.len();

    if nb < na {
        let q = vec![0.0];
        let r = b;
        let q_da = DenseArray::from_vec_unchecked(vec![1, 1], q);
        let r_da = DenseArray::from_vec_unchecked(vec![1, r.len()], r);
        return if nargout >= 2 {
            Ok(vec![Value::Double(q_da), Value::Double(r_da)])
        } else {
            Ok(vec![Value::Double(q_da)])
        };
    }

    let nq = nb - na + 1;
    let mut q = vec![0.0; nq];
    let mut rem = b.clone();

    for i in 0..nq {
        let val = rem[i] / a[0];
        q[i] = val;
        for j in 0..na {
            rem[i + j] -= val * a[j];
        }
    }

    let q_da = DenseArray::from_vec_unchecked(vec![1, nq], q);
    let r_da = DenseArray::from_vec_unchecked(vec![1, nb], rem);
    if nargout >= 2 {
        Ok(vec![Value::Double(q_da), Value::Double(r_da)])
    } else {
        Ok(vec![Value::Double(q_da)])
    }
}

/// Frequency response of digital filter: [h, w] = freqz(b, a, n, ...).
pub fn freqz_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "freqz requires at least 1 input.",
        ));
    }
    let b = extract_f64_slice(&args[0])?;
    let a = if args.len() > 1 && !args[1].is_empty() {
        extract_f64_slice(&args[1])?
    } else {
        vec![1.0]
    };
    if a.is_empty() || a[0].abs() < 1e-15 {
        return Err(user_err(
            "RUSTLAB:freqz:zeroDenominator",
            "Denominator cannot be empty or zero.",
        ));
    }

    let n_pts = if args.len() > 2 && !args[2].is_empty() {
        args[2]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:freqz:invalidN", "N must be a scalar integer."))?
            as usize
    } else {
        512
    };

    let whole = args.iter().any(|arg| {
        if let Value::Char(c) = arg {
            let s: String = c.as_slice().iter().collect();
            s.eq_ignore_ascii_case("whole")
        } else if let Value::String(sa) = arg {
            sa.as_slice()
                .first()
                .map(|s| s.eq_ignore_ascii_case("whole"))
                .unwrap_or(false)
        } else {
            false
        }
    });

    let max_freq = if whole { 2.0 * PI } else { PI };
    let mut w = Vec::with_capacity(n_pts);
    let mut h = Vec::with_capacity(n_pts);

    for k in 0..n_pts {
        let omega = max_freq * (k as f64) / (n_pts as f64);
        w.push(omega);

        let mut b_sum = Complex64::new(0.0, 0.0);
        for (m, &bm) in b.iter().enumerate() {
            let phase = -omega * (m as f64);
            b_sum += bm * Complex64::new(phase.cos(), phase.sin());
        }

        let mut a_sum = Complex64::new(0.0, 0.0);
        for (m, &am) in a.iter().enumerate() {
            let phase = -omega * (m as f64);
            a_sum += am * Complex64::new(phase.cos(), phase.sin());
        }

        let resp = b_sum / a_sum;
        h.push(resp);
    }

    let h_da = DenseArray::from_vec_unchecked(vec![n_pts, 1], h);
    let w_da = DenseArray::from_vec_unchecked(vec![n_pts, 1], w);

    if nargout >= 2 {
        Ok(vec![Value::ComplexDouble(h_da), Value::Double(w_da)])
    } else {
        Ok(vec![Value::ComplexDouble(h_da)])
    }
}

/// Partial fraction expansion (residues, poles, and direct term): [r, p, k] = residue(b, a).
pub fn residue_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() < 2 {
        return Err(user_err(
            "MATLAB:minrhs",
            "residue requires at least 2 inputs: (b, a).",
        ));
    }
    let b = extract_f64_slice(&args[0])?;
    let a = extract_f64_slice(&args[1])?;

    if a.is_empty() {
        return Err(user_err(
            "RUSTLAB:residue:emptyA",
            "Denominator cannot be empty.",
        ));
    }

    // Direct polynomial quotient k and remainder b_rem if deg(b) >= deg(a)
    let (k_vec, b_rem) = if b.len() >= a.len() {
        let nq = b.len() - a.len() + 1;
        let mut q = vec![0.0; nq];
        let mut rem = b.clone();
        for i in 0..nq {
            let q_i = rem[i] / a[0];
            q[i] = q_i;
            for j in 0..a.len() {
                rem[i + j] -= q_i * a[j];
            }
        }
        let trim_idx = rem
            .iter()
            .position(|&x| x.abs() > 1e-12)
            .unwrap_or(rem.len());
        (q, rem[trim_idx..].to_vec())
    } else {
        (vec![], b.clone())
    };

    // Find poles: roots of polynomial a
    let a_val = Value::Double(DenseArray::from_vec_unchecked(vec![1, a.len()], a.clone()));
    let roots_out = crate::numerical::roots_fn(&[a_val], 1)?;
    let p_val = roots_out.into_iter().next().unwrap_or_else(Value::empty);

    let poles: Vec<Complex64> = match &p_val {
        Value::ComplexDouble(cd) => cd.as_slice().to_vec(),
        Value::Double(d) => d
            .as_slice()
            .iter()
            .map(|&x| Complex64::new(x, 0.0))
            .collect(),
        _ => vec![],
    };

    let np = poles.len();
    let mut residues = Vec::with_capacity(np);

    // Compute derivative of a: a'(s)
    let na = a.len();
    let mut a_der = Vec::with_capacity(na.saturating_sub(1));
    for i in 0..na.saturating_sub(1) {
        let power = (na - 1 - i) as f64;
        a_der.push(a[i] * power);
    }

    let eval_poly_c = |poly: &[f64], s: Complex64| -> Complex64 {
        let mut val = Complex64::new(0.0, 0.0);
        for &c in poly {
            val = val * s + Complex64::new(c, 0.0);
        }
        val
    };

    for &pole in &poles {
        let b_val = eval_poly_c(&b_rem, pole);
        let a_der_val = eval_poly_c(&a_der, pole);
        let res = if a_der_val.norm() > 1e-14 {
            b_val / a_der_val
        } else {
            Complex64::new(0.0, 0.0)
        };
        residues.push(res);
    }

    let r_da = DenseArray::from_vec_unchecked(vec![np, 1], residues);
    let p_da = DenseArray::from_vec_unchecked(vec![np, 1], poles);
    let k_da = DenseArray::from_vec_unchecked(vec![1, k_vec.len()], k_vec);

    if nargout >= 3 {
        Ok(vec![
            Value::ComplexDouble(r_da),
            Value::ComplexDouble(p_da),
            Value::Double(k_da),
        ])
    } else if nargout == 2 {
        Ok(vec![Value::ComplexDouble(r_da), Value::ComplexDouble(p_da)])
    } else {
        Ok(vec![Value::ComplexDouble(r_da)])
    }
}

/// Welch's power spectral density estimate: [pxx, f] = pwelch(x, window, noverlap, nfft, fs).
pub fn pwelch_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "pwelch requires at least 1 input.",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let nx = x.len();
    if nx == 0 {
        return Ok(vec![
            Value::Double(DenseArray::empty()),
            Value::Double(DenseArray::empty()),
        ]);
    }

    let win: Vec<f64> = if args.len() > 1 && !args[1].is_empty() {
        if let Ok(w_slice) = extract_f64_slice(&args[1]) {
            if w_slice.len() > 1 {
                w_slice
            } else {
                let w_len = (w_slice[0] as usize).min(nx);
                let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
                extract_f64_slice(&w_out[0])?
            }
        } else {
            let w_len = 256.min(nx);
            let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
            extract_f64_slice(&w_out[0])?
        }
    } else {
        let w_len = 256.min(nx);
        let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
        extract_f64_slice(&w_out[0])?
    };

    let nwin = win.len();
    let noverlap = if args.len() > 2 && !args[2].is_empty() {
        args[2]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:pwelch:invalidOverlap", "noverlap must be scalar."))?
            as usize
    } else {
        nwin / 2
    };

    let nfft = if args.len() > 3 && !args[3].is_empty() {
        args[3]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:pwelch:invalidNfft", "nfft must be scalar."))?
            as usize
    } else {
        nwin.max(256)
    };

    let fs = if args.len() > 4 && !args[4].is_empty() {
        args[4]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:pwelch:invalidFs", "fs must be scalar."))?
    } else {
        1.0
    };

    let step = nwin.saturating_sub(noverlap).max(1);
    let mut num_segments = 0;
    let n_freq = nfft / 2 + 1;
    let mut psd_sum = vec![0.0; n_freq];

    let win_power: f64 = win.iter().map(|&w| w * w).sum();

    let mut start = 0;
    while start + nwin <= nx {
        num_segments += 1;
        let mut seg = Vec::with_capacity(nfft);
        for i in 0..nwin {
            seg.push(Complex64::new(x[start + i] * win[i], 0.0));
        }
        seg.resize(nfft, Complex64::new(0.0, 0.0));

        let fft_res = crate::signal::fft_vector(&seg, false);

        for k in 0..n_freq {
            let pwr = fft_res[k].norm_sqr();
            if k == 0 || (nfft % 2 == 0 && k == n_freq - 1) {
                psd_sum[k] += pwr;
            } else {
                psd_sum[k] += 2.0 * pwr;
            }
        }
        start += step;
    }

    if num_segments == 0 {
        num_segments = 1;
    }
    let norm = (num_segments as f64) * fs * win_power;
    let pxx: Vec<f64> = psd_sum.iter().map(|&v| v / norm).collect();

    let mut f = Vec::with_capacity(n_freq);
    for k in 0..n_freq {
        f.push((k as f64) * fs / (nfft as f64));
    }

    let pxx_da = DenseArray::from_vec_unchecked(vec![n_freq, 1], pxx);
    let f_da = DenseArray::from_vec_unchecked(vec![n_freq, 1], f);

    if nargout >= 2 {
        Ok(vec![Value::Double(pxx_da), Value::Double(f_da)])
    } else {
        Ok(vec![Value::Double(pxx_da)])
    }
}

/// Short-Time Fourier Transform spectrogram: [s, f, t] = spectrogram(x, window, noverlap, nfft, fs).
pub fn spectrogram_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        return Err(user_err(
            "MATLAB:minrhs",
            "spectrogram requires at least 1 input.",
        ));
    }
    let x = extract_f64_slice(&args[0])?;
    let nx = x.len();
    if nx == 0 {
        return Ok(vec![Value::ComplexDouble(DenseArray::empty())]);
    }

    let win: Vec<f64> = if args.len() > 1 && !args[1].is_empty() {
        if let Ok(w_slice) = extract_f64_slice(&args[1]) {
            if w_slice.len() > 1 {
                w_slice
            } else {
                let w_len = (w_slice[0] as usize).min(nx);
                let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
                extract_f64_slice(&w_out[0])?
            }
        } else {
            let w_len = 256.min(nx);
            let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
            extract_f64_slice(&w_out[0])?
        }
    } else {
        let w_len = 256.min(nx);
        let w_out = hamming_fn(&[Value::scalar(w_len as f64)], 1)?;
        extract_f64_slice(&w_out[0])?
    };

    let nwin = win.len();
    let noverlap = if args.len() > 2 && !args[2].is_empty() {
        args[2].as_scalar_f64().map_err(|_| {
            user_err(
                "RUSTLAB:spectrogram:invalidOverlap",
                "noverlap must be scalar.",
            )
        })? as usize
    } else {
        nwin / 2
    };

    let nfft = if args.len() > 3 && !args[3].is_empty() {
        args[3]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:spectrogram:invalidNfft", "nfft must be scalar."))?
            as usize
    } else {
        nwin.max(256)
    };

    let fs = if args.len() > 4 && !args[4].is_empty() {
        args[4]
            .as_scalar_f64()
            .map_err(|_| user_err("RUSTLAB:spectrogram:invalidFs", "fs must be scalar."))?
    } else {
        1.0
    };

    let step = nwin.saturating_sub(noverlap).max(1);
    let n_freq = nfft / 2 + 1;
    let mut columns: Vec<Vec<Complex64>> = Vec::new();
    let mut t_centers: Vec<f64> = Vec::new();

    let mut start = 0;
    while start + nwin <= nx {
        let mut seg = Vec::with_capacity(nfft);
        for i in 0..nwin {
            seg.push(Complex64::new(x[start + i] * win[i], 0.0));
        }
        seg.resize(nfft, Complex64::new(0.0, 0.0));

        let fft_res = crate::signal::fft_vector(&seg, false);
        columns.push(fft_res[0..n_freq].to_vec());
        let center_sample = start as f64 + (nwin as f64 - 1.0) * 0.5;
        t_centers.push(center_sample / fs);

        start += step;
    }

    let n_cols = columns.len();
    let mut flat_s = vec![Complex64::new(0.0, 0.0); n_freq * n_cols];
    for col in 0..n_cols {
        for row in 0..n_freq {
            flat_s[row + col * n_freq] = columns[col][row];
        }
    }

    let mut f = Vec::with_capacity(n_freq);
    for k in 0..n_freq {
        f.push((k as f64) * fs / (nfft as f64));
    }

    let s_da = DenseArray::from_vec_unchecked(vec![n_freq, n_cols], flat_s);
    let f_da = DenseArray::from_vec_unchecked(vec![n_freq, 1], f);
    let t_da = DenseArray::from_vec_unchecked(vec![1, n_cols], t_centers);

    if nargout >= 3 {
        Ok(vec![
            Value::ComplexDouble(s_da),
            Value::Double(f_da),
            Value::Double(t_da),
        ])
    } else if nargout == 2 {
        Ok(vec![Value::ComplexDouble(s_da), Value::Double(f_da)])
    } else {
        Ok(vec![Value::ComplexDouble(s_da)])
    }
}
