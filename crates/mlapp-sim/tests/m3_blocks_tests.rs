//! Tests for Milestone M3: Sources and Control Blocks.

use mlapp_sim::engine::feedthrough;
use mlapp_sim::model::{all_block_specs, block_spec, Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimError, SimulationConfig, SolverType};

#[test]
fn test_m3_spec_table_definitions() {
    let specs = all_block_specs();
    assert!(
        specs.len() >= 25,
        "Spec table should contain all block types"
    );

    // Test Sawtooth spec
    let saw = block_spec(&BlockType::Sawtooth).expect("Sawtooth spec");
    assert_eq!(saw.min_inputs, 0);
    assert_eq!(saw.outputs, 1);
    assert!(!saw.default_feedthrough);
    assert!(saw.params.iter().any(|p| p.key == "Frequency"));

    // Test Pulse Generator spec
    let pulse = block_spec(&BlockType::PulseGenerator).expect("Pulse spec");
    assert_eq!(pulse.min_inputs, 0);
    assert_eq!(pulse.outputs, 1);
    assert!(!pulse.default_feedthrough);

    // Test Transfer Function feedthrough rule:
    // Strictly proper: num=[1], den=[1, 1] -> deg(num) < deg(den) -> feedthrough is false
    let mut tf_proper_params = std::collections::HashMap::new();
    tf_proper_params.insert("Numerator".to_string(), "1".to_string());
    tf_proper_params.insert("Denominator".to_string(), "1, 1".to_string());
    assert!(
        !feedthrough(&BlockType::TransferFunction, &tf_proper_params, 0),
        "Strictly proper transfer function must NOT have direct feedthrough"
    );

    // Proper with deg(num) == deg(den): num=[2, 1], den=[1, 1] -> feedthrough is true
    let mut tf_feedthrough_params = std::collections::HashMap::new();
    tf_feedthrough_params.insert("Numerator".to_string(), "2, 1".to_string());
    tf_feedthrough_params.insert("Denominator".to_string(), "1, 1".to_string());
    assert!(
        feedthrough(&BlockType::TransferFunction, &tf_feedthrough_params, 0),
        "Proper transfer function with deg(num) == deg(den) must have direct feedthrough"
    );

    // Test PID feedthrough: must be true
    let pid_params = std::collections::HashMap::new();
    assert!(
        feedthrough(&BlockType::Pid, &pid_params, 0),
        "PID controller must have direct feedthrough"
    );

    // Test Zero-Order Hold: feedthrough must be false
    let zoh_params = std::collections::HashMap::new();
    assert!(
        !feedthrough(&BlockType::ZeroOrderHold, &zoh_params, 0),
        "ZOH must NOT have direct feedthrough"
    );
}

#[test]
fn test_m3_sources_eval_and_validation() {
    // 1. Validation errors: Frequency <= 0 on Sawtooth, Triangle, SineWave
    let mut bad_saw = BlockDiagram::new("BadSaw");
    bad_saw.add_block(Block::new("1", "Saw", BlockType::Sawtooth).with_param("Frequency", "0.0"));
    bad_saw.add_block(Block::new("2", "Scope", BlockType::Scope));
    bad_saw.add_line("1", 1, "2", 1);

    let cfg = SimulationConfig::default();
    let err_saw = simulate_diagram(&bad_saw, &cfg).expect_err("Freq 0 should fail");
    match err_saw {
        SimError::Parameter { code, .. } => assert_eq!(code, "E-PARAM"),
        other => panic!("Expected E-PARAM, got {other:?}"),
    }

    // 2. Validation error: Empty vectors on Repeating Sequence
    let mut bad_seq = BlockDiagram::new("BadSeq");
    bad_seq.add_block(
        Block::new("1", "Seq", BlockType::RepeatingSequence)
            .with_param("TimeValues", "[]")
            .with_param("OutputValues", "[]"),
    );
    bad_seq.add_block(Block::new("2", "Scope", BlockType::Scope));
    bad_seq.add_line("1", 1, "2", 1);
    let err_seq = simulate_diagram(&bad_seq, &cfg).expect_err("Empty seq should fail");
    match err_seq {
        SimError::Parameter { code, .. } => assert_eq!(code, "E-PARAM"),
        other => panic!("Expected E-PARAM, got {other:?}"),
    }

    // 3. Sine Wave units: rad/s vs Hz
    // SineWave with freq 1 Hz: at t = 0.25 s, sin(2*pi*1*0.25) = sin(pi/2) = 1.0
    let mut diag_hz = BlockDiagram::new("SineHz");
    diag_hz.add_block(
        Block::new("1", "SinHz", BlockType::SineWave)
            .with_param("Frequency", "1.0")
            .with_param("Units", "Hz"),
    );
    diag_hz.add_block(Block::new("2", "Scope", BlockType::Scope));
    diag_hz.add_line("1", 1, "2", 1);

    let cfg_hz = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.25,
        step_size: 0.25,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };
    let out_hz = simulate_diagram(&diag_hz, &cfg_hz).expect("sim sine hz");
    let val_at_025 = out_hz.yout.last().copied().unwrap();
    assert!(
        (val_at_025 - 1.0).abs() < 1e-12,
        "Sine wave at 1 Hz at t=0.25s must be 1.0; got {val_at_025}"
    );

    // 4. Clock block outputs current time t
    let mut diag_clk = BlockDiagram::new("ClockTest");
    diag_clk.add_block(Block::new("1", "Clk", BlockType::Clock));
    diag_clk.add_block(Block::new("2", "Scope", BlockType::Scope));
    diag_clk.add_line("1", 1, "2", 1);
    let cfg_clk = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.1,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };
    let out_clk = simulate_diagram(&diag_clk, &cfg_clk).expect("sim clock");
    for (t, y) in out_clk.tout.iter().zip(&out_clk.yout) {
        assert!(
            (t - y).abs() < 1e-12,
            "Clock output must equal time t; got y={y} vs t={t}"
        );
    }
}

#[test]
fn test_m3_source_breakpoints_hit_under_ode45() {
    // Pulse Generator with Period=2.0, Duty=50% (on=1.0s), Delay=0.5s
    // Discontinuities occur at t = 0.5 (rising edge), 1.5 (falling edge), 2.5 (rising edge)
    let mut diag = BlockDiagram::new("PulseBreakpoints");
    diag.add_block(
        Block::new("1", "Pulse", BlockType::PulseGenerator)
            .with_param("Amplitude", "5.0")
            .with_param("Period", "2.0")
            .with_param("Duty", "50.0")
            .with_param("Delay", "0.5"),
    );
    diag.add_block(Block::new("2", "Scope", BlockType::Scope));
    diag.add_line("1", 1, "2", 1);

    let cfg = SimulationConfig {
        start_time: 0.0,
        stop_time: 3.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };

    let out = simulate_diagram(&diag, &cfg).expect("sim pulse ode45");

    // The variable-step solver must land exactly on the edge times 0.5, 1.5, 2.5
    for &edge_t in &[0.5, 1.5, 2.5] {
        let hit = out.tout.iter().any(|&t| (t - edge_t).abs() < 1e-11);
        assert!(
            hit,
            "Variable-step solver must land exactly on pulse edge {edge_t}; tout={:?}",
            out.tout
        );
    }
}

#[test]
fn test_m3_control_eval_and_feedthrough() {
    let mut diag = BlockDiagram::new("ControlBlocks");

    // Constant -4.0 -> Abs -> 4.0
    diag.add_block(Block::new("1", "ConstNeg", BlockType::Constant).with_param("Value", "-4.0"));
    diag.add_block(Block::new("2", "AbsBlock", BlockType::Abs));
    // MathFunction sqrt(4.0) -> 2.0
    diag.add_block(
        Block::new("3", "SqrtBlock", BlockType::MathFunction).with_param("Function", "sqrt"),
    );
    // Relational Operator: 2.0 > 1.0 -> 1.0
    diag.add_block(Block::new("4", "ConstOne", BlockType::Constant).with_param("Value", "1.0"));
    diag.add_block(
        Block::new("5", "RelOp", BlockType::RelationalOperator).with_param("Operator", ">"),
    );
    // Switch: if In2 (1.0) >= Threshold (0.5), pass In1 (100.0) else In3 (200.0)
    diag.add_block(Block::new("6", "ValA", BlockType::Constant).with_param("Value", "100.0"));
    diag.add_block(Block::new("7", "ValB", BlockType::Constant).with_param("Value", "200.0"));
    diag.add_block(
        Block::new("8", "SwitchBlock", BlockType::Switch)
            .with_param("Criteria", ">=")
            .with_param("Threshold", "0.5"),
    );
    diag.add_block(Block::new("9", "Scope", BlockType::Scope));

    diag.add_line("1", 1, "2", 1);
    diag.add_line("2", 1, "3", 1);
    diag.add_line("3", 1, "5", 1);
    diag.add_line("4", 1, "5", 2);
    diag.add_line("6", 1, "8", 1);
    diag.add_line("5", 1, "8", 2);
    diag.add_line("7", 1, "8", 3);
    diag.add_line("8", 1, "9", 1);

    let cfg = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.1,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&diag, &cfg).expect("sim control blocks");
    let final_val = out.yout.last().copied().unwrap();
    assert_eq!(final_val, 100.0, "Switch should output 100.0");
}

#[test]
fn test_m3_transfer_function_analytical_step() {
    // First-order system: H(s) = 1 / (s + 1)
    // Unit step response y(t) = 1 - e^(-t)
    // At t = 1.0: y(1.0) = 1 - e^(-1) = 0.6321205588285577
    let mut diag = BlockDiagram::new("TransferFunctionStep");
    diag.add_block(
        Block::new("1", "Step", BlockType::Step)
            .with_param("Time", "0.0")
            .with_param("Before", "0.0")
            .with_param("After", "1.0"),
    );
    diag.add_block(
        Block::new("2", "TF", BlockType::TransferFunction)
            .with_param("Numerator", "1")
            .with_param("Denominator", "1, 1"),
    );
    diag.add_block(Block::new("3", "Scope", BlockType::Scope));

    diag.add_line("1", 1, "2", 1);
    diag.add_line("2", 1, "3", 1);

    let cfg = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        abs_tol: 1e-8,
        rel_tol: 1e-6,
        max_step: Some(0.05),
        ..Default::default()
    };

    let out = simulate_diagram(&diag, &cfg).expect("sim TF step");

    for (t, y) in out.tout.iter().zip(&out.yout) {
        let expected = 1.0 - (-t).exp();
        let err = (y - expected).abs();
        assert!(
            err < 1e-4,
            "TF step error at t={t} exceeds tolerance: actual={y}, expected={expected}, err={err}"
        );
    }
}

#[test]
fn test_m3_discrete_blocks_e_sample_rule() {
    // Discrete Transfer Function without SampleTime under DP triggers E-SAMPLE
    let mut diag_missing = BlockDiagram::new("DiscreteTfMissingTs");
    diag_missing
        .add_block(Block::new("1", "Const", BlockType::Constant).with_param("Value", "1.0"));
    diag_missing.add_block(
        Block::new("2", "DTF", BlockType::DiscreteTransferFunction)
            .with_param("Numerator", "1")
            .with_param("Denominator", "1, -0.5"),
    );
    diag_missing.add_block(Block::new("3", "Scope", BlockType::Scope));
    diag_missing.add_line("1", 1, "2", 1);
    diag_missing.add_line("2", 1, "3", 1);

    let cfg_dp = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };
    let err_missing = simulate_diagram(&diag_missing, &cfg_dp).expect_err("missing Ts");
    match err_missing {
        SimError::SampleTime { code, block, .. } => {
            assert_eq!(code, "E-SAMPLE");
            assert_eq!(block.sid, "2");
        }
        other => panic!("Expected E-SAMPLE, got {other:?}"),
    }

    // Zero-Order Hold holds values between sample hits
    let mut diag_zoh = BlockDiagram::new("ZohTest");
    // Ramp via integrator: t
    diag_zoh.add_block(Block::new("1", "ConstOne", BlockType::Constant).with_param("Value", "1.0"));
    diag_zoh.add_block(Block::new("2", "Integ", BlockType::Integrator));
    diag_zoh.add_block(
        Block::new("3", "ZOH", BlockType::ZeroOrderHold).with_param("SampleTime", "0.5"),
    );
    diag_zoh.add_block(Block::new("4", "Scope", BlockType::Scope));

    diag_zoh.add_line("1", 1, "2", 1);
    diag_zoh.add_line("2", 1, "3", 1);
    diag_zoh.add_line("3", 1, "4", 1);

    let cfg_zoh = SimulationConfig {
        start_time: 0.0,
        stop_time: 2.0,
        step_size: 0.01,
        solver: SolverType::VariableStepDormandPrince,
        ..Default::default()
    };
    let out_zoh = simulate_diagram(&diag_zoh, &cfg_zoh).expect("sim zoh");

    // Sample hits at 0.0, 0.5, 1.0, 1.5, 2.0
    // Check that tout hits these exactly
    for &hit_t in &[0.5, 1.0, 1.5, 2.0] {
        let landed = out_zoh.tout.iter().any(|&t| (t - hit_t).abs() < 1e-12);
        assert!(landed, "ZOH solver must land on sample hit {hit_t}");
    }
}

#[test]
fn test_m3_pid_analytical_and_antiwindup() {
    // Closed-loop PI control of plant G(s) = 1 / (s + 1)
    // Controller: C(s) = Kp + Ki/s with Kp = 2.0, Ki = 3.0
    // Closed-loop transfer function:
    // T(s) = C(s)G(s) / (1 + C(s)G(s)) = (2s + 3) / (s^2 + 3s + 3)
    let mut diag = BlockDiagram::new("PidClosedLoop");
    diag.add_block(
        Block::new("1", "Step", BlockType::Step)
            .with_param("Time", "0.0")
            .with_param("Before", "0.0")
            .with_param("After", "1.0"),
    );
    diag.add_block(Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "+-"));
    diag.add_block(
        Block::new("3", "PID", BlockType::Pid)
            .with_param("P", "2.0")
            .with_param("I", "3.0")
            .with_param("D", "0.0")
            .with_param("UpperSaturationLimit", "1.5")
            .with_param("LowerSaturationLimit", "-1.5")
            .with_param("AntiWindupMode", "Clamping"),
    );
    diag.add_block(
        Block::new("4", "Plant", BlockType::TransferFunction)
            .with_param("Numerator", "1")
            .with_param("Denominator", "1, 1"),
    );
    diag.add_block(Block::new("5", "Scope", BlockType::Scope));

    diag.add_line("1", 1, "2", 1);
    diag.add_line("2", 1, "3", 1);
    diag.add_line("3", 1, "4", 1);
    diag.add_line("4", 1, "5", 1);
    diag.add_line("4", 1, "2", 2);

    let cfg = SimulationConfig {
        start_time: 0.0,
        stop_time: 5.0,
        step_size: 0.01,
        solver: SolverType::FixedStepRk4,
        ..Default::default()
    };

    let out = simulate_diagram(&diag, &cfg).expect("sim pid closed loop");
    let final_val = out.yout.last().copied().unwrap();
    // System is stable with integral action -> final steady-state value reaches reference 1.0 within 1%
    assert!(
        (final_val - 1.0).abs() < 0.01,
        "Closed loop PI control should drive error to zero; final value: {final_val}"
    );
}
