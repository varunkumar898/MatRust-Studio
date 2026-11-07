//! Execution engine: direct-feedthrough analysis, Tarjan SCC algebraic loop detection,
//! deterministic topological scheduling, and pure evaluation for block diagrams.

use crate::model::{BlockDiagram, BlockType};
use crate::solver::{BlockRef, SimError};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Returns whether the specified input port of a block type has direct feedthrough to its output.
pub fn feedthrough(
    block_type: &BlockType,
    _parameters: &HashMap<String, String>,
    _input_port: usize,
) -> bool {
    match block_type {
        BlockType::Gain
        | BlockType::Sum
        | BlockType::Product
        | BlockType::Saturation
        | BlockType::Scope
        | BlockType::ToWorkspace
        | BlockType::Outport => true,
        BlockType::Integrator
        | BlockType::UnitDelay
        | BlockType::Constant
        | BlockType::SineWave
        | BlockType::Step
        | BlockType::Inport => false,
        BlockType::Subsystem | BlockType::Unknown(_) => true,
    }
}

/// Compiled block type with pre-parsed parameters for allocation-free execution.
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledKind {
    Constant {
        value: f64,
    },
    SineWave {
        amp: f64,
        freq: f64,
        phase: f64,
        bias: f64,
    },
    Step {
        step_time: f64,
        before: f64,
        after: f64,
    },
    Gain {
        gain: f64,
    },
    Sum {
        signs: Vec<bool>,
    }, // true for '+', false for '-'
    Product,
    Integrator {
        state_idx: usize,
    },
    UnitDelay {
        state_idx: usize,
    },
    Saturation {
        lower: f64,
        upper: f64,
    },
    Scope,
    ToWorkspace,
    Outport,
    Inport,
    Other,
}

/// Pre-compiled block node inside the execution engine.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBlock {
    pub sid: String,
    pub name: String,
    pub kind: CompiledKind,
    pub input_sources: Vec<Option<usize>>, // None = unconnected (evaluates to 0.0)
}

/// State vector layout showing continuous and discrete state variable counts and offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelLayout {
    pub num_continuous_states: usize,
    pub num_discrete_states: usize,
    pub continuous_offsets: Vec<Option<usize>>,
    pub discrete_offsets: Vec<Option<usize>>,
}

/// Reusable workspace buffers for allocation-free simulation steps.
#[derive(Debug, Clone, Default)]
pub struct SimWorkspace {
    pub outputs: Vec<f64>,
    pub derivatives: Vec<f64>,
}

impl SimWorkspace {
    pub fn new(num_blocks: usize, num_continuous_states: usize) -> Self {
        Self {
            outputs: vec![0.0; num_blocks],
            derivatives: vec![0.0; num_continuous_states],
        }
    }
}

/// Execution engine holding compiled blocks, state layout, and deterministic execution schedule.
#[derive(Debug, Clone)]
pub struct ExecutionEngine {
    pub layout: ModelLayout,
    pub schedule: Vec<usize>,
    pub compiled_blocks: Vec<CompiledBlock>,
    pub initial_continuous_states: Vec<f64>,
    pub initial_discrete_states: Vec<f64>,
}

impl ExecutionEngine {
    /// Compiles a block diagram into an execution engine: analyzes direct feedthrough,
    /// performs Tarjan SCC loop detection for E-ALG errors, and schedules blocks topologically.
    pub fn compile(diagram: &BlockDiagram) -> Result<Self, SimError> {
        let num_blocks = diagram.blocks.len();

        // 1. Build SID/name to index lookup
        let mut block_indices: HashMap<String, usize> = HashMap::with_capacity(num_blocks * 2);
        for (i, b) in diagram.blocks.iter().enumerate() {
            block_indices.insert(b.sid.clone(), i);
            block_indices.insert(b.name.clone(), i);
        }

        // 2. Compute state offsets and initial conditions
        let mut continuous_offsets = Vec::with_capacity(num_blocks);
        let mut discrete_offsets = Vec::with_capacity(num_blocks);
        let mut initial_continuous_states = Vec::new();
        let mut initial_discrete_states = Vec::new();

        for b in &diagram.blocks {
            let init_cond = b
                .get_param("InitialCondition")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            match b.block_type {
                BlockType::Integrator => {
                    let idx = initial_continuous_states.len();
                    continuous_offsets.push(Some(idx));
                    initial_continuous_states.push(init_cond);
                    discrete_offsets.push(None);
                }
                BlockType::UnitDelay => {
                    let idx = initial_discrete_states.len();
                    discrete_offsets.push(Some(idx));
                    initial_discrete_states.push(init_cond);
                    continuous_offsets.push(None);
                }
                _ => {
                    continuous_offsets.push(None);
                    discrete_offsets.push(None);
                }
            }
        }

        let layout = ModelLayout {
            num_continuous_states: initial_continuous_states.len(),
            num_discrete_states: initial_discrete_states.len(),
            continuous_offsets,
            discrete_offsets,
        };

        // 3. Compile block configurations and map input wiring
        let mut compiled_blocks = Vec::with_capacity(num_blocks);

        for (i, b) in diagram.blocks.iter().enumerate() {
            let num_inputs = b.input_ports();
            let mut input_sources = vec![None; num_inputs];

            // Resolve incoming wires
            for line in &diagram.lines {
                let matches_dst = line.dst.block_sid == b.sid || line.dst.block_sid == b.name;
                if matches_dst {
                    let port_idx = if line.dst.port_index > 0 {
                        line.dst.port_index - 1
                    } else {
                        0
                    };
                    if let Some(&src_idx) = block_indices.get(&line.src.block_sid) {
                        if port_idx < num_inputs {
                            input_sources[port_idx] = Some(src_idx);
                        } else {
                            input_sources.push(Some(src_idx));
                        }
                    }
                }
            }

            let kind = match b.block_type {
                BlockType::Constant => {
                    let val = b
                        .get_param("Value")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    CompiledKind::Constant { value: val }
                }
                BlockType::SineWave => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let freq = b
                        .get_param("Frequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let phase = b
                        .get_param("Phase")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let bias = b
                        .get_param("Bias")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    CompiledKind::SineWave {
                        amp,
                        freq,
                        phase,
                        bias,
                    }
                }
                BlockType::Step => {
                    let step_t = b
                        .get_param("Time")
                        .or_else(|| b.get_param("StepTime"))
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let before = b
                        .get_param("Before")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let after = b
                        .get_param("After")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    CompiledKind::Step {
                        step_time: step_t,
                        before,
                        after,
                    }
                }
                BlockType::Gain => {
                    let gain = b
                        .get_param("Gain")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    CompiledKind::Gain { gain }
                }
                BlockType::Sum => {
                    let signs_str = b.get_param("Inputs").unwrap_or("++");
                    let mut signs = Vec::new();
                    for c in signs_str.chars() {
                        if c == '+' {
                            signs.push(true);
                        } else if c == '-' {
                            signs.push(false);
                        }
                    }
                    if signs.is_empty() {
                        signs = vec![true, true];
                    }
                    CompiledKind::Sum { signs }
                }
                BlockType::Product => CompiledKind::Product,
                BlockType::Integrator => {
                    let state_idx = layout.continuous_offsets[i].unwrap_or(0);
                    CompiledKind::Integrator { state_idx }
                }
                BlockType::UnitDelay => {
                    let state_idx = layout.discrete_offsets[i].unwrap_or(0);
                    CompiledKind::UnitDelay { state_idx }
                }
                BlockType::Saturation => {
                    let lower = b
                        .get_param("LowerLimit")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(-1.0);
                    let upper = b
                        .get_param("UpperLimit")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    CompiledKind::Saturation { lower, upper }
                }
                BlockType::Scope => CompiledKind::Scope,
                BlockType::ToWorkspace => CompiledKind::ToWorkspace,
                BlockType::Outport => CompiledKind::Outport,
                BlockType::Inport => CompiledKind::Inport,
                _ => CompiledKind::Other,
            };

            compiled_blocks.push(CompiledBlock {
                sid: b.sid.clone(),
                name: b.name.clone(),
                kind,
                input_sources,
            });
        }

        // 4. Build direct-feedthrough adjacency graph
        // An edge u -> v exists if u feeds into an input port of v that has direct feedthrough.
        let mut df_adj: Vec<Vec<usize>> = vec![Vec::new(); num_blocks];
        let mut df_in_degree: Vec<usize> = vec![0; num_blocks];
        let mut df_predecessors: Vec<HashSet<usize>> = vec![HashSet::new(); num_blocks];

        for (v, b) in diagram.blocks.iter().enumerate() {
            for (port_idx, &src_opt) in compiled_blocks[v].input_sources.iter().enumerate() {
                if let Some(u) = src_opt {
                    if feedthrough(&b.block_type, &b.parameters, port_idx) {
                        df_adj[u].push(v);
                        if df_predecessors[v].insert(u) {
                            df_in_degree[v] += 1;
                        }
                    }
                }
            }
        }

        // 5. Tarjan SCC detection on the direct-feedthrough graph
        let sccs = run_tarjan_scc(num_blocks, &df_adj);
        let mut loop_blocks = Vec::new();

        for scc in &sccs {
            let has_cycle = if scc.len() > 1 {
                true
            } else if scc.len() == 1 {
                let u = scc[0];
                df_adj[u].contains(&u)
            } else {
                false
            };

            if has_cycle {
                for &idx in scc {
                    loop_blocks.push(BlockRef {
                        sid: diagram.blocks[idx].sid.clone(),
                        name: diagram.blocks[idx].name.clone(),
                    });
                }
            }
        }

        if !loop_blocks.is_empty() {
            // Sort deterministically by diagram block order
            loop_blocks.sort_by(|a, b| {
                let pos_a = diagram
                    .blocks
                    .iter()
                    .position(|blk| blk.sid == a.sid)
                    .unwrap_or(0);
                let pos_b = diagram
                    .blocks
                    .iter()
                    .position(|blk| blk.sid == b.sid)
                    .unwrap_or(0);
                pos_a.cmp(&pos_b)
            });
            loop_blocks.dedup();
            return Err(SimError::algebraic_loop(loop_blocks));
        }

        // 6. Stable Topological Sort (Kahn's algorithm with min-heap tie-breaking)
        let mut ready_heap = BinaryHeap::new();
        for (i, &deg) in df_in_degree.iter().enumerate() {
            if deg == 0 {
                ready_heap.push(Reverse(i));
            }
        }

        let mut schedule = Vec::with_capacity(num_blocks);
        while let Some(Reverse(u)) = ready_heap.pop() {
            schedule.push(u);
            for &v in &df_adj[u] {
                if df_predecessors[v].remove(&u) {
                    df_in_degree[v] -= 1;
                    if df_in_degree[v] == 0 {
                        ready_heap.push(Reverse(v));
                    }
                }
            }
        }

        Ok(Self {
            layout,
            schedule,
            compiled_blocks,
            initial_continuous_states,
            initial_discrete_states,
        })
    }

    /// Pure evaluation function: computes algebraic outputs and continuous derivatives
    /// at time t without mutating engine or solver state.
    pub fn eval(
        &self,
        t: f64,
        continuous_states: &[f64],
        discrete_states: &[f64],
        workspace: &mut SimWorkspace,
    ) {
        let outputs = &mut workspace.outputs;

        // Evaluate all blocks in precomputed topological schedule order
        for &idx in &self.schedule {
            let b = &self.compiled_blocks[idx];
            match &b.kind {
                CompiledKind::Constant { value } => {
                    outputs[idx] = *value;
                }
                CompiledKind::SineWave {
                    amp,
                    freq,
                    phase,
                    bias,
                } => {
                    outputs[idx] = amp * (freq * t + phase).sin() + bias;
                }
                CompiledKind::Step {
                    step_time,
                    before,
                    after,
                } => {
                    outputs[idx] = if t >= *step_time { *after } else { *before };
                }
                CompiledKind::Integrator { state_idx } => {
                    outputs[idx] = continuous_states[*state_idx];
                }
                CompiledKind::UnitDelay { state_idx } => {
                    outputs[idx] = discrete_states[*state_idx];
                }
                CompiledKind::Gain { gain } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[idx] = gain * u;
                }
                CompiledKind::Sum { signs } => {
                    let mut sum = 0.0;
                    for (port_idx, &is_positive) in signs.iter().enumerate() {
                        let u = b
                            .input_sources
                            .get(port_idx)
                            .and_then(|opt| *opt)
                            .map(|src| outputs[src])
                            .unwrap_or(0.0);
                        if is_positive {
                            sum += u;
                        } else {
                            sum -= u;
                        }
                    }
                    outputs[idx] = sum;
                }
                CompiledKind::Product => {
                    let mut prod = 1.0;
                    let mut has_input = false;
                    for opt in &b.input_sources {
                        let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                        prod *= u;
                        has_input = true;
                    }
                    outputs[idx] = if has_input { prod } else { 0.0 };
                }
                CompiledKind::Saturation { lower, upper } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[idx] = u.clamp(*lower, *upper);
                }
                CompiledKind::Scope | CompiledKind::ToWorkspace | CompiledKind::Outport => {
                    outputs[idx] = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                }
                _ => {
                    outputs[idx] = 0.0;
                }
            }
        }

        // Evaluate continuous state derivatives
        let derivatives = &mut workspace.derivatives;
        for b in &self.compiled_blocks {
            if let CompiledKind::Integrator { state_idx } = b.kind {
                let u = b
                    .input_sources
                    .first()
                    .and_then(|opt| *opt)
                    .map(|src| outputs[src])
                    .unwrap_or(0.0);
                derivatives[state_idx] = u;
            }
        }
    }

    /// Discrete state update: updates UnitDelay states from current outputs.
    /// Called once per accepted major simulation step.
    pub fn update_discrete(&self, outputs: &[f64], discrete_states: &mut [f64]) {
        for b in &self.compiled_blocks {
            if let CompiledKind::UnitDelay { state_idx } = b.kind {
                let u = b
                    .input_sources
                    .first()
                    .and_then(|opt| *opt)
                    .map(|src| outputs[src])
                    .unwrap_or(0.0);
                discrete_states[state_idx] = u;
            }
        }
    }
}

/// Runs Tarjan's strongly connected components algorithm on an adjacency graph.
fn run_tarjan_scc(num_nodes: usize, adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct TarjanState<'a> {
        adj: &'a [Vec<usize>],
        index: usize,
        indices: Vec<Option<usize>>,
        lowlink: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        sccs: Vec<Vec<usize>>,
    }

    fn strongconnect(u: usize, state: &mut TarjanState) {
        state.indices[u] = Some(state.index);
        state.lowlink[u] = state.index;
        state.index += 1;
        state.stack.push(u);
        state.on_stack[u] = true;

        for &v in &state.adj[u] {
            if state.indices[v].is_none() {
                strongconnect(v, state);
                state.lowlink[u] = state.lowlink[u].min(state.lowlink[v]);
            } else if state.on_stack[v] {
                state.lowlink[u] = state.lowlink[u].min(state.indices[v].unwrap_or(0));
            }
        }

        if state.lowlink[u] == state.indices[u].unwrap_or(0) {
            let mut scc = Vec::new();
            while let Some(w) = state.stack.pop() {
                state.on_stack[w] = false;
                scc.push(w);
                if w == u {
                    break;
                }
            }
            state.sccs.push(scc);
        }
    }

    let mut state = TarjanState {
        adj,
        index: 0,
        indices: vec![None; num_nodes],
        lowlink: vec![0; num_nodes],
        on_stack: vec![false; num_nodes],
        stack: Vec::new(),
        sccs: Vec::new(),
    };

    for i in 0..num_nodes {
        if state.indices[i].is_none() {
            strongconnect(i, &mut state);
        }
    }

    state.sccs
}
