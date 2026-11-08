//! Milestone M4 integration tests: Electrical data model, terminals, nets via union-find,
//! block rotation, parameter validation, domain separation, and pre-M5 simulation guards.

use mlapp_sim::diagnostics::DiagnosticSeverity;
use mlapp_sim::model::{
    Block, BlockDiagram, BlockRotation, BlockType, Line, PortDesc, PortKind, PortRef, PortRole,
};
use mlapp_sim::netlist::resolve_electrical_nets;
use mlapp_sim::solver::{simulate_diagram, SimulationConfig};
use mlapp_sim::spec::{pin_geometry, validate_block_params};
use std::collections::HashMap;

#[test]
fn test_terminal_count_and_port_kinds_per_block_type() {
    let empty_params = HashMap::new();

    // Electrical components
    let r = Block::new("1", "R1", BlockType::Resistor);
    assert_eq!(r.terminal_count(), 2);
    assert_eq!(r.input_ports(), 0);
    assert_eq!(r.output_ports(), 0);
    let r_ports = r.ports();
    assert_eq!(r_ports.len(), 2);
    assert_eq!(r_ports[0].kind, PortKind::Electrical);
    assert_eq!(r_ports[0].role, PortRole::Terminal);
    assert_eq!(r_ports[0].label, "p");
    assert_eq!(r_ports[1].label, "n");

    let gnd = Block::new("2", "GND", BlockType::Ground);
    assert_eq!(gnd.terminal_count(), 1);
    assert_eq!(gnd.input_ports(), 0);
    assert_eq!(gnd.output_ports(), 0);
    let gnd_ports = gnd.ports();
    assert_eq!(gnd_ports.len(), 1);
    assert_eq!(gnd_ports[0].kind, PortKind::Electrical);
    assert_eq!(gnd_ports[0].label, "gnd");

    let sw = Block::new("3", "SW", BlockType::IdealSwitch);
    assert_eq!(sw.terminal_count(), 2);
    assert_eq!(sw.input_ports(), 1); // 1 signal control input
    assert_eq!(sw.output_ports(), 0);
    let sw_ports = sw.ports();
    assert_eq!(sw_ports.len(), 3);
    assert_eq!(
        sw_ports[0],
        PortDesc {
            kind: PortKind::Signal,
            role: PortRole::Input,
            label: "v".to_string(),
            index: 0,
        }
    );
    assert_eq!(
        sw_ports[1],
        PortDesc {
            kind: PortKind::Electrical,
            role: PortRole::Terminal,
            label: "p".to_string(),
            index: 0,
        }
    );
    assert_eq!(
        sw_ports[2],
        PortDesc {
            kind: PortKind::Electrical,
            role: PortRole::Terminal,
            label: "n".to_string(),
            index: 1,
        }
    );

    // Pure signal block has 0 terminals
    let gain = Block::new("4", "K", BlockType::Gain);
    assert_eq!(gain.terminal_count(), 0);
    assert_eq!(gain.input_ports(), 1);
    assert_eq!(gain.output_ports(), 1);
    let gain_ports = gain.ports();
    assert_eq!(gain_ports.len(), 2);
    assert_eq!(gain_ports[0].kind, PortKind::Signal);
    assert_eq!(gain_ports[1].kind, PortKind::Signal);

    // Other two-terminal blocks
    for bt in [
        BlockType::Capacitor,
        BlockType::Inductor,
        BlockType::DcVoltageSource,
        BlockType::AcVoltageSource,
        BlockType::DcCurrentSource,
        BlockType::Diode,
    ] {
        assert_eq!(bt.terminal_count(), 2);
        assert_eq!(bt.input_ports(&empty_params), 0);
        assert_eq!(bt.output_ports(), 0);
    }
}

#[test]
fn test_pin_geometry_and_rotation_invariance() {
    // Resistor terminal 0 (p) and terminal 1 (n)
    // R0: Left (0.0, 0.5) and Right (1.0, 0.5)
    let p_r0 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        0,
        BlockRotation::R0,
    );
    let n_r0 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        1,
        BlockRotation::R0,
    );
    assert_eq!(p_r0, (0.0, 0.5));
    assert_eq!(n_r0, (1.0, 0.5));

    // R90: Top (0.5, 0.0) and Bottom (0.5, 1.0)
    let p_r90 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        0,
        BlockRotation::R90,
    );
    let n_r90 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        1,
        BlockRotation::R90,
    );
    assert_eq!(p_r90, (0.5, 0.0));
    assert_eq!(n_r90, (0.5, 1.0));

    // R180: Right (1.0, 0.5) and Left (0.0, 0.5)
    let p_r180 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        0,
        BlockRotation::R180,
    );
    let n_r180 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        1,
        BlockRotation::R180,
    );
    assert_eq!(p_r180, (1.0, 0.5));
    assert_eq!(n_r180, (0.0, 0.5));

    // R270: Bottom (0.5, 1.0) and Top (0.5, 0.0)
    let p_r270 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        0,
        BlockRotation::R270,
    );
    let n_r270 = pin_geometry(
        &BlockType::Resistor,
        PortKind::Electrical,
        1,
        BlockRotation::R270,
    );
    assert_eq!(p_r270, (0.5, 1.0));
    assert_eq!(n_r270, (0.5, 0.0));

    // Ground terminal
    assert_eq!(
        pin_geometry(
            &BlockType::Ground,
            PortKind::Electrical,
            0,
            BlockRotation::R0
        ),
        (0.5, 0.0)
    );
    assert_eq!(
        pin_geometry(
            &BlockType::Ground,
            PortKind::Electrical,
            0,
            BlockRotation::R90
        ),
        (1.0, 0.5)
    );
    assert_eq!(
        pin_geometry(
            &BlockType::Ground,
            PortKind::Electrical,
            0,
            BlockRotation::R180
        ),
        (0.5, 1.0)
    );
    assert_eq!(
        pin_geometry(
            &BlockType::Ground,
            PortKind::Electrical,
            0,
            BlockRotation::R270
        ),
        (0.0, 0.5)
    );
}

#[test]
fn test_multi_wire_terminal_and_reversed_line_invariance() {
    let mut d1 = BlockDiagram::new("Circuit1");
    d1.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    d1.add_block(Block::new("r1", "R1", BlockType::Resistor));
    d1.add_block(Block::new("r2", "R2", BlockType::Resistor));

    // Connect R1(n) to Ground, R2(n) to Ground (multi-wire on Ground terminal)
    // Connect R1(p) to R2(p)
    d1.add_electrical_line("r1", 1, "gnd", 0);
    d1.add_electrical_line("r2", 1, "gnd", 0);
    d1.add_electrical_line("r1", 0, "r2", 0);

    let res1 = resolve_electrical_nets(&d1);
    assert!(res1
        .diagnostics
        .iter()
        .all(|d| d.severity != DiagnosticSeverity::Error));
    let top1 = res1.topology.expect("topology must be valid");

    // Ground net must be 0
    assert_eq!(top1.terminal_to_net.get(&("gnd".to_string(), 0)), Some(&0));
    assert_eq!(top1.terminal_to_net.get(&("r1".to_string(), 1)), Some(&0));
    assert_eq!(top1.terminal_to_net.get(&("r2".to_string(), 1)), Some(&0));

    // Top nodes R1(p) and R2(p) are in node 1
    assert_eq!(top1.terminal_to_net.get(&("r1".to_string(), 0)), Some(&1));
    assert_eq!(top1.terminal_to_net.get(&("r2".to_string(), 0)), Some(&1));

    // Now build diagram with lines completely reversed (undirected test)
    let mut d2 = BlockDiagram::new("Circuit2");
    d2.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    d2.add_block(Block::new("r1", "R1", BlockType::Resistor));
    d2.add_block(Block::new("r2", "R2", BlockType::Resistor));

    d2.add_electrical_line("gnd", 0, "r1", 1);
    d2.add_electrical_line("gnd", 0, "r2", 1);
    d2.add_electrical_line("r2", 0, "r1", 0);

    let res2 = resolve_electrical_nets(&d2);
    let top2 = res2.topology.expect("topology must be valid");
    assert_eq!(top1.terminal_to_net, top2.terminal_to_net);
    assert_eq!(top1.net_ids, top2.net_ids);
}

#[test]
fn test_duplicate_and_self_wire_warnings() {
    let mut d = BlockDiagram::new("WarningsCircuit");
    d.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor));

    // Normal connection to Ground
    d.add_electrical_line("r1", 1, "gnd", 0);

    // Duplicate wire between r1(1) and gnd(0) (in reverse direction)
    d.add_electrical_line("gnd", 0, "r1", 1);

    // Self wire on r1(0)
    d.add_electrical_line("r1", 0, "r1", 0);

    let res = resolve_electrical_nets(&d);
    // Warnings do not invalidate topology
    assert!(res.topology.is_some());

    let w_wires: Vec<_> = res
        .diagnostics
        .iter()
        .filter(|d| d.code == "W-WIRE")
        .collect();
    assert_eq!(w_wires.len(), 2);
    assert!(
        w_wires[0].message.contains("Duplicate wire") || w_wires[0].message.contains("self-loop")
    );
    assert!(
        w_wires[1].message.contains("Duplicate wire") || w_wires[1].message.contains("self-loop")
    );
}

#[test]
fn test_unknown_sid_or_terminal_error_not_panic() {
    let mut d = BlockDiagram::new("FaultyLines");
    d.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor));

    // Connect to non-existent block
    d.add_electrical_line("r1", 0, "nonexistent_block", 0);

    // Connect to out of bounds terminal index on Resistor (only 0 and 1 exist)
    d.add_electrical_line("r1", 99, "gnd", 0);

    let res = resolve_electrical_nets(&d);
    assert!(res.topology.is_none()); // Errors prevent valid topology

    let errors: Vec<_> = res
        .diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert_eq!(errors.len(), 2);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("unknown block SID 'nonexistent_block'")));
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Unknown terminal index 99")));
}

#[test]
fn test_ground_merging_and_missing_ground_detection() {
    // 1. Two separate grounds merge into node 0
    let mut d1 = BlockDiagram::new("TwoGrounds");
    d1.add_block(Block::new("gnd1", "GND1", BlockType::Ground));
    d1.add_block(Block::new("gnd2", "GND2", BlockType::Ground));
    d1.add_block(Block::new("r1", "R1", BlockType::Resistor));
    d1.add_electrical_line("r1", 0, "gnd1", 0);
    d1.add_electrical_line("r1", 1, "gnd2", 0);

    let res1 = resolve_electrical_nets(&d1);
    let top1 = res1
        .topology
        .expect("topology with multiple grounds must be valid");
    assert_eq!(top1.terminal_to_net.get(&("gnd1".to_string(), 0)), Some(&0));
    assert_eq!(top1.terminal_to_net.get(&("gnd2".to_string(), 0)), Some(&0));
    assert_eq!(top1.terminal_to_net.get(&("r1".to_string(), 0)), Some(&0));
    assert_eq!(top1.terminal_to_net.get(&("r1".to_string(), 1)), Some(&0));

    // 2. Island without Ground produces E-GND
    let mut d2 = BlockDiagram::new("FloatingIsland");
    d2.add_block(Block::new("r1", "R1", BlockType::Resistor));
    d2.add_block(Block::new("c1", "C1", BlockType::Capacitor));
    d2.add_electrical_line("r1", 0, "c1", 0);
    d2.add_electrical_line("r1", 1, "c1", 1);

    let res2 = resolve_electrical_nets(&d2);
    assert!(res2.topology.is_none());
    let e_gnd = res2
        .diagnostics
        .iter()
        .find(|d| d.code == "E-GND")
        .expect("E-GND expected");
    assert!(e_gnd.message.contains("no Ground reference"));
    assert!(e_gnd.block_sids.contains(&"r1".to_string()));
    assert!(e_gnd.block_sids.contains(&"c1".to_string()));

    // 3. Two separate islands without Ground reported separately
    let mut d3 = BlockDiagram::new("TwoFloatingIslands");
    d3.add_block(Block::new("r1", "R1", BlockType::Resistor));
    d3.add_block(Block::new("r2", "R2", BlockType::Resistor));
    d3.add_block(Block::new("r3", "R3", BlockType::Resistor));
    d3.add_block(Block::new("r4", "R4", BlockType::Resistor));
    // Island 1: R1 and R2
    d3.add_electrical_line("r1", 0, "r2", 0);
    d3.add_electrical_line("r1", 1, "r2", 1);
    // Island 2: R3 and R4
    d3.add_electrical_line("r3", 0, "r4", 0);
    d3.add_electrical_line("r3", 1, "r4", 1);

    let res3 = resolve_electrical_nets(&d3);
    let gnd_errors: Vec<_> = res3
        .diagnostics
        .iter()
        .filter(|d| d.code == "E-GND")
        .collect();
    assert_eq!(gnd_errors.len(), 2);

    // 4. Ground with unconnected terminal emits W-OPEN
    let mut d4 = BlockDiagram::new("OpenGround");
    d4.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    let res4 = resolve_electrical_nets(&d4);
    let w_open = res4
        .diagnostics
        .iter()
        .find(|d| d.code == "W-OPEN")
        .expect("W-OPEN expected");
    assert!(w_open.message.contains("Unconnected electrical terminal"));
}

#[test]
fn test_deterministic_net_ids_invariant_to_wire_order() {
    let create_diagram_with_lines = |line_order: &[usize]| -> BlockDiagram {
        let mut d = BlockDiagram::new("DeterministicNets");
        d.add_block(Block::new("gnd", "Ground", BlockType::Ground));
        d.add_block(Block::new("src", "V1", BlockType::DcVoltageSource));
        d.add_block(Block::new("r1", "R1", BlockType::Resistor));
        d.add_block(Block::new("r2", "R2", BlockType::Resistor));

        let lines = [
            Line {
                src: PortRef::electrical("src", 1),
                dst: PortRef::electrical("gnd", 0),
                name: None,
            },
            Line {
                src: PortRef::electrical("r2", 1),
                dst: PortRef::electrical("gnd", 0),
                name: None,
            },
            Line {
                src: PortRef::electrical("src", 0),
                dst: PortRef::electrical("r1", 0),
                name: None,
            },
            Line {
                src: PortRef::electrical("r1", 1),
                dst: PortRef::electrical("r2", 0),
                name: None,
            },
        ];

        for &idx in line_order {
            d.lines.push(lines[idx].clone());
        }
        d
    };

    let d_forward = create_diagram_with_lines(&[0, 1, 2, 3]);
    let d_reversed = create_diagram_with_lines(&[3, 2, 1, 0]);
    let d_shuffled = create_diagram_with_lines(&[2, 0, 3, 1]);

    let res_f = resolve_electrical_nets(&d_forward);
    let res_r = resolve_electrical_nets(&d_reversed);
    let res_s = resolve_electrical_nets(&d_shuffled);

    let top_f = res_f.topology.expect("forward topology valid");
    let top_r = res_r.topology.expect("reversed topology valid");
    let top_s = res_s.topology.expect("shuffled topology valid");

    assert_eq!(top_f.terminal_to_net, top_r.terminal_to_net);
    assert_eq!(top_f.terminal_to_net, top_s.terminal_to_net);
    assert_eq!(top_f.net_ids, top_r.net_ids);
    assert_eq!(top_f.net_ids, top_s.net_ids);
}

#[test]
fn test_spec_parameter_validation_rules() {
    // 1. Resistor R <= 0 and non-finite
    let mut p_r = HashMap::new();
    p_r.insert("Resistance".to_string(), "-10.0".to_string());
    let diags_r = validate_block_params(&BlockType::Resistor, "r1", &p_r);
    assert_eq!(diags_r.len(), 1);
    assert_eq!(diags_r[0].code, "E-PARAM");

    let mut p_r_nan = HashMap::new();
    p_r_nan.insert("Resistance".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Resistor, "r1_nan", &p_r_nan)[0].code,
        "E-PARAM"
    );

    // 2. Capacitor C <= 0, non-finite C, non-finite V0
    let mut p_c1 = HashMap::new();
    p_c1.insert("Capacitance".to_string(), "0.0".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Capacitor, "c1", &p_c1)[0].code,
        "E-PARAM"
    );
    let mut p_c_inf = HashMap::new();
    p_c_inf.insert("Capacitance".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Capacitor, "c_inf", &p_c_inf)[0].code,
        "E-PARAM"
    );
    let mut p_c2 = HashMap::new();
    p_c2.insert("InitialVoltage".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Capacitor, "c2", &p_c2)[0].code,
        "E-PARAM"
    );

    // 3. Inductor L <= 0, non-finite L, non-finite I0
    let mut p_l1 = HashMap::new();
    p_l1.insert("Inductance".to_string(), "-0.001".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Inductor, "l1", &p_l1)[0].code,
        "E-PARAM"
    );
    let mut p_l_nan = HashMap::new();
    p_l_nan.insert("Inductance".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Inductor, "l_nan", &p_l_nan)[0].code,
        "E-PARAM"
    );
    let mut p_l2 = HashMap::new();
    p_l2.insert("InitialCurrent".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Inductor, "l2", &p_l2)[0].code,
        "E-PARAM"
    );

    // 4. DC Voltage Source non-finite V
    let mut p_v = HashMap::new();
    p_v.insert("Voltage".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::DcVoltageSource, "v1", &p_v)[0].code,
        "E-PARAM"
    );

    // 5. DC Current Source non-finite I
    let mut p_i = HashMap::new();
    p_i.insert("Current".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::DcCurrentSource, "i1", &p_i)[0].code,
        "E-PARAM"
    );

    // 6. AC Voltage Source Amplitude < 0, Amplitude non-finite, Frequency <= 0, Frequency non-finite, Phase non-finite, Offset non-finite
    let mut p_ac1 = HashMap::new();
    p_ac1.insert("Amplitude".to_string(), "-5.0".to_string());
    assert_eq!(
        validate_block_params(&BlockType::AcVoltageSource, "ac1", &p_ac1)[0].code,
        "E-PARAM"
    );
    let mut p_ac_amp_nan = HashMap::new();
    p_ac_amp_nan.insert("Amplitude".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::AcVoltageSource, "ac_amp_nan", &p_ac_amp_nan)[0].code,
        "E-PARAM"
    );

    let mut p_ac2 = HashMap::new();
    p_ac2.insert("Frequency".to_string(), "0.0".to_string());
    let ac2_err = validate_block_params(&BlockType::AcVoltageSource, "ac2", &p_ac2);
    assert_eq!(ac2_err[0].code, "E-PARAM");
    assert!(ac2_err[0]
        .message
        .contains("use a DC Voltage Source for 0 Hz"));

    let mut p_ac_freq_inf = HashMap::new();
    p_ac_freq_inf.insert("Frequency".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::AcVoltageSource, "ac_freq_inf", &p_ac_freq_inf)[0].code,
        "E-PARAM"
    );

    let mut p_ac_phase_nan = HashMap::new();
    p_ac_phase_nan.insert("Phase".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::AcVoltageSource, "ac_phase_nan", &p_ac_phase_nan)[0].code,
        "E-PARAM"
    );

    let mut p_ac_offset_inf = HashMap::new();
    p_ac_offset_inf.insert("Offset".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(
            &BlockType::AcVoltageSource,
            "ac_offset_inf",
            &p_ac_offset_inf
        )[0]
        .code,
        "E-PARAM"
    );

    // 7. Diode Vf < 0, non-finite Vf, Ron <= 0, non-finite Ron, Roff <= Ron, non-finite Roff
    let mut p_d1 = HashMap::new();
    p_d1.insert("ForwardVoltage".to_string(), "-0.7".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Diode, "d1", &p_d1)[0].code,
        "E-PARAM"
    );
    let mut p_d_vf_nan = HashMap::new();
    p_d_vf_nan.insert("ForwardVoltage".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Diode, "d_vf_nan", &p_d_vf_nan)[0].code,
        "E-PARAM"
    );

    let mut p_d2 = HashMap::new();
    p_d2.insert("OnResistance".to_string(), "0.0".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Diode, "d2", &p_d2)[0].code,
        "E-PARAM"
    );
    let mut p_d_ron_inf = HashMap::new();
    p_d_ron_inf.insert("OnResistance".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Diode, "d_ron_inf", &p_d_ron_inf)[0].code,
        "E-PARAM"
    );

    let mut p_d3 = HashMap::new();
    p_d3.insert("OnResistance".to_string(), "10.0".to_string());
    p_d3.insert("OffResistance".to_string(), "5.0".to_string()); // Roff < Ron
    let d3_err = validate_block_params(&BlockType::Diode, "d3", &p_d3);
    assert_eq!(d3_err[0].code, "E-PARAM");
    assert!(d3_err[0]
        .message
        .contains("off-resistance must be strictly greater than OnResistance"));

    let mut p_d_roff_nan = HashMap::new();
    p_d_roff_nan.insert("OnResistance".to_string(), "1.0".to_string());
    p_d_roff_nan.insert("OffResistance".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::Diode, "d_roff_nan", &p_d_roff_nan)[0].code,
        "E-PARAM"
    );

    // 8. Ideal Switch Ron <= 0, non-finite Ron, Roff <= Ron, non-finite Roff, non-finite Threshold
    let mut p_sw1 = HashMap::new();
    p_sw1.insert("OnResistance".to_string(), "-0.01".to_string());
    assert_eq!(
        validate_block_params(&BlockType::IdealSwitch, "sw1", &p_sw1)[0].code,
        "E-PARAM"
    );
    let mut p_sw_ron_nan = HashMap::new();
    p_sw_ron_nan.insert("OnResistance".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::IdealSwitch, "sw_ron_nan", &p_sw_ron_nan)[0].code,
        "E-PARAM"
    );

    let mut p_sw2 = HashMap::new();
    p_sw2.insert("OnResistance".to_string(), "100.0".to_string());
    p_sw2.insert("OffResistance".to_string(), "100.0".to_string()); // Roff == Ron
    assert_eq!(
        validate_block_params(&BlockType::IdealSwitch, "sw2", &p_sw2)[0].code,
        "E-PARAM"
    );
    let mut p_sw_roff_inf = HashMap::new();
    p_sw_roff_inf.insert("OnResistance".to_string(), "10.0".to_string());
    p_sw_roff_inf.insert("OffResistance".to_string(), "Inf".to_string());
    assert_eq!(
        validate_block_params(&BlockType::IdealSwitch, "sw_roff_inf", &p_sw_roff_inf)[0].code,
        "E-PARAM"
    );

    let mut p_sw3 = HashMap::new();
    p_sw3.insert("Threshold".to_string(), "NaN".to_string());
    assert_eq!(
        validate_block_params(&BlockType::IdealSwitch, "sw3", &p_sw3)[0].code,
        "E-PARAM"
    );
}

#[test]
fn test_cross_domain_signal_to_electrical_error() {
    let mut d = BlockDiagram::new("CrossDomain");
    d.add_block(Block::new("sine", "Sine Wave", BlockType::SineWave));
    d.add_block(Block::new("r1", "Resistor", BlockType::Resistor));

    // Wire signal output of Sine Wave directly to terminal 0 of Resistor
    d.lines.push(Line {
        src: PortRef::signal("sine", 0),
        dst: PortRef::electrical("r1", 0),
        name: None,
    });

    let res = resolve_electrical_nets(&d);
    assert!(res.topology.is_none());
    let e_domain = res
        .diagnostics
        .iter()
        .find(|d| d.code == "E-DOMAIN")
        .expect("E-DOMAIN expected");
    assert!(e_domain
        .message
        .contains("Direct wire between signal port and electrical terminal"));
    assert!(e_domain.block_sids.contains(&"sine".to_string()));
    assert!(e_domain.block_sids.contains(&"r1".to_string()));
}

#[test]
fn test_simulate_diagram_with_electrical_blocks_gives_e_unsupported() {
    let mut d = BlockDiagram::new("Over10SwitchesCircuit");
    d.add_block(Block::new("gnd", "Ground", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "5.0"));
    d.add_electrical_line("vs", 1, "gnd", 0);

    let mut prev_sid = "vs".to_string();
    let mut prev_term = 0;
    for k in 0..11 {
        let sid = format!("d{k}");
        d.add_block(Block::new(&sid, format!("D{k}"), BlockType::Diode));
        d.add_electrical_line(&prev_sid, prev_term, &sid, 0);
        prev_sid = sid;
        prev_term = 1;
    }
    d.add_block(Block::new("r", "R", BlockType::Resistor).with_param("Resistance", "100.0"));
    d.add_electrical_line(&prev_sid, prev_term, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);

    let config = SimulationConfig::default();
    let result = simulate_diagram(&d, &config);

    assert!(
        result.is_err(),
        "Circuit with > 10 switches must return error"
    );
    let err = result.err().unwrap();
    assert_eq!(err.code(), "E-UNSUPPORTED");
    assert!(err.message().contains("11 switches/diodes"));
    assert!(err
        .message()
        .contains("maximum supported topologies: 2^10 = 1024"));
}

#[test]
fn test_schema_v1_v2_v3_backward_compatibility_and_rotation_serde() {
    // 1. Deserializing older schema v1 JSON without rotation
    let json_v1 = r#"{
        "name": "LegacyDiagram",
        "schema_version": 1,
        "blocks": [
            {
                "sid": "1",
                "name": "Const",
                "block_type": "Constant",
                "parameters": {"Value": "42.0"}
            }
        ],
        "lines": []
    }"#;

    let mut d: BlockDiagram =
        serde_json::from_str(json_v1).expect("Legacy v1 diagram must deserialize cleanly");
    assert_eq!(d.schema_version, 1);
    assert_eq!(d.blocks[0].rotation, BlockRotation::R0); // Default applied

    // Migrate schema upgrades version to 3
    d.migrate_schema();
    assert_eq!(d.schema_version, 3);

    // 2. Round-tripping v3 diagram with rotation preserves rotation
    let mut d3 = BlockDiagram::new("V3Diagram");
    let mut b = Block::new("r1", "R1", BlockType::Resistor);
    b = b.with_rotation(BlockRotation::R90);
    d3.add_block(b);

    let serialized = serde_json::to_string(&d3).expect("Serialization succeeds");
    let deserialized: BlockDiagram =
        serde_json::from_str(&serialized).expect("Deserialization succeeds");
    assert_eq!(deserialized.schema_version, 3);
    assert_eq!(deserialized.blocks[0].rotation, BlockRotation::R90);
}

#[test]
fn test_slx_safety_foreign_resistor_stays_unknown() {
    // BlockType::from_str_name only maps standard Simulink builtins.
    // Foreign physical blocks must remain Unknown.
    let foreign1 = BlockType::from_str_name("Resistor");
    assert_eq!(foreign1, BlockType::Unknown("Resistor".to_string()));

    let foreign2 = BlockType::from_str_name("fl_lib/Elements/Resistor");
    assert_eq!(
        foreign2,
        BlockType::Unknown("fl_lib/Elements/Resistor".to_string())
    );

    let foreign3 = BlockType::from_str_name("SimscapeBlock");
    assert_eq!(foreign3, BlockType::Unknown("SimscapeBlock".to_string()));
}

#[test]
fn test_migration_v1_v2_handwritten_json_identical_simulation() {
    // Minimal hand-written v1 diagram (only required fields: name, blocks with sid/name/block_type/parameters, lines with src/dst)
    // Non-core fields omitted: version, release, schema_version, simulation_config, toolboxes, properties, position, rotation, line name, port_kind.
    let json_v1 = r#"{
        "name": "MinimalV1",
        "blocks": [
            {
                "sid": "1",
                "name": "Const",
                "block_type": "Constant",
                "parameters": {"Value": "5.0"}
            },
            {
                "sid": "2",
                "name": "Gain",
                "block_type": "Gain",
                "parameters": {"Gain": "3.0"}
            },
            {
                "sid": "3",
                "name": "Out",
                "block_type": "Outport"
            }
        ],
        "lines": [
            {
                "src": {"block_sid": "1", "port_index": 0},
                "dst": {"block_sid": "2", "port_index": 0}
            },
            {
                "src": {"block_sid": "2", "port_index": 0},
                "dst": {"block_sid": "3", "port_index": 0}
            }
        ]
    }"#;

    // Minimal hand-written v2 diagram (with partial simulation_config)
    let json_v2 = r#"{
        "name": "MinimalV2",
        "schema_version": 2,
        "simulation_config": {
            "start_time": 0.0,
            "stop_time": 1.0,
            "step_size": 0.1,
            "solver": "FixedStepEuler"
        },
        "blocks": [
            {
                "sid": "1",
                "name": "Const",
                "block_type": "Constant",
                "parameters": {"Value": "5.0"}
            },
            {
                "sid": "2",
                "name": "Gain",
                "block_type": "Gain",
                "parameters": {"Gain": "3.0"}
            },
            {
                "sid": "3",
                "name": "Out",
                "block_type": "Outport"
            }
        ],
        "lines": [
            {
                "src": {"block_sid": "1", "port_index": 0},
                "dst": {"block_sid": "2", "port_index": 0}
            },
            {
                "src": {"block_sid": "2", "port_index": 0},
                "dst": {"block_sid": "3", "port_index": 0}
            }
        ]
    }"#;

    let d_v1: BlockDiagram = serde_json::from_str(json_v1).expect("Minimal v1 must deserialize");
    let d_v2: BlockDiagram = serde_json::from_str(json_v2).expect("Minimal v2 must deserialize");

    assert_eq!(d_v1.schema_version, 1);
    assert_eq!(d_v2.schema_version, 2);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.1,
        solver: mlapp_sim::solver::SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res_v1 = simulate_diagram(&d_v1, &config).expect("v1 simulation succeeds");
    let res_v2 = simulate_diagram(&d_v2, &config).expect("v2 simulation succeeds");

    // Both simulations must produce bit-for-bit identical results
    assert_eq!(res_v1.tout.len(), res_v2.tout.len());
    for (t1, t2) in res_v1.tout.iter().zip(res_v2.tout.iter()) {
        assert_eq!(t1.to_bits(), t2.to_bits());
    }
    assert_eq!(res_v1.yout.len(), res_v2.yout.len());
    for (y1, y2) in res_v1.yout.iter().zip(res_v2.yout.iter()) {
        assert_eq!(y1.to_bits(), y2.to_bits());
        assert_eq!(*y1, 15.0); // 5.0 * 3.0
    }
    let out_sig_v1 = res_v1.signals.get("Out").expect("Out signal exists");
    let out_sig_v2 = res_v2.signals.get("Out").expect("Out signal exists");
    assert_eq!(out_sig_v1.len(), out_sig_v2.len());
    for (s1, s2) in out_sig_v1.iter().zip(out_sig_v2.iter()) {
        assert_eq!(s1.to_bits(), s2.to_bits());
        assert_eq!(*s1, 15.0); // 5.0 * 3.0
    }
}

#[test]
fn test_island_semantics_shared_ground_and_partitioning() {
    // Island A: V1 (dc voltage source) in parallel with R1, grounded at GND_A.
    // Island B: V2 (dc voltage source) in parallel with R2, grounded at GND_B.
    // There are NO direct connections between Island A and Island B except their Ground references.
    let mut d = BlockDiagram::new("DualIslandSubcircuits");
    // Island A blocks
    d.add_block(Block::new("gnd_a", "GND_A", BlockType::Ground));
    d.add_block(Block::new("v1", "V1", BlockType::DcVoltageSource));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor));

    // Island B blocks
    d.add_block(Block::new("gnd_b", "GND_B", BlockType::Ground));
    d.add_block(Block::new("v2", "V2", BlockType::DcVoltageSource));
    d.add_block(Block::new("r2", "R2", BlockType::Resistor));

    // Island A wires: V1(n) to GND_A, R1(n) to GND_A, V1(p) to R1(p)
    d.add_electrical_line("v1", 1, "gnd_a", 0);
    d.add_electrical_line("r1", 1, "gnd_a", 0);
    d.add_electrical_line("v1", 0, "r1", 0);

    // Island B wires: V2(n) to GND_B, R2(n) to GND_B, V2(p) to R2(p)
    d.add_electrical_line("v2", 1, "gnd_b", 0);
    d.add_electrical_line("r2", 1, "gnd_b", 0);
    d.add_electrical_line("v2", 0, "r2", 0);

    let res = resolve_electrical_nets(&d);
    assert!(
        res.diagnostics
            .iter()
            .all(|diag| diag.severity != DiagnosticSeverity::Error),
        "Islands sharing ground must produce no E-GND errors: {:?}",
        res.diagnostics
    );
    let top = res.topology.expect("Topology must be valid");

    // Both GND_A and GND_B merge into global reference Net 0
    let gnd_a_net = top.terminal_to_net.get(&("gnd_a".to_string(), 0)).copied();
    let gnd_b_net = top.terminal_to_net.get(&("gnd_b".to_string(), 0)).copied();
    assert_eq!(gnd_a_net, Some(0));
    assert_eq!(gnd_b_net, Some(0));

    // Non-ground nodes in Island A and Island B are strictly distinct
    let v1_p_net = *top.terminal_to_net.get(&("v1".to_string(), 0)).unwrap();
    let r1_p_net = *top.terminal_to_net.get(&("r1".to_string(), 0)).unwrap();
    assert_eq!(v1_p_net, r1_p_net);
    assert_ne!(v1_p_net, 0);

    let v2_p_net = *top.terminal_to_net.get(&("v2".to_string(), 0)).unwrap();
    let r2_p_net = *top.terminal_to_net.get(&("r2".to_string(), 0)).unwrap();
    assert_eq!(v2_p_net, r2_p_net);
    assert_ne!(v2_p_net, 0);

    // Island A and Island B have disjoint non-ground net sets
    assert_ne!(v1_p_net, v2_p_net);
    // In M5, since Node 0 is eliminated from MNA equations, the conductance matrix [G]
    // for this system has zero coupling between Net {v1_p_net} and Net {v2_p_net},
    // forming a block-diagonal matrix.
}
