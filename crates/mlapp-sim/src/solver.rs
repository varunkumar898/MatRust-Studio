//! Fixed-step simulation engine for Simulink block diagrams.

use crate::model::{Block, BlockDiagram, BlockType};
use rustlab_math::{DenseArray, StructArray, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockRef {
    pub sid: String,
    pub name: String,
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum SimError {
    #[error("Simulation error: {0}")]
    Execution(String),

    #[error("{message}")]
    AlgebraicLoop {
        code: String,
        blocks: Vec<BlockRef>,
        message: String,
    },

    #[error("Block not found: {0}")]
    BlockNotFound(String),
}

impl SimError {
    pub fn algebraic_loop(blocks: Vec<BlockRef>) -> Self {
        let block_list = blocks
            .iter()
            .map(|b| format!("{} (SID: {})", b.name, b.sid))
            .collect::<Vec<_>>()
            .join(", ");
        let message = format!(
            "Algebraic loop detected (E-ALG): Direct-feedthrough dependency cycle detected among blocks: [{}]. The outputs of these blocks instantaneously depend on their own inputs. To fix: insert a Unit Delay or Integrator in the loop to break the instantaneous algebraic feedback.",
            block_list
        );
        SimError::AlgebraicLoop {
            code: "E-ALG".to_string(),
            blocks,
            message,
        }
    }
}

/// Simulation configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfig {
    pub start_time: f64,
    pub stop_time: f64,
    pub step_size: f64,
    pub solver: String,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            stop_time: 10.0,
            step_size: 0.01,
            solver: "FixedStepEuler".to_string(),
        }
    }
}

/// Simulation execution results containing logged time points and signals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationOutput {
    pub tout: Vec<f64>,
    pub yout: Vec<f64>,
    pub signals: HashMap<String, Vec<f64>>,
}

impl SimulationOutput {
    /// Converts the simulation results into a MATLAB struct representation (`simOut.tout`, `simOut.yout`).
    pub fn to_matlab_value(&self) -> Value {
        let mut fields = HashMap::new();

        // tout
        if let Ok(tout_arr) = DenseArray::from_vec(vec![self.tout.len(), 1], self.tout.clone()) {
            fields.insert("tout".to_string(), Value::Double(tout_arr));
        }

        // yout
        if let Ok(yout_arr) = DenseArray::from_vec(vec![self.yout.len(), 1], self.yout.clone()) {
            fields.insert("yout".to_string(), Value::Double(yout_arr));
        }

        // additional signals
        for (name, sig) in &self.signals {
            if let Ok(sig_arr) = DenseArray::from_vec(vec![sig.len(), 1], sig.clone()) {
                fields.insert(name.clone(), Value::Double(sig_arr));
            }
        }

        Value::Struct(StructArray::scalar(fields))
    }
}

/// Runs a fixed-step simulation of the provided block diagram using the execution engine.
pub fn simulate_diagram(
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, SimError> {
    if config.step_size <= 0.0 {
        return Err(SimError::Execution("Step size must be positive".into()));
    }
    if config.stop_time < config.start_time {
        return Err(SimError::Execution(
            "Stop time must be >= start time".into(),
        ));
    }

    // 1. Compile diagram into execution engine (topological order, E-ALG cycle check, pre-parsed blocks)
    let engine = crate::engine::ExecutionEngine::compile(diagram)?;
    let num_steps = ((config.stop_time - config.start_time) / config.step_size).round() as usize;

    let mut continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();

    // 2. Pre-allocate workspace buffers once per run (zero heap allocations in the simulation step loop)
    let mut workspace =
        crate::engine::SimWorkspace::new(diagram.blocks.len(), engine.layout.num_continuous_states);

    let mut tout = Vec::with_capacity(num_steps + 1);
    let mut yout = Vec::with_capacity(num_steps + 1);
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();

    // Identify log sinks (Scope, ToWorkspace, Outport)
    let log_blocks: Vec<(usize, &Block)> = diagram
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            matches!(
                b.block_type,
                BlockType::Scope | BlockType::ToWorkspace | BlockType::Outport
            )
        })
        .collect();

    for (_, b) in &log_blocks {
        signals.insert(b.name.clone(), Vec::with_capacity(num_steps + 1));
    }

    let dt = config.step_size;

    for step in 0..=num_steps {
        let t = config.start_time + (step as f64) * dt;

        // Pure evaluation of block outputs and derivatives
        engine.eval(t, &continuous_states, &discrete_states, &mut workspace);

        // Record logged signals at this time step
        tout.push(t);

        let primary_val = log_blocks
            .first()
            .map(|(idx, _)| workspace.outputs[*idx])
            .unwrap_or_else(|| workspace.outputs.last().copied().unwrap_or(0.0));
        yout.push(primary_val);

        for (idx, b) in &log_blocks {
            if let Some(vec) = signals.get_mut(&b.name) {
                vec.push(workspace.outputs[*idx]);
            }
        }

        // Advance continuous states using fixed-step Euler integration
        for (state, &deriv) in continuous_states.iter_mut().zip(&workspace.derivatives) {
            *state += dt * deriv;
        }

        // Advance discrete states (UnitDelay) once per accepted major step
        engine.update_discrete(&workspace.outputs, &mut discrete_states);
    }

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
    })
}
