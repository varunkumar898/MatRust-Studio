//! Integration and oracle tests for Milestone M7:
//! Sensors, Controlled Sources, and Hybrid Coupling.

use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SolverType};

#[test]
fn test_voltage_sensor_resistive_divider() {
    // 10V DC source -> R1 (1000 Ohm) -> R2 (1000 Ohm) -> GND.
    // VoltageSensor across R2: p at midpoint, n at GND.
    // Signal output wired to Outport "Vout".
    let mut d = BlockDiagram::new("VoltageSensorDivider");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    d.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "1000.0"));
    d.add_block(Block::new("vsens", "VSensor", BlockType::VoltageSensor));
    d.add_block(Block::new("out", "Vout", BlockType::Outport));

    // Electrical connections
    d.add_electrical_line("vs", 0, "r1", 0);
    d.add_electrical_line("r1", 1, "r2", 0);
    d.add_electrical_line("r2", 1, "gnd", 0);
    d.add_electrical_line("vs", 1, "gnd", 0);

    // Sensor electrical terminals: p=0 (midpoint), n=1 (gnd)
    d.add_electrical_line("vsens", 0, "r1", 1);
    d.add_electrical_line("vsens", 1, "gnd", 0);

    // Sensor signal output (port 0) -> Outport (port 0)
    d.add_line("vsens", 0, "out", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("Simulation with VoltageSensor must succeed");
    let vout = res.signals.get("Vout").expect("Vout signal logged");

    for &v in vout {
        assert!(
            (v - 5.0).abs() < 1e-10,
            "Voltage sensor must measure exactly 5.0V across R2, got {v}"
        );
    }
}

#[test]
fn test_current_sensor_series_circuit() {
    // 12V DC source -> CurrentSensor -> R (600 Ohm) -> GND.
    // Theoretical current: I = 12 / 600 = 0.020 A (20 mA).
    // CurrentSensor signal output wired to Outport "Iout".
    let mut d = BlockDiagram::new("CurrentSensorSeries");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "12.0"));
    d.add_block(Block::new("isens", "ISensor", BlockType::CurrentSensor));
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "600.0"));
    d.add_block(Block::new("out", "Iout", BlockType::Outport));

    // Electrical lines: Vs+ -> ISensor in (terminal 0); ISensor out (terminal 1) -> R in; R out -> GND; Vs- -> GND
    d.add_electrical_line("vs", 0, "isens", 0);
    d.add_electrical_line("isens", 1, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vs", 1, "gnd", 0);

    // Signal line: ISensor (port 0) -> Outport (port 0)
    d.add_line("isens", 0, "out", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("Simulation with CurrentSensor must succeed");
    let iout = res.signals.get("Iout").expect("Iout signal logged");

    for &i in iout {
        assert!(
            (i - 0.020).abs() < 1e-10,
            "Current sensor must measure exactly 0.020 A, got {i}"
        );
    }

    // Verify voltage drop across ideal CurrentSensor is identically 0.0V
    if let Some(vsens_v) = res.signals.get("ISensor.v") {
        for &v in vsens_v {
            assert!(
                v.abs() < 1e-10,
                "Voltage drop across ideal ammeter must be 0V, got {v}"
            );
        }
    }
}

#[test]
fn test_voltage_controlled_voltage_source_gain() {
    // Constant signal 2.5V -> VCVS (Gain = 4.0) -> Rload (100 Ohm) -> GND.
    // Expected output voltage: Vout = 4.0 * 2.5 = 10.0V.
    // Expected load current: Iout = 10.0 / 100 = 0.1A.
    let mut d = BlockDiagram::new("VcvsTest");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("ctrl", "Ctrl", BlockType::Constant).with_param("Value", "2.5"));
    d.add_block(
        Block::new("vcvs", "VCVS", BlockType::VoltageControlledVoltageSource)
            .with_param("Gain", "4.0"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "100.0"));
    d.add_block(Block::new("vsens", "VSens", BlockType::VoltageSensor));
    d.add_block(Block::new("out", "Vout", BlockType::Outport));

    // Signal connection to VCVS control input (port 0)
    d.add_line("ctrl", 0, "vcvs", 0);

    // Electrical connections: VCVS+ -> Rload -> GND; VCVS- -> GND
    d.add_electrical_line("vcvs", 0, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vcvs", 1, "gnd", 0);

    // Measure voltage across Rload
    d.add_electrical_line("vsens", 0, "r", 0);
    d.add_electrical_line("vsens", 1, "gnd", 0);
    d.add_line("vsens", 0, "out", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("VCVS simulation must succeed");
    let vout = res.signals.get("Vout").expect("Vout signal logged");

    for &v in vout {
        assert!(
            (v - 10.0).abs() < 1e-10,
            "VCVS must produce 10.0V (Gain 4.0 * 2.5V), got {v}"
        );
    }
}

#[test]
fn test_voltage_controlled_current_source_transconductance() {
    // Constant signal 3.0V -> VCCS (Transconductance = 0.05 A/V) -> Rload (200 Ohm) -> GND.
    // Injected current: I = 3.0 * 0.05 = 0.15 A.
    // Expected output voltage across Rload: V = 0.15 * 200 = 30.0V.
    let mut d = BlockDiagram::new("VccsTest");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("ctrl", "Ctrl", BlockType::Constant).with_param("Value", "3.0"));
    d.add_block(
        Block::new("vccs", "VCCS", BlockType::VoltageControlledCurrentSource)
            .with_param("Transconductance", "0.05"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "200.0"));
    d.add_block(Block::new("vsens", "VSens", BlockType::VoltageSensor));
    d.add_block(Block::new("out", "Vout", BlockType::Outport));

    // Signal connection to VCCS control input
    d.add_line("ctrl", 0, "vccs", 0);

    // Electrical connections: VCCS+ (terminal 0) -> Rload -> GND; VCCS- (terminal 1) -> GND
    d.add_electrical_line("vccs", 0, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vccs", 1, "gnd", 0);

    // Sensor
    d.add_electrical_line("vsens", 0, "r", 0);
    d.add_electrical_line("vsens", 1, "gnd", 0);
    d.add_line("vsens", 0, "out", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("VCCS simulation must succeed");
    let vout = res.signals.get("Vout").expect("Vout logged");

    for &v in vout {
        assert!(
            (v - 30.0).abs() < 1e-10,
            "VCCS must produce 30.0V across 200 Ohm (0.15A * 200), got {v}"
        );
    }
}

#[test]
fn test_ammeter_insertion_parity() {
    // Verify inserting CurrentSensor does NOT alter any circuit voltages or currents.
    // Circuit A: 10V -> 100 Ohm -> 400 Ohm -> GND.
    // Circuit B: 10V -> 100 Ohm -> CurrentSensor -> 400 Ohm -> GND.
    let mut da = BlockDiagram::new("CircuitA");
    da.add_block(Block::new("gnd", "GND", BlockType::Ground));
    da.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    da.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "100.0"));
    da.add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "400.0"));
    da.add_electrical_line("vs", 0, "r1", 0);
    da.add_electrical_line("r1", 1, "r2", 0);
    da.add_electrical_line("r2", 1, "gnd", 0);
    da.add_electrical_line("vs", 1, "gnd", 0);

    let mut db = BlockDiagram::new("CircuitB");
    db.add_block(Block::new("gnd", "GND", BlockType::Ground));
    db.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"));
    db.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "100.0"));
    db.add_block(Block::new("isens", "ISens", BlockType::CurrentSensor));
    db.add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "400.0"));
    db.add_electrical_line("vs", 0, "r1", 0);
    db.add_electrical_line("r1", 1, "isens", 0);
    db.add_electrical_line("isens", 1, "r2", 0);
    db.add_electrical_line("r2", 1, "gnd", 0);
    db.add_electrical_line("vs", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out_a = simulate_diagram(&da, &config).expect("sim A");
    let out_b = simulate_diagram(&db, &config).expect("sim B");

    for (k, &va) in out_a.signals["R2.v"].iter().enumerate() {
        let vb = out_b.signals["R2.v"][k];
        assert!(
            (va - vb).abs() < 1e-12,
            "Node voltages must match exactly with/without ammeter"
        );
    }
    for (k, &ia) in out_a.signals["R2.i"].iter().enumerate() {
        let ib = out_b.signals["R2.i"][k];
        assert!(
            (ia - ib).abs() < 1e-12,
            "Branch currents must match exactly with/without ammeter"
        );
    }
}

#[test]
fn test_direct_algebraic_loop_triggers_w_delay() {
    // Closed cross-domain feedback: VoltageSensor directly drives VCVS in the same circuit
    // without any state block (Integrator, UnitDelay, ZOH).
    // The engine must emit warning W-DELAY and insert a 1-step delay so execution proceeds.
    let mut d = BlockDiagram::new("CrossDomainAlgebraicLoop");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vsens", "VSens", BlockType::VoltageSensor));
    d.add_block(
        Block::new("vcvs", "VCVS", BlockType::VoltageControlledVoltageSource)
            .with_param("Gain", "2.0"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "100.0"));

    // VCVS drives Rload
    d.add_electrical_line("vcvs", 0, "r", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vcvs", 1, "gnd", 0);

    // Sensor measures Rload voltage
    d.add_electrical_line("vsens", 0, "r", 0);
    d.add_electrical_line("vsens", 1, "gnd", 0);

    // Direct signal line from sensor back to VCVS control input!
    d.add_line("vsens", 0, "vcvs", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.005,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("Simulation must succeed with W-DELAY fallback");
    let has_w_delay = res.diagnostics.iter().any(|diag| diag.code == "W-DELAY");
    assert!(
        has_w_delay,
        "Direct algebraic loop between sensor and source must emit W-DELAY warning"
    );
}

#[test]
fn test_op_amp_inverting_amplifier_closed_loop_gain() {
    // Inverting op-amp macro-model with dominant-pole compensation (integrator):
    // Vin (1.0V) -> Rin (10k Ohm) -> Summing Node
    // Rf (100k Ohm) from Summing Node to Vout
    // VoltageSensor: p = GND (0), n = Summing Node -> measures (0 - Vsum) = -Vsum (virtual ground error)
    // Op-amp core: Gain (K = 20,000 rad/s) -> Integrator -> VCVS (Gain = 1.0)
    // Theoretical closed-loop steady-state gain: Vout/Vin = -Rf/Rin = -10.0V.
    let mut d = BlockDiagram::new("InvertingOpAmp");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vin", "Vin", BlockType::DcVoltageSource).with_param("Voltage", "1.0"));
    d.add_block(Block::new("rin", "Rin", BlockType::Resistor).with_param("Resistance", "10000.0"));
    d.add_block(Block::new("rf", "Rf", BlockType::Resistor).with_param("Resistance", "100000.0"));
    d.add_block(Block::new("vsens", "VSens", BlockType::VoltageSensor));
    d.add_block(
        Block::new("opamp_gain", "OpAmpGain", BlockType::Gain).with_param("Gain", "20000.0"),
    );
    d.add_block(Block::new("opamp_int", "OpAmpInt", BlockType::Integrator));
    d.add_block(
        Block::new(
            "opamp_out",
            "OpAmpOut",
            BlockType::VoltageControlledVoltageSource,
        )
        .with_param("Gain", "1.0"),
    );
    d.add_block(Block::new("out", "Vout", BlockType::Outport));

    // Vin -> Rin -> Summing Node
    d.add_electrical_line("vin", 0, "rin", 0);
    d.add_electrical_line("vin", 1, "gnd", 0);

    // Summing Node connects Rin.terminal_1 and Rf.terminal_0
    d.add_electrical_line("rin", 1, "rf", 0);

    // Rf.terminal_1 connects to OpAmpOut output terminal 0 (Vout)
    d.add_electrical_line("rf", 1, "opamp_out", 0);
    d.add_electrical_line("opamp_out", 1, "gnd", 0);

    // Sensor measures: p=GND, n=Summing Node (rin.terminal_1)
    d.add_electrical_line("vsens", 0, "gnd", 0);
    d.add_electrical_line("vsens", 1, "rin", 1);

    // Feedback path: VSens -> OpAmpGain -> OpAmpInt -> OpAmpOut (VCVS)
    d.add_line("vsens", 0, "opamp_gain", 0);
    d.add_line("opamp_gain", 0, "opamp_int", 0);
    d.add_line("opamp_int", 0, "opamp_out", 0);

    // Measure Vout via sensor
    d.add_block(Block::new(
        "vsens_out",
        "VSensOut",
        BlockType::VoltageSensor,
    ));
    d.add_electrical_line("vsens_out", 0, "opamp_out", 0);
    d.add_electrical_line("vsens_out", 1, "gnd", 0);
    d.add_line("vsens_out", 0, "out", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.02,
        step_size: 1e-4,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let res = simulate_diagram(&d, &config).expect("Op-amp simulation must succeed");
    let vout = res.signals.get("Vout").expect("Vout logged");

    // After settling (tau ~ 0.55 ms; 20 ms ~ 36 tau), Vout must equal -10.0V
    let final_v = *vout.last().expect("last step");
    assert!(
        (final_v - (-10.0)).abs() < 1e-4,
        "Closed loop inverting op-amp must achieve -10.0V, got {final_v}"
    );
}
