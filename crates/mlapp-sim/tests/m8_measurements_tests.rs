use mlapp_sim::measurements::{
    spectrum, spectrum_with_window, NeumaierSum, PowerMeasurementBuffer, ScalarWindowBuffer,
    ThdBuffer, WindowFunction,
};
use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SimulationOutput, SolverType};
use std::collections::HashMap;

fn rel_eq(a: f64, b: f64, tol: f64) -> bool {
    let diff = (a - b).abs();
    let denom = a.abs().max(b.abs());
    if denom == 0.0 {
        diff <= tol
    } else {
        diff / denom <= tol
    }
}

#[test]
fn test_pure_sine_wave_rms_mean_dc() {
    let f = 50.0;
    let vm = 10.0;
    let n_w = 200; // >= 3 integer samples per period
    let dt = (1.0 / f) / (n_w as f64);

    let mut rms_buf = ScalarWindowBuffer::new(n_w, 0.0, true);
    let mut mean_buf = ScalarWindowBuffer::new(n_w, 0.0, false);

    // Warm up one period
    for i in 0..n_w {
        let t = (i as f64) * dt;
        let v = vm * (2.0 * std::f64::consts::PI * f * t).sin();
        rms_buf.update(v);
        mean_buf.update(v);
    }

    // Steady state after full window
    let v_next = vm * (2.0 * std::f64::consts::PI * f * (n_w as f64) * dt).sin();
    rms_buf.update(v_next);
    mean_buf.update(v_next);

    let expected_rms = vm / 2.0_f64.sqrt();
    let calculated_rms = rms_buf.eval_output();
    let calculated_mean = mean_buf.eval_output();

    assert!(
        rel_eq(calculated_rms, expected_rms, 1e-12),
        "Sine RMS failed: expected {expected_rms}, got {calculated_rms}, rel diff: {:e}",
        (calculated_rms - expected_rms).abs() / expected_rms
    );
    assert!(
        calculated_mean.abs() <= 1e-12,
        "Sine Mean failed: expected 0.0, got {calculated_mean}"
    );

    // DC test: RMS = Mean = V0
    let v0 = 7.5;
    let mut dc_rms = ScalarWindowBuffer::new(n_w, 0.0, true);
    let mut dc_mean = ScalarWindowBuffer::new(n_w, 0.0, false);
    for _ in 0..=n_w {
        dc_rms.update(v0);
        dc_mean.update(v0);
    }
    assert!(
        rel_eq(dc_rms.eval_output(), v0, 1e-12),
        "DC RMS failed: expected {v0}, got {}",
        dc_rms.eval_output()
    );
    assert!(
        rel_eq(dc_mean.eval_output(), v0, 1e-12),
        "DC Mean failed: expected {v0}, got {}",
        dc_mean.eval_output()
    );

    // Square wave test: RMS = Vm
    let mut sq_rms = ScalarWindowBuffer::new(n_w, 0.0, true);
    for i in 0..=n_w {
        let s = if (i % n_w) < n_w / 2 { vm } else { -vm };
        sq_rms.update(s);
    }
    assert!(
        rel_eq(sq_rms.eval_output(), vm, 1e-12),
        "Square wave RMS failed: expected {vm}, got {}",
        sq_rms.eval_output()
    );
}

#[test]
fn test_thd_pure_sine_and_distorted() {
    let n_w = 256;
    let mut thd_pure = ThdBuffer::new(n_w, 10, false);

    // Pure sine wave
    for i in 0..=n_w {
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (n_w as f64);
        thd_pure.update(theta.sin());
    }
    assert!(
        thd_pure.eval_output() < 1e-9,
        "Pure sine THD must be ~0, got {}",
        thd_pure.eval_output()
    );

    // Sine + 10% 3rd harmonic
    let mut thd_distorted = ThdBuffer::new(n_w, 10, false);
    for i in 0..=n_w {
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (n_w as f64);
        let s = theta.sin() + 0.10 * (3.0 * theta).sin();
        thd_distorted.update(s);
    }
    let expected = 0.10;
    assert!(
        rel_eq(thd_distorted.eval_output(), expected, 1e-6),
        "Distorted THD failed: expected {expected}, got {}, rel diff: {:e}",
        thd_distorted.eval_output(),
        (thd_distorted.eval_output() - expected).abs() / expected
    );
}

#[test]
fn test_sampled_square_wave_discrete_vs_continuous_thd() {
    let n_w = 512;
    let max_h = 15;
    let mut thd_buf = ThdBuffer::new(n_w, max_h, false);

    for i in 0..=n_w {
        let idx = i % n_w;
        let s = if idx < n_w / 2 { 1.0 } else { -1.0 };
        thd_buf.update(s);
    }

    let calculated_thd = thd_buf.eval_output();

    // Discrete-domain theoretical formula:
    // |X_k| / |X_1| = sin(pi / N) / sin(pi * k / N)
    let pi_over_n = std::f64::consts::PI / (n_w as f64);
    let mut disc_sum = 0.0;
    let mut cont_sum = 0.0;
    for k in (3..=max_h).step_by(2) {
        let ratio_disc = pi_over_n.sin() / (pi_over_n * (k as f64)).sin();
        disc_sum += ratio_disc * ratio_disc;

        let ratio_cont = 1.0 / (k as f64);
        cont_sum += ratio_cont * ratio_cont;
    }
    let thd_discrete_theoretical = disc_sum.sqrt();
    let thd_continuous_theoretical = cont_sum.sqrt();

    assert!(
        rel_eq(calculated_thd, thd_discrete_theoretical, 1e-9),
        "Calculated THD ({calculated_thd}) must match discrete theoretical ({thd_discrete_theoretical})"
    );

    let rel_diff = (calculated_thd - thd_continuous_theoretical).abs() / thd_continuous_theoretical;
    assert!(
        rel_diff < 0.02,
        "Discrete THD ({calculated_thd}) vs continuous ({thd_continuous_theoretical}) rel diff {rel_diff} >= 2%"
    );
}

#[test]
fn test_rl_load_power_measurement_oracles_and_trapezoidal_ratio() {
    let f = 50.0;
    let omega = 2.0 * std::f64::consts::PI * f;
    let r = 10.0;
    let l = 0.0318309886; // omega * L = 10.0
    let vm = 100.0;
    let v_rms = vm / 2.0_f64.sqrt();

    let z_sq = r * r + (omega * l) * (omega * l);
    let expected_p = v_rms * v_rms * r / z_sq;
    let expected_q1 = v_rms * v_rms * (omega * l) / z_sq;
    let expected_i_rms = v_rms / z_sq.sqrt();
    let expected_s = v_rms * expected_i_rms;
    let expected_pf = expected_p / expected_s;
    let phi = (omega * l).atan2(r);
    let expected_dpf = phi.cos();

    // Step size h = T / 200
    let t_period = 1.0 / f;
    let n_w1 = 200;
    let dt1 = t_period / (n_w1 as f64);

    let mut power_buf1 = PowerMeasurementBuffer::new(n_w1);
    for i in 0..=(n_w1 * 2) {
        let t = (i as f64) * dt1;
        let v = vm * (omega * t).sin();
        let i_sig = (vm / z_sq.sqrt()) * (omega * t - phi).sin();
        power_buf1.update(v, i_sig);
    }
    let (p1, q1_1, s1, pf1, dpf1) = power_buf1.all_outputs();

    assert!(
        rel_eq(p1, expected_p, 1e-3),
        "P1 failed: {p1} vs {expected_p}"
    );
    assert!(
        rel_eq(q1_1, expected_q1, 1e-3),
        "Q1_1 failed: {q1_1} vs {expected_q1}"
    );
    assert!(
        rel_eq(s1, expected_s, 1e-3),
        "S1 failed: {s1} vs {expected_s}"
    );
    assert!(
        rel_eq(pf1, expected_pf, 1e-3),
        "PF1 failed: {pf1} vs {expected_pf}"
    );
    assert!(
        rel_eq(dpf1, expected_dpf, 1e-3),
        "DPF1 failed: {dpf1} vs {expected_dpf}"
    );

    // Half step size h2 = T / 400
    let n_w2 = 400;
    let dt2 = t_period / (n_w2 as f64);
    let mut power_buf2 = PowerMeasurementBuffer::new(n_w2);
    for i in 0..=(n_w2 * 2) {
        let t = (i as f64) * dt2;
        let v = vm * (omega * t).sin();
        let i_sig = (vm / z_sq.sqrt()) * (omega * t - phi).sin();
        power_buf2.update(v, i_sig);
    }
    let (p2, _, _, _, _) = power_buf2.all_outputs();

    let err1 = (p1 - expected_p).abs();
    let err2 = (p2 - expected_p).abs();
    if err1 > 1e-9 && err2 > 1e-9 {
        let ratio = err1 / err2;
        assert!(
            ratio > 3.0 && ratio < 5.0,
            "Trapezoidal error ratio should be ~4, got {ratio}"
        );
    }
}

#[test]
fn test_distorted_current_power_measurement() {
    let n_w = 256;
    let mut power_buf = PowerMeasurementBuffer::new(n_w);

    let phi = 0.5; // rad
    let omega_dt = 2.0 * std::f64::consts::PI / (n_w as f64);

    for i in 0..=(n_w * 2) {
        let theta = (i as f64) * omega_dt;
        let v = theta.sin();
        let i_sig = (theta - phi).sin() + 0.3 * (3.0 * theta).sin();
        power_buf.update(v, i_sig);
    }

    let (p, q1, s, pf, dpf) = power_buf.all_outputs();

    let expected_p = 0.5 * phi.cos();
    let expected_q1 = 0.5 * phi.sin();
    let expected_s = 0.5 * (1.0 + 0.3 * 0.3_f64).sqrt();
    let expected_dpf = phi.cos();
    let expected_pf = expected_p / expected_s;

    assert!(rel_eq(p, expected_p, 1e-3), "P failed: {p} vs {expected_p}");
    assert!(
        rel_eq(q1, expected_q1, 1e-3),
        "Q1 failed: {q1} vs {expected_q1}"
    );
    assert!(rel_eq(s, expected_s, 1e-3), "S failed: {s} vs {expected_s}");
    assert!(
        rel_eq(pf, expected_pf, 1e-3),
        "PF failed: {pf} vs {expected_pf}"
    );
    assert!(
        rel_eq(dpf, expected_dpf, 1e-3),
        "DPF failed: {dpf} vs {expected_dpf}"
    );
    assert!(pf < dpf, "PF ({pf}) must be strictly less than DPF ({dpf})");
}

#[test]
fn test_neumaier_drift_and_underflow_long_run() {
    let nw = 100;
    let mut sum = NeumaierSum::new();
    let mut ring = vec![0.0; nw];

    // 1 million steps of sliding window addition and subtraction
    for i in 0..1_000_000 {
        let x = 1.0 + 1e-7 * ((i % 100) as f64);
        let x_old = ring[i % nw];
        ring[i % nw] = x;
        sum.sub(x_old);
        sum.add(x);
    }

    let mut exact_recompute = 0.0;
    for &val in &ring {
        exact_recompute += val;
    }

    let drift = (sum.total() - exact_recompute).abs() / exact_recompute;
    assert!(
        drift <= 1e-12,
        "Neumaier drift exceeded 1e-12: got {drift:e}"
    );

    let mut tiny_buf = ScalarWindowBuffer::new(100, 0.0, true);
    for _ in 0..200 {
        tiny_buf.update(1e-12);
    }
    let out = tiny_buf.eval_output();
    assert!(!out.is_nan(), "Tiny signal underflowed to NaN!");
    assert!(out > 0.0, "Tiny signal output must be positive");
}

#[test]
fn test_algebraic_loop_through_measurement_blocks() {
    let mut diagram = BlockDiagram::new("LoopWithRms");

    diagram.add_block(Block::new("1", "Const", BlockType::Constant).with_param("Value", "5.0"));
    diagram.add_block(Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "++"));
    diagram.add_block(
        Block::new("3", "RmsMeas", BlockType::Rms)
            .with_param("FundamentalFrequency", "50.0")
            .with_param("SampleTime", "0.001"),
    );
    diagram.add_block(Block::new("4", "Gain", BlockType::Gain).with_param("Gain", "0.5"));
    diagram.add_block(Block::new("5", "Scope", BlockType::Scope));

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);
    diagram.add_line("4", 1, "2", 2); // feedback loop
    diagram.add_line("3", 1, "5", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.05,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&diagram, &config);
    assert!(
        res.is_ok(),
        "Loop containing Rms block must not be an algebraic loop: {:?}",
        res.err()
    );
}

#[test]
fn test_simulation_output_write_csv() {
    let tout = vec![0.0, 0.01, 0.02];
    let yout = vec![1.0, 2.0, 3.0];
    let mut signals = HashMap::new();
    signals.insert("sig_a".to_string(), vec![1.5, 2.5, 3.5]);
    signals.insert("=injection".to_string(), vec![10.0, 20.0, 30.0]);
    signals.insert("+formula".to_string(), vec![1.1, 2.2, 3.3]);
    signals.insert("-minus".to_string(), vec![4.0, 5.0, 6.0]);
    signals.insert("@at".to_string(), vec![7.0, 8.0, 9.0]);
    signals.insert("quoted,name".to_string(), vec![0.1, 0.2, 0.3]);

    let output = SimulationOutput {
        tout,
        yout,
        signals,
        diagnostics: Vec::new(),
    };

    let mut buf = Vec::new();
    output.write_csv(&mut buf).expect("write_csv must succeed");
    let csv_str = String::from_utf8(buf).expect("valid utf-8");

    assert!(csv_str.starts_with("time,"));
    assert!(csv_str.contains("'+formula"));
    assert!(csv_str.contains("'-minus"));
    assert!(csv_str.contains("'=injection"));
    assert!(csv_str.contains("'@at"));
    assert!(csv_str.contains("\"quoted,name\""));
}

#[test]
fn test_spectrum_fft_and_dft_and_window_functions() {
    let n = 256;
    let fs = 800.0; // 800 / 256 = 3.125 Hz bin spacing, 50 Hz is bin 16
    let dt = 1.0 / fs;
    let f0 = 50.0;

    let mut tout = Vec::with_capacity(n);
    let mut sig = Vec::with_capacity(n);

    for i in 0..n {
        let t = (i as f64) * dt;
        tout.push(t);
        sig.push(
            10.0 * (2.0 * std::f64::consts::PI * f0 * t).sin()
                + 2.0 * (2.0 * std::f64::consts::PI * 3.0 * f0 * t).sin(),
        );
    }

    // FFT on power of 2
    let res = spectrum(&sig, &tout, Some(n), f0).expect("spectrum succeeds");
    assert_eq!(res.fundamental_freq, 50.0);
    let bin_50 = res
        .frequencies
        .iter()
        .position(|&f| (f - 50.0).abs() < 1e-3)
        .expect("50Hz bin found");
    assert!(rel_eq(res.magnitudes[bin_50], 10.0, 1e-2));

    // Non-power-of-2 tests direct DFT fallback (160 samples, 800 / 160 = 5.0 Hz, 50 Hz is bin 10)
    let sig_dft = &sig[..160];
    let tout_dft = &tout[..160];
    let res_dft = spectrum(sig_dft, tout_dft, Some(160), f0).expect("direct DFT succeeds");
    let bin_50_dft = res_dft
        .frequencies
        .iter()
        .position(|&f| (f - 50.0).abs() < 1e-3)
        .expect("50Hz bin found in DFT");
    assert!(rel_eq(res_dft.magnitudes[bin_50_dft], 10.0, 1e-2));

    // Test Hann and FlatTop window functions
    let res_hann = spectrum_with_window(&sig, &tout, Some(n), f0, WindowFunction::Hann)
        .expect("Hann succeeds");
    assert!(res_hann.magnitudes[bin_50] > 0.0);

    let res_flattop = spectrum_with_window(&sig, &tout, Some(n), f0, WindowFunction::FlatTop)
        .expect("FlatTop succeeds");
    assert!(res_flattop.magnitudes[bin_50] > 0.0);

    // Non-integer multiple of fundamental period produces W-WINDOW
    let sig_nonint = &sig[..150];
    let tout_nonint = &tout[..150];
    let res_win = spectrum(sig_nonint, tout_nonint, Some(150), f0).expect("succeeds with warning");
    assert!(res_win.warnings.iter().any(|w| w.code == "W-WINDOW"));
}
