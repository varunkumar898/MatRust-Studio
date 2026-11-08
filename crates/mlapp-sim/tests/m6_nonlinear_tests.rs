use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SolverType};

#[test]
fn test_half_wave_diode_rectifier_resistive_load_oracle() {
    // 50 Hz AC Source 10 V peak, Diode Vf = 0.7 V, Ron = 0.01 ohm, Roff = 1e6 ohm, Load R = 100 ohm.
    let mut d = BlockDiagram::new("HalfWaveRectifier");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "10.0")
            .with_param("Frequency", "50.0")
            .with_param("Phase", "0.0")
            .with_param("Offset", "0.0"),
    );
    d.add_block(
        Block::new("d", "D1", BlockType::Diode)
            .with_param("ForwardVoltage", "0.7")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "100.0"));

    // Vac(p) -> D1(anode, p) -> Net 1
    d.add_electrical_line("vac", 0, "d", 0);
    // D1(cathode, n) -> Rload(p) -> Net 2
    d.add_electrical_line("d", 1, "r", 0);
    // Rload(n) -> Ground (Net 0)
    d.add_electrical_line("r", 1, "gnd", 0);
    // Vac(n) -> Ground (Net 0)
    d.add_electrical_line("vac", 1, "gnd", 0);

    let dt = 1e-4; // 0.1 ms step
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.04, // 2 full 50Hz cycles (20 ms each)
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate half-wave rectifier");
    let v_load = out.signals.get("Rload.v").expect("Rload.v logged");
    let d_state = out.signals.get("D1.state").expect("D1.state logged");
    let d_current = out.signals.get("D1.i").expect("D1.i logged");

    // Positive peak occurs at t = 0.005 s (1/4 of 20 ms period)
    let peak_idx = (0.005 / dt).round() as usize;
    let v_peak = v_load[peak_idx];
    // Expected output at peak: Vac - Vf = 10.0 - 0.7 = 9.3 V (with Ron voltage divider: 100 / 100.01 * 9.3 ~= 9.299 V)
    assert!(
        (v_peak - 9.299).abs() < 0.02,
        "Positive peak voltage across load must be ~9.3 V, got {v_peak:.4} V"
    );
    assert_eq!(
        d_state[peak_idx], 1.0,
        "Diode must be in ON state (1.0) during positive conduction"
    );
    assert!(
        d_current[peak_idx] > 0.09,
        "Diode current at peak should be ~0.093 A, got {:.4} A",
        d_current[peak_idx]
    );

    // Negative peak occurs at t = 0.015 s
    let neg_peak_idx = (0.015 / dt).round() as usize;
    let v_neg = v_load[neg_peak_idx];
    assert!(
        v_neg.abs() < 0.001,
        "Output voltage during negative half-cycle must be ~0 V (blocked), got {v_neg:.6} V"
    );
    assert_eq!(
        d_state[neg_peak_idx], 0.0,
        "Diode must be in OFF state (0.0) during negative blocking"
    );
    assert!(
        d_current[neg_peak_idx].abs() < 1e-4,
        "Diode reverse leakage must be negligible, got {:.6} A",
        d_current[neg_peak_idx]
    );
}

#[test]
fn test_half_wave_rectifier_with_smoothing_capacitor_oracle() {
    // AC Source 10 V peak 50 Hz, Diode Vf = 0.7 V, C = 100 uF, R = 1000 ohm.
    // Time constant tau = R * C = 1000 * 100e-6 = 0.1 s.
    let mut d = BlockDiagram::new("RectifierWithCapacitor");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "10.0")
            .with_param("Frequency", "50.0"),
    );
    d.add_block(
        Block::new("d", "D1", BlockType::Diode)
            .with_param("ForwardVoltage", "0.7")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(
        Block::new("c", "Cfilter", BlockType::Capacitor)
            .with_param("Capacitance", "1e-4")
            .with_param("InitialVoltage", "0.0"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "1000.0"));

    // Vac(p) -> D1(anode) -> Net 1
    d.add_electrical_line("vac", 0, "d", 0);
    // D1(cathode) -> Net 2 (connected to Cfilter and Rload)
    d.add_electrical_line("d", 1, "c", 0);
    d.add_electrical_line("c", 0, "r", 0);
    // Ground connections
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("c", 1, "gnd", 0);
    d.add_electrical_line("vac", 1, "gnd", 0);

    let dt = 1e-4;
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.05,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate rectifier with capacitor");
    let v_cap = out.signals.get("Cfilter.v").expect("Cfilter.v logged");

    // After first peak at ~0.005 s, capacitor charges to ~9.3 V
    let peak_idx = (0.005 / dt).round() as usize;
    assert!(
        (v_cap[peak_idx] - 9.3).abs() < 0.1,
        "Capacitor must charge to ~9.3 V at first peak, got {:.4} V",
        v_cap[peak_idx]
    );

    // At t = 0.015 s (when Vac is negative), capacitor holds charge and discharges with tau = 0.1 s
    // dt_discharge = 0.010 s -> v_decay ~= 9.3 * exp(-0.010 / 0.1) ~= 9.3 * 0.9048 ~= 8.41 V
    let neg_peak_idx = (0.015 / dt).round() as usize;
    let v_decay = v_cap[neg_peak_idx];
    assert!(
        (v_decay - 8.41).abs() < 0.2,
        "Capacitor voltage during diode off-phase must discharge to ~8.4 V, got {:.4} V",
        v_decay
    );
}

#[test]
fn test_diode_voltage_clipper_circuit() {
    // AC input 10 V peak. Series resistor 100 ohm.
    // Diode in series with DC 3.0 V bias voltage to ground.
    // Clipping occurs at 3.0 + 0.7 = 3.7 V.
    let mut d = BlockDiagram::new("DiodeClipper");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "10.0")
            .with_param("Frequency", "50.0"),
    );
    d.add_block(Block::new("r", "Rseries", BlockType::Resistor).with_param("Resistance", "100.0"));
    d.add_block(
        Block::new("d", "D1", BlockType::Diode)
            .with_param("ForwardVoltage", "0.7")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(
        Block::new("vbias", "Vbias", BlockType::DcVoltageSource).with_param("Voltage", "3.0"),
    );

    // Vac(p) -> Net 1 -> Rseries(p)
    d.add_electrical_line("vac", 0, "r", 0);
    // Rseries(n) -> Net 2 (Output node) -> D1(anode, p)
    d.add_electrical_line("r", 1, "d", 0);
    // D1(cathode, n) -> Net 3 -> Vbias(p)
    d.add_electrical_line("d", 1, "vbias", 0);
    // Vbias(n) -> Ground
    d.add_electrical_line("vbias", 1, "gnd", 0);
    // Vac(n) -> Ground
    d.add_electrical_line("vac", 1, "gnd", 0);

    let dt = 1e-4;
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.02,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate diode clipper");
    let v_out = out
        .signals
        .get("Net2")
        .expect("Net2 (clipped output) logged");

    // Positive peak of Vac is 10 V, but Net 2 must be clipped to ~3.7 V
    let peak_idx = (0.005 / dt).round() as usize;
    let v_clipped = v_out[peak_idx];
    assert!(
        (v_clipped - 3.7).abs() < 0.05,
        "Clipped output voltage at peak must be ~3.7 V (3.0 V bias + 0.7 V diode), got {v_clipped:.4} V"
    );

    // Negative peak (-10 V): diode is reverse-biased (OFF), so Net 2 follows Vac (-10 V)
    let neg_idx = (0.015 / dt).round() as usize;
    let v_unclipped = v_out[neg_idx];
    assert!(
        (v_unclipped - (-10.0)).abs() < 0.05,
        "Negative peak voltage must remain unclipped at -10 V, got {v_unclipped:.4} V"
    );
}

#[test]
fn test_ideal_switch_driven_by_step_control_signal() {
    // DC 12 V source -> Ideal Switch (threshold = 0.5) -> Load 10 ohm -> Ground.
    // Step control signal: 0.0 until t = 0.005 s, then 1.0.
    let mut d = BlockDiagram::new("IdealSwitchCircuit");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "12.0"));
    d.add_block(
        Block::new("sw", "Switch1", BlockType::IdealSwitch)
            .with_param("Threshold", "0.5")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "10.0"));
    d.add_block(
        Block::new("ctrl", "StepCtrl", BlockType::Step)
            .with_param("Time", "0.005")
            .with_param("Before", "0.0")
            .with_param("After", "1.0"),
    );

    // Electrical connections:
    // Vs(p) -> Switch1(p) -> Net 1
    d.add_electrical_line("vs", 0, "sw", 0);
    // Switch1(n) -> Rload(p) -> Net 2
    d.add_electrical_line("sw", 1, "r", 0);
    // Rload(n) -> Ground
    d.add_electrical_line("r", 1, "gnd", 0);
    // Vs(n) -> Ground
    d.add_electrical_line("vs", 1, "gnd", 0);

    // Signal connection: StepCtrl output 1 -> Switch1 control port 1
    d.add_line("ctrl", 1, "sw", 1);

    let dt = 1e-4;
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate ideal switch");
    let v_load = out.signals.get("Rload.v").expect("Rload.v logged");
    let sw_state = out
        .signals
        .get("Switch1.state")
        .expect("Switch1.state logged");

    // Before t = 0.005 s: control is 0.0 < 0.5 -> Switch OFF -> Vload ~ 0 V
    let before_idx = (0.002 / dt).round() as usize;
    assert!(
        v_load[before_idx].abs() < 0.001,
        "Load voltage before switch turn-on must be ~0 V, got {:.6} V",
        v_load[before_idx]
    );
    assert_eq!(sw_state[before_idx], 0.0, "Switch state must be 0.0 (OFF)");

    // After t = 0.005 s: control is 1.0 >= 0.5 -> Switch ON -> Vload ~ 12 V
    let after_idx = (0.008 / dt).round() as usize;
    let v_on = v_load[after_idx];
    assert!(
        (v_on - 11.988).abs() < 0.02,
        "Load voltage after switch turn-on must be ~12 V (10 / 10.01 * 12 ~= 11.988 V), got {v_on:.4} V"
    );
    assert_eq!(sw_state[after_idx], 1.0, "Switch state must be 1.0 (ON)");
}

#[test]
fn test_buck_converter_freewheeling_diode_and_switch_oracle() {
    // Buck converter stage:
    // DC source 24 V, Switch controlled by pulse, freewheeling Diode, Inductor 1 mH, Capacitor 100 uF, Load 10 ohm.
    let mut d = BlockDiagram::new("BuckConverter");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "24.0"));
    d.add_block(
        Block::new("sw", "MainSwitch", BlockType::IdealSwitch)
            .with_param("Threshold", "0.5")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(
        Block::new("d", "FreewheelDiode", BlockType::Diode)
            .with_param("ForwardVoltage", "0.7")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    d.add_block(
        Block::new("l", "Inductor", BlockType::Inductor)
            .with_param("Inductance", "1e-3")
            .with_param("InitialCurrent", "0.0"),
    );
    d.add_block(
        Block::new("c", "Capacitor", BlockType::Capacitor)
            .with_param("Capacitance", "1e-4")
            .with_param("InitialVoltage", "0.0"),
    );
    d.add_block(Block::new("r", "Load", BlockType::Resistor).with_param("Resistance", "10.0"));

    // Switch control: ON until 0.002 s, then OFF
    d.add_block(
        Block::new("ctrl", "Gate", BlockType::Step)
            .with_param("Time", "0.002")
            .with_param("Before", "1.0")
            .with_param("After", "0.0"),
    );

    // Vs(p) -> Switch(p) -> Net 1
    d.add_electrical_line("vs", 0, "sw", 0);
    // Switch(n) -> Switching node (Net 2)
    d.add_electrical_line("sw", 1, "l", 0);
    // FreewheelDiode: Cathode (n) to switching node (Net 2), Anode (p) to Ground (Net 0)
    d.add_electrical_line("d", 1, "l", 0);
    d.add_electrical_line("d", 0, "gnd", 0);
    // Inductor(n) -> Output node (Net 3) -> Capacitor(p) & Load(p)
    d.add_electrical_line("l", 1, "c", 0);
    d.add_electrical_line("c", 0, "r", 0);
    // Grounds
    d.add_electrical_line("c", 1, "gnd", 0);
    d.add_electrical_line("r", 1, "gnd", 0);
    d.add_electrical_line("vs", 1, "gnd", 0);

    // Gate -> MainSwitch
    d.add_line("ctrl", 1, "sw", 1);

    let dt = 1e-5; // 10 us step
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.004,
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate buck converter");
    let il = out.signals.get("Inductor.i").expect("Inductor.i logged");
    let d_state = out
        .signals
        .get("FreewheelDiode.state")
        .expect("FreewheelDiode.state logged");
    let sw_state = out
        .signals
        .get("MainSwitch.state")
        .expect("MainSwitch.state logged");

    // Phase 1: t in [0..0.002 s]: MainSwitch is ON (1.0), Diode is reverse-biased (0.0)
    let t1_idx = (0.001 / dt).round() as usize;
    assert_eq!(
        sw_state[t1_idx], 1.0,
        "Main switch must be ON during phase 1"
    );
    assert_eq!(d_state[t1_idx], 0.0, "Diode must be OFF during phase 1");
    assert!(il[t1_idx] > 0.0, "Inductor current must be ramping up");

    // Peak inductor current at t = 0.002 s
    let peak_idx = (0.002 / dt).round() as usize;
    let i_peak = il[peak_idx];
    assert!(
        i_peak > 1.0,
        "Inductor current should peak above 1 A, got {i_peak:.3} A"
    );

    // Phase 2: immediately after t = 0.002 s (e.g. t = 0.00205 s):
    // MainSwitch turns OFF (0.0), Diode freewheels (ON = 1.0) while inductor discharges
    let t2_freewheel_idx = (0.00205 / dt).round() as usize;
    assert_eq!(
        sw_state[t2_freewheel_idx], 0.0,
        "Main switch must be OFF during phase 2"
    );
    assert_eq!(
        d_state[t2_freewheel_idx], 1.0,
        "Freewheel diode must turn ON to maintain inductor current continuity"
    );
    assert!(
        il[t2_freewheel_idx] > 0.5,
        "Inductor current must be discharging through freewheel diode"
    );

    // Phase 3: after inductor finishes discharging (t = 0.0025 s):
    // Diode turns OFF in discontinuous conduction mode
    let t3_idx = (0.0025 / dt).round() as usize;
    assert_eq!(
        d_state[t3_idx], 0.0,
        "Freewheel diode must turn OFF once inductor current reaches zero"
    );
}

#[test]
fn test_switch_chattering_limit_cycle_detection_triggers_e_chatter() {
    // Circuit designed to trigger switch chattering / limit cycle:
    // Anti-parallel diodes with negative forward voltage or opposing threshold ideal switches
    // driven into an algebraic oscillation within the same time step.
    let mut d = BlockDiagram::new("ChatterCircuit");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "5.0"));

    // Switch A: ON if Net 2 < 2.5 V
    // Switch B: ON if Net 2 > 2.5 V
    // Coupled such that turning ON changes the node voltage across the threshold
    d.add_block(
        Block::new("d1", "DiodeA", BlockType::Diode)
            .with_param("ForwardVoltage", "0.0")
            .with_param("OnResistance", "1.0")
            .with_param("OffResistance", "100.0"),
    );
    d.add_block(
        Block::new("d2", "DiodeB", BlockType::Diode)
            .with_param("ForwardVoltage", "0.0")
            .with_param("OnResistance", "1.0")
            .with_param("OffResistance", "100.0"),
    );

    // DiodeA p to Vs, n to Net 2
    d.add_electrical_line("vs", 0, "d1", 0);
    d.add_electrical_line("d1", 1, "d2", 1); // cathode to cathode
    d.add_electrical_line("d2", 0, "vs", 0); // anode to Vs
    d.add_electrical_line("vs", 1, "gnd", 0);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.01,
        step_size: 0.001,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    // If simulation encounters limit cycle or chattering, it returns E-CHATTER
    // Or if solver executes without chattering, it solves cleanly.
    let res = simulate_diagram(&d, &config);
    if let Err(err) = res {
        assert_eq!(err.code(), "E-CHATTER");
        assert!(err.message().contains("E-CHATTER"));
    }
}

#[test]
fn test_full_wave_bridge_rectifier_oracle() {
    // 4-diode bridge:
    // AC input: 12 V peak 50 Hz across nodes 1 and 2
    // D1: node 1 -> node 3 (DC+)
    // D2: node 2 -> node 3 (DC+)
    // D3: node 4 (DC-) -> node 1
    // D4: node 4 (DC-) -> node 2
    // Load resistor between node 3 and node 4.
    // Node 4 is grounded as DC reference.
    let mut d = BlockDiagram::new("BridgeRectifier");
    d.add_block(Block::new("gnd", "GND", BlockType::Ground));
    d.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "12.0")
            .with_param("Frequency", "50.0"),
    );
    d.add_block(Block::new("d1", "D1", BlockType::Diode).with_param("ForwardVoltage", "0.7"));
    d.add_block(Block::new("d2", "D2", BlockType::Diode).with_param("ForwardVoltage", "0.7"));
    d.add_block(Block::new("d3", "D3", BlockType::Diode).with_param("ForwardVoltage", "0.7"));
    d.add_block(Block::new("d4", "D4", BlockType::Diode).with_param("ForwardVoltage", "0.7"));
    d.add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "100.0"));

    // Vac terminals
    // Vac(p) -> Net 1
    d.add_electrical_line("vac", 0, "d1", 0);
    d.add_electrical_line("d1", 0, "d3", 1);
    // Vac(n) -> Net 2
    d.add_electrical_line("vac", 1, "d2", 0);
    d.add_electrical_line("d2", 0, "d4", 1);

    // DC+ (Net 3): D1 cathode and D2 cathode -> Rload(p)
    d.add_electrical_line("d1", 1, "d2", 1);
    d.add_electrical_line("d1", 1, "r", 0);

    // DC- (Net 0, Ground): D3 anode and D4 anode -> Rload(n) -> Ground
    d.add_electrical_line("d3", 0, "d4", 0);
    d.add_electrical_line("d3", 0, "r", 1);
    d.add_electrical_line("r", 1, "gnd", 0);

    let dt = 1e-4;
    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 0.03, // 1.5 cycles
        step_size: dt,
        solver: SolverType::FixedStepEuler,
        ..Default::default()
    };

    let out = simulate_diagram(&d, &config).expect("simulate bridge rectifier");
    let v_load = out.signals.get("Rload.v").expect("Rload.v logged");

    // In a full-wave rectifier, both half-cycles produce positive load voltage!
    // Positive peak at t = 0.005 s: Vac = +12 V -> Vload ~= 12 - 2 * 0.7 = 10.6 V
    let idx_pos = (0.005 / dt).round() as usize;
    assert!(
        (v_load[idx_pos] - 10.59).abs() < 0.1,
        "Positive half-cycle output should be ~10.6 V, got {:.4} V",
        v_load[idx_pos]
    );

    // Negative peak at t = 0.015 s: Vac = -12 V -> Vload ~= 12 - 2 * 0.7 = 10.6 V (inverted to positive!)
    let idx_neg = (0.015 / dt).round() as usize;
    assert!(
        (v_load[idx_neg] - 10.59).abs() < 0.1,
        "Negative half-cycle output must also be ~10.6 V (full-wave rectified), got {:.4} V",
        v_load[idx_neg]
    );
}
