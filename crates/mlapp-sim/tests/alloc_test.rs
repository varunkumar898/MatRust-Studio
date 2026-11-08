use mlapp_sim::engine::SimWorkspace;
use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::ExecutionEngine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

struct CountingAlloc;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING_ACTIVE.try_with(|t| t.get()).unwrap_or(false) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static A: CountingAlloc = CountingAlloc;

#[test]
fn test_zero_allocations_during_eval_and_update_discrete() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut diagram = BlockDiagram::new("AllocTestModel");

    let step = Block::new("1", "Step", BlockType::Step)
        .with_param("Time", "1.0")
        .with_param("Before", "0.0")
        .with_param("After", "2.0");

    let sum = Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "+-");

    let integ =
        Block::new("3", "Integrator", BlockType::Integrator).with_param("InitialCondition", "0.0");

    let delay =
        Block::new("4", "UnitDelay", BlockType::UnitDelay).with_param("InitialCondition", "0.0");

    let gain = Block::new("5", "Gain", BlockType::Gain).with_param("Gain", "0.5");

    let sat = Block::new("6", "Saturation", BlockType::Saturation)
        .with_param("LowerLimit", "-10.0")
        .with_param("UpperLimit", "10.0");

    let scope = Block::new("7", "Scope", BlockType::Scope);

    diagram.add_block(step);
    diagram.add_block(sum);
    diagram.add_block(integ);
    diagram.add_block(delay);
    diagram.add_block(gain);
    diagram.add_block(sat);
    diagram.add_block(scope);

    // Wire: Step -> Sum[0]
    diagram.add_line("1", 1, "2", 1);
    // Sum -> Integrator
    diagram.add_line("2", 1, "3", 1);
    // Integrator -> UnitDelay
    diagram.add_line("3", 1, "4", 1);
    // UnitDelay -> Saturation
    diagram.add_line("4", 1, "6", 1);
    // Saturation -> Gain
    diagram.add_line("6", 1, "5", 1);
    // Gain -> Sum[1] (feedback)
    diagram.add_line("5", 1, "2", 2);
    // Integrator -> Scope
    diagram.add_line("3", 1, "7", 1);

    // Compile engine and pre-allocate workspace outside tracking region
    let engine = ExecutionEngine::compile(&diagram).expect("compile engine");
    let mut workspace =
        SimWorkspace::new(diagram.blocks.len(), engine.layout.num_continuous_states);
    let continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();

    // Start tracking allocations on this thread
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ACTIVE.with(|t| t.set(true));

    for step_idx in 0..10_000 {
        let t = step_idx as f64 * 0.001;
        engine
            .eval(t, &continuous_states, &discrete_states, &mut workspace)
            .expect("eval");
        engine.update_discrete(&workspace.outputs, &mut discrete_states);
    }

    TRACKING_ACTIVE.with(|t| t.set(false));
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "SimWorkspace eval and update_discrete must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}

#[test]
fn test_zero_allocations_m3_blocks_during_eval_and_update_discrete() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut diagram = BlockDiagram::new("AllocTestM3Model");

    diagram.add_block(Block::new("1", "Clock", BlockType::Clock));
    diagram.add_block(
        Block::new("2", "Saw", BlockType::Sawtooth)
            .with_param("Amplitude", "2.0")
            .with_param("Frequency", "10.0"),
    );
    diagram.add_block(
        Block::new("3", "Tri", BlockType::Triangle)
            .with_param("Amplitude", "1.5")
            .with_param("Frequency", "5.0"),
    );
    diagram.add_block(
        Block::new("4", "Pulse", BlockType::PulseGenerator)
            .with_param("Period", "0.1")
            .with_param("Duty", "50.0"),
    );
    diagram.add_block(
        Block::new("5", "RepSeq", BlockType::RepeatingSequence)
            .with_param("TimeValues", "[0, 0.05, 0.1]")
            .with_param("OutputValues", "[0, 1, 0]"),
    );
    diagram.add_block(
        Block::new("6", "RepStair", BlockType::RepeatingSequenceStair)
            .with_param("TimeValues", "[0, 0.05, 0.1]")
            .with_param("OutputValues", "[0, 1, 0]"),
    );
    diagram.add_block(
        Block::new("7", "TF", BlockType::TransferFunction)
            .with_param("Numerator", "1")
            .with_param("Denominator", "1, 2, 1"),
    );
    diagram.add_block(
        Block::new("8", "DTF", BlockType::DiscreteTransferFunction)
            .with_param("Numerator", "1")
            .with_param("Denominator", "1, -0.5")
            .with_param("SampleTime", "0.01"),
    );
    diagram.add_block(
        Block::new("9", "ContPID", BlockType::Pid)
            .with_param("P", "2.0")
            .with_param("I", "3.0")
            .with_param("D", "0.1")
            .with_param("N", "50.0")
            .with_param("AntiWindupMode", "Clamping")
            .with_param("UpperSaturationLimit", "5.0")
            .with_param("LowerSaturationLimit", "-5.0"),
    );
    diagram.add_block(
        Block::new("10", "DiscPID", BlockType::Pid)
            .with_param("TimeDomain", "Discrete")
            .with_param("P", "1.5")
            .with_param("I", "2.0")
            .with_param("SampleTime", "0.01"),
    );
    diagram.add_block(
        Block::new("11", "ZOH", BlockType::ZeroOrderHold).with_param("SampleTime", "0.01"),
    );
    diagram.add_block(Block::new("12", "Abs", BlockType::Abs));
    diagram.add_block(
        Block::new("13", "Math", BlockType::MathFunction).with_param("Function", "sqrt"),
    );
    diagram.add_block(
        Block::new("14", "RelOp", BlockType::RelationalOperator).with_param("Operator", ">="),
    );
    diagram.add_block(
        Block::new("15", "LogOp", BlockType::LogicalOperator).with_param("Operator", "AND"),
    );
    diagram.add_block(
        Block::new("16", "Switch", BlockType::Switch)
            .with_param("Criteria", ">=")
            .with_param("Threshold", "0.0"),
    );
    diagram.add_block(Block::new("17", "Scope", BlockType::Scope));

    // Connect wires
    diagram.add_line("1", 1, "7", 1); // Clock -> TF
    diagram.add_line("2", 1, "8", 1); // Saw -> DTF
    diagram.add_line("3", 1, "9", 1); // Tri -> ContPID
    diagram.add_line("4", 1, "10", 1); // Pulse -> DiscPID
    diagram.add_line("5", 1, "11", 1); // RepSeq -> ZOH
    diagram.add_line("6", 1, "12", 1); // RepStair -> Abs
    diagram.add_line("12", 1, "13", 1); // Abs -> Math(sqrt)
    diagram.add_line("7", 1, "14", 1); // TF -> RelOp port 1
    diagram.add_line("8", 1, "14", 2); // DTF -> RelOp port 2
    diagram.add_line("14", 1, "15", 1); // RelOp -> LogOp port 1
    diagram.add_line("4", 1, "15", 2); // Pulse -> LogOp port 2
    diagram.add_line("9", 1, "16", 1); // ContPID -> Switch port 1
    diagram.add_line("15", 1, "16", 2); // LogOp -> Switch port 2
    diagram.add_line("10", 1, "16", 3); // DiscPID -> Switch port 3
    diagram.add_line("16", 1, "17", 1); // Switch -> Scope

    let engine = ExecutionEngine::compile(&diagram).expect("compile M3 engine");
    let mut workspace =
        SimWorkspace::new(diagram.blocks.len(), engine.layout.num_continuous_states);
    let continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();
    let config = mlapp_sim::solver::SimulationConfig {
        start_time: 0.0,
        stop_time: 10.0,
        step_size: 0.001,
        solver: mlapp_sim::solver::SolverType::FixedStepEuler,
        ..Default::default()
    };

    // Track allocations over 10,000 steps on this thread
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ACTIVE.with(|t| t.set(true));

    for step_idx in 0..10_000 {
        let t = step_idx as f64 * 0.001;
        engine
            .eval(t, &continuous_states, &discrete_states, &mut workspace)
            .expect("eval M3 blocks");
        engine.update_discrete_at(t, 0.0, &workspace.outputs, &mut discrete_states, &config);
    }

    TRACKING_ACTIVE.with(|t| t.set(false));
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "M3 blocks eval and update_discrete_at must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}

#[test]
fn test_zero_allocations_during_mna_circuit_simulation_step() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut diagram = BlockDiagram::new("AllocTestCircuit");

    diagram.add_block(Block::new("gnd", "GND", BlockType::Ground));
    diagram.add_block(
        Block::new("vs", "Vs", BlockType::DcVoltageSource).with_param("Voltage", "10.0"),
    );
    diagram.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "5.0")
            .with_param("Frequency", "60.0"),
    );
    diagram
        .add_block(Block::new("is", "Is", BlockType::DcCurrentSource).with_param("Current", "1.0"));
    diagram
        .add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "100.0"));
    diagram
        .add_block(Block::new("r2", "R2", BlockType::Resistor).with_param("Resistance", "200.0"));
    diagram.add_block(Block::new("c", "C", BlockType::Capacitor).with_param("Capacitance", "1e-5"));
    diagram.add_block(Block::new("l", "L", BlockType::Inductor).with_param("Inductance", "1e-2"));

    // Vs(p) -> Net 1
    diagram.add_electrical_line("vs", 0, "r1", 0);
    // R1(n) -> Net 2
    diagram.add_electrical_line("r1", 1, "c", 0);
    // C(n) -> Net 3
    diagram.add_electrical_line("c", 1, "l", 0);
    // L(n) -> Net 0
    diagram.add_electrical_line("l", 1, "gnd", 0);
    // Vs(n) -> Net 0
    diagram.add_electrical_line("vs", 1, "gnd", 0);
    // Vac parallel with R2 to Ground
    diagram.add_electrical_line("vac", 0, "r2", 0);
    diagram.add_electrical_line("r2", 1, "gnd", 0);
    diagram.add_electrical_line("vac", 1, "gnd", 0);
    // Is into Net 2
    diagram.add_electrical_line("is", 0, "r1", 1);
    diagram.add_electrical_line("is", 1, "gnd", 0);

    let dt = 1e-4;
    let mut solver = mlapp_sim::electrical::LinearCircuitSolver::compile(&diagram, dt)
        .expect("compile circuit")
        .expect("circuit solver exists");

    solver.init_solve(0.0).expect("init solve");

    // Warm-up step 1 (Backward Euler)
    solver.step(dt, dt, true).expect("first step");
    // Warm-up step 2 (Trapezoidal) and signal query
    solver.step(2.0 * dt, dt, false).expect("second step");
    let _ = solver.branch_voltage("r1");
    let _ = solver.branch_current("r1");
    let _ = solver.node_voltage(1);

    // Track allocations over 10,000 steps of Trapezoidal MNA stepping on this thread
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ACTIVE.with(|t| t.set(true));

    for step_idx in 1..=10_000 {
        let t = (step_idx as f64 + 2.0) * dt;
        solver.step(t, dt, false).expect("mna step");
        let _ = solver.branch_voltage("r1");
        let _ = solver.branch_current("r1");
        let _ = solver.node_voltage(1);
    }

    TRACKING_ACTIVE.with(|t| t.set(false));
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "MNA circuit stepping (LinearCircuitSolver::step) must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}

#[test]
fn test_zero_allocations_during_nonlinear_circuit_simulation_step() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut diagram = BlockDiagram::new("AllocTestNonlinearCircuit");

    diagram.add_block(Block::new("gnd", "GND", BlockType::Ground));
    diagram.add_block(
        Block::new("vac", "Vac", BlockType::AcVoltageSource)
            .with_param("Amplitude", "12.0")
            .with_param("Frequency", "100.0"),
    );
    diagram.add_block(
        Block::new("d1", "D1", BlockType::Diode)
            .with_param("ForwardVoltage", "0.7")
            .with_param("OnResistance", "0.01")
            .with_param("OffResistance", "1e6"),
    );
    diagram.add_block(
        Block::new("sw1", "Switch1", BlockType::IdealSwitch)
            .with_param("Threshold", "0.5")
            .with_param("OnResistance", "0.02")
            .with_param("OffResistance", "1e5"),
    );
    diagram
        .add_block(Block::new("r", "Rload", BlockType::Resistor).with_param("Resistance", "50.0"));
    diagram.add_block(
        Block::new("c", "Cfilter", BlockType::Capacitor).with_param("Capacitance", "1e-5"),
    );

    // Vac(p) -> D1(p) -> Net 1
    diagram.add_electrical_line("vac", 0, "d1", 0);
    // D1(n) -> Switch1(p) -> Net 2
    diagram.add_electrical_line("d1", 1, "sw1", 0);
    // Switch1(n) -> Rload(p) & Cfilter(p) -> Net 3
    diagram.add_electrical_line("sw1", 1, "r", 0);
    diagram.add_electrical_line("r", 0, "c", 0);
    // Grounds
    diagram.add_electrical_line("vac", 1, "gnd", 0);
    diagram.add_electrical_line("r", 1, "gnd", 0);
    diagram.add_electrical_line("c", 1, "gnd", 0);

    let dt = 1e-4;
    let mut solver = mlapp_sim::electrical::LinearCircuitSolver::compile(&diagram, dt)
        .expect("compile nonlinear circuit")
        .expect("circuit solver exists");

    solver.init_solve(0.0).expect("init solve");

    // Warm-up step 1 (Backward Euler)
    solver.set_switch_control("sw1", 1.0);
    solver.step(dt, dt, true).expect("first step");
    // Warm-up step 2 (Trapezoidal)
    solver.step(2.0 * dt, dt, false).expect("second step");
    let _ = solver.branch_voltage("d1");
    let _ = solver.branch_current("d1");
    let _ = solver.element_state("d1");
    let _ = solver.branch_voltage("sw1");
    let _ = solver.branch_current("sw1");
    let _ = solver.element_state("sw1");

    // Track allocations over 10,000 steps of nonlinear switching fixed-point iterations
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ACTIVE.with(|t| t.set(true));

    for step_idx in 1..=10_000 {
        let t = (step_idx as f64 + 2.0) * dt;
        // Alternate switch state every 50 steps
        let ctrl = if (step_idx / 50) % 2 == 0 { 1.0 } else { 0.0 };
        solver.set_switch_control("sw1", ctrl);
        solver.step(t, dt, false).expect("nonlinear step");
        let _ = solver.branch_voltage("d1");
        let _ = solver.branch_current("d1");
        let _ = solver.element_state("d1");
        let _ = solver.branch_voltage("sw1");
        let _ = solver.branch_current("sw1");
        let _ = solver.element_state("sw1");
        let _ = solver.node_voltage(1);
    }

    TRACKING_ACTIVE.with(|t| t.set(false));
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "Nonlinear switching circuit stepping with fixed-point iterations must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}

#[test]
fn test_zero_allocations_during_sensors_and_controlled_sources_stepping() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut diagram = BlockDiagram::new("SensorsSourcesAllocModel");

    diagram.add_block(Block::new("gnd", "GND", BlockType::Ground));
    diagram.add_block(
        Block::new("vdc", "Vdc", BlockType::DcVoltageSource).with_param("Voltage", "10.0"),
    );
    diagram.add_block(Block::new("r1", "R1", BlockType::Resistor).with_param("Resistance", "50.0"));
    diagram.add_block(Block::new("isens", "ISens", BlockType::CurrentSensor));
    diagram.add_block(Block::new("vsens", "VSens", BlockType::VoltageSensor));
    diagram.add_block(
        Block::new("vcvs", "VCVS", BlockType::VoltageControlledVoltageSource)
            .with_param("Gain", "2.0"),
    );
    diagram.add_block(
        Block::new("vccs", "VCCS", BlockType::VoltageControlledCurrentSource)
            .with_param("Transconductance", "0.01"),
    );
    diagram.add_block(
        Block::new("rload", "Rload", BlockType::Resistor).with_param("Resistance", "100.0"),
    );

    // Connections:
    // Vdc+ -> R1 -> ISens+ -> ISens- -> GND
    diagram.add_electrical_line("vdc", 0, "r1", 0);
    diagram.add_electrical_line("r1", 1, "isens", 0);
    diagram.add_electrical_line("isens", 1, "gnd", 0);
    diagram.add_electrical_line("vdc", 1, "gnd", 0);

    // VSens across R1
    diagram.add_electrical_line("vsens", 0, "r1", 0);
    diagram.add_electrical_line("vsens", 1, "r1", 1);

    // VCVS powered across Rload
    diagram.add_electrical_line("vcvs", 0, "rload", 0);
    diagram.add_electrical_line("rload", 1, "gnd", 0);
    diagram.add_electrical_line("vcvs", 1, "gnd", 0);

    // VCCS into Rload
    diagram.add_electrical_line("vccs", 0, "rload", 0);
    diagram.add_electrical_line("vccs", 1, "gnd", 0);

    let dt = 1e-4;
    let mut solver = mlapp_sim::electrical::LinearCircuitSolver::compile(&diagram, dt)
        .expect("compile sensors and sources circuit")
        .expect("circuit solver exists");

    solver.init_solve(0.0).expect("init solve");

    // Warm-up step
    solver.set_vcvs_control("vcvs", 1.5);
    solver.set_vccs_control("vccs", 2.0);
    solver.step(dt, dt, true).expect("first step");
    let _ = solver.sensor_voltage("vsens");
    let _ = solver.sensor_current("isens");

    // Track allocations over 10,000 steps
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ACTIVE.with(|t| t.set(true));

    for step_idx in 1..=10_000 {
        let t = (step_idx as f64 + 1.0) * dt;
        solver.set_vcvs_control("vcvs", 1.0 + (step_idx as f64 * 0.001).sin());
        solver.set_vccs_control("vccs", 2.0 + (step_idx as f64 * 0.001).cos());
        solver.step(t, dt, false).expect("step");
        let _ = solver.sensor_voltage("vsens");
        let _ = solver.sensor_current("isens");
        let _ = solver.branch_current("isens");
        let _ = solver.branch_voltage("vsens");
        let _ = solver.branch_current("vcvs");
        let _ = solver.branch_current("vccs");
    }

    TRACKING_ACTIVE.with(|t| t.set(false));
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "Sensors and controlled sources stepping must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}
