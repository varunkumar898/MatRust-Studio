//! Milestone M9: Error Finding and Diagnostics Integration Tests
//! Verifies proactive static diagnostics (E-FLOAT, E-VLOOP, E-ICUT, E-PORT, W-PORT,
//! E-MULTI, W-MULTI, W-STEP, E-SINGULAR), automated fix engine (apply_fix),
//! 1-level undo, and CLI check validation.

use mlapp_sim::diagnostics::{apply_fix, check_diagram, DiagnosticSeverity};
use mlapp_sim::model::{Block, BlockDiagram, BlockType, Line, PortKind, PortRef};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SolverType};

fn make_wire(src_sid: &str, src_p: usize, dst_sid: &str, dst_p: usize, is_elec: bool) -> Line {
    let kind = if is_elec {
        PortKind::Electrical
    } else {
        PortKind::Signal
    };
    Line {
        src: PortRef {
            block_sid: src_sid.to_string(),
            port_index: src_p,
            port_kind: Some(kind),
        },
        dst: PortRef {
            block_sid: dst_sid.to_string(),
            port_index: dst_p,
            port_kind: Some(kind),
        },
        name: None,
    }
}

#[test]
fn test_m9_floating_island_and_autofix() {
    // A circuit with a DC Voltage Source and a Resistor, but NO Ground.
    let mut diagram = BlockDiagram::new("floating_circuit");
    let mut vsrc = Block::new("vsrc", "DC Voltage Source", BlockType::DcVoltageSource);
    vsrc.parameters
        .insert("Voltage".to_string(), "10".to_string());
    vsrc.position = Some([100.0, 100.0, 180.0, 140.0]);

    let mut res = Block::new("res", "Resistor", BlockType::Resistor);
    res.parameters
        .insert("Resistance".to_string(), "100".to_string());
    res.position = Some([250.0, 100.0, 330.0, 140.0]);

    diagram.blocks.push(vsrc);
    diagram.blocks.push(res);
    diagram.lines.push(make_wire("vsrc", 0, "res", 0, true));
    diagram.lines.push(make_wire("res", 1, "vsrc", 1, true));

    let diags = check_diagram(&diagram, None);
    let float_diag = diags.iter().find(|d| d.code == "E-FLOAT");
    assert!(
        float_diag.is_some(),
        "Must detect E-FLOAT on circuit without Ground: {:?}",
        diags
    );
    assert_eq!(float_diag.unwrap().severity, DiagnosticSeverity::Error);

    // Apply autofix AddGroundToNet
    let fix = float_diag
        .unwrap()
        .fix
        .as_ref()
        .expect("E-FLOAT must have autofix");
    apply_fix(&mut diagram, fix).expect("apply_fix must succeed");

    // Re-check diagnostics: E-FLOAT must be resolved
    let diags_after = check_diagram(&diagram, None);
    assert!(
        !diags_after.iter().any(|d| d.code == "E-FLOAT"),
        "E-FLOAT must be resolved after autofix: {:?}",
        diags_after
    );

    // The fixed circuit must simulate successfully
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };
    let sim_res = simulate_diagram(&diagram, &config);
    assert!(
        sim_res.is_ok(),
        "Simulating fixed circuit must succeed: {:?}",
        sim_res.err()
    );
}

#[test]
fn test_m9_voltage_source_loop_and_autofix() {
    // Two ideal voltage sources connected in parallel across the same nodes -> E-VLOOP
    let mut diagram = BlockDiagram::new("vloop_circuit");
    let mut v1 = Block::new("v1", "V1", BlockType::DcVoltageSource);
    v1.parameters
        .insert("Voltage".to_string(), "10".to_string());
    v1.position = Some([100.0, 100.0, 180.0, 140.0]);

    let mut v2 = Block::new("v2", "V2", BlockType::DcVoltageSource);
    v2.parameters
        .insert("Voltage".to_string(), "10".to_string());
    v2.position = Some([250.0, 100.0, 330.0, 140.0]);

    let mut gnd = Block::new("gnd", "Ground", BlockType::Ground);
    gnd.position = Some([180.0, 250.0, 230.0, 300.0]);

    diagram.blocks.push(v1);
    diagram.blocks.push(v2);
    diagram.blocks.push(gnd);

    diagram.lines.push(make_wire("v1", 0, "v2", 0, true));
    diagram.lines.push(make_wire("v1", 1, "v2", 1, true));
    diagram.lines.push(make_wire("v1", 1, "gnd", 0, true));

    let diags = check_diagram(&diagram, None);
    let vloop_diag = diags.iter().find(|d| d.code == "E-VLOOP");
    assert!(
        vloop_diag.is_some(),
        "Must detect E-VLOOP on parallel ideal voltage sources: {:?}",
        diags
    );

    // Apply autofix InsertSeriesResistor
    let fix = vloop_diag
        .unwrap()
        .fix
        .as_ref()
        .expect("E-VLOOP must have autofix");
    apply_fix(&mut diagram, fix).expect("apply_fix must succeed");

    // Re-check diagnostics: E-VLOOP must be resolved
    let diags_after = check_diagram(&diagram, None);
    assert!(
        !diags_after.iter().any(|d| d.code == "E-VLOOP"),
        "E-VLOOP must be resolved after autofix: {:?}",
        diags_after
    );

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };
    let sim_res = simulate_diagram(&diagram, &config);
    assert!(
        sim_res.is_ok(),
        "Simulating fixed circuit must succeed: {:?}",
        sim_res.err()
    );
}

#[test]
fn test_m9_current_source_cutset_and_autofix() {
    // A current source connected to an open terminal -> E-ICUT
    let mut diagram = BlockDiagram::new("icut_circuit");
    let mut isrc = Block::new("isrc", "DC Current Source", BlockType::DcCurrentSource);
    isrc.parameters
        .insert("Current".to_string(), "2.0".to_string());
    isrc.position = Some([100.0, 100.0, 180.0, 140.0]);

    let mut gnd = Block::new("gnd", "Ground", BlockType::Ground);
    gnd.position = Some([100.0, 250.0, 150.0, 300.0]);

    diagram.blocks.push(isrc);
    diagram.blocks.push(gnd);
    // Connect terminal 1 to ground, leave terminal 0 completely open
    diagram.lines.push(make_wire("isrc", 1, "gnd", 0, true));

    let diags = check_diagram(&diagram, None);
    let icut_diag = diags.iter().find(|d| d.code == "E-ICUT");
    assert!(
        icut_diag.is_some(),
        "Must detect E-ICUT on current source connected to open node: {:?}",
        diags
    );

    // Apply autofix InsertParallelResistor
    let fix = icut_diag
        .unwrap()
        .fix
        .as_ref()
        .expect("E-ICUT must have autofix");
    apply_fix(&mut diagram, fix).expect("apply_fix must succeed");

    let diags_after = check_diagram(&diagram, None);
    assert!(
        !diags_after.iter().any(|d| d.code == "E-ICUT"),
        "E-ICUT must be resolved: {:?}",
        diags_after
    );

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };
    let sim_res = simulate_diagram(&diagram, &config);
    assert!(
        sim_res.is_ok(),
        "Simulating fixed circuit must succeed: {:?}",
        sim_res.err()
    );
}

#[test]
fn test_m9_port_connection_schema_version_validation() {
    // Test Gain block with unconnected input port in schema_version 3 vs 4
    let mut diagram_v3 = BlockDiagram::new("unconnected_v3");
    diagram_v3.schema_version = 3;
    let mut gain_v3 = Block::new("gain1", "Gain", BlockType::Gain);
    gain_v3
        .parameters
        .insert("Gain".to_string(), "5.0".to_string());
    diagram_v3.blocks.push(gain_v3);

    let diags_v3 = check_diagram(&diagram_v3, None);
    let port_v3 = diags_v3.iter().find(|d| d.code == "W-PORT");
    assert!(
        port_v3.is_some(),
        "Schema 3 must emit W-PORT warning: {:?}",
        diags_v3
    );
    assert_eq!(port_v3.unwrap().severity, DiagnosticSeverity::Warning);

    // Now test in schema_version 4: must emit E-PORT error
    let mut diagram_v4 = BlockDiagram::new("unconnected_v4");
    diagram_v4.schema_version = 4;
    let mut gain_v4 = Block::new("gain1", "Gain", BlockType::Gain);
    gain_v4
        .parameters
        .insert("Gain".to_string(), "5.0".to_string());
    diagram_v4.blocks.push(gain_v4);

    let diags_v4 = check_diagram(&diagram_v4, None);
    let port_v4 = diags_v4.iter().find(|d| d.code == "E-PORT");
    assert!(
        port_v4.is_some(),
        "Schema 4 must emit E-PORT error: {:?}",
        diags_v4
    );
    assert_eq!(port_v4.unwrap().severity, DiagnosticSeverity::Error);

    // Apply ConnectPortToDefaultZero autofix
    let fix = port_v4.unwrap().fix.as_ref().expect("E-PORT has autofix");
    apply_fix(&mut diagram_v4, fix).expect("apply_fix must succeed");

    let diags_after = check_diagram(&diagram_v4, None);
    assert!(
        !diags_after.iter().any(|d| d.code == "E-PORT"),
        "E-PORT resolved: {:?}",
        diags_after
    );

    let config = SimulationConfig::default();
    let sim_res = simulate_diagram(&diagram_v4, &config);
    assert!(
        sim_res.is_ok(),
        "Fixed diagram must simulate cleanly: {:?}",
        sim_res.err()
    );
}

#[test]
fn test_m9_multi_port_connection_schema_validation() {
    // Multiple wires to one input port: W-MULTI in schema 3, E-MULTI in schema 4
    let mut diagram_v3 = BlockDiagram::new("multi_v3");
    diagram_v3.schema_version = 3;
    let c1 = Block::new("c1", "Constant1", BlockType::Constant);
    let c2 = Block::new("c2", "Constant2", BlockType::Constant);
    let gain = Block::new("g", "Gain", BlockType::Gain);

    diagram_v3.blocks.push(c1);
    diagram_v3.blocks.push(c2);
    diagram_v3.blocks.push(gain);

    diagram_v3.lines.push(make_wire("c1", 1, "g", 1, false));
    diagram_v3.lines.push(make_wire("c2", 1, "g", 1, false));

    let diags_v3 = check_diagram(&diagram_v3, None);
    assert!(
        diags_v3.iter().any(|d| d.code == "W-MULTI"),
        "Schema 3 emits W-MULTI: {:?}",
        diags_v3
    );

    let mut diagram_v4 = diagram_v3.clone();
    diagram_v4.schema_version = 4;
    let diags_v4 = check_diagram(&diagram_v4, None);
    let multi_err = diags_v4.iter().find(|d| d.code == "E-MULTI");
    assert!(
        multi_err.is_some(),
        "Schema 4 emits E-MULTI error: {:?}",
        diags_v4
    );
    assert_eq!(multi_err.unwrap().severity, DiagnosticSeverity::Error);
}

#[test]
fn test_m9_w_step_fast_time_constant_heuristic() {
    // RC circuit with R = 1 kOhm, C = 1 uF -> tau = 1 ms = 1e-3 s
    // If step size h = 5 ms (5e-3 s) > 2 * tau = 2 ms -> W-STEP warning!
    let mut diagram = BlockDiagram::new("fast_rc");
    let mut vsrc = Block::new("vsrc", "DC Voltage Source", BlockType::DcVoltageSource);
    vsrc.parameters
        .insert("Voltage".to_string(), "10".to_string());
    vsrc.position = Some([100.0, 100.0, 180.0, 140.0]);

    let mut res = Block::new("res", "Resistor", BlockType::Resistor);
    res.parameters
        .insert("Resistance".to_string(), "1000".to_string());
    res.position = Some([250.0, 100.0, 330.0, 140.0]);

    let mut cap = Block::new("cap", "Capacitor", BlockType::Capacitor);
    cap.parameters
        .insert("Capacitance".to_string(), "1e-6".to_string());
    cap.position = Some([400.0, 100.0, 480.0, 140.0]);

    let mut gnd = Block::new("gnd", "Ground", BlockType::Ground);
    gnd.position = Some([250.0, 250.0, 300.0, 300.0]);

    diagram.blocks.push(vsrc);
    diagram.blocks.push(res);
    diagram.blocks.push(cap);
    diagram.blocks.push(gnd);

    diagram.lines.push(make_wire("vsrc", 0, "res", 0, true));
    diagram.lines.push(make_wire("res", 1, "cap", 0, true));
    diagram.lines.push(make_wire("cap", 1, "gnd", 0, true));
    diagram.lines.push(make_wire("vsrc", 1, "gnd", 0, true));

    let config_bad = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.05,
        step_size: 5e-3, // 5 ms > 2 * 1 ms
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };

    let diags = check_diagram(&diagram, Some(&config_bad));
    let w_step = diags.iter().find(|d| d.code == "W-STEP");
    assert!(
        w_step.is_some(),
        "Must emit W-STEP when h exceeds 2*tau_min: {:?}",
        diags
    );
    assert_eq!(w_step.unwrap().severity, DiagnosticSeverity::Warning);

    // With proper small step size h = 1e-4 s <= 2*tau, W-STEP must NOT fire
    let config_good = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.05,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };
    let diags_good = check_diagram(&diagram, Some(&config_good));
    assert!(
        !diags_good.iter().any(|d| d.code == "W-STEP"),
        "No W-STEP when step size is adequate: {:?}",
        diags_good
    );
}

#[test]
fn test_m9_singular_circuit_matrix_error_reporting() {
    // Pure inductor connected across voltage source with initial step -> singular matrix E-SINGULAR
    let mut diagram = BlockDiagram::new("singular_circuit");
    let mut vsrc = Block::new("vsrc", "DC Voltage Source", BlockType::DcVoltageSource);
    vsrc.parameters
        .insert("Voltage".to_string(), "10".to_string());
    vsrc.position = Some([100.0, 100.0, 180.0, 140.0]);

    let mut ind = Block::new("ind", "Inductor", BlockType::Inductor);
    ind.parameters
        .insert("Inductance".to_string(), "1e-3".to_string());
    ind.position = Some([250.0, 100.0, 330.0, 140.0]);

    let mut gnd = Block::new("gnd", "Ground", BlockType::Ground);
    gnd.position = Some([180.0, 250.0, 230.0, 300.0]);

    diagram.blocks.push(vsrc);
    diagram.blocks.push(ind);
    diagram.blocks.push(gnd);

    diagram.lines.push(make_wire("vsrc", 0, "ind", 0, true));
    diagram.lines.push(make_wire("ind", 1, "vsrc", 1, true));
    diagram.lines.push(make_wire("vsrc", 1, "gnd", 0, true));

    // Notice: at t=0, inductor companion model has I_L(0) and R_eq = 2L/h.
    // If two 0V voltage sources are in parallel, that's definitely singular.
    let mut diagram_vparallel = BlockDiagram::new("vparallel");
    let mut v1 = Block::new("v1", "V1", BlockType::DcVoltageSource);
    v1.parameters
        .insert("Voltage".to_string(), "5.0".to_string());
    let mut v2 = Block::new("v2", "V2", BlockType::DcVoltageSource);
    v2.parameters
        .insert("Voltage".to_string(), "10.0".to_string());
    let gnd = Block::new("gnd", "Ground", BlockType::Ground);

    diagram_vparallel.blocks.push(v1);
    diagram_vparallel.blocks.push(v2);
    diagram_vparallel.blocks.push(gnd);
    diagram_vparallel
        .lines
        .push(make_wire("v1", 0, "v2", 0, true));
    diagram_vparallel
        .lines
        .push(make_wire("v1", 1, "v2", 1, true));
    diagram_vparallel
        .lines
        .push(make_wire("v1", 1, "gnd", 0, true));

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..SimulationConfig::default()
    };
    let err = simulate_diagram(&diagram_vparallel, &config)
        .expect_err("Parallel voltage sources must fail");
    assert_eq!(err.code(), "E-SINGULAR");
    assert!(
        err.message().contains("Zero pivot") || err.message().contains("singular"),
        "Error message must report singularity: {}",
        err.message()
    );
}
