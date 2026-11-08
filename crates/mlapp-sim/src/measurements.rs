//! Measurements and spectral analysis algorithms for MatRust Studio.
//!
//! Provides allocation-free running window RMS, Mean, Power Measurement (P, Q1, S, PF, DPF),
//! and Total Harmonic Distortion (THD) blocks, plus offline spectral analysis (FFT/DFT).
//!
//! # References
//! - Mohan, Undeland, Robbins: Power Electronics (Converters, Applications, and Design)
//! - Neumaier, A. (1974): Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen

use crate::diagnostics::Diagnostic;
use serde::{Deserialize, Serialize};

/// Neumaier compensated summation for high numerical precision in running windows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeumaierSum {
    pub sum: f64,
    pub c: f64,
}

impl Default for NeumaierSum {
    fn default() -> Self {
        Self::new()
    }
}

impl NeumaierSum {
    pub const fn new() -> Self {
        Self { sum: 0.0, c: 0.0 }
    }

    pub fn add(&mut self, val: f64) {
        let t = self.sum + val;
        if self.sum.abs() >= val.abs() {
            self.c += (self.sum - t) + val;
        } else {
            self.c += (val - t) + self.sum;
        }
        self.sum = t;
    }

    pub fn sub(&mut self, val: f64) {
        self.add(-val);
    }

    pub fn total(&self) -> f64 {
        self.sum + self.c
    }

    pub fn reset(&mut self, val: f64) {
        self.sum = val;
        self.c = 0.0;
    }
}

/// Running window configuration and circular buffer state for single-input blocks (Rms, Mean).
#[derive(Debug, Clone, PartialEq)]
pub struct ScalarWindowBuffer {
    pub buffer: Vec<f64>,
    pub head: usize,
    pub count: usize,
    pub window_size: usize,
    pub sum: NeumaierSum,
    pub initial_value: f64,
    pub last_output: f64,
    pub steps_since_recompute: usize,
    pub is_rms: bool,
}

impl ScalarWindowBuffer {
    pub fn new(window_size: usize, initial_value: f64, is_rms: bool) -> Self {
        Self {
            buffer: vec![0.0; window_size.max(1)],
            head: 0,
            count: 0,
            window_size: window_size.max(1),
            sum: NeumaierSum::new(),
            initial_value,
            last_output: initial_value,
            steps_since_recompute: 0,
            is_rms,
        }
    }

    /// Evaluates the block output during major or minor steps.
    /// Strictly zero direct feedthrough: returns output calculated up to previous sample.
    pub fn eval_output(&self) -> f64 {
        self.last_output
    }

    /// Updates the circular buffer on accepted sample hit.
    pub fn update(&mut self, u: f64) {
        let nw = self.window_size;
        let x = if self.is_rms { u * u } else { u };

        if self.count < nw {
            self.buffer[self.head] = x;
            self.sum.add(x);
            self.head = (self.head + 1) % nw;
            self.count += 1;

            if self.count == nw {
                self.steps_since_recompute = 0;
                let tot = self.sum.total();
                self.last_output = if self.is_rms {
                    (tot.max(0.0) / (nw as f64)).sqrt()
                } else {
                    tot / (nw as f64)
                };
            } else {
                self.last_output = self.initial_value;
            }
        } else {
            let x_old = self.buffer[self.head];
            self.buffer[self.head] = x;
            self.head = (self.head + 1) % nw;
            self.steps_since_recompute += 1;

            if self.steps_since_recompute >= nw {
                // Exact recompute from ring buffer every Nw steps to prevent drift
                let mut recomputed = NeumaierSum::new();
                for &val in &self.buffer {
                    recomputed.add(val);
                }
                self.sum = recomputed;
                self.steps_since_recompute = 0;
            } else {
                self.sum.sub(x_old);
                self.sum.add(x);
            }

            let tot = self.sum.total();
            self.last_output = if self.is_rms {
                (tot.max(0.0) / (nw as f64)).sqrt()
            } else {
                tot / (nw as f64)
            };
        }
    }
}

/// Running window state for PowerMeasurement block:
/// Calculates P, Q1, S, PF, DPF.
#[derive(Debug, Clone, PartialEq)]
pub struct PowerMeasurementBuffer {
    pub v_buf: Vec<f64>,
    pub i_buf: Vec<f64>,
    pub head: usize,
    pub count: usize,
    pub window_size: usize,
    pub p_sum: NeumaierSum,
    pub v_sq_sum: NeumaierSum,
    pub i_sq_sum: NeumaierSum,
    pub steps_since_recompute: usize,
    pub cos_table: Vec<f64>,
    pub sin_table: Vec<f64>,
    // Outputs held: [P, Q1, S, PF, DPF]
    pub outputs: [f64; 5],
}

impl PowerMeasurementBuffer {
    pub fn new(window_size: usize) -> Self {
        let nw = window_size.max(4);
        let mut cos_table = Vec::with_capacity(nw);
        let mut sin_table = Vec::with_capacity(nw);
        let two_pi = 2.0 * std::f64::consts::PI;
        for n in 0..nw {
            let angle = two_pi * (n as f64) / (nw as f64);
            cos_table.push(angle.cos());
            sin_table.push(angle.sin());
        }

        Self {
            v_buf: vec![0.0; nw],
            i_buf: vec![0.0; nw],
            head: 0,
            count: 0,
            window_size: nw,
            p_sum: NeumaierSum::new(),
            v_sq_sum: NeumaierSum::new(),
            i_sq_sum: NeumaierSum::new(),
            steps_since_recompute: 0,
            cos_table,
            sin_table,
            outputs: [0.0; 5],
        }
    }

    pub fn eval_output(&self, port: usize) -> f64 {
        if port < 5 {
            self.outputs[port]
        } else {
            0.0
        }
    }

    pub fn all_outputs(&self) -> (f64, f64, f64, f64, f64) {
        (
            self.outputs[0],
            self.outputs[1],
            self.outputs[2],
            self.outputs[3],
            self.outputs[4],
        )
    }

    pub fn update(&mut self, v: f64, i: f64) {
        let nw = self.window_size;
        let vi = v * i;
        let v2 = v * v;
        let i2 = i * i;

        if self.count < nw {
            self.v_buf[self.head] = v;
            self.i_buf[self.head] = i;
            self.p_sum.add(vi);
            self.v_sq_sum.add(v2);
            self.i_sq_sum.add(i2);
            self.head = (self.head + 1) % nw;
            self.count += 1;

            if self.count == nw {
                self.steps_since_recompute = 0;
                self.recompute_all();
            } else {
                self.outputs = [0.0, 0.0, 0.0, 1.0, 1.0];
            }
        } else {
            let v_old = self.v_buf[self.head];
            let i_old = self.i_buf[self.head];
            self.v_buf[self.head] = v;
            self.i_buf[self.head] = i;
            self.head = (self.head + 1) % nw;
            self.steps_since_recompute += 1;

            if self.steps_since_recompute >= nw {
                let mut re_p = NeumaierSum::new();
                let mut re_v2 = NeumaierSum::new();
                let mut re_i2 = NeumaierSum::new();
                for k in 0..nw {
                    let vk = self.v_buf[k];
                    let ik = self.i_buf[k];
                    re_p.add(vk * ik);
                    re_v2.add(vk * vk);
                    re_i2.add(ik * ik);
                }
                self.p_sum = re_p;
                self.v_sq_sum = re_v2;
                self.i_sq_sum = re_i2;
                self.steps_since_recompute = 0;

                // Recompute fundamental DFT for Q1 and DPF once per window
                self.update_fundamental_dft();
            } else {
                self.p_sum.sub(v_old * i_old);
                self.p_sum.add(vi);
                self.v_sq_sum.sub(v_old * v_old);
                self.v_sq_sum.add(v2);
                self.i_sq_sum.sub(i_old * i_old);
                self.i_sq_sum.add(i2);
            }

            // P, S, PF update every sample
            let p = self.p_sum.total() / (nw as f64);
            let v_rms = (self.v_sq_sum.total().max(0.0) / (nw as f64)).sqrt();
            let i_rms = (self.i_sq_sum.total().max(0.0) / (nw as f64)).sqrt();
            let s = v_rms * i_rms;
            let pf = if s < 1e-9 {
                1.0
            } else {
                (p / s).clamp(-1.0, 1.0)
            };

            self.outputs[0] = p;
            self.outputs[2] = s;
            self.outputs[3] = pf;
        }
    }

    fn recompute_all(&mut self) {
        let nw = self.window_size;
        let p = self.p_sum.total() / (nw as f64);
        let v_rms = (self.v_sq_sum.total().max(0.0) / (nw as f64)).sqrt();
        let i_rms = (self.i_sq_sum.total().max(0.0) / (nw as f64)).sqrt();
        let s = v_rms * i_rms;
        let pf = if s < 1e-9 {
            1.0
        } else {
            (p / s).clamp(-1.0, 1.0)
        };
        self.outputs[0] = p;
        self.outputs[2] = s;
        self.outputs[3] = pf;
        self.update_fundamental_dft();
    }

    fn update_fundamental_dft(&mut self) {
        let nw = self.window_size;
        let mut a_v = 0.0;
        let mut b_v = 0.0;
        let mut a_i = 0.0;
        let mut b_i = 0.0;

        for n in 0..nw {
            // Buffer oldest sample is at self.head
            let buf_idx = (self.head + n) % nw;
            let vn = self.v_buf[buf_idx];
            let in_val = self.i_buf[buf_idx];
            let c = self.cos_table[n];
            let s = self.sin_table[n];
            a_v += vn * c;
            b_v += vn * s;
            a_i += in_val * c;
            b_i += in_val * s;
        }

        let v_mag = (a_v * a_v + b_v * b_v).sqrt();
        let i_mag = (a_i * a_i + b_i * b_i).sqrt();

        if v_mag < 1e-9 || i_mag < 1e-9 {
            self.outputs[1] = 0.0;
            self.outputs[4] = 1.0;
        } else {
            // Q1 = (2 / Nw^2) * (A_v * B_i - B_v * A_i)
            let q1 = (2.0 / ((nw * nw) as f64)) * (a_v * b_i - b_v * a_i);
            // DPF = cos(theta_v - theta_i) = (A_v * A_i + B_v * B_i) / (v_mag * i_mag)
            let dpf = ((a_v * a_i + b_v * b_i) / (v_mag * i_mag)).clamp(-1.0, 1.0);
            self.outputs[1] = q1;
            self.outputs[4] = dpf;
        }
    }
}

/// Running window state for Total Harmonic Distortion (Thd) block.
#[derive(Debug, Clone, PartialEq)]
pub struct ThdBuffer {
    pub buffer: Vec<f64>,
    pub head: usize,
    pub count: usize,
    pub window_size: usize,
    pub max_harmonic: usize,
    pub as_percent: bool,
    pub cos_tables: Vec<Vec<f64>>, // [h-1][n]
    pub sin_tables: Vec<Vec<f64>>,
    pub last_output: f64,
    pub steps_since_recompute: usize,
    pub emitted_nofund_info: bool,
}

impl ThdBuffer {
    pub fn new(window_size: usize, max_harmonic: usize, as_percent: bool) -> Self {
        let nw = window_size.max(4);
        let h_max = max_harmonic.min((nw - 1) / 2).max(1);
        let two_pi = 2.0 * std::f64::consts::PI;

        let mut cos_tables = Vec::with_capacity(h_max);
        let mut sin_tables = Vec::with_capacity(h_max);

        for h in 1..=h_max {
            let mut cos_h = Vec::with_capacity(nw);
            let mut sin_h = Vec::with_capacity(nw);
            for n in 0..nw {
                let angle = two_pi * (h as f64) * (n as f64) / (nw as f64);
                cos_h.push(angle.cos());
                sin_h.push(angle.sin());
            }
            cos_tables.push(cos_h);
            sin_tables.push(sin_h);
        }

        Self {
            buffer: vec![0.0; nw],
            head: 0,
            count: 0,
            window_size: nw,
            max_harmonic: h_max,
            as_percent,
            cos_tables,
            sin_tables,
            last_output: 0.0,
            steps_since_recompute: 0,
            emitted_nofund_info: false,
        }
    }

    pub fn eval_output(&self) -> f64 {
        self.last_output
    }

    pub fn update(&mut self, u: f64) -> Option<Diagnostic> {
        let nw = self.window_size;
        let mut diag = None;

        self.buffer[self.head] = u;
        self.head = (self.head + 1) % nw;
        if self.count < nw {
            self.count += 1;
            if self.count == nw {
                self.steps_since_recompute = 0;
                diag = self.compute_thd();
            } else {
                self.last_output = 0.0;
            }
        } else {
            self.steps_since_recompute += 1;
            if self.steps_since_recompute >= nw {
                self.steps_since_recompute = 0;
                diag = self.compute_thd();
            }
        }
        diag
    }

    fn compute_thd(&mut self) -> Option<Diagnostic> {
        let nw = self.window_size;
        let h_max = self.max_harmonic;

        // Compute fundamental (h=1)
        let mut a1 = 0.0;
        let mut b1 = 0.0;
        let c1 = &self.cos_tables[0];
        let s1 = &self.sin_tables[0];
        for n in 0..nw {
            let idx = (self.head + n) % nw;
            let val = self.buffer[idx];
            a1 += val * c1[n];
            b1 += val * s1[n];
        }

        let v1_sq = a1 * a1 + b1 * b1;
        let v1 = (v1_sq).sqrt();

        if v1 < 1e-9 {
            self.last_output = 0.0;
            if !self.emitted_nofund_info {
                self.emitted_nofund_info = true;
                return Some(Diagnostic::info(
                    "I-THD-NOFUND",
                    "Fundamental harmonic magnitude is near zero (< 1e-9): THD output is set to 0. Why: THD denominator is undefined without fundamental. How to fix: Ensure input signal has non-zero fundamental frequency component.",
                    vec![],
                ));
            }
            return None;
        }

        let mut sum_harm_sq = 0.0;
        for h in 2..=h_max {
            let mut ah = 0.0;
            let mut bh = 0.0;
            let ch = &self.cos_tables[h - 1];
            let sh = &self.sin_tables[h - 1];
            for n in 0..nw {
                let idx = (self.head + n) % nw;
                let val = self.buffer[idx];
                ah += val * ch[n];
                bh += val * sh[n];
            }
            sum_harm_sq += ah * ah + bh * bh;
        }

        let thd_ratio = (sum_harm_sq / v1_sq).sqrt();
        self.last_output = if self.as_percent {
            thd_ratio * 100.0
        } else {
            thd_ratio
        };
        None
    }
}

/// Window function choices for spectral analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WindowFunction {
    #[default]
    Rectangular,
    Hann,
    FlatTop,
}

impl WindowFunction {
    pub fn apply(&self, data: &[f64]) -> Vec<f64> {
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }
        match self {
            WindowFunction::Rectangular => data.to_vec(),
            WindowFunction::Hann => {
                let mut out = Vec::with_capacity(n);
                for (i, &val) in data.iter().enumerate() {
                    let w =
                        0.5 * (1.0 - (2.0 * std::f64::consts::PI * (i as f64) / (n as f64)).cos());
                    out.push(val * w * 2.0); // scaled for peak amplitude
                }
                out
            }
            WindowFunction::FlatTop => {
                let mut out = Vec::with_capacity(n);
                const A0: f64 = 0.21557895;
                const A1: f64 = 0.41663158;
                const A2: f64 = 0.277263158;
                const A3: f64 = 0.083578947;
                const A4: f64 = 0.006947368;
                for (i, &val) in data.iter().enumerate() {
                    let z = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
                    let w = A0 - A1 * z.cos() + A2 * (2.0 * z).cos() - A3 * (3.0 * z).cos()
                        + A4 * (4.0 * z).cos();
                    out.push(val * w * (1.0 / A0)); // scaled for peak amplitude
                }
                out
            }
        }
    }
}

/// Offline spectral analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumResult {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
    pub phases: Vec<f64>,
    pub thd: f64,
    pub fundamental_freq: f64,
    pub warnings: Vec<Diagnostic>,
}

/// Computes spectral analysis (harmonics, magnitude, phase, THD) for a logged simulation signal.
pub fn spectrum(
    signal: &[f64],
    tout: &[f64],
    window: Option<usize>,
    f0: f64,
) -> Result<SpectrumResult, Diagnostic> {
    spectrum_with_window(signal, tout, window, f0, WindowFunction::Rectangular)
}

/// Computes spectral analysis using a specified window function.
pub fn spectrum_with_window(
    signal: &[f64],
    tout: &[f64],
    window: Option<usize>,
    f0: f64,
    window_fn: WindowFunction,
) -> Result<SpectrumResult, Diagnostic> {
    if signal.is_empty() || tout.is_empty() || signal.len() != tout.len() {
        return Err(Diagnostic::error(
            "E-PARAM",
            "Empty or mismatched signal and time vectors in spectrum analysis.",
            vec![],
        ));
    }

    if f0 <= 0.0 {
        return Err(Diagnostic::error(
            "E-PARAM",
            format!("Fundamental frequency f0 ({f0}) must be strictly positive."),
            vec![],
        ));
    }

    // Check uniform time step
    if tout.len() > 1 {
        let dt0 = tout[1] - tout[0];
        if dt0 <= 0.0 {
            return Err(Diagnostic::error(
                "E-PARAM",
                "Non-increasing time vector in spectrum analysis.",
                vec![],
            ));
        }
        for i in 1..tout.len() - 1 {
            let dti = tout[i + 1] - tout[i];
            if (dti - dt0).abs() > 1e-6 * dt0 {
                return Err(Diagnostic::error(
                    "E-PARAM",
                    "Time points are non-uniform: Spectrum requires a fixed-step simulation. Why: FFT/DFT assumes constant sample spacing. How to fix: Use a fixed-step solver (e.g., FixedStepEuler or FixedStepRk4) with a constant step size.",
                    vec![],
                ));
            }
        }
    }

    let dt = if tout.len() > 1 {
        tout[1] - tout[0]
    } else {
        1.0
    };
    let n_total = signal.len();
    let n_win = window.unwrap_or(n_total).min(n_total);
    let start_idx = n_total - n_win;
    let data = &signal[start_idx..];

    let t_period = 1.0 / f0;
    let nw_period = (t_period / dt).round() as usize;

    let mut warnings = Vec::new();
    if nw_period > 0 && !n_win.is_multiple_of(nw_period) {
        warnings.push(Diagnostic::warning(
            "W-WINDOW",
            format!(
                "Spectral analysis window ({n_win} samples) is not an integer multiple of fundamental period ({nw_period} samples): spectral leakage / ripple expected.",
            ),
            vec![],
        ));
    }

    // Windowing and DFT/FFT
    let windowed = window_fn.apply(data);
    let n = windowed.len();
    let is_pow2 = n > 0 && (n & (n - 1)) == 0;

    let (magnitudes, phases) = if is_pow2 && n >= 4 {
        fft_radix2(&windowed)
    } else {
        dft_direct(&windowed)
    };

    let fs = 1.0 / dt;
    let num_bins = n / 2 + 1;
    let mut freqs = Vec::with_capacity(num_bins);
    let mut mags = Vec::with_capacity(num_bins);
    let mut phs = Vec::with_capacity(num_bins);

    for k in 0..num_bins {
        freqs.push((k as f64) * fs / (n as f64));
        mags.push(magnitudes[k]);
        phs.push(phases[k]);
    }

    // Find fundamental bin near f0
    let mut bin1 = 1;
    let mut min_diff = f64::MAX;
    for (k, &f) in freqs.iter().enumerate().skip(1) {
        let diff = (f - f0).abs();
        if diff < min_diff {
            min_diff = diff;
            bin1 = k;
        }
    }

    let v1 = mags.get(bin1).copied().unwrap_or(0.0);
    let thd = if v1 > 1e-9 {
        let mut sum_h_sq = 0.0;
        let mut h = 2;
        while h * bin1 < num_bins {
            let vh = mags[h * bin1];
            sum_h_sq += vh * vh;
            h += 1;
        }
        (sum_h_sq.sqrt() / v1) * 100.0
    } else {
        0.0
    };

    Ok(SpectrumResult {
        frequencies: freqs,
        magnitudes: mags,
        phases: phs,
        thd,
        fundamental_freq: f0,
        warnings,
    })
}

/// Direct discrete Fourier transform for arbitrary sequence length N.
pub fn dft_direct(data: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = data.len();
    let num_bins = n / 2 + 1;
    let mut mags = Vec::with_capacity(num_bins);
    let mut phs = Vec::with_capacity(num_bins);
    let two_pi = 2.0 * std::f64::consts::PI;

    for k in 0..num_bins {
        let mut re = 0.0;
        let mut im = 0.0;
        for (m, &val) in data.iter().enumerate() {
            let angle = two_pi * (k as f64) * (m as f64) / (n as f64);
            re += val * angle.cos();
            im -= val * angle.sin();
        }
        let scale = if k == 0 || (n.is_multiple_of(2) && k == n / 2) {
            1.0 / (n as f64)
        } else {
            2.0 / (n as f64)
        };
        mags.push((re * re + im * im).sqrt() * scale);
        phs.push(im.atan2(re));
    }

    (mags, phs)
}

/// Self-contained Radix-2 Cooley-Tukey decimation-in-time FFT.
pub fn fft_radix2(data: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = data.len();
    let mut re: Vec<f64> = data.to_vec();
    let mut im: Vec<f64> = vec![0.0; n];

    // Bit reversal permutation
    let mut j = 0;
    for i in 0..n - 1 {
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
        let mut k = n >> 1;
        while k <= j {
            j -= k;
            k >>= 1;
        }
        j += k;
    }

    // Cooley-Tukey butterflies
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let theta = -2.0 * std::f64::consts::PI / (len as f64);
        let w_step_re = theta.cos();
        let w_step_im = theta.sin();

        let mut i = 0;
        while i < n {
            let mut w_re = 1.0;
            let mut w_im = 0.0;
            for j in 0..half {
                let u_re = re[i + j];
                let u_im = im[i + j];
                let v_re = re[i + j + half] * w_re - im[i + j + half] * w_im;
                let v_im = re[i + j + half] * w_im + im[i + j + half] * w_re;

                re[i + j] = u_re + v_re;
                im[i + j] = u_im + v_im;
                re[i + j + half] = u_re - v_re;
                im[i + j + half] = u_im - v_im;

                let next_w_re = w_re * w_step_re - w_im * w_step_im;
                let next_w_im = w_re * w_step_im + w_im * w_step_re;
                w_re = next_w_re;
                w_im = next_w_im;
            }
            i += len;
        }
        len <<= 1;
    }

    let num_bins = n / 2 + 1;
    let mut mags = Vec::with_capacity(num_bins);
    let mut phs = Vec::with_capacity(num_bins);

    for k in 0..num_bins {
        let r = re[k];
        let i_val = im[k];
        let scale = if k == 0 || (n.is_multiple_of(2) && k == n / 2) {
            1.0 / (n as f64)
        } else {
            2.0 / (n as f64)
        };
        mags.push((r * r + i_val * i_val).sqrt() * scale);
        phs.push(i_val.atan2(r));
    }

    (mags, phs)
}
