//! Milestone M5 integration tests: Modified Nodal Analysis (MNA), companion models,
//! dense self-contained LU decomposition with partial pivoting, linear electrical circuits,
//! analytical oracles, and solver diagnostics (E-SINGULAR, E-SOLVER, E-UNSUPPORTED).

use mlapp_sim::electrical::LuDecomposition;
use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SolverType};

#[test]
fn test_lu_decomposition_pivoting_and_singular_rejection() {
    // 1. Matrix requiring row swap:
    // [0  2] [x1] = [4]
    // [3  1] [x2]   [5]
    // Solution: x1 = 1, x2 = 2
    let a = vec![0.0, 2.0, 3.0, 1.0];
    let lu = LuDecomposition::decompose(&a, 2).expect("Decomposition must succeed with pivoting");
    let b = vec![4.0, 5.0];
    let mut x = vec![0.0; 2];
    lu.solve(&b, &mut x);
    assert!((x[0] - 1.0).abs() < 1e-12, "x[0] must be 1.0, got {}", x[0]);
    assert!((x[1] - 2.0).abs() < 1e-12, "x[1] must be 2.0, got {}", x[1]);

    // 2. 3x3 general system:
    // [ 1  2  3 ] [x1]   [ 14 ]
    // [ 2 -1  1 ] [x2] = [  3 ]
    // [ 3  0 -1 ] [x3]   [  0 ]
    // Solution: x1 = 1, x2 = 2, x3 = 3
    let a3 = vec![1.0, 2.0, 3.0, 2.0, -1.0, 1.0, 3.0, 0.0, -1.0];
    let lu3 = LuDecomposition::decompose(&a3, 3).expect("Decomposition must succeed");
    let b3 = vec![14.0, 3.0, 0.0];
    let mut x3 = vec![0.0; 3];
    lu3.solve(&b3, &mut x3);
    assert!((x3[0] - 1.0).abs() < 1e-12);
    assert!((x3[1] - 2.0).abs() < 1e-12);
    assert!((x3[2] - 3.0).abs() < 1e-12);

    // 3. Singular matrix (linearly dependent rows):
    // [ 1  2 ]
    // [ 2  4 ]
    let a_sing = vec![1.0, 2.0, 2.0, 4.0];
    let err = LuDecomposition::decompose(&a_sing, 2).expect_err("Singular matrix must fail");
    assert_eq!(err.code(), "E-SINGULAR");
    assert!(err.message().contains("Zero pivot"));
    assert!(err.message().contains("Why:"));
    assert!(err.message().contains("How to fix:"));
}

#[test]
fn test_resistive_voltage_divider_dc_mna() {
    // 10V DC Voltage Source, R1 = 1000 Ohm, R2 = 1000 Ohm
    // Vsrc: p -> Net 1, n -> Net 0 (Ground)
    // R1:   p -> Net 1, n -> Net 2
    // R2:   p -> Net 2, n -> Net 0 (Ground)
    let mut d = BlockDiagram::new("VoltageDivider");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vsrc", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "1000.0"));

    // Vs(p) to R1(p) -> Net 1
    d.add_electrical_line("vsrc", 0, "r1", 0);
    // R1(n) to R2(p) -> Net 2
    d.add_electrical_line("r1", 1, "r2", 0);
    // Vs(n) to GND -> Net 0
    d.add_electrical_line("vsrc", 1, "gnd", 0);
    // R2(n) to GND -> Net 0
    d.add_electrical_line("r2", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    // Theoretical values:
    // Net 1 (source node) = 10.0 V
    // Net 2 (divider node) = 5.0 V
    // Current through R1 and R2 = 10 / 2000 = 0.005 A (5 mA)
    // Voltage across R1 = 5.0 V
    // Voltage across R2 = 5.0 V
    for &val in &out.signals["Net1"] {
        assert!((val - 10.0).abs() < 1e-10, "Net1 must be 10V, got {val}");
    }
    for &val in &out.signals["Net2"] {
        assert!((val - 5.0).abs() < 1e-10, "Net2 must be 5V, got {val}");
    }
    for &val in &out.signals["R1.v"] {
        assert!((val - 5.0).abs() < 1e-10, "R1.v must be 5V, got {val}");
    }
    for &val in &out.signals["R2.v"] {
        assert!((val - 5.0).abs() < 1e-10, "R2.v must be 5V, got {val}");
    }
    for &val in &out.signals["R1.i"] {
        assert!((val - 0.005).abs() < 1e-10, "R1.i must be 5mA, got {val}");
    }
    for &val in &out.signals["R2.i"] {
        assert!((val - 0.005).abs() < 1e-10, "R2.i must be 5mA, got {val}");
    }
}

#[test]
fn test_dc_current_source_parallel_resistor() {
    // Current source 2.0 A in parallel with 50.0 Ohm resistor
    let mut d = BlockDiagram::new("CurrentSourceParallelResistor");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("isrc", "Is", BlockType::DcCurrentSource).with_param("Current", "2.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "50.0"));

    // Is(p) to R(p) -> Net 1
    d.add_electrical_line("isrc", 0, "r", 0);
    // Is(n) to GND -> Net 0
    d.add_electrical_line("isrc", 1, "gnd", 0);
    // R(n) to GND -> Net 0
    d.add_electrical_line("r", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    // V = I * R = 2.0 * 50.0 = 100.0 V
    for &val in &out.signals["Net1"] {
        assert!((val - 100.0).abs() < 1e-10, "Net1 must be 100V, got {val}");
    }
    for &val in &out.signals["R.v"] {
        assert!((val - 100.0).abs() < 1e-10, "R.v must be 100V, got {val}");
    }
    for &val in &out.signals["R.i"] {
        assert!((val - 2.0).abs() < 1e-10, "R.i must be 2A, got {val}");
    }
}

#[test]
fn test_rc_circuit_step_response_analytical_oracle() {
    // Series RC circuit:
    // Vs = 10.0 V, R = 1000 Ohm, C = 1 uF (1e-6 F).
    // Time constant tau = R * C = 1e-3 s = 1.0 ms.
    // Initial condition V0 = 0.0 V.
    // Analytical oracle:
    //   vc(t) = 10 * (1 - exp(-t / tau))
    //   ic(t) = (10 / R) * exp(-t / tau) = 0.01 * exp(-t / tau)
    let mut d = BlockDiagram::new("RcTransient");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(
        Block::new("c", "C", BlockType::Capacitor)
            .with_param("Capacitance", "1e-6")
            .with_param("InitialVoltage", "0.0"),
    );

    // Vs(p) to R(p) -> Net 1
    d.add_electrical_line("vs", 0, "r", 0);
    // R(n) to C(p) -> Net 2
    d.add_electrical_line("r", 1, "c", 0);
    // C(n) to GND -> Net 0
    d.add_electrical_line("c", 1, "gnd", 0);
    // Vs(n) to GND -> Net 0
    d.add_electrical_line("vs", 1, "gnd", 0);

    let dt = 1e-5; // 10 us
    let t_stop = 0.005; // 5 ms (5 time constants)
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: t_stop,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    let tau = 0.001;
    let mut max_vc_err: f64 = 0.0;
    let mut max_ic_err: f64 = 0.0;

    let c_v = &out.signals["C.v"];
    let c_i = &out.signals["C.i"];

    for (k, &t) in out.tout.iter().enumerate() {
        let expected_vc = 10.0 * (1.0 - (-t / tau).exp());
        let expected_ic = 0.01 * (-t / tau).exp();

        let vc = c_v[k];
        let ic = c_i[k];

        let err_vc = (vc - expected_vc).abs();
        let err_ic = (ic - expected_ic).abs();

        if err_vc > max_vc_err {
            max_vc_err = err_vc;
        }
        if err_ic > max_ic_err {
            max_ic_err = err_ic;
        }
    }

    // Dommel Trapezoidal rule has O(dt^2) local truncation error, with Backward Euler step 1
    // introducing ~0.5*dt/tau truncation error on the initial transition (~0.49 mV on 10V scale).
    assert!(
        max_vc_err < 1e-3,
        "Max capacitor voltage error {max_vc_err} exceeds 1e-3 V tolerance"
    );
    assert!(
        max_ic_err < 1e-6,
        "Max capacitor current error {max_ic_err} exceeds 1e-6 A tolerance"
    );

    // Check specific value at t = tau (1 ms): vc(tau) = 10 * (1 - 1/e) ~ 6.3212 V
    let idx_1ms = (0.001 / dt).round() as usize;
    let vc_1ms = c_v[idx_1ms];
    let exact_1ms = 10.0 * (1.0 - (-1.0f64).exp());
    assert!(
        (vc_1ms - exact_1ms).abs() < 1e-3,
        "At t = 1ms, expected {exact_1ms}, got {vc_1ms}"
    );
}

#[test]
fn test_rl_circuit_step_response_analytical_oracle() {
    // Series RL circuit:
    // Vs = 10.0 V, R = 100 Ohm, L = 0.1 H.
    // Time constant tau = L / R = 0.1 / 100 = 0.001 s = 1.0 ms.
    // Steady state current = Vs / R = 10 / 100 = 0.1 A.
    // Analytical oracle:
    //   il(t) = (Vs / R) * (1 - exp(-t / tau)) = 0.1 * (1 - exp(-t / 0.001))
    //   vl(t) = Vs * exp(-t / tau) = 10.0 * exp(-t / 0.001)
    let mut d = BlockDiagram::new("RlTransient");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "100.0"));
    d.add_block(
        Block::new("l", "L", BlockType::Inductor)
            .with_param("Inductance", "0.1")
            .with_param("InitialCurrent", "0.0"),
    );

    // Vs(p) to R(p) -> Net 1
    d.add_electrical_line("vs", 0, "r", 0);
    // R(n) to L(p) -> Net 2
    d.add_electrical_line("r", 1, "l", 0);
    // L(n) to GND -> Net 0
    d.add_electrical_line("l", 1, "gnd", 0);
    // Vs(n) to GND -> Net 0
    d.add_electrical_line("vs", 1, "gnd", 0);

    let dt = 1e-5; // 10 us
    let t_stop = 0.005; // 5 ms
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: t_stop,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    let tau = 0.001;
    let mut max_il_err: f64 = 0.0;
    let mut max_vl_err: f64 = 0.0;

    let l_v = &out.signals["L.v"];
    let l_i = &out.signals["L.i"];

    for (k, &t) in out.tout.iter().enumerate() {
        let expected_il = 0.1 * (1.0 - (-t / tau).exp());
        let expected_vl = 10.0 * (-t / tau).exp();

        let il = l_i[k];
        let vl = l_v[k];

        let err_il = (il - expected_il).abs();
        let err_vl = (vl - expected_vl).abs();

        if err_il > max_il_err {
            max_il_err = err_il;
        }
        if err_vl > max_vl_err {
            max_vl_err = err_vl;
        }
    }

    assert!(
        max_il_err < 1e-4,
        "Max inductor current error {max_il_err} exceeds 1e-4 A tolerance"
    );
    assert!(
        max_vl_err < 1e-3,
        "Max inductor voltage error {max_vl_err} exceeds 1e-3 V tolerance"
    );
}

#[test]
fn test_rlc_second_order_underdamped_transient_oracle() {
    // Series RLC circuit:
    // Vs = 10.0 V, R = 20 Ohm, L = 0.01 H (10 mH), C = 10 uF (1e-5 F).
    // omega0^2 = 1 / (L * C) = 1 / (0.01 * 1e-5) = 1e7
    // alpha = R / (2 * L) = 20 / (2 * 0.01) = 1000 rad/s
    // alpha^2 = 1e6 < omega0^2 = 1e7 => Underdamped!
    // omega_d = sqrt(omega0^2 - alpha^2) = sqrt(1e7 - 1e6) = sqrt(9e6) = 3000 rad/s
    // Analytical step response for capacitor voltage:
    //   vc(t) = Vs * [1 - exp(-alpha * t) * (cos(omega_d * t) + (alpha / omega_d) * sin(omega_d * t))]
    // Peak time: t_peak = pi / omega_d = pi / 3000 ~ 1.0472 ms
    // Peak overshoot: vc(t_peak) = Vs * (1 + exp(-alpha * pi / omega_d)) = 10 * (1 + exp(-pi / 3)) ~ 13.5124 V
    let mut d = BlockDiagram::new("RlcUnderdamped");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "20.0"));
    d.add_block(
        Block::new("l", "L", BlockType::Inductor)
            .with_param("Inductance", "0.01")
            .with_param("InitialCurrent", "0.0"),
    );
    d.add_block(
        Block::new("c", "C", BlockType::Capacitor)
            .with_param("Capacitance", "1e-5")
            .with_param("InitialVoltage", "0.0"),
    );

    // Vs(p) to R(p) -> Net 1
    d.add_electrical_line("vs", 0, "r", 0);
    // R(n) to L(p) -> Net 2
    d.add_electrical_line("r", 1, "l", 0);
    // L(n) to C(p) -> Net 3
    d.add_electrical_line("l", 1, "c", 0);
    // C(n) to GND -> Net 0
    d.add_electrical_line("c", 1, "gnd", 0);
    // Vs(n) to GND -> Net 0
    d.add_electrical_line("vs", 1, "gnd", 0);

    let dt = 5e-6; // 5 us
    let t_stop = 0.005; // 5 ms
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: t_stop,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    let alpha = 1000.0;
    let omega_d = 3000.0;
    let mut max_err: f64 = 0.0;
    let c_v = &out.signals["C.v"];

    for (k, &t) in out.tout.iter().enumerate() {
        let expected_vc = 10.0
            * (1.0
                - (-alpha * t).exp()
                    * ((omega_d * t).cos() + (alpha / omega_d) * (omega_d * t).sin()));
        let vc = c_v[k];
        let err = (vc - expected_vc).abs();
        if err > max_err {
            max_err = err;
        }
    }

    assert!(
        max_err < 0.02,
        "Max RLC transient error {max_err} exceeds 0.02 V tolerance"
    );

    // Check peak voltage near t_peak = pi / 3000 ~ 1.0472 ms
    let t_peak = std::f64::consts::PI / 3000.0;
    let peak_idx = (t_peak / dt).round() as usize;
    let vc_peak = c_v[peak_idx];
    let expected_peak = 10.0 * (1.0 + (-std::f64::consts::PI / 3.0).exp());
    assert!(
        (vc_peak - expected_peak).abs() < 0.02,
        "Peak overshoot expected {expected_peak}, got {vc_peak}"
    );
}

#[test]
fn test_ac_voltage_source_frequency_response() {
    // AC Source: 10 V amplitude, 1000 Hz frequency, 0 phase, 0 bias.
    // RC Low-pass filter: R = 1000 Ohm, C = 1 / (2 * pi * 1000 * 1000) = 1.5915494309e-7 F
    // Cut-off frequency fc = 1000 Hz.
    // At fc, gain |H(j w)| = 1 / sqrt(2) ~ 0.70710678, phase shift = -45 deg.
    // Expected steady state amplitude across C = 10.0 / sqrt(2) ~ 7.071 V.
    let mut d = BlockDiagram::new("AcRcFilter");
    let c_val = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1000.0);
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "10.0")
            .with_param("Frequency", "1000.0")
            .with_param("Phase", "0.0")
            .with_param("Bias", "0.0"),
    );
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(
        Block::new("c", "C", BlockType::Capacitor)
            .with_param("Capacitance", format!("{c_val:.12e}"))
            .with_param("InitialVoltage", "0.0"),
    );

    // Vac(p) to R(p) -> Net 1
    d.add_electrical_line("vac", 0, "r", 0);
    // R(n) to C(p) -> Net 2
    d.add_electrical_line("r", 1, "c", 0);
    // C(n) to GND -> Net 0
    d.add_electrical_line("c", 1, "gnd", 0);
    // Vac(n) to GND -> Net 0
    d.add_electrical_line("vac", 1, "gnd", 0);

    let dt = 1e-6; // 1 us step size (1000 points per cycle)
    let t_stop = 0.01; // 10 ms (10 full cycles)
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: t_stop,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");
    let c_v = &out.signals["C.v"];

    // Inspect peak value in the final cycle (t in [9 ms, 10 ms])
    let start_idx = (0.009 / dt).round() as usize;
    let end_idx = (0.010 / dt).round() as usize;
    let mut peak_val: f64 = 0.0;
    for &v in &c_v[start_idx..=end_idx] {
        if v.abs() > peak_val {
            peak_val = v.abs();
        }
    }

    let theoretical_peak = 10.0 / (2.0f64).sqrt();
    let rel_err = (peak_val - theoretical_peak).abs() / theoretical_peak;
    assert!(
        rel_err < 0.005,
        "Peak amplitude {peak_val} deviates from theoretical {theoretical_peak} by {rel_err:.4}"
    );
}

#[test]
fn test_singular_circuit_ideal_voltage_source_loop_gives_e_singular() {
    // Two ideal voltage sources directly in parallel between Net 1 and Ground.
    // This creates an infinite current branch / singular MNA matrix.
    let mut d = BlockDiagram::new("ParallelVoltageSources");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("v1", "V1", BlockType::DcVoltageSource).with_param("Voltage", "5.0"));
    d.add_block(Block::new("v2", "V2", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));

    // V1(p) to V2(p) -> Net 1
    d.add_electrical_line("v1", 0, "v2", 0);
    // V1(n) to GND -> Net 0
    d.add_electrical_line("v1", 1, "gnd", 0);
    // V2(n) to GND -> Net 0
    d.add_electrical_line("v2", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let err = simulate_diagram(&d, &config).expect_err("Parallel voltage sources must fail");
    assert_eq!(err.code(), "E-SINGULAR");
    assert!(err.message().contains("Zero pivot"));
    assert!(err.message().contains("Why:"));
}

#[test]
fn test_variable_step_ode45_on_electrical_circuit_gives_e_solver() {
    // Circuits require fixed-step solver in MNA companion models.
    // Variable-step solvers must return typed E-SOLVER error.
    let mut d = BlockDiagram::new("ElectricalWithVariableStep");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "100.0"));

    d.add_electrical_line("vs", 0, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vs", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };

    let err = simulate_diagram(&d, &config).expect_err("Variable-step on electrical must fail");
    assert_eq!(err.code(), "E-SOLVER");
    assert!(err.message().contains("Variable-step Dormand-Prince"));
    assert!(err.message().contains("Why:"));
    assert!(err.message().contains("How to fix:"));
}

#[test]
fn test_nonlinear_electrical_blocks_give_e_unsupported() {
    // Milestone M5 supports linear circuits. Diode and IdealSwitch arrive in M6.
    let mut d = BlockDiagram::new("DiodeCircuit");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "5.0"));
    d.add_block(Block::new("d", "D1", BlockType::Diode));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "100.0"));

    d.add_electrical_line("vs", 0, "d", 0);
    d.add_electrical_line("d", 1, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vs", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let err = simulate_diagram(&d, &config).expect_err("Diode must produce E-UNSUPPORTED in M5");
    assert_eq!(err.code(), "E-UNSUPPORTED");
    assert!(err.message().contains("Milestone M6"));
}

#[test]
fn test_dual_ground_and_partitioned_subcircuits() {
    // Two independent subcircuits sharing reference Ground:
    // Subcircuit 1: V1 (10V) -> R1 (1000 Ohm) -> Ground
    // Subcircuit 2: V2 (5V) -> R2 (500 Ohm) -> Ground
    let mut d = BlockDiagram::new("DualSubcircuits");
    d.add_block(Block::new("gnd1", "GND1", BlockType::Ground));
    d.add_block(Block::new("gnd2", "GND2", BlockType::Ground));
    d.add_block(Block::new("v1", "V1", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(Block::new("v2", "V2", BlockType::DcVoltageSource).with_param("Voltage", "5.0"));
    d.add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "500.0"));

    // Subcircuit 1
    d.add_electrical_line("v1", 0, "r1", 0);
    d.add_electrical_line("r1", 1, "gnd1", 0);
    d.add_electrical_line("v1", 1, "gnd1", 0);

    // Subcircuit 2
    d.add_electrical_line("v2", 0, "r2", 0);
    d.add_electrical_line("r2", 1, "gnd2", 0);
    d.add_electrical_line("v2", 1, "gnd2", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("Simulation must succeed");

    for &v in &out.signals["R1.v"] {
        assert!((v - 10.0).abs() < 1e-10, "R1.v must be 10V, got {v}");
    }
    for &i in &out.signals["R1.i"] {
        assert!((i - 0.010).abs() < 1e-10, "R1.i must be 10mA, got {i}");
    }

    for &v in &out.signals["R2.v"] {
        assert!((v - 5.0).abs() < 1e-10, "R2.v must be 5V, got {v}");
    }
    for &i in &out.signals["R2.i"] {
        assert!((i - 0.010).abs() < 1e-10, "R2.i must be 10mA, got {i}");
    }
}

#[test]
fn test_rk4_solver_with_electrical_circuit() {
    // Test that FixedStepRk4 also executes MNA stepping correctly.
    let mut d = BlockDiagram::new("Rk4Circuit");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vsrc", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "12.0"));
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "600.0"));

    d.add_electrical_line("vsrc", 0, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vsrc", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 0.001,
        solver: SolverType::FixedStepRk4,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("RK4 circuit simulation must succeed");
    for &v in &out.signals["R.v"] {
        assert!((v - 12.0).abs() < 1e-10);
    }
    for &i in &out.signals["R.i"] {
        assert!((i - 0.020).abs() < 1e-10);
    }
}
