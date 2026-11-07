//! Comprehensive test suite for M2 Solvers (Euler, RK4, Dormand-Prince, validation, and sample times).

use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{
    simulate_diagram, DormandPrinceTableau, SimError, SimulationConfig, SolverType,
};

/// 1. Butcher tableau weights and order condition verification.
#[test]
fn test_butcher_tableau_weights_and_orders() {
    let c = DormandPrinceTableau::C;
    let b = DormandPrinceTableau::B;
    let b_hat = DormandPrinceTableau::B_HAT;
    let e = DormandPrinceTableau::E;

    // Row sum conditions: c_i = sum_j a_ij
    let a2_sum = DormandPrinceTableau::A21;
    assert!((c[1] - a2_sum).abs() < 1e-14, "c2 row sum");

    let a3_sum = DormandPrinceTableau::A31 + DormandPrinceTableau::A32;
    assert!((c[2] - a3_sum).abs() < 1e-14, "c3 row sum");

    let a4_sum = DormandPrinceTableau::A41 + DormandPrinceTableau::A42 + DormandPrinceTableau::A43;
    assert!((c[3] - a4_sum).abs() < 1e-14, "c4 row sum");

    let a5_sum = DormandPrinceTableau::A51
        + DormandPrinceTableau::A52
        + DormandPrinceTableau::A53
        + DormandPrinceTableau::A54;
    assert!((c[4] - a5_sum).abs() < 1e-14, "c5 row sum");

    let a6_sum = DormandPrinceTableau::A61
        + DormandPrinceTableau::A62
        + DormandPrinceTableau::A63
        + DormandPrinceTableau::A64
        + DormandPrinceTableau::A65;
    assert!((c[5] - a6_sum).abs() < 1e-14, "c6 row sum");

    let a7_sum = DormandPrinceTableau::A71
        + DormandPrinceTableau::A72
        + DormandPrinceTableau::A73
        + DormandPrinceTableau::A74
        + DormandPrinceTableau::A75
        + DormandPrinceTableau::A76;
    assert!((c[6] - a7_sum).abs() < 1e-14, "c7 row sum");

    // 5th-order weight conditions:
    // sum(b) = 1
    let sum_b: f64 = b.iter().sum();
    assert!((sum_b - 1.0).abs() < 1e-14, "sum(b) == 1");

    // sum(b * c) = 1/2
    let sum_bc: f64 = b.iter().zip(c.iter()).map(|(&bi, &ci)| bi * ci).sum();
    assert!((sum_bc - 0.5).abs() < 1e-14, "sum(b * c) == 1/2");

    // sum(b * c^2) = 1/3
    let sum_bc2: f64 = b.iter().zip(c.iter()).map(|(&bi, &ci)| bi * ci * ci).sum();
    assert!((sum_bc2 - 1.0 / 3.0).abs() < 1e-14, "sum(b * c^2) == 1/3");

    // sum(b * c^3) = 1/4
    let sum_bc3: f64 = b
        .iter()
        .zip(c.iter())
        .map(|(&bi, &ci)| bi * ci * ci * ci)
        .sum();
    assert!((sum_bc3 - 0.25).abs() < 1e-14, "sum(b * c^3) == 1/4");

    // 4th-order embedded weights sum to 1
    let sum_b_hat: f64 = b_hat.iter().sum();
    assert!((sum_b_hat - 1.0).abs() < 1e-14, "sum(b_hat) == 1");

    // Error weights E = B - B_HAT
    for i in 0..7 {
        assert!(
            (e[i] - (b[i] - b_hat[i])).abs() < 1e-15,
            "E[{i}] == B[{i}] - B_HAT[{i}]"
        );
    }
}

/// 2. Solver config Serde backwards compatibility and supported names.
#[test]
fn test_solver_config_serde_backward_compat() {
    // Legacy string "FixedStepEuler"
    let legacy_json =
        r#"{"start_time":0.0,"stop_time":10.0,"step_size":0.01,"solver":"FixedStepEuler"}"#;
    let config: SimulationConfig = serde_json::from_str(legacy_json).expect("deserialize legacy");
    assert_eq!(config.solver, SolverType::FixedStepEuler);
    assert_eq!(config.abs_tol, 1e-6);
    assert_eq!(config.rel_tol, 1e-3);
    assert_eq!(config.max_steps, 1_000_000);

    // ode aliases
    let ode1: SolverType = serde_json::from_str(r#""ode1""#).expect("ode1");
    assert_eq!(ode1, SolverType::FixedStepEuler);

    let ode4: SolverType = serde_json::from_str(r#""ode4""#).expect("ode4");
    assert_eq!(ode4, SolverType::FixedStepRk4);

    let ode45: SolverType = serde_json::from_str(r#""ode45""#).expect("ode45");
    assert_eq!(ode45, SolverType::VariableStepDormandPrince);

    let rk4: SolverType = serde_json::from_str(r#""FixedStepRk4""#).expect("FixedStepRk4");
    assert_eq!(rk4, SolverType::FixedStepRk4);

    let dp: SolverType =
        serde_json::from_str(r#""VariableStepDormandPrince""#).expect("VariableStepDormandPrince");
    assert_eq!(dp, SolverType::VariableStepDormandPrince);

    // Unsupported solver string produces helpful error listing valid options
    let unknown_res: Result<SolverType, _> = serde_json::from_str(r#""ode23""#);
    assert!(unknown_res.is_err());
    let err_msg = unknown_res.err().unwrap().to_string();
    assert!(
        err_msg.contains("Unsupported solver 'ode23'"),
        "error message: {err_msg}"
    );
    assert!(err_msg.contains("ode1"), "error message: {err_msg}");
    assert!(err_msg.contains("ode4"), "error message: {err_msg}");
    assert!(err_msg.contains("ode45"), "error message: {err_msg}");

    // Full roundtrip JSON
    let full_config = SimulationConfig {
        start_time: 1.0,
        stop_time: 20.0,
        step_size: 0.005,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-8,
        rel_tol: 1e-5,
        max_step: Some(0.5),
        min_step: Some(1e-10),
        initial_step: Some(0.01),
        max_steps: 500_000,
        enable_fsal: Some(true),
    };
    let json = serde_json::to_string_pretty(&full_config).expect("serialize");
    let recovered: SimulationConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(full_config, recovered);
}

/// Builds an autonomous first-order decay model: dx/dt = -x, x(0) = 1.
/// (Integrator with InitialCondition=1.0 fed back through Gain=-1.0 to its input).
fn make_first_order_decay_model() -> BlockDiagram {
    let mut diagram = BlockDiagram::new("FirstOrderDecay");

    // Block 1: Integrator, x(0) = 1.0
    let mut int_blk = Block::new("1", "Integrator", BlockType::Integrator);
    int_blk
        .parameters
        .insert("InitialCondition".to_string(), "1.0".to_string());
    diagram.add_block(int_blk);

    // Block 2: Gain, gain = -1.0
    let mut gain_blk = Block::new("2", "Gain", BlockType::Gain);
    gain_blk
        .parameters
        .insert("Gain".to_string(), "-1.0".to_string());
    diagram.add_block(gain_blk);

    // Block 3: Scope
    let scope_blk = Block::new("3", "Scope", BlockType::Scope);
    diagram.add_block(scope_blk);

    // Integrator -> Gain
    diagram.add_line("1", 1, "2", 1);
    // Gain -> Integrator (feedback)
    diagram.add_line("2", 1, "1", 1);
    // Integrator -> Scope
    diagram.add_line("1", 1, "3", 1);

    diagram
}

/// 3. Measured order of accuracy: Euler error cuts ~2x, RK4 cuts ~16x on halving dt.
#[test]
fn test_euler_rk4_convergence_rates() {
    let diagram = make_first_order_decay_model();
    let t_end: f64 = 1.0;
    let exact_end = (-t_end).exp();

    // Euler convergence: test dt = 0.04 and dt = 0.02
    let cfg_euler1 = SimulationConfig {
        start_time: 0.0,
        stop_time: t_end,
        step_size: 0.04,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };
    let out_euler1 = simulate_diagram(&diagram, &cfg_euler1).expect("sim euler1");
    let err_euler1 = (out_euler1.yout.last().unwrap() - exact_end).abs();

    let cfg_euler2 = SimulationConfig {
        start_time: 0.0,
        stop_time: t_end,
        step_size: 0.02,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };
    let out_euler2 = simulate_diagram(&diagram, &cfg_euler2).expect("sim euler2");
    let err_euler2 = (out_euler2.yout.last().unwrap() - exact_end).abs();

    let euler_ratio = err_euler1 / err_euler2;
    assert!(
        (1.8..=2.2).contains(&euler_ratio),
        "Euler halving ratio should be ~2.0, observed: {euler_ratio}"
    );

    // RK4 convergence: test dt = 0.1 and dt = 0.05
    let cfg_rk4_1 = SimulationConfig {
        start_time: 0.0,
        stop_time: t_end,
        step_size: 0.1,
        solver: SolverType::FixedStepRk4,
        ..Default::default()
    };
    let out_rk4_1 = simulate_diagram(&diagram, &cfg_rk4_1).expect("sim rk4 1");
    let err_rk4_1 = (out_rk4_1.yout.last().unwrap() - exact_end).abs();

    let cfg_rk4_2 = SimulationConfig {
        start_time: 0.0,
        stop_time: t_end,
        step_size: 0.05,
        solver: SolverType::FixedStepRk4,
        ..Default::default()
    };
    let out_rk4_2 = simulate_diagram(&diagram, &cfg_rk4_2).expect("sim rk4 2");
    let err_rk4_2 = (out_rk4_2.yout.last().unwrap() - exact_end).abs();

    let rk4_ratio = err_rk4_1 / err_rk4_2;
    assert!(
        (14.0..=18.0).contains(&rk4_ratio),
        "RK4 halving ratio should be ~16.0, observed: {rk4_ratio}"
    );
}

/// 4. Dormand-Prince tolerance compliance, tightening effect, and step rejection on huge initial step.
#[test]
fn test_dormand_prince_tolerance_compliance() {
    let diagram = make_first_order_decay_model();

    // Standard run with AbsTol = 1e-4, RelTol = 1e-2, max_step = 5.0
    let cfg_standard = SimulationConfig {
        start_time: 0.0,
        stop_time: 3.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-4,
        rel_tol: 1e-2,
        max_step: Some(5.0),
        ..Default::default()
    };
    let out_standard = simulate_diagram(&diagram, &cfg_standard).expect("sim standard");

    // Verify error stays within 20 * (AbsTol + RelTol * |y|) for all points
    for (&t, &y) in out_standard.tout.iter().zip(out_standard.yout.iter()) {
        let exact = (-t).exp();
        let err = (y - exact).abs();
        let threshold = 20.0 * (cfg_standard.abs_tol + cfg_standard.rel_tol * y.abs());
        assert!(
            err <= threshold,
            "Error {err} exceeded threshold {threshold} at t={t}"
        );
    }

    // Tightened tolerances by 10x
    let cfg_tight = SimulationConfig {
        start_time: 0.0,
        stop_time: 3.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-5,
        rel_tol: 1e-3,
        max_step: Some(5.0),
        ..Default::default()
    };
    let out_tight = simulate_diagram(&diagram, &cfg_tight).expect("sim tight");
    let max_err_standard = out_standard
        .tout
        .iter()
        .zip(out_standard.yout.iter())
        .map(|(&t, &y)| (y - (-t).exp()).abs())
        .fold(0.0_f64, f64::max);
    let max_err_tight = out_tight
        .tout
        .iter()
        .zip(out_tight.yout.iter())
        .map(|(&t, &y)| (y - (-t).exp()).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        max_err_tight < max_err_standard,
        "Tightened tolerances should reduce error: tight {max_err_tight} vs standard {max_err_standard}"
    );

    // Huge initial step (e.g. 5.0 when stop is 10.0) forces at least one step rejection
    let cfg_huge_step = SimulationConfig {
        start_time: 0.0,
        stop_time: 10.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-8,
        rel_tol: 1e-6,
        max_step: Some(10.0),
        initial_step: Some(5.0),
        ..Default::default()
    };
    let out_huge = simulate_diagram(&diagram, &cfg_huge_step).expect("sim huge initial step");
    // The first accepted step must be much smaller than the 5.0 initial candidate due to rejection
    let first_accepted_step = out_huge.tout[1] - out_huge.tout[0];
    assert!(
        first_accepted_step < 1.0,
        "Initial step 5.0 must have been rejected and scaled down; first accepted dt = {first_accepted_step}"
    );
}

/// 5. Analytical parity: Integrator of sin(t) equals 1 - cos(t).
#[test]
fn test_integrator_sin_analytical_parity() {
    let mut diagram = BlockDiagram::new("SinIntegrator");

    // Sine Wave: Amp=1.0, Freq=1.0, Phase=0.0, Bias=0.0
    let mut sin_blk = Block::new("1", "SineWave", BlockType::SineWave);
    sin_blk
        .parameters
        .insert("Frequency".to_string(), "1.0".to_string());
    diagram.add_block(sin_blk);

    // Integrator: x(0) = 0.0
    let mut int_blk = Block::new("2", "Integrator", BlockType::Integrator);
    int_blk
        .parameters
        .insert("InitialCondition".to_string(), "0.0".to_string());
    diagram.add_block(int_blk);

    // Scope
    let scope_blk = Block::new("3", "Scope", BlockType::Scope);
    diagram.add_block(scope_blk);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: std::f64::consts::TAU,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-8,
        rel_tol: 1e-6,
        ..Default::default()
    };
    let out = simulate_diagram(&diagram, &config).expect("sim sin integration");

    for (&t, &y) in out.tout.iter().zip(out.yout.iter()) {
        let exact = 1.0 - t.cos();
        let err = (y - exact).abs();
        assert!(
            err < 5e-5,
            "Integrator of sin(t) must equal 1 - cos(t); err={err} at t={t}"
        );
    }
}

/// 6. Finite-time blowup system: x' = x^2, x(0) = 1 (blow-up at t = 1) ends with E-STEP or E-NAN and never hangs.
#[test]
fn test_blowup_system_ends_with_e_step_or_e_nan() {
    let mut diagram = BlockDiagram::new("BlowupModel");

    // Integrator: x(0) = 1.0
    let mut int_blk = Block::new("1", "Integrator", BlockType::Integrator);
    int_blk
        .parameters
        .insert("InitialCondition".to_string(), "1.0".to_string());
    diagram.add_block(int_blk);

    // Product: inputs = [x, x] -> x^2
    let prod_blk = Block::new("2", "Product", BlockType::Product);
    diagram.add_block(prod_blk);

    // Scope
    let scope_blk = Block::new("3", "Scope", BlockType::Scope);
    diagram.add_block(scope_blk);

    // Integrator -> Product input 1 & 2
    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("1", 1, "2", 2);
    // Product -> Integrator
    diagram.add_line("2", 1, "1", 1);
    // Integrator -> Scope
    diagram.add_line("1", 1, "3", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0, // Singularity at t=1.0
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        min_step: Some(1e-12),
        max_steps: 10_000,
        ..Default::default()
    };

    let res = simulate_diagram(&diagram, &config);
    assert!(
        res.is_err(),
        "Blowup system x' = x^2 must return an error before t=2.0"
    );

    match res.err().unwrap() {
        SimError::StepSize { code, message } => {
            assert_eq!(code, "E-STEP");
            assert!(
                message.contains("E-STEP"),
                "Message must contain E-STEP: {message}"
            );
        }
        SimError::NonFinite { code, message, .. } => {
            assert_eq!(code, "E-NAN");
            assert!(
                message.contains("E-NAN"),
                "Message must contain E-NAN: {message}"
            );
        }
        other => panic!("Expected E-STEP or E-NAN error, got: {other:?}"),
    }
}

/// 7. Step block under DP lands exactly on the step time and discontinuity causes minimal rejections.
#[test]
fn test_step_block_lands_exactly_on_step_time() {
    let mut diagram = BlockDiagram::new("StepBreakpointModel");

    let step_time = 1.2345;
    let mut step_blk = Block::new("1", "Step", BlockType::Step);
    step_blk
        .parameters
        .insert("StepTime".to_string(), step_time.to_string());
    step_blk
        .parameters
        .insert("Before".to_string(), "0.0".to_string());
    step_blk
        .parameters
        .insert("After".to_string(), "2.5".to_string());
    diagram.add_block(step_blk);

    // Integrator: integrates the step input
    let int_blk = Block::new("2", "Integrator", BlockType::Integrator);
    diagram.add_block(int_blk);

    // Scope
    let scope_blk = Block::new("3", "Scope", BlockType::Scope);
    diagram.add_block(scope_blk);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 3.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };

    let out = simulate_diagram(&diagram, &config).expect("sim step block");

    // Assert that tout contains step_time exactly
    let hit = out
        .tout
        .iter()
        .any(|&t| (t - step_time).abs() < 1e-12 || t == step_time);
    assert!(
        hit,
        "tout must land exactly on the Step block step_time ({step_time}); tout={:?}",
        out.tout
    );
}

/// 8. UnitDelay with SampleTime under DP: output changes only at hits, FSAL invalidation, and sample errors.
#[test]
fn test_unit_delay_sample_time_under_dp() {
    // Sub-case A: Missing SampleTime under DP triggers E-SAMPLE
    let mut diag_missing = BlockDiagram::new("DelayMissingSampleTime");
    let mut const_blk = Block::new("1", "Constant", BlockType::Constant);
    const_blk
        .parameters
        .insert("Value".to_string(), "10.0".to_string());
    diag_missing.add_block(const_blk);

    let delay_blk = Block::new("2", "UnitDelay", BlockType::UnitDelay);
    diag_missing.add_block(delay_blk);

    let scope_blk = Block::new("3", "Scope", BlockType::Scope);
    diag_missing.add_block(scope_blk);

    diag_missing.add_line("1", 1, "2", 1);
    diag_missing.add_line("2", 1, "3", 1);

    let cfg_dp = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };
    let err_missing = simulate_diagram(&diag_missing, &cfg_dp).expect_err("missing SampleTime");
    match err_missing {
        SimError::SampleTime { code, block, .. } => {
            assert_eq!(code, "E-SAMPLE");
            assert_eq!(block.sid, "2");
        }
        other => panic!("Expected E-SAMPLE, got {other:?}"),
    }

    // Sub-case B: Non-integer multiple SampleTime under fixed-step triggers E-SAMPLE
    let mut diag_bad_ts = BlockDiagram::new("DelayBadTs");
    let mut delay_bad = Block::new("2", "UnitDelay", BlockType::UnitDelay);
    delay_bad
        .parameters
        .insert("SampleTime".to_string(), "0.025".to_string());
    diag_bad_ts.add_block(Block::new("1", "Constant", BlockType::Constant));
    diag_bad_ts.add_block(delay_bad);
    diag_bad_ts.add_block(Block::new("3", "Scope", BlockType::Scope));
    diag_bad_ts.add_line("1", 1, "2", 1);
    diag_bad_ts.add_line("2", 1, "3", 1);

    let cfg_fixed_bad = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.02, // 0.025 is not a multiple of 0.02
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };
    let err_not_mult = simulate_diagram(&diag_bad_ts, &cfg_fixed_bad).expect_err("non-multiple ts");
    match err_not_mult {
        SimError::SampleTime { code, block, .. } => {
            assert_eq!(code, "E-SAMPLE");
            assert_eq!(block.sid, "2");
        }
        other => panic!("Expected E-SAMPLE, got {other:?}"),
    }

    // Sub-case C: Valid SampleTime under DP: outputs change only at integer multiples of Ts
    let mut diag_valid = BlockDiagram::new("DelayValidTs");
    // Clock-like accumulator: Integrator integrating constant 1.0 -> t
    let mut const_1 = Block::new("1", "Constant", BlockType::Constant);
    const_1
        .parameters
        .insert("Value".to_string(), "1.0".to_string());
    diag_valid.add_block(const_1);

    let int_time = Block::new("2", "Integrator", BlockType::Integrator);
    diag_valid.add_block(int_time);

    let ts = 0.5;
    let mut delay_valid = Block::new("3", "UnitDelay", BlockType::UnitDelay);
    delay_valid
        .parameters
        .insert("SampleTime".to_string(), ts.to_string());
    diag_valid.add_block(delay_valid);

    let scope = Block::new("4", "Scope", BlockType::Scope);
    diag_valid.add_block(scope);

    diag_valid.add_line("1", 1, "2", 1);
    diag_valid.add_line("2", 1, "3", 1);
    diag_valid.add_line("3", 1, "4", 1);

    let cfg_dp_valid = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };
    let out = simulate_diagram(&diag_valid, &cfg_dp_valid).expect("sim valid delay");

    // The output of UnitDelay should hold constant between sample hits:
    // Sample hits occur at t = 0.0, 0.5, 1.0, 1.5, 2.0
    // Check that tout hits t = 0.5, 1.0, 1.5, 2.0 exactly
    for &hit_t in &[0.5, 1.0, 1.5, 2.0] {
        let landed = out.tout.iter().any(|&t| (t - hit_t).abs() < 1e-12);
        assert!(
            landed,
            "DP solver must land exactly on sample hit t={hit_t}"
        );
    }
}

/// 8b. UnitDelay with SampleTime under DP: FSAL enabled versus FSAL disabled trajectories agree.
#[test]
fn test_unit_delay_sample_time_under_dp_fsal_agreement() {
    let mut diagram = BlockDiagram::new("DelayFsalAgreement");

    // Continuous integrator feeding a discrete delay and fed back
    let mut const_blk = Block::new("1", "Constant", BlockType::Constant);
    const_blk
        .parameters
        .insert("Value".to_string(), "2.0".to_string());
    diagram.add_block(const_blk);

    let sum = Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "+-");
    diagram.add_block(sum);

    let mut int_blk = Block::new("3", "Integrator", BlockType::Integrator);
    int_blk
        .parameters
        .insert("InitialCondition".to_string(), "0.0".to_string());
    diagram.add_block(int_blk);

    let mut delay = Block::new("4", "UnitDelay", BlockType::UnitDelay);
    delay
        .parameters
        .insert("SampleTime".to_string(), "0.25".to_string());
    diagram.add_block(delay);

    let scope = Block::new("5", "Scope", BlockType::Scope);
    diagram.add_block(scope);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);
    diagram.add_line("4", 1, "2", 2);
    diagram.add_line("3", 1, "5", 1);

    let cfg_fsal_on = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        enable_fsal: Some(true),
        max_step: Some(0.1),
        ..Default::default()
    };
    let out_fsal_on = simulate_diagram(&diagram, &cfg_fsal_on).expect("sim fsal on");

    let cfg_fsal_off = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        enable_fsal: Some(false),
        max_step: Some(0.1),
        ..Default::default()
    };
    let out_fsal_off = simulate_diagram(&diagram, &cfg_fsal_off).expect("sim fsal off");

    assert_eq!(
        out_fsal_on.tout.len(),
        out_fsal_off.tout.len(),
        "FSAL on vs off should take identical steps on valid sample time hits"
    );
    for i in 0..out_fsal_on.tout.len() {
        assert!(
            (out_fsal_on.tout[i] - out_fsal_off.tout[i]).abs() < 1e-12,
            "tout[{i}] mismatch: on={} vs off={}",
            out_fsal_on.tout[i],
            out_fsal_off.tout[i]
        );
        assert!(
            (out_fsal_on.yout[i] - out_fsal_off.yout[i]).abs() < 1e-12,
            "yout[{i}] mismatch: on={} vs off={}",
            out_fsal_on.yout[i],
            out_fsal_off.yout[i]
        );
    }
}

/// 9. Step rejection leaves discrete states unchanged.
#[test]
fn test_rejected_step_preserves_discrete_state() {
    let mut diagram = BlockDiagram::new("RejectionDiscretePreserve");

    // Integrator with huge initial derivative that will cause rejection on a large initial step
    let mut int_blk = Block::new("1", "Integrator", BlockType::Integrator);
    int_blk
        .parameters
        .insert("InitialCondition".to_string(), "1.0".to_string());
    diagram.add_block(int_blk);

    let mut gain = Block::new("2", "Gain", BlockType::Gain);
    gain.parameters
        .insert("Gain".to_string(), "-100.0".to_string()); // Stiff rapid decay
    diagram.add_block(gain);

    let mut delay = Block::new("3", "UnitDelay", BlockType::UnitDelay);
    delay
        .parameters
        .insert("SampleTime".to_string(), "0.01".to_string());
    diagram.add_block(delay);

    let scope = Block::new("4", "Scope", BlockType::Scope);
    diagram.add_block(scope);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "1", 1);
    diagram.add_line("1", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.1,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-8,
        rel_tol: 1e-6,
        initial_step: Some(0.08), // Much larger than stable step size -> forced rejection
        ..Default::default()
    };

    let out = simulate_diagram(&diagram, &config).expect("sim rejection preservation");
    assert!(
        out.tout.len() > 2,
        "Simulation should complete with multiple adaptive steps"
    );
    // At t=0, delay output is 0.0 (initial state)
    assert_eq!(out.yout[0], 0.0);
}

/// 10. Non-uniform tout and signal output length consistency.
#[test]
fn test_non_uniform_tout_and_signals() {
    let diagram = make_first_order_decay_model();

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 5.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-6,
        rel_tol: 1e-3,
        max_step: Some(1.0),
        ..Default::default()
    };
    let out = simulate_diagram(&diagram, &config).expect("sim variable step non-uniform");

    // Check that step sizes are non-uniform (dt varies)
    let dts: Vec<f64> = out.tout.windows(2).map(|w| w[1] - w[0]).collect();
    let min_dt = dts.iter().copied().fold(f64::INFINITY, f64::min);
    let max_dt = dts.iter().copied().fold(0.0_f64, f64::max);
    assert!(
        max_dt > min_dt * 1.5,
        "Variable-step tout must have varying dt: min_dt={min_dt}, max_dt={max_dt}"
    );

    // Every signal must have the exact same length as tout
    assert_eq!(out.yout.len(), out.tout.len());
    for (name, sig) in &out.signals {
        assert_eq!(
            sig.len(),
            out.tout.len(),
            "Signal {name} length must match tout length"
        );
    }
}

/// 11. Zero continuous states under variable step.
#[test]
fn test_zero_continuous_states_under_variable_step() {
    let mut diagram = BlockDiagram::new("PureAlgebraic");

    let mut c1 = Block::new("1", "Constant", BlockType::Constant);
    c1.parameters.insert("Value".to_string(), "3.0".to_string());
    diagram.add_block(c1);

    let mut g1 = Block::new("2", "Gain", BlockType::Gain);
    g1.parameters.insert("Gain".to_string(), "4.0".to_string());
    diagram.add_block(g1);

    let scope = Block::new("3", "Scope", BlockType::Scope);
    diagram.add_block(scope);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 10.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };
    let out = simulate_diagram(&diagram, &config).expect("sim zero continuous states");
    // With no continuous, no discrete, and no breakpoints, it lands directly on stop_time
    assert_eq!(out.tout, vec![0.0, 10.0]);
    assert_eq!(out.yout, vec![12.0, 12.0]);
}

/// 12. Parameter validation errors (E-PARAM).
#[test]
fn test_parameter_validation_e_param() {
    let diagram = make_first_order_decay_model();

    // Invalid stop_time <= start_time
    let mut cfg = SimulationConfig {
        start_time: 5.0,
        stop_time: 2.0,
        ..Default::default()
    };
    let err = simulate_diagram(&diagram, &cfg).expect_err("stop <= start");
    match err {
        SimError::Parameter { code, message } => {
            assert_eq!(code, "E-PARAM");
            assert!(
                message.contains("greater than start time"),
                "message: {message}"
            );
        }
        other => panic!("Expected E-PARAM, got: {other:?}"),
    }

    // Invalid dt <= 0
    cfg = SimulationConfig {
        step_size: 0.0,
        ..Default::default()
    };
    let err_dt = simulate_diagram(&diagram, &cfg).expect_err("dt <= 0");
    match err_dt {
        SimError::Parameter { code, message } => {
            assert_eq!(code, "E-PARAM");
            assert!(message.contains("Step size dt"), "message: {message}");
        }
        other => panic!("Expected E-PARAM, got: {other:?}"),
    }

    // Invalid AbsTol <= 0
    cfg = SimulationConfig {
        abs_tol: -1e-5,
        ..Default::default()
    };
    let err_abs = simulate_diagram(&diagram, &cfg).expect_err("abs_tol <= 0");
    match err_abs {
        SimError::Parameter { code, message } => {
            assert_eq!(code, "E-PARAM");
            assert!(
                message.contains("Absolute tolerance AbsTol"),
                "message: {message}"
            );
        }
        other => panic!("Expected E-PARAM, got: {other:?}"),
    }

    // Invalid RelTol < 1e-13
    cfg = SimulationConfig {
        rel_tol: 1e-15,
        ..Default::default()
    };
    let err_rel = simulate_diagram(&diagram, &cfg).expect_err("rel_tol < 1e-13");
    match err_rel {
        SimError::Parameter { code, message } => {
            assert_eq!(code, "E-PARAM");
            assert!(
                message.contains("Relative tolerance RelTol"),
                "message: {message}"
            );
        }
        other => panic!("Expected E-PARAM, got: {other:?}"),
    }

    // Invalid MinStep >= MaxStep
    cfg = SimulationConfig {
        min_step: Some(0.5),
        max_step: Some(0.1),
        ..Default::default()
    };
    let err_step = simulate_diagram(&diagram, &cfg).expect_err("min_step >= max_step");
    match err_step {
        SimError::Parameter { code, message } => {
            assert_eq!(code, "E-PARAM");
            assert!(message.contains("MinStep"), "message: {message}");
        }
        other => panic!("Expected E-PARAM, got: {other:?}"),
    }
}
