//! Integration tests for Milestone M12: Parameter Sweep and Workflow Polish.

use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::SimulationConfig;
use mlapp_sim::sweep::{run_parameter_sweep, ParameterSweepConfig};

#[test]
fn test_parameter_sweep_gain_variation() {
    let mut diagram = BlockDiagram::new("sweep_gain");
    let mut cst = Block::new("1", "Constant", BlockType::Constant);
    cst.parameters
        .insert("Value".to_string(), "2.0".to_string());

    let mut gain = Block::new("2", "Gain", BlockType::Gain);
    gain.parameters
        .insert("Gain".to_string(), "1.0".to_string());

    let scope = Block::new("3", "Scope", BlockType::Scope);

    diagram.blocks = vec![cst, gain, scope];
    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let sweep_cfg = ParameterSweepConfig {
        block_id: "2".to_string(),
        param_name: "Gain".to_string(),
        values: vec![1.0, 2.5, 5.0, 10.0],
        sim_config: SimulationConfig {
            stop_time: 0.1,
            step_size: 0.01,
            ..Default::default()
        },
    };

    let results = run_parameter_sweep(&diagram, &sweep_cfg).expect("Sweep should succeed");
    assert_eq!(results.len(), 4);

    // Expected final outputs: 2.0 * gain_val
    let expected_outs = [2.0, 5.0, 10.0, 20.0];
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r.param_value, sweep_cfg.values[i]);
        let scope_out = r.output.signals.get("Scope").expect("Scope signal exists");
        let last_val = *scope_out.last().expect("Has output values");
        assert!(
            (last_val - expected_outs[i]).abs() < 1e-9,
            "Expected {}, got {}",
            expected_outs[i],
            last_val
        );
    }
}

#[test]
fn test_parameter_sweep_resistor_divider() {
    let mut diagram = BlockDiagram::new("sweep_divider");
    let mut vsrc = Block::new("1", "V1", BlockType::DcVoltageSource);
    vsrc.parameters
        .insert("Voltage".to_string(), "12.0".to_string());

    let mut r1 = Block::new("2", "R1", BlockType::Resistor);
    r1.parameters
        .insert("Resistance".to_string(), "1000.0".to_string());

    let mut r2 = Block::new("3", "R2", BlockType::Resistor);
    r2.parameters
        .insert("Resistance".to_string(), "1000.0".to_string());

    let gnd = Block::new("4", "GND", BlockType::Ground);
    let vsense = Block::new("5", "V_Sense", BlockType::VoltageSensor);

    diagram.blocks = vec![vsrc, r1, r2, gnd, vsense];
    diagram.add_electrical_line("1", 0, "2", 0);
    diagram.add_electrical_line("2", 1, "3", 0);
    diagram.add_electrical_line("2", 1, "5", 0);
    diagram.add_electrical_line("3", 1, "4", 0);
    diagram.add_electrical_line("1", 1, "4", 0);
    diagram.add_electrical_line("5", 1, "4", 0);

    let r2_values = vec![500.0, 1000.0, 2000.0, 3000.0];
    let sweep_cfg = ParameterSweepConfig {
        block_id: "3".to_string(),
        param_name: "Resistance".to_string(),
        values: r2_values.clone(),
        sim_config: SimulationConfig {
            stop_time: 0.01,
            step_size: 0.001,
            ..Default::default()
        },
    };

    let results = run_parameter_sweep(&diagram, &sweep_cfg).expect("Sweep succeeds");
    assert_eq!(results.len(), 4);

    for (i, r) in results.iter().enumerate() {
        let r2_val = r2_values[i];
        let theoretical_v = 12.0 * r2_val / (1000.0 + r2_val);
        let measured_v = *r
            .final_values
            .get("V_Sense")
            .expect("V_Sense output present");
        assert!(
            (measured_v - theoretical_v).abs() < 1e-4,
            "R2={r2_val}: expected {theoretical_v}, got {measured_v}"
        );
    }
}

#[test]
fn test_parameter_sweep_invalid_target_error() {
    let diagram = BlockDiagram::new("empty");
    let sweep_cfg = ParameterSweepConfig {
        block_id: "nonexistent_99".to_string(),
        param_name: "Gain".to_string(),
        values: vec![1.0, 2.0],
        sim_config: SimulationConfig::default(),
    };

    let err =
        run_parameter_sweep(&diagram, &sweep_cfg).expect_err("Must fail for nonexistent block");
    assert!(err
        .iter()
        .any(|d| d.code == "E-PARAM" && d.message.contains("nonexistent_99")));
}
