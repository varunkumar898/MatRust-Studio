use mlapp_sim::engine::SimWorkspace;
use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::ExecutionEngine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct CountingAlloc;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_TRACKING_ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOC_TRACKING_ACTIVE.load(Ordering::Relaxed) {
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

    // Start tracking allocations
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    ALLOC_TRACKING_ACTIVE.store(true, Ordering::SeqCst);

    for step_idx in 0..10_000 {
        let t = step_idx as f64 * 0.001;
        engine
            .eval(t, &continuous_states, &discrete_states, &mut workspace)
            .expect("eval");
        engine.update_discrete(&workspace.outputs, &mut discrete_states);
    }

    ALLOC_TRACKING_ACTIVE.store(false, Ordering::SeqCst);
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "SimWorkspace eval and update_discrete must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}

#[test]
fn test_zero_allocations_m3_blocks_during_eval_and_update_discrete() {
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

    // Track allocations over 10,000 steps
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    ALLOC_TRACKING_ACTIVE.store(true, Ordering::SeqCst);

    for step_idx in 0..10_000 {
        let t = step_idx as f64 * 0.001;
        engine
            .eval(t, &continuous_states, &discrete_states, &mut workspace)
            .expect("eval M3 blocks");
        engine.update_discrete_at(t, 0.0, &workspace.outputs, &mut discrete_states, &config);
    }

    ALLOC_TRACKING_ACTIVE.store(false, Ordering::SeqCst);
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "M3 blocks eval and update_discrete_at must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}
