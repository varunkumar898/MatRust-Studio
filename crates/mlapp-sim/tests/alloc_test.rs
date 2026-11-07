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
        engine.eval(t, &continuous_states, &discrete_states, &mut workspace);
        engine.update_discrete(&workspace.outputs, &mut discrete_states);
    }

    ALLOC_TRACKING_ACTIVE.store(false, Ordering::SeqCst);
    let total_allocs = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(
        total_allocs, 0,
        "SimWorkspace eval and update_discrete must execute with ZERO heap allocations; observed {total_allocs} allocations"
    );
}
