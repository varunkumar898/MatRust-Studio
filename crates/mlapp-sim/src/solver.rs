//! Solvers and execution drivers for Simulink block diagrams.
//!
//! Supports:
//! - Fixed-step Euler (ode1)
//! - Fixed-step Runge-Kutta 4th order (ode4)
//! - Variable-step Dormand-Prince 5(4) with FSAL (ode45)

use crate::engine::{ExecutionEngine, SimWorkspace};
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

    #[error("{message}")]
    StepSize { code: String, message: String },

    #[error("{message}")]
    NonFinite {
        code: String,
        block: Option<BlockRef>,
        message: String,
    },

    #[error("{message}")]
    SampleTime {
        code: String,
        block: BlockRef,
        message: String,
    },

    #[error("{message}")]
    Parameter { code: String, message: String },

    #[error("{message}")]
    Unsupported { code: String, message: String },

    #[error("{message}")]
    Singular { code: String, message: String },

    #[error("{message}")]
    Solver { code: String, message: String },

    #[error("{message}")]
    Chatter { code: String, message: String },

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

    pub fn step_size_limit(min_step: f64, t: f64) -> Self {
        let message = format!(
            "Step size error (E-STEP): Solver reached minimum step size ({min_step:e}) at t = {t} without meeting error tolerances. Why: The system equations may be stiff, experiencing a discontinuity, or approaching a singularity. How to fix: Relax AbsTol/RelTol, decrease MinStep, or inspect model dynamics for rapid state transitions."
        );
        SimError::StepSize {
            code: "E-STEP".to_string(),
            message,
        }
    }

    pub fn max_steps_exceeded(max_steps: usize, t: f64) -> Self {
        let message = format!(
            "Step size error (E-STEP): Maximum simulation step count ({max_steps}) exceeded at t = {t}. Why: Step sizes may be extremely small or stop time is excessively large. How to fix: Increase MaxSteps, relax error tolerances, or adjust MinStep."
        );
        SimError::StepSize {
            code: "E-STEP".to_string(),
            message,
        }
    }

    pub fn non_finite(block: Option<BlockRef>, val: f64, t: f64, context: &str) -> Self {
        let (blk_name, sid_str) = match &block {
            Some(b) => (b.name.as_str(), format!(" (SID: {})", b.sid)),
            None => ("Unknown", String::new()),
        };
        let message = format!(
            "Non-finite value encountered (E-NAN): Non-finite {context} ({val}) encountered at t = {t} in block '{blk_name}'{sid_str}. Why: Division by zero, numerical overflow, or unstable feedback occurred. How to fix: Check input signals, parameters, and saturation bounds to prevent arithmetic overflow."
        );
        SimError::NonFinite {
            code: "E-NAN".to_string(),
            block,
            message,
        }
    }

    pub fn sample_time_missing(block: BlockRef) -> Self {
        let message = format!(
            "Sample time error (E-SAMPLE): Discrete block '{}' (SID: {}) is missing parameter 'SampleTime' under variable-step solver. Why: Variable-step solvers cannot infer discrete block update rates without an explicit sample period. How to fix: Set parameter 'SampleTime' on block '{}' (e.g. '0.01').",
            block.name, block.sid, block.name
        );
        SimError::SampleTime {
            code: "E-SAMPLE".to_string(),
            block,
            message,
        }
    }

    pub fn sample_time_invalid(block: BlockRef, val: f64) -> Self {
        let message = format!(
            "Sample time error (E-SAMPLE): Discrete block '{}' (SID: {}) has invalid SampleTime '{val}'. Why: Sample time must be a finite positive number of seconds. How to fix: Set parameter 'SampleTime' to a positive number (e.g. '0.01').",
            block.name, block.sid
        );
        SimError::SampleTime {
            code: "E-SAMPLE".to_string(),
            block,
            message,
        }
    }

    pub fn sample_time_not_multiple(block: BlockRef, ts: f64, dt: f64) -> Self {
        let nearest_k = ((ts / dt).round() as i64).max(1);
        let suggested = (nearest_k as f64) * dt;
        let message = format!(
            "Sample time error (E-SAMPLE): Discrete block '{}' (SID: {}) has SampleTime = {ts}, which is not a positive integer multiple of step size dt = {dt}. Why: Fixed-step execution requires discrete sample times to be exact integer multiples of the fundamental step size dt. How to fix: Adjust SampleTime to be k * dt (e.g. {suggested}).",
            block.name, block.sid
        );
        SimError::SampleTime {
            code: "E-SAMPLE".to_string(),
            block,
            message,
        }
    }

    pub fn parameter(msg: impl Into<String>) -> Self {
        let detail = msg.into();
        let message = format!("Invalid simulation parameters (E-PARAM): {detail}");
        SimError::Parameter {
            code: "E-PARAM".to_string(),
            message,
        }
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        let detail = msg.into();
        let message = format!("Unsupported simulation capability (E-UNSUPPORTED): {detail}");
        SimError::Unsupported {
            code: "E-UNSUPPORTED".to_string(),
            message,
        }
    }

    pub fn singular(msg: impl Into<String>) -> Self {
        let detail = msg.into();
        let message = format!("Singular electrical circuit matrix (E-SINGULAR): {detail}");
        SimError::Singular {
            code: "E-SINGULAR".to_string(),
            message,
        }
    }

    pub fn solver(msg: impl Into<String>) -> Self {
        let detail = msg.into();
        let message = format!("Unsupported solver for electrical simulation (E-SOLVER): {detail}");
        SimError::Solver {
            code: "E-SOLVER".to_string(),
            message,
        }
    }

    pub fn chatter(msg: impl Into<String>) -> Self {
        let detail = msg.into();
        let message = format!("Switch chattering or limit cycle detected (E-CHATTER): {detail}");
        SimError::Chatter {
            code: "E-CHATTER".to_string(),
            message,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            SimError::Execution(_) => "E-EXEC",
            SimError::AlgebraicLoop { code, .. } => code.as_str(),
            SimError::StepSize { code, .. } => code.as_str(),
            SimError::NonFinite { code, .. } => code.as_str(),
            SimError::SampleTime { code, .. } => code.as_str(),
            SimError::Parameter { code, .. } => code.as_str(),
            SimError::Unsupported { code, .. } => code.as_str(),
            SimError::Singular { code, .. } => code.as_str(),
            SimError::Solver { code, .. } => code.as_str(),
            SimError::Chatter { code, .. } => code.as_str(),
            SimError::BlockNotFound(_) => "E-PARAM",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            SimError::Execution(msg) => msg.as_str(),
            SimError::AlgebraicLoop { message, .. } => message.as_str(),
            SimError::StepSize { message, .. } => message.as_str(),
            SimError::NonFinite { message, .. } => message.as_str(),
            SimError::SampleTime { message, .. } => message.as_str(),
            SimError::Parameter { message, .. } => message.as_str(),
            SimError::Unsupported { message, .. } => message.as_str(),
            SimError::Singular { message, .. } => message.as_str(),
            SimError::Solver { message, .. } => message.as_str(),
            SimError::Chatter { message, .. } => message.as_str(),
            SimError::BlockNotFound(msg) => msg.as_str(),
        }
    }
}

/// Supported numerical integration solvers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolverType {
    FixedStepEuler,
    FixedStepRk4,
    VariableStepDormandPrince,
}

impl Serialize for SolverType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SolverType::FixedStepEuler => serializer.serialize_str("FixedStepEuler"),
            SolverType::FixedStepRk4 => serializer.serialize_str("FixedStepRk4"),
            SolverType::VariableStepDormandPrince => {
                serializer.serialize_str("VariableStepDormandPrince")
            }
        }
    }
}

impl<'de> Deserialize<'de> for SolverType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SolverTypeVisitor;
        impl<'de> serde::de::Visitor<'de> for SolverTypeVisitor {
            type Value = SolverType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "a supported solver name ('FixedStepEuler', 'FixedStepRk4', 'VariableStepDormandPrince', 'ode1', 'ode4', 'ode45')",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<SolverType, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "FixedStepEuler" | "ode1" | "Euler" => Ok(SolverType::FixedStepEuler),
                    "FixedStepRk4" | "ode4" | "RK4" => Ok(SolverType::FixedStepRk4),
                    "VariableStepDormandPrince" | "ode45" | "DormandPrince" | "DormandPrince5" => {
                        Ok(SolverType::VariableStepDormandPrince)
                    }
                    other => Err(serde::de::Error::custom(format!(
                        "Unsupported solver '{other}'. Supported solvers are: 'FixedStepEuler' (ode1), 'FixedStepRk4' (ode4), 'VariableStepDormandPrince' (ode45)"
                    ))),
                }
            }
        }
        deserializer.deserialize_str(SolverTypeVisitor)
    }
}

fn default_start_time() -> f64 {
    0.0
}
fn default_stop_time() -> f64 {
    10.0
}
fn default_step_size() -> f64 {
    0.01
}
fn default_solver() -> SolverType {
    SolverType::FixedStepEuler
}
fn default_abs_tol() -> f64 {
    1e-6
}
fn default_rel_tol() -> f64 {
    1e-3
}
fn default_max_steps() -> usize {
    1_000_000
}

/// Simulation configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfig {
    #[serde(default = "default_start_time")]
    pub start_time: f64,
    #[serde(default = "default_stop_time")]
    pub stop_time: f64,
    #[serde(default = "default_step_size")]
    pub step_size: f64,
    #[serde(default = "default_solver")]
    pub solver: SolverType,
    #[serde(default = "default_abs_tol")]
    pub abs_tol: f64,
    #[serde(default = "default_rel_tol")]
    pub rel_tol: f64,
    #[serde(default)]
    pub max_step: Option<f64>,
    #[serde(default)]
    pub min_step: Option<f64>,
    #[serde(default)]
    pub initial_step: Option<f64>,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default)]
    pub enable_fsal: Option<bool>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            stop_time: 10.0,
            step_size: 0.01,
            solver: SolverType::FixedStepEuler,
            abs_tol: default_abs_tol(),
            rel_tol: default_rel_tol(),
            max_step: None,
            min_step: None,
            initial_step: None,
            max_steps: default_max_steps(),
            enable_fsal: None,
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), SimError> {
        if !self.start_time.is_finite() {
            return Err(SimError::parameter(format!(
                "Start time ({}) must be a finite number. Why: Simulation bounds must be well-defined. How to fix: Set start time to a finite value.",
                self.start_time
            )));
        }
        if !self.stop_time.is_finite() {
            return Err(SimError::parameter(format!(
                "Stop time ({}) must be a finite number. Why: Simulation bounds must be well-defined. How to fix: Set stop time to a finite value.",
                self.stop_time
            )));
        }
        if self.stop_time <= self.start_time {
            return Err(SimError::parameter(format!(
                "Stop time ({}) must be strictly greater than start time ({}). Why: Simulation time must advance forward. How to fix: Increase stop time or decrease start time.",
                self.stop_time, self.start_time
            )));
        }
        if self.step_size <= 0.0 || !self.step_size.is_finite() {
            return Err(SimError::parameter(format!(
                "Step size dt ({}) must be positive and finite. Why: Numerical stepping requires a positive increment. How to fix: Set step size to a positive number (default: 0.01).",
                self.step_size
            )));
        }
        if self.abs_tol <= 0.0 || !self.abs_tol.is_finite() {
            return Err(SimError::parameter(format!(
                "Absolute tolerance AbsTol ({}) must be strictly positive and finite. Why: Error bounds define numerical precision. How to fix: Set AbsTol to a positive value (default: 1e-6).",
                self.abs_tol
            )));
        }
        if self.rel_tol < 1e-13 || !self.rel_tol.is_finite() {
            return Err(SimError::parameter(format!(
                "Relative tolerance RelTol ({}) must be >= 1e-13. Why: Floating-point precision prohibits tolerances below machine epsilon. How to fix: Set RelTol >= 1e-13 (default: 1e-3).",
                self.rel_tol
            )));
        }
        if let (Some(min), Some(max)) = (self.min_step, self.max_step) {
            if min >= max {
                return Err(SimError::parameter(format!(
                    "MinStep ({min}) must be strictly less than MaxStep ({max}). Why: The adaptive step size must have a valid non-empty interval. How to fix: Set MinStep < MaxStep.",
                )));
            }
        }
        if let Some(init) = self.initial_step {
            if init <= 0.0 || !init.is_finite() {
                return Err(SimError::parameter(format!(
                    "InitialStep ({init}) must be positive and finite. Why: Step size must be non-zero. How to fix: Set InitialStep to a positive value.",
                )));
            }
            if let Some(min) = self.min_step {
                if init < min {
                    return Err(SimError::parameter(format!(
                        "InitialStep ({init}) must be >= MinStep ({min}). Why: Initial step cannot violate minimum step bound. How to fix: Set InitialStep >= MinStep.",
                    )));
                }
            }
            if let Some(max) = self.max_step {
                if init > max {
                    return Err(SimError::parameter(format!(
                        "InitialStep ({init}) must be <= MaxStep ({max}). Why: Initial step cannot exceed maximum step bound. How to fix: Set InitialStep <= MaxStep.",
                    )));
                }
            }
        }
        if self.max_steps == 0 {
            return Err(SimError::parameter(
                "MaxSteps must be greater than zero. Why: Simulation requires a positive iteration limit. How to fix: Set MaxSteps to a value >= 1 (default: 1,000,000).",
            ));
        }
        Ok(())
    }
}

/// Simulation execution results containing logged time points and signals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationOutput {
    pub tout: Vec<f64>,
    pub yout: Vec<f64>,
    pub signals: HashMap<String, Vec<f64>>,
    #[serde(default)]
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
}

impl SimulationOutput {
    /// Converts the simulation results into a MATLAB struct representation (`simOut.tout`, `simOut.yout`).
    pub fn to_matlab_value(&self) -> Value {
        let mut fields = HashMap::new();

        if let Ok(tout_arr) = DenseArray::from_vec(vec![self.tout.len(), 1], self.tout.clone()) {
            fields.insert("tout".to_string(), Value::Double(tout_arr));
        }

        if let Ok(yout_arr) = DenseArray::from_vec(vec![self.yout.len(), 1], self.yout.clone()) {
            fields.insert("yout".to_string(), Value::Double(yout_arr));
        }

        for (name, sig) in &self.signals {
            if let Ok(sig_arr) = DenseArray::from_vec(vec![sig.len(), 1], sig.clone()) {
                fields.insert(name.clone(), Value::Double(sig_arr));
            }
        }

        Value::Struct(StructArray::scalar(fields))
    }
}

/// Butcher tableau coefficients for Dormand-Prince 5(4) (DOPRI5 / MATLAB ode45).
pub struct DormandPrinceTableau;

impl DormandPrinceTableau {
    pub const C: [f64; 7] = [0.0, 1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0, 8.0 / 9.0, 1.0, 1.0];

    pub const A21: f64 = 1.0 / 5.0;

    pub const A31: f64 = 3.0 / 40.0;
    pub const A32: f64 = 9.0 / 40.0;

    pub const A41: f64 = 44.0 / 45.0;
    pub const A42: f64 = -56.0 / 15.0;
    pub const A43: f64 = 32.0 / 9.0;

    pub const A51: f64 = 19372.0 / 6561.0;
    pub const A52: f64 = -25360.0 / 2187.0;
    pub const A53: f64 = 64448.0 / 6561.0;
    pub const A54: f64 = -212.0 / 729.0;

    pub const A61: f64 = 9017.0 / 3168.0;
    pub const A62: f64 = -355.0 / 33.0;
    pub const A63: f64 = 46732.0 / 5247.0;
    pub const A64: f64 = 49.0 / 176.0;
    pub const A65: f64 = -5103.0 / 18656.0;

    pub const A71: f64 = 35.0 / 384.0;
    pub const A72: f64 = 0.0;
    pub const A73: f64 = 500.0 / 1113.0;
    pub const A74: f64 = 125.0 / 192.0;
    pub const A75: f64 = -2187.0 / 6784.0;
    pub const A76: f64 = 11.0 / 84.0;

    /// 5th-order weights b
    pub const B: [f64; 7] = [
        35.0 / 384.0,
        0.0,
        500.0 / 1113.0,
        125.0 / 192.0,
        -2187.0 / 6784.0,
        11.0 / 84.0,
        0.0,
    ];

    /// 4th-order embedded weights b_hat
    pub const B_HAT: [f64; 7] = [
        5179.0 / 57600.0,
        0.0,
        7571.0 / 16695.0,
        393.0 / 640.0,
        -92097.0 / 339200.0,
        187.0 / 2100.0,
        1.0 / 40.0,
    ];

    /// Error weights E = B - B_HAT
    pub const E: [f64; 7] = [
        71.0 / 57600.0,
        0.0,
        -71.0 / 16695.0,
        71.0 / 1920.0,
        -17253.0 / 339200.0,
        22.0 / 525.0,
        -1.0 / 40.0,
    ];
}

/// Runs a simulation of the provided block diagram using the requested solver.
pub fn simulate_diagram(
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, SimError> {
    config.validate()?;

    let has_electrical = diagram.blocks.iter().any(|b| b.block_type.is_electrical());
    if has_electrical && config.solver == SolverType::VariableStepDormandPrince {
        return Err(SimError::solver(
            "Variable-step Dormand-Prince (ode45) is not supported for electrical circuits in Milestone M5. Why: Companion-model MNA requires a fixed step size h to maintain constant-matrix factorization and avoid companion conductance restructuring. How to fix: Select a fixed-step solver such as FixedStepEuler (ode1) or FixedStepRk4 (ode4) in Simulation Parameters."
        ));
    }

    let engine = ExecutionEngine::compile(diagram)?;
    engine.validate_sample_times(config)?;

    match config.solver {
        SolverType::FixedStepEuler => simulate_fixed_step_euler(&engine, diagram, config),
        SolverType::FixedStepRk4 => simulate_fixed_step_rk4(&engine, diagram, config),
        SolverType::VariableStepDormandPrince => simulate_dormand_prince(&engine, diagram, config),
    }
}

/// Fixed-step Euler simulation. Preserves exact time arithmetic for bit-identical backwards compatibility.
fn simulate_fixed_step_euler(
    engine: &ExecutionEngine,
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, SimError> {
    let dt = config.step_size;
    let num_steps = ((config.stop_time - config.start_time) / dt).round() as usize;

    let mut circuit_solver = crate::electrical::LinearCircuitSolver::compile(diagram, dt)?;
    let electrical_blocks: Vec<(usize, &Block)> = diagram
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.block_type.is_electrical())
        .collect();

    let mut continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();

    let mut workspace =
        SimWorkspace::new(diagram.blocks.len(), engine.layout.num_continuous_states);

    let mut tout = Vec::with_capacity(num_steps + 1);
    let mut yout = Vec::with_capacity(num_steps + 1);
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();

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

    if let Some(ref c) = circuit_solver {
        for (_, b) in &electrical_blocks {
            signals.insert(b.name.clone(), Vec::with_capacity(num_steps + 1));
            signals.insert(format!("{}.v", b.name), Vec::with_capacity(num_steps + 1));
            signals.insert(format!("{}.i", b.name), Vec::with_capacity(num_steps + 1));
            if matches!(b.block_type, BlockType::Diode | BlockType::IdealSwitch) {
                signals.insert(
                    format!("{}.state", b.name),
                    Vec::with_capacity(num_steps + 1),
                );
            }
        }
        for net in 1..=c.num_nodes {
            signals.insert(format!("Net{net}"), Vec::with_capacity(num_steps + 1));
        }
    }

    if let Some(ref mut c) = circuit_solver {
        engine.eval(
            config.start_time,
            &continuous_states,
            &discrete_states,
            &mut workspace,
        )?;
        for (_, b) in &electrical_blocks {
            if let Some(src_idx) = c.get_switch_driver_idx(&b.sid) {
                c.set_switch_control(&b.sid, workspace.outputs[src_idx]);
            }
        }
        c.init_solve(config.start_time)?;
        for &(idx, b) in &electrical_blocks {
            if let Some(v) = c.branch_voltage(&b.sid) {
                workspace.outputs[idx] = v;
            }
        }
    }

    for step in 0..=num_steps {
        // Exact time arithmetic retained for bit-identical fixture parity
        let t = config.start_time + (step as f64) * dt;

        engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;

        tout.push(t);

        let primary_val = log_blocks
            .first()
            .map(|(idx, _)| workspace.outputs[*idx])
            .unwrap_or_else(|| {
                if let Some(&(idx, _)) = electrical_blocks.last() {
                    workspace.outputs[idx]
                } else {
                    workspace.outputs.last().copied().unwrap_or(0.0)
                }
            });
        yout.push(primary_val);

        for (idx, b) in &log_blocks {
            if let Some(vec) = signals.get_mut(&b.name) {
                vec.push(workspace.outputs[*idx]);
            }
        }

        if let Some(ref c) = circuit_solver {
            for (_, b) in &electrical_blocks {
                let v = c.branch_voltage(&b.sid).unwrap_or(0.0);
                let i = c.branch_current(&b.sid).unwrap_or(0.0);
                if let Some(vec) = signals.get_mut(&b.name) {
                    vec.push(v);
                }
                if let Some(vec) = signals.get_mut(&format!("{}.v", b.name)) {
                    vec.push(v);
                }
                if let Some(vec) = signals.get_mut(&format!("{}.i", b.name)) {
                    vec.push(i);
                }
                if let Some(st) = c.element_state(&b.sid) {
                    if let Some(vec) = signals.get_mut(&format!("{}.state", b.name)) {
                        vec.push(st);
                    }
                }
            }
            for net in 1..=c.num_nodes {
                let vn = c.node_voltage(net);
                if let Some(vec) = signals.get_mut(&format!("Net{net}")) {
                    vec.push(vn);
                }
            }
        }

        if step < num_steps {
            // Advance continuous states using Euler integration
            for (state, &deriv) in continuous_states.iter_mut().zip(&workspace.derivatives) {
                *state += dt * deriv;
            }

            // Advance circuit solver to t + dt
            if let Some(ref mut c) = circuit_solver {
                for (_, b) in &electrical_blocks {
                    if let Some(src_idx) = c.get_switch_driver_idx(&b.sid) {
                        c.set_switch_control(&b.sid, workspace.outputs[src_idx]);
                    }
                }
                let t_next = config.start_time + ((step + 1) as f64) * dt;
                c.step(t_next, dt, step == 0)?;
                for &(idx, b) in &electrical_blocks {
                    if let Some(v) = c.branch_voltage(&b.sid) {
                        workspace.outputs[idx] = v;
                    }
                }
            }

            // Discrete state update with sample time check
            engine.update_discrete_at(
                t,
                config.start_time,
                &workspace.outputs,
                &mut discrete_states,
                config,
            );
        }
    }

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
        diagnostics: circuit_solver.map(|c| c.diagnostics).unwrap_or_default(),
    })
}

/// Fixed-step classical 4th-order Runge-Kutta (RK4) simulation.
fn simulate_fixed_step_rk4(
    engine: &ExecutionEngine,
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, SimError> {
    let dt = config.step_size;
    let num_steps = ((config.stop_time - config.start_time) / dt).round() as usize;
    let num_states = engine.layout.num_continuous_states;

    let mut circuit_solver = crate::electrical::LinearCircuitSolver::compile(diagram, dt)?;
    let electrical_blocks: Vec<(usize, &Block)> = diagram
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.block_type.is_electrical())
        .collect();

    let mut continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();

    let mut workspace = SimWorkspace::new(diagram.blocks.len(), num_states);

    let mut tout = Vec::with_capacity(num_steps + 1);
    let mut yout = Vec::with_capacity(num_steps + 1);
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();

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

    if let Some(ref c) = circuit_solver {
        for (_, b) in &electrical_blocks {
            signals.insert(b.name.clone(), Vec::with_capacity(num_steps + 1));
            signals.insert(format!("{}.v", b.name), Vec::with_capacity(num_steps + 1));
            signals.insert(format!("{}.i", b.name), Vec::with_capacity(num_steps + 1));
            if matches!(b.block_type, BlockType::Diode | BlockType::IdealSwitch) {
                signals.insert(
                    format!("{}.state", b.name),
                    Vec::with_capacity(num_steps + 1),
                );
            }
        }
        for net in 1..=c.num_nodes {
            signals.insert(format!("Net{net}"), Vec::with_capacity(num_steps + 1));
        }
    }

    if let Some(ref mut c) = circuit_solver {
        engine.eval(
            config.start_time,
            &continuous_states,
            &discrete_states,
            &mut workspace,
        )?;
        for (_, b) in &electrical_blocks {
            if let Some(src_idx) = c.get_switch_driver_idx(&b.sid) {
                c.set_switch_control(&b.sid, workspace.outputs[src_idx]);
            }
        }
        c.init_solve(config.start_time)?;
        for &(idx, b) in &electrical_blocks {
            if let Some(v) = c.branch_voltage(&b.sid) {
                workspace.outputs[idx] = v;
            }
        }
    }

    let mut k1 = vec![0.0; num_states];
    let mut k2 = vec![0.0; num_states];
    let mut k3 = vec![0.0; num_states];
    let mut k4 = vec![0.0; num_states];
    let mut y_temp = vec![0.0; num_states];

    for step in 0..num_steps {
        let t = config.start_time + (step as f64) * dt;

        // Stage 1: evaluate at t, y
        engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;

        // Log outputs at this step
        tout.push(t);
        let primary_val = log_blocks
            .first()
            .map(|(idx, _)| workspace.outputs[*idx])
            .unwrap_or_else(|| {
                if let Some(&(idx, _)) = electrical_blocks.last() {
                    workspace.outputs[idx]
                } else {
                    workspace.outputs.last().copied().unwrap_or(0.0)
                }
            });
        yout.push(primary_val);
        for (idx, b) in &log_blocks {
            if let Some(vec) = signals.get_mut(&b.name) {
                vec.push(workspace.outputs[*idx]);
            }
        }

        if let Some(ref c) = circuit_solver {
            for (_, b) in &electrical_blocks {
                let v = c.branch_voltage(&b.sid).unwrap_or(0.0);
                let i = c.branch_current(&b.sid).unwrap_or(0.0);
                if let Some(vec) = signals.get_mut(&b.name) {
                    vec.push(v);
                }
                if let Some(vec) = signals.get_mut(&format!("{}.v", b.name)) {
                    vec.push(v);
                }
                if let Some(vec) = signals.get_mut(&format!("{}.i", b.name)) {
                    vec.push(i);
                }
                if let Some(st) = c.element_state(&b.sid) {
                    if let Some(vec) = signals.get_mut(&format!("{}.state", b.name)) {
                        vec.push(st);
                    }
                }
            }
            for net in 1..=c.num_nodes {
                let vn = c.node_voltage(net);
                if let Some(vec) = signals.get_mut(&format!("Net{net}")) {
                    vec.push(vn);
                }
            }
        }

        if num_states > 0 {
            k1.copy_from_slice(&workspace.derivatives);

            // Stage 2: evaluate at t + dt/2, y + dt/2 * k1
            let t_mid = t + 0.5 * dt;
            for i in 0..num_states {
                y_temp[i] = continuous_states[i] + 0.5 * dt * k1[i];
            }
            engine.eval(t_mid, &y_temp, &discrete_states, &mut workspace)?;
            k2.copy_from_slice(&workspace.derivatives);

            // Stage 3: evaluate at t + dt/2, y + dt/2 * k2
            for i in 0..num_states {
                y_temp[i] = continuous_states[i] + 0.5 * dt * k2[i];
            }
            engine.eval(t_mid, &y_temp, &discrete_states, &mut workspace)?;
            k3.copy_from_slice(&workspace.derivatives);

            // Stage 4: evaluate at t + dt, y + dt * k3
            let t_next = config.start_time + ((step + 1) as f64) * dt;
            for i in 0..num_states {
                y_temp[i] = continuous_states[i] + dt * k3[i];
            }
            engine.eval(t_next, &y_temp, &discrete_states, &mut workspace)?;
            k4.copy_from_slice(&workspace.derivatives);

            // Advance continuous states
            for i in 0..num_states {
                continuous_states[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
        }

        // Advance circuit solver to t + dt
        if let Some(ref mut c) = circuit_solver {
            for (_, b) in &electrical_blocks {
                if let Some(src_idx) = c.get_switch_driver_idx(&b.sid) {
                    c.set_switch_control(&b.sid, workspace.outputs[src_idx]);
                }
            }
            let t_next = config.start_time + ((step + 1) as f64) * dt;
            c.step(t_next, dt, step == 0)?;
            for &(idx, b) in &electrical_blocks {
                if let Some(v) = c.branch_voltage(&b.sid) {
                    workspace.outputs[idx] = v;
                }
            }
        }

        // Advance discrete states at t
        engine.update_discrete_at(
            t,
            config.start_time,
            &workspace.outputs,
            &mut discrete_states,
            config,
        );
    }

    // Final point at stop_time
    let t_stop = config.start_time + (num_steps as f64) * dt;
    engine.eval(t_stop, &continuous_states, &discrete_states, &mut workspace)?;
    tout.push(t_stop);
    let primary_val = log_blocks
        .first()
        .map(|(idx, _)| workspace.outputs[*idx])
        .unwrap_or_else(|| {
            if let Some(&(idx, _)) = electrical_blocks.last() {
                workspace.outputs[idx]
            } else {
                workspace.outputs.last().copied().unwrap_or(0.0)
            }
        });
    yout.push(primary_val);
    for (idx, b) in &log_blocks {
        if let Some(vec) = signals.get_mut(&b.name) {
            vec.push(workspace.outputs[*idx]);
        }
    }
    if let Some(ref c) = circuit_solver {
        for (_, b) in &electrical_blocks {
            let v = c.branch_voltage(&b.sid).unwrap_or(0.0);
            let i = c.branch_current(&b.sid).unwrap_or(0.0);
            if let Some(vec) = signals.get_mut(&b.name) {
                vec.push(v);
            }
            if let Some(vec) = signals.get_mut(&format!("{}.v", b.name)) {
                vec.push(v);
            }
            if let Some(vec) = signals.get_mut(&format!("{}.i", b.name)) {
                vec.push(i);
            }
        }
        for net in 1..=c.num_nodes {
            let vn = c.node_voltage(net);
            if let Some(vec) = signals.get_mut(&format!("Net{net}")) {
                vec.push(vn);
            }
        }
    }
    engine.update_discrete_at(
        t_stop,
        config.start_time,
        &workspace.outputs,
        &mut discrete_states,
        config,
    );

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
        diagnostics: circuit_solver.map(|c| c.diagnostics).unwrap_or_default(),
    })
}

/// Variable-step Dormand-Prince 5(4) simulation with adaptive step size and FSAL.
fn simulate_dormand_prince(
    engine: &ExecutionEngine,
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, SimError> {
    let num_states = engine.layout.num_continuous_states;
    let t_start = config.start_time;
    let t_stop = config.stop_time;

    let mut continuous_states = engine.initial_continuous_states.clone();
    let mut discrete_states = engine.initial_discrete_states.clone();

    let mut workspace = SimWorkspace::new(diagram.blocks.len(), num_states);

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

    let mut tout = Vec::new();
    let mut yout = Vec::new();
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();
    for (_, b) in &log_blocks {
        signals.insert(b.name.clone(), Vec::new());
    }

    let default_max_step = (t_stop - t_start) / 50.0;
    let max_step = config.max_step.unwrap_or(default_max_step);

    // Case 1: Zero continuous states
    if num_states == 0 {
        let mut t = t_start;
        engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;
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
        engine.update_discrete_at(t, t_start, &workspace.outputs, &mut discrete_states, config);

        // With no continuous AND no discrete states and no breakpoints, just land on t_stop
        if engine.layout.num_discrete_states == 0 && engine.next_breakpoint(t_start).is_none() {
            engine.eval(t_stop, &continuous_states, &discrete_states, &mut workspace)?;
            tout.push(t_stop);
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
            return Ok(SimulationOutput {
                tout,
                yout,
                signals,
                diagnostics: Vec::new(),
            });
        }

        let mut step_count = 0;
        while t < t_stop - 1e-14 {
            step_count += 1;
            if step_count > config.max_steps {
                return Err(SimError::max_steps_exceeded(config.max_steps, t));
            }
            let next_bp = engine.next_breakpoint(t);
            let next_hit = engine.next_sample_hit(t, t_start, config);

            let mut next_t = t + max_step;
            if let Some(bp) = next_bp {
                next_t = next_t.min(bp);
            }
            if let Some(sh) = next_hit {
                next_t = next_t.min(sh);
            }
            next_t = next_t.min(t_stop);

            t = next_t;
            engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;
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
            engine.update_discrete_at(t, t_start, &workspace.outputs, &mut discrete_states, config);
        }

        return Ok(SimulationOutput {
            tout,
            yout,
            signals,
            diagnostics: Vec::new(),
        });
    }

    // Case 2: One or more continuous states
    let abs_tol = config.abs_tol;
    let rel_tol = config.rel_tol;

    // Initial evaluation at t_start
    let mut t = t_start;
    engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;

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
    engine.update_discrete_at(t, t_start, &workspace.outputs, &mut discrete_states, config);

    let mut k_stages = vec![vec![0.0; num_states]; 7];
    k_stages[0].copy_from_slice(&workspace.derivatives);

    // Initial step computation (Hairer starting step if not specified)
    let mut h = if let Some(init) = config.initial_step {
        init.min(max_step).min(t_stop - t_start)
    } else {
        compute_hairer_starting_step(
            engine,
            t,
            &continuous_states,
            &discrete_states,
            &k_stages[0],
            abs_tol,
            rel_tol,
            max_step,
            t_stop - t_start,
            &mut workspace,
        )?
    };

    let mut y_stage = vec![0.0; num_states];
    let mut y_new = vec![0.0; num_states];
    let mut step_count = 0;
    let mut last_step_rejected = false;
    let use_fsal = config.enable_fsal.unwrap_or(true);
    let mut fsal_valid = true;

    while t < t_stop - 1e-14 {
        step_count += 1;
        if step_count > config.max_steps {
            return Err(SimError::max_steps_exceeded(config.max_steps, t));
        }

        let dyn_min_step = config
            .min_step
            .unwrap_or_else(|| (1e-12_f64).max(16.0 * f64::EPSILON * t.abs()));

        let next_bp = engine.next_breakpoint(t);
        let next_hit = engine.next_sample_hit(t, t_start, config);

        // Clamp step size to max_step, stop_time, and upcoming breakpoints / sample hits
        let mut h_step = h.min(max_step);
        if let Some(bp) = next_bp {
            if bp > t + 1e-12 {
                h_step = h_step.min(bp - t);
            }
        }
        if let Some(sh) = next_hit {
            if sh > t + 1e-12 {
                h_step = h_step.min(sh - t);
            }
        }
        let is_final_step = t + h_step >= t_stop;
        if is_final_step {
            h_step = t_stop - t;
        }

        if h_step <= 0.0 {
            break;
        }

        // Stage 1: k1 is either reused from k7 (FSAL) or recomputed
        if !fsal_valid {
            engine.eval(t, &continuous_states, &discrete_states, &mut workspace)?;
            k_stages[0].copy_from_slice(&workspace.derivatives);
        }

        // Stage 2: c2 = 1/5
        let t2 = t + DormandPrinceTableau::C[1] * h_step;
        for i in 0..num_states {
            y_stage[i] = continuous_states[i] + h_step * DormandPrinceTableau::A21 * k_stages[0][i];
        }
        engine.eval(t2, &y_stage, &discrete_states, &mut workspace)?;
        k_stages[1].copy_from_slice(&workspace.derivatives);

        // Stage 3: c3 = 3/10
        let t3 = t + DormandPrinceTableau::C[2] * h_step;
        for i in 0..num_states {
            y_stage[i] = continuous_states[i]
                + h_step
                    * (DormandPrinceTableau::A31 * k_stages[0][i]
                        + DormandPrinceTableau::A32 * k_stages[1][i]);
        }
        engine.eval(t3, &y_stage, &discrete_states, &mut workspace)?;
        k_stages[2].copy_from_slice(&workspace.derivatives);

        // Stage 4: c4 = 4/5
        let t4 = t + DormandPrinceTableau::C[3] * h_step;
        for i in 0..num_states {
            y_stage[i] = continuous_states[i]
                + h_step
                    * (DormandPrinceTableau::A41 * k_stages[0][i]
                        + DormandPrinceTableau::A42 * k_stages[1][i]
                        + DormandPrinceTableau::A43 * k_stages[2][i]);
        }
        engine.eval(t4, &y_stage, &discrete_states, &mut workspace)?;
        k_stages[3].copy_from_slice(&workspace.derivatives);

        // Stage 5: c5 = 8/9
        let t5 = t + DormandPrinceTableau::C[4] * h_step;
        for i in 0..num_states {
            y_stage[i] = continuous_states[i]
                + h_step
                    * (DormandPrinceTableau::A51 * k_stages[0][i]
                        + DormandPrinceTableau::A52 * k_stages[1][i]
                        + DormandPrinceTableau::A53 * k_stages[2][i]
                        + DormandPrinceTableau::A54 * k_stages[3][i]);
        }
        engine.eval(t5, &y_stage, &discrete_states, &mut workspace)?;
        k_stages[4].copy_from_slice(&workspace.derivatives);

        // Stage 6: c6 = 1.0
        let t6 = t + DormandPrinceTableau::C[5] * h_step;
        for i in 0..num_states {
            y_stage[i] = continuous_states[i]
                + h_step
                    * (DormandPrinceTableau::A61 * k_stages[0][i]
                        + DormandPrinceTableau::A62 * k_stages[1][i]
                        + DormandPrinceTableau::A63 * k_stages[2][i]
                        + DormandPrinceTableau::A64 * k_stages[3][i]
                        + DormandPrinceTableau::A65 * k_stages[4][i]);
        }
        engine.eval(t6, &y_stage, &discrete_states, &mut workspace)?;
        k_stages[5].copy_from_slice(&workspace.derivatives);

        // 5th-order solution candidate y_new
        for i in 0..num_states {
            y_new[i] = continuous_states[i]
                + h_step
                    * (DormandPrinceTableau::B[0] * k_stages[0][i]
                        + DormandPrinceTableau::B[2] * k_stages[2][i]
                        + DormandPrinceTableau::B[3] * k_stages[3][i]
                        + DormandPrinceTableau::B[4] * k_stages[4][i]
                        + DormandPrinceTableau::B[5] * k_stages[5][i]);
        }

        // Stage 7: FSAL evaluation at t + h_step, y_new
        let t7 = t + h_step;
        engine.eval(t7, &y_new, &discrete_states, &mut workspace)?;
        k_stages[6].copy_from_slice(&workspace.derivatives);

        // Scaled RMS error norm computation
        let mut sum_sq = 0.0;
        for i in 0..num_states {
            let error_i = h_step
                * (DormandPrinceTableau::E[0] * k_stages[0][i]
                    + DormandPrinceTableau::E[2] * k_stages[2][i]
                    + DormandPrinceTableau::E[3] * k_stages[3][i]
                    + DormandPrinceTableau::E[4] * k_stages[4][i]
                    + DormandPrinceTableau::E[5] * k_stages[5][i]
                    + DormandPrinceTableau::E[6] * k_stages[6][i]);
            let scale_i = abs_tol + rel_tol * continuous_states[i].abs().max(y_new[i].abs());
            let ratio = error_i / scale_i;
            sum_sq += ratio * ratio;
        }
        let err = (sum_sq / (num_states as f64)).sqrt();

        if err <= 1.0 {
            // STEP ACCEPTED
            t += h_step;
            continuous_states.copy_from_slice(&y_new);

            // Record logged signals at the accepted time step
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

            // Update discrete states only at sample hits
            engine.update_discrete_at(t, t_start, &workspace.outputs, &mut discrete_states, config);

            // Check if step landed on a breakpoint or sample hit: if so, invalidate FSAL
            let hit_breakpoint = next_bp.is_some_and(|bp| (t - bp).abs() < 1e-11);
            let hit_sample = next_hit.is_some_and(|sh| (t - sh).abs() < 1e-11);
            if hit_breakpoint || hit_sample || !use_fsal {
                fsal_valid = false;
            } else {
                fsal_valid = true;
                let (first, rest) = k_stages.split_at_mut(1);
                first[0].copy_from_slice(&rest[5]);
            }

            // Adapt step size for next step
            let fac = if err == 0.0 {
                5.0
            } else {
                (0.9 * err.powf(-0.2)).clamp(0.2, 5.0)
            };
            let fac = if last_step_rejected {
                fac.min(1.0)
            } else {
                fac
            };
            h = h_step * fac;
            last_step_rejected = false;
        } else {
            // STEP REJECTED: do not advance time, continuous, or discrete states
            fsal_valid = false;
            let fac = (0.9 * err.powf(-0.2)).clamp(0.2, 1.0);
            h = h_step * fac;
            last_step_rejected = true;

            if h < dyn_min_step && !is_final_step {
                return Err(SimError::step_size_limit(dyn_min_step, t));
            }
        }
    }

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
        diagnostics: Vec::new(),
    })
}

/// Hairer's starting step size heuristic for ODE initial value problems.
#[allow(clippy::too_many_arguments)]
fn compute_hairer_starting_step(
    engine: &ExecutionEngine,
    t0: f64,
    y0: &[f64],
    discrete_states: &[f64],
    f0: &[f64],
    abs_tol: f64,
    rel_tol: f64,
    max_step: f64,
    interval_len: f64,
    workspace: &mut SimWorkspace,
) -> Result<f64, SimError> {
    let n = y0.len();
    let mut sum_d0 = 0.0;
    let mut sum_d1 = 0.0;
    for i in 0..n {
        let sc = abs_tol + rel_tol * y0[i].abs();
        let r0 = y0[i] / sc;
        let r1 = f0[i] / sc;
        sum_d0 += r0 * r0;
        sum_d1 += r1 * r1;
    }
    let d0 = (sum_d0 / (n as f64)).sqrt();
    let d1 = (sum_d1 / (n as f64)).sqrt();

    let h0 = if d0 < 1e-5 || d1 < 1e-5 {
        1e-6
    } else {
        0.01 * (d0 / d1)
    };

    let mut y1 = vec![0.0; n];
    for i in 0..n {
        y1[i] = y0[i] + h0 * f0[i];
    }
    engine.eval(t0 + h0, &y1, discrete_states, workspace)?;
    let f1 = &workspace.derivatives;

    let mut sum_d2 = 0.0;
    for i in 0..n {
        let sc = abs_tol + rel_tol * y0[i].abs();
        let diff = (f1[i] - f0[i]) / sc;
        sum_d2 += diff * diff;
    }
    let d2 = (sum_d2 / (n as f64)).sqrt() / h0;

    let max_d = d1.max(d2);
    let h1 = if max_d <= 1e-15 {
        (1e-6_f64).max(h0 * 1e-3)
    } else {
        (0.01 / max_d).powf(0.2)
    };

    let h = (100.0 * h0).min(h1).min(max_step).min(interval_len);
    Ok(h.max(1e-10))
}
