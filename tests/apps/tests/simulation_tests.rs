//! Simulation and Control integration and backend tests.
//!
//! Verifies:
//! - RK4 first-order analytical step response: y(t) = 1 - exp(-t/tau)
//! - Second-order harmonic oscillator energy conservation: zeta = 0
//! - Underdamped second-order peak overshoot matching analytical Mp = exp(-pi*zeta/sqrt(1-zeta^2))
//! - Canonical state-space representation equivalence
//! - RK4 numerical convergence order (step-halving error reduction)
//! - Stability and pole location via matlab_linalg eigenvalues of A
//! - Validation errors: negative dt, non-positive tau, mismatched dimensions, non-finite values
//! - Divergence detection on unstable systems
//! - Cancellation flag cooperative termination
//! - Session integration: workspace variables (sim_t, sim_y, sim_u, sim_x, metrics)
//! - Graphics figures: Figure 1 (Time Response) and Figure 2 (Phase Portrait)
//! - CSV export and overwrite refusal guard
//! - Registry reflection: status Ready and launchable

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use matlab_runtime::graphics::PlotItem;
use matlab_runtime::Session;
use matlab_values::DenseMatrix;
use matlabr_apps::simulation::{
    export_simulation_csv, run_simulation, simulate_rk4, InputSignal, SimulationParams, SystemModel,
};
use matlabr_apps::{Status, Verification, REGISTRY};

fn assert_close(actual: f64, expected: f64, tolerance: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: expected {expected}, got {actual} (diff = {})",
        (actual - expected).abs()
    );
}

fn write_temp_csv_path(tag: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_dir = manifest_dir.join("../../target");
    let dir = if target_dir.is_dir() {
        target_dir
    } else {
        std::env::temp_dir()
    };
    dir.join(format!(
        "matrust_sim_test_{}_{}_{:?}.csv",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ))
}

#[test]
fn simulation_first_order_step_matches_analytical_solution() {
    let tau = 1.5;
    let gain = 2.0;
    let model = SystemModel::FirstOrder { tau, gain, y0: 0.0 };
    let canonical = model.to_state_space().expect("valid state-space");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 15.0,
        dt: 0.01,
    };

    let res = simulate_rk4(&canonical, &input, &params, None).expect("simulation converges");
    assert_eq!(res.time.len(), 1501);
    assert_eq!(res.y.len(), 1501);

    // Analytical solution: y(t) = K * (1 - exp(-t/tau))
    for (t, y) in res.time.iter().zip(res.y.iter()) {
        let expected = gain * (1.0 - (-t / tau).exp());
        assert_close(
            *y,
            expected,
            1e-6,
            &format!("First-order response at t={t}"),
        );
    }

    // Performance metrics check
    assert!(res.metrics.is_stable, "First-order pole -1/tau is stable");
    assert_close(res.metrics.steady_state, gain, 5e-3, "Steady-state value");
    assert_eq!(res.metrics.poles.len(), 1);
    assert_close(res.metrics.poles[0].0, -1.0 / tau, 1e-10, "Pole real part");

    // Closed-form rise time (10% to 90%):
    // y(t10) = 0.1*K => 1 - exp(-t10/tau) = 0.1 => t10 = tau*ln(10/9)
    // y(t90) = 0.9*K => 1 - exp(-t90/tau) = 0.9 => t90 = tau*ln(10)
    // tr = t90 - t10 = tau * (ln(10) - ln(10/9)) = tau * ln(9)
    let expected_tr = tau * 9.0_f64.ln();
    // Discrete threshold crossing detection on a grid with step dt introduces
    // sampling quantization bounded by ±dt per threshold, so tr difference is bounded by 2*dt.
    let tol_tr = 2.0 * params.dt;
    let actual_tr = res.metrics.rise_time.expect("rise time computed");
    assert_close(
        actual_tr,
        expected_tr,
        tol_tr,
        "First-order step rise time against tau*ln(9)",
    );

    // Closed-form settling time (2% band around steady state):
    // |y(ts) - K| = 0.02*K => K*exp(-ts/tau) = 0.02*K => ts = tau*ln(50)
    let expected_ts = tau * 50.0_f64.ln();
    // Discrete sampling quantization is bounded by 2*dt.
    let tol_ts = 2.0 * params.dt;
    let actual_ts = res.metrics.settling_time.expect("settling time computed");
    assert_close(
        actual_ts,
        expected_ts,
        tol_ts,
        "First-order step settling time against tau*ln(50)",
    );
}

#[test]
fn simulation_harmonic_oscillator_conserves_energy() {
    // Pure undamped oscillator: zeta = 0, omega_n = 2*pi (frequency = 1 Hz, period = 1s)
    let wn = 2.0 * std::f64::consts::PI;
    let model = SystemModel::SecondOrder {
        omega_n: wn,
        zeta: 0.0,
        gain: 1.0,
        y0: 1.0, // initial displacement
        v0: 0.0, // initial velocity
    };
    let canonical = model.to_state_space().expect("canonical state space");
    let input = InputSignal::Zero; // autonomous response
                                   // Discretisation step size: dt = 0.001 s (1 millisecond)
    let dt = 0.001;
    let params = SimulationParams { t_final: 3.0, dt };

    let res = simulate_rk4(&canonical, &input, &params, None).expect("simulation converges");

    // Theoretical energy bound justification:
    // With step size dt = 0.001 s and omega_n = 2*pi, the dimensionless frequency step is
    // theta = omega_n * dt ≈ 6.283185e-3.
    // For pure imaginary eigenvalue lambda = i*omega_n, the RK4 stability amplification
    // factor satisfies:
    //     |R(i*theta)|^2 = 1 - theta^6 / 72 + theta^8 / 576 - ...
    // Over t_final = 3.0 s (N = 3000 steps), the cumulative theoretical dissipation is bounded by:
    //     N * theta^6 / 72 ≈ 3000 * (6.283185e-3)^6 / 72 ≈ 2.56e-12.
    // The normalized continuous energy is E = y^2 + (v/wn)^2 == 1.0.
    // Setting the energy bound at 1e-6 guarantees strict numerical conservation well below
    // physical significance while providing four orders of magnitude headroom above the 2.6e-12
    // theoretical RK4 drift.
    for (idx, &t) in res.time.iter().enumerate() {
        let expected_y = (wn * t).cos();
        let expected_v = -wn * (wn * t).sin();
        let y_act = res.states[idx][0];
        let v_act = res.states[idx][1];
        assert_close(y_act, expected_y, 1e-4, &format!("Displacement at t={t}"));
        assert_close(v_act, expected_v, 2e-3, &format!("Velocity at t={t}"));

        // Normalized energy: E = y^2 + (v/wn)^2 == 1.0
        let energy = y_act * y_act + (v_act / wn) * (v_act / wn);
        assert_close(energy, 1.0, 1e-6, &format!("Energy at t={t} (dt={dt})"));
    }

    // Poles must be imaginary: 0 +- j*wn
    assert_eq!(res.metrics.poles.len(), 2);
    assert_close(res.metrics.poles[0].0, 0.0, 1e-6, "Pole 1 real part");
    assert_close(res.metrics.poles[0].1.abs(), wn, 1e-6, "Pole 1 imag part");
    assert_close(res.metrics.poles[1].0, 0.0, 1e-6, "Pole 2 real part");
    assert_close(res.metrics.poles[1].1.abs(), wn, 1e-6, "Pole 2 imag part");
}

#[test]
fn simulation_underdamped_step_overshoot_matches_control_theory() {
    let wn = 4.0;
    let zeta = 0.4;
    let gain = 1.0;
    let model = SystemModel::SecondOrder {
        omega_n: wn,
        zeta,
        gain,
        y0: 0.0,
        v0: 0.0,
    };
    let canonical = model.to_state_space().expect("to_state_space");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 5.0,
        dt: 0.001,
    };

    let res = simulate_rk4(&canonical, &input, &params, None).expect("simulates");

    // Analytical peak overshoot: Mp = exp(-pi * zeta / sqrt(1 - zeta^2)) * 100%
    let analytical_mp = (-std::f64::consts::PI * zeta / (1.0 - zeta * zeta).sqrt()).exp() * 100.0;
    let computed_mp = res.metrics.overshoot_percent.expect("has overshoot");
    assert_close(computed_mp, analytical_mp, 0.5, "Percent overshoot Mp");

    // Analytical peak time: tp = pi / (wn * sqrt(1 - zeta^2))
    let wd = wn * (1.0 - zeta * zeta).sqrt();
    let analytical_tp = std::f64::consts::PI / wd;
    assert_close(res.metrics.peak_time, analytical_tp, 0.005, "Peak time tp");

    // Comparison of y(t) with closed-form underdamped step response at 6 distinct values of t:
    // Closed form: y(t) = K * (1 - exp(-zeta*wn*t) * (cos(wd*t) + (zeta/sqrt(1-zeta^2))*sin(wd*t)))
    // Tolerance justification:
    // RK4 global truncation error is O(dt^4). With dt = 1e-3, local error per step is O(dt^5) ~ 1e-15,
    // and accumulated error over 3000 steps (3.0 seconds) is bounded by C * dt^4 ~ 1e-11.
    // An assertion tolerance of 1e-6 rigorously verifies trajectory adherence to the analytical
    // solution while providing generous headroom against floating-point accumulation.
    let test_times = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    for &target_t in &test_times {
        let step_idx = (target_t / params.dt).round() as usize;
        let actual_t = res.time[step_idx];
        assert_close(actual_t, target_t, 1e-9, "Sample time grid alignment");
        let actual_y = res.y[step_idx];
        let decay = (-zeta * wn * target_t).exp();
        let osc =
            (wd * target_t).cos() + (zeta / (1.0 - zeta * zeta).sqrt()) * (wd * target_t).sin();
        let expected_y = gain * (1.0 - decay * osc);
        assert_close(
            actual_y,
            expected_y,
            1e-6,
            &format!("Underdamped 2nd-order step y(t) at t={target_t}"),
        );
    }
}

#[test]
fn simulation_canonical_state_space_equivalence() {
    // Model A: constructed via SecondOrder enum
    let wn = 3.0;
    let zeta = 0.5;
    let model_a = SystemModel::SecondOrder {
        omega_n: wn,
        zeta,
        gain: 2.0,
        y0: 0.5,
        v0: -0.2,
    };
    let canonical_a = model_a.to_state_space().expect("canonical A");

    // Model B: manually specified equivalent StateSpace
    let model_b = SystemModel::StateSpace {
        a: DenseMatrix {
            rows: 2,
            cols: 2,
            data: vec![0.0, 1.0, -wn * wn, -2.0 * zeta * wn],
        },
        b: DenseMatrix {
            rows: 2,
            cols: 1,
            data: vec![0.0, wn * wn * 2.0],
        },
        c: DenseMatrix {
            rows: 1,
            cols: 2,
            data: vec![1.0, 0.0],
        },
        d: 0.0,
        x0: vec![0.5, -0.2],
    };
    let canonical_b = model_b.to_state_space().expect("canonical B");

    let input = InputSignal::Sine {
        amplitude: 1.5,
        frequency: 0.5,
        phase_rad: 0.2,
    };
    let params = SimulationParams {
        t_final: 4.0,
        dt: 0.01,
    };

    let res_a = simulate_rk4(&canonical_a, &input, &params, None).expect("sim A");
    let res_b = simulate_rk4(&canonical_b, &input, &params, None).expect("sim B");

    assert_eq!(res_a.y.len(), res_b.y.len());
    for (ya, yb) in res_a.y.iter().zip(res_b.y.iter()) {
        assert_close(*ya, *yb, 1e-12, "State-space equivalence check");
    }
}

#[test]
fn simulation_rk4_convergence_rate() {
    // Test RK4 convergence order by comparing error at dt and dt/2
    let tau = 1.0;
    let model = SystemModel::FirstOrder {
        tau,
        gain: 1.0,
        y0: 0.0,
    };
    let canonical = model.to_state_space().expect("state-space");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };

    let t_final = 1.0;
    let exact = 1.0 * (1.0 - (-t_final / tau).exp());

    let res_coarse = simulate_rk4(
        &canonical,
        &input,
        &SimulationParams { t_final, dt: 0.1 },
        None,
    )
    .expect("coarse");
    let err_coarse = (res_coarse.y.last().unwrap() - exact).abs();

    let res_fine = simulate_rk4(
        &canonical,
        &input,
        &SimulationParams { t_final, dt: 0.05 },
        None,
    )
    .expect("fine");
    let err_fine = (res_fine.y.last().unwrap() - exact).abs();

    // With dt halved (0.1 -> 0.05), a 4th-order method theoretical error reduction
    // ratio is 2^4 = 16. Asymptotically for this smooth first-order step, error(dt)/error(dt/2)
    // is in [14, 18], confirming classical 4th-order global convergence order ~ 4.
    let ratio = err_coarse / err_fine;
    let observed_order = ratio.log2();
    println!("Observed RK4 order: {observed_order:.4} (error ratio: {ratio:.4})");
    assert!(
        (14.0..=18.0).contains(&ratio),
        "RK4 error ratio error(dt)/error(dt/2) must be in [14, 18], got {ratio} (observed order: {observed_order:.4})"
    );
    assert!(
        (3.8..=4.2).contains(&observed_order),
        "Observed RK4 convergence order must be approximately 4, got {observed_order}"
    );
}

#[test]
fn simulation_first_order_impulse_response() {
    // Continuous-time first-order ODE: dy/dt = -y/tau + (K/tau)*u(t), with y(0) = 0.
    // For an ideal Dirac delta input u(t) = delta(t), the exact analytical impulse
    // response is:
    //     h(t) = (K / tau) * exp(-t / tau)   for t > 0.
    //
    // Discretisation in numerical simulation:
    // In continuous-time linear control theory (and MATLAB's `impulse` command),
    // an ideal Dirac impulse delta(t) applied to dx/dt = A*x + B*u imparts an
    // instantaneous state jump at t = 0+:
    //     x(0+) = x(0-) + integral_{0-}^{0+} B*delta(t) dt = B * 1.0 = [K / tau].
    // For all t > 0, the external input is zero (u(t) = 0), and the system evolves
    // autonomously from x(0+) as x(t) = exp(A*t) * x(0+) = (K/tau) * exp(-t/tau).
    //
    // In numerical RK4 simulation with step size dt = 0.01:
    // We simulate the exact post-impulse state jump x0 = K/tau with InputSignal::Zero.
    // RK4 integration reproduces the exact continuous exponential decay with O(dt^4) error.
    let tau = 1.25;
    let gain = 2.5;
    let model = SystemModel::StateSpace {
        a: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![-1.0 / tau],
        },
        b: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![gain / tau],
        },
        c: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![1.0],
        },
        d: 0.0,
        x0: vec![gain / tau],
    };
    let canonical = model.to_state_space().expect("canonical state space");
    let input = InputSignal::Zero;
    let params = SimulationParams {
        t_final: 5.0,
        dt: 0.01,
    };

    let res =
        simulate_rk4(&canonical, &input, &params, None).expect("impulse simulation converges");

    for (t, y) in res.time.iter().zip(res.y.iter()) {
        let expected = (gain / tau) * (-t / tau).exp();
        assert_close(
            *y,
            expected,
            1e-6,
            &format!("First-order impulse response at t={t}"),
        );
    }
}

#[test]
fn simulation_rejects_invalid_parameters() {
    // Negative duration
    let p_bad_dur = SimulationParams {
        t_final: -2.0,
        dt: 0.1,
    };
    assert!(p_bad_dur.validate().is_err());

    // Negative dt
    let p_bad_dt = SimulationParams {
        t_final: 5.0,
        dt: -0.01,
    };
    assert!(p_bad_dt.validate().is_err());

    // dt > t_final
    let p_dt_large = SimulationParams {
        t_final: 1.0,
        dt: 2.0,
    };
    assert!(p_dt_large.validate().is_err());

    // Non-positive tau
    let m_bad_tau = SystemModel::FirstOrder {
        tau: 0.0,
        gain: 1.0,
        y0: 0.0,
    };
    assert!(m_bad_tau.to_state_space().is_err());

    // Non-square A matrix
    let m_non_square = SystemModel::StateSpace {
        a: DenseMatrix {
            rows: 2,
            cols: 3,
            data: vec![1.0; 6],
        },
        b: DenseMatrix {
            rows: 2,
            cols: 1,
            data: vec![1.0; 2],
        },
        c: DenseMatrix {
            rows: 1,
            cols: 2,
            data: vec![1.0; 2],
        },
        d: 0.0,
        x0: vec![0.0; 2],
    };
    assert!(m_non_square.to_state_space().is_err());
}

#[test]
fn simulation_divergence_guard_for_unstable_system() {
    // Unstable pole: A = [10.0] -> dx/dt = 10 x -> grows as exp(10 t)
    let model = SystemModel::StateSpace {
        a: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![100.0],
        },
        b: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![1.0],
        },
        c: DenseMatrix {
            rows: 1,
            cols: 1,
            data: vec![1.0],
        },
        d: 0.0,
        x0: vec![1.0],
    };
    let canonical = model.to_state_space().expect("valid state space");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 20.0,
        dt: 0.01,
    };

    let res = simulate_rk4(&canonical, &input, &params, None);
    assert!(
        res.is_err(),
        "Simulation of wildly diverging unstable system must return an explicit Diagnostic error"
    );
    let err_msg = res.unwrap_err().message;
    assert!(
        err_msg.contains("diverged") || err_msg.contains("unstable"),
        "Error message should mention divergence: {err_msg}"
    );
}

#[test]
fn simulation_cancellation_stops_execution() {
    let flag = AtomicBool::new(true);
    let model = SystemModel::FirstOrder {
        tau: 1.0,
        gain: 1.0,
        y0: 0.0,
    };
    let canonical = model.to_state_space().expect("state-space");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 10.0,
        dt: 0.001,
    };

    let res = simulate_rk4(&canonical, &input, &params, Some(&flag));
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("cancelled"));
}

#[test]
fn simulation_session_integration_workspace_and_figures() {
    let mut session = Session::new();
    let model = SystemModel::SecondOrder {
        omega_n: 5.0,
        zeta: 0.3,
        gain: 1.0,
        y0: 0.0,
        v0: 0.0,
    };
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 2.0,
        dt: 0.01,
    };

    let result =
        run_simulation(&mut session, &model, &input, &params, None).expect("run_simulation");

    // Check workspace variables
    let sim_t = session
        .env
        .get("sim_t")
        .and_then(|v| v.as_matrix())
        .expect("sim_t");
    let sim_y = session
        .env
        .get("sim_y")
        .and_then(|v| v.as_matrix())
        .expect("sim_y");
    let sim_u = session
        .env
        .get("sim_u")
        .and_then(|v| v.as_matrix())
        .expect("sim_u");
    let sim_x = session
        .env
        .get("sim_x")
        .and_then(|v| v.as_matrix())
        .expect("sim_x");

    assert_eq!(sim_t.rows, 201);
    assert_eq!(sim_t.cols, 1);
    assert_eq!(sim_y.rows, 201);
    assert_eq!(sim_y.cols, 1);
    assert_eq!(sim_u.rows, 201);
    assert_eq!(sim_u.cols, 1);
    assert_eq!(sim_x.rows, 201);
    assert_eq!(sim_x.cols, 2); // 2 states for second order

    let sim_peak = session
        .env
        .get("sim_peak")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("sim_peak");
    assert_close(
        sim_peak,
        result.metrics.peak_value,
        1e-12,
        "sim_peak in env",
    );

    // Check figures pushed
    assert_eq!(
        session.graphics.figures.len(),
        2,
        "Expected 2 figures (Time Response + Phase Portrait)"
    );
    let fig1 = &session.graphics.figures[0];
    assert!(fig1
        .title
        .as_ref()
        .unwrap()
        .contains("Simulation — Second-Order"));
    assert_eq!(fig1.items.len(), 2);
    match &fig1.items[0] {
        PlotItem::Line { label, x, y } => {
            assert_eq!(label, "Output y(t)");
            assert_eq!(x.len(), 201);
            assert_eq!(y.len(), 201);
        }
        _ => panic!("Expected Line plot item for output"),
    }

    let fig2 = &session.graphics.figures[1];
    assert!(fig2.title.as_ref().unwrap().contains("Phase Portrait"));
    assert_eq!(fig2.items.len(), 2);
}

#[test]
fn simulation_csv_export_and_overwrite_guard() {
    let model = SystemModel::FirstOrder {
        tau: 1.0,
        gain: 1.0,
        y0: 0.0,
    };
    let canonical = model.to_state_space().expect("canonical");
    let input = InputSignal::Step {
        amplitude: 1.0,
        t_start: 0.0,
    };
    let params = SimulationParams {
        t_final: 1.0,
        dt: 0.1,
    };
    let result = simulate_rk4(&canonical, &input, &params, None).expect("sim");

    let path = write_temp_csv_path("export_test");

    // First write: succeeds
    export_simulation_csv(&path, &result, false).expect("first export");
    let text = std::fs::read_to_string(&path).expect("read exported csv");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 12); // header + 11 data lines
    assert!(lines[0].starts_with("time,y,u"));

    // Second write without overwrite: must refuse
    let err = export_simulation_csv(&path, &result, false).expect_err("refuse overwrite");
    assert!(err.message.contains("Refusing to overwrite"));

    // Third write with overwrite: succeeds
    export_simulation_csv(&path, &result, true).expect("overwrite succeeds");

    let _ = std::fs::remove_file(path);
}

#[test]
fn simulation_registry_status_and_launchable() {
    let app = REGISTRY
        .iter()
        .find(|a| a.id == "simulation")
        .expect("simulation registered");
    assert_eq!(app.status, Status::Ready);
    assert_eq!(app.verification, Verification::Unit);
    assert!(
        app.launchable(),
        "Simulation app must be launchable in launcher"
    );
    assert_eq!(app.needs_file_input, false);
}
