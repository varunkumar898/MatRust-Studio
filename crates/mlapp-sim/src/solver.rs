//! Fixed-step simulation engine for Simulink block diagrams.

use crate::model::{Block, BlockDiagram, BlockType};
use rustlab_math::{DenseArray, StructArray, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimError {
    #[error("Simulation error: {0}")]
    Execution(String),

    #[error("Algebraic loop failed to converge after {0} iterations")]
    AlgebraicLoop(usize),

    #[error("Block not found: {0}")]
    BlockNotFound(String),
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

/// Internal runtime state for a single block during simulation.
struct BlockState {
    continuous_state: f64,
    discrete_state: f64,
    derivative: f64,
    output: f64,
}

/// Runs a fixed-step simulation of the provided block diagram.
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

    let num_steps = ((config.stop_time - config.start_time) / config.step_size).round() as usize;

    // Initialize block states
    let mut states: HashMap<String, BlockState> = HashMap::new();
    for b in &diagram.blocks {
        let init_cond = b
            .get_param("InitialCondition")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        states.insert(
            b.sid.clone(),
            BlockState {
                continuous_state: init_cond,
                discrete_state: init_cond,
                derivative: 0.0,
                output: 0.0,
            },
        );
    }

    let mut tout = Vec::with_capacity(num_steps + 1);
    let mut yout = Vec::with_capacity(num_steps + 1);
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();

    // Identify log sinks (Scope, ToWorkspace, Outport)
    let log_blocks: Vec<&Block> = diagram
        .blocks
        .iter()
        .filter(|b| {
            matches!(
                b.block_type,
                BlockType::Scope | BlockType::ToWorkspace | BlockType::Outport
            )
        })
        .collect();

    for b in &log_blocks {
        signals.insert(b.name.clone(), Vec::with_capacity(num_steps + 1));
    }

    let dt = config.step_size;

    for step in 0..=num_steps {
        let t = config.start_time + (step as f64) * dt;

        // 1. Initial output evaluation from stateful and source blocks
        for b in &diagram.blocks {
            let st = states
                .get_mut(&b.sid)
                .ok_or_else(|| SimError::BlockNotFound(b.sid.clone()))?;
            match b.block_type {
                BlockType::Constant => {
                    let val = b
                        .get_param("Value")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    st.output = val;
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
                    st.output = amp * (freq * t + phase).sin() + bias;
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
                    st.output = if t >= step_t { after } else { before };
                }
                BlockType::Integrator => {
                    st.output = st.continuous_state;
                }
                BlockType::UnitDelay => {
                    st.output = st.discrete_state;
                }
                _ => {}
            }
        }

        // 2. Feedforward algebraic resolution (Gauss-Seidel iterations for feedforward blocks)
        let max_iter = 20;
        let mut converged = false;

        for _iter in 0..max_iter {
            let mut max_change = 0.0f64;

            for b in &diagram.blocks {
                let inputs = get_block_inputs(b, diagram, &states);

                let new_output = match b.block_type {
                    BlockType::Gain => {
                        let gain = b
                            .get_param("Gain")
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(1.0);
                        let u = inputs.first().copied().unwrap_or(0.0);
                        gain * u
                    }
                    BlockType::Sum => {
                        let signs = b.get_param("Inputs").unwrap_or("++");
                        let mut sum = 0.0;
                        let mut sign_chars = signs.chars().filter(|&c| c == '+' || c == '-');
                        for &u in &inputs {
                            let s = sign_chars.next().unwrap_or('+');
                            if s == '-' {
                                sum -= u;
                            } else {
                                sum += u;
                            }
                        }
                        sum
                    }
                    BlockType::Product => {
                        if inputs.is_empty() {
                            0.0
                        } else {
                            inputs.iter().product()
                        }
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
                        let u = inputs.first().copied().unwrap_or(0.0);
                        u.clamp(lower, upper)
                    }
                    BlockType::Scope | BlockType::ToWorkspace | BlockType::Outport => {
                        inputs.first().copied().unwrap_or(0.0)
                    }
                    _ => {
                        // Already computed in step 1
                        states.get(&b.sid).map(|s| s.output).unwrap_or(0.0)
                    }
                };

                if let Some(st) = states.get_mut(&b.sid) {
                    let diff = (new_output - st.output).abs();
                    if diff > max_change {
                        max_change = diff;
                    }
                    st.output = new_output;
                }
            }

            if max_change < 1e-9 {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(SimError::AlgebraicLoop(max_iter));
        }

        // 3. Record logged signals at this time step
        tout.push(t);

        // Find primary output signal (first outport or scope)
        let primary_val = log_blocks
            .first()
            .and_then(|b| states.get(&b.sid).map(|s| s.output))
            .unwrap_or_else(|| {
                // If no log block, pick the last block in diagram
                diagram
                    .blocks
                    .last()
                    .and_then(|b| states.get(&b.sid).map(|s| s.output))
                    .unwrap_or(0.0)
            });
        yout.push(primary_val);

        for b in &log_blocks {
            if let Some(st) = states.get(&b.sid) {
                if let Some(vec) = signals.get_mut(&b.name) {
                    vec.push(st.output);
                }
            }
        }

        // 4. Update states for next step: Integrator and UnitDelay
        for b in &diagram.blocks {
            let inputs = get_block_inputs(b, diagram, &states);
            let u = inputs.first().copied().unwrap_or(0.0);

            if let Some(st) = states.get_mut(&b.sid) {
                match b.block_type {
                    BlockType::Integrator => {
                        st.derivative = u;
                        st.continuous_state += dt * u;
                    }
                    BlockType::UnitDelay => {
                        st.discrete_state = u;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
    })
}

/// Helper to get input signal values to a given block from connected lines.
fn get_block_inputs(
    block: &Block,
    diagram: &BlockDiagram,
    states: &HashMap<String, BlockState>,
) -> Vec<f64> {
    let mut inputs = Vec::new();

    // Find all lines whose destination is this block
    let mut incoming_lines: Vec<_> = diagram
        .lines
        .iter()
        .filter(|l| l.dst.block_sid == block.sid || l.dst.block_sid == block.name)
        .collect();

    // Sort by destination port index
    incoming_lines.sort_by_key(|l| l.dst.port_index);

    for line in incoming_lines {
        let src_val = diagram
            .find_block(&line.src.block_sid)
            .and_then(|src_block| states.get(&src_block.sid).map(|st| st.output))
            .unwrap_or(0.0);
        inputs.push(src_val);
    }

    inputs
}
