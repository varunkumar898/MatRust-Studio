//! Execution engine: direct-feedthrough analysis, Tarjan SCC algebraic loop detection,
//! deterministic topological scheduling, and pure evaluation for block diagrams.

use crate::measurements::{PowerMeasurementBuffer, ScalarWindowBuffer, ThdBuffer};
use crate::model::{BlockDiagram, BlockType};
use crate::solver::{BlockRef, SimError, SimulationConfig, SolverType};
use crate::spec::parse_float_vector;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Returns whether the specified input port of a block type has direct feedthrough to its output.
pub fn feedthrough(
    block_type: &BlockType,
    parameters: &HashMap<String, String>,
    input_port: usize,
) -> bool {
    crate::spec::spec_feedthrough(block_type, parameters, input_port)
}

/// Relational comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

impl std::str::FromStr for RelOp {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let op = match s.trim() {
            "==" => RelOp::Equal,
            "~=" | "!=" => RelOp::NotEqual,
            "<" => RelOp::LessThan,
            "<=" => RelOp::LessEqual,
            ">" => RelOp::GreaterThan,
            ">=" => RelOp::GreaterEqual,
            _ => RelOp::LessEqual,
        };
        Ok(op)
    }
}

impl RelOp {
    #[inline]
    pub fn evaluate(&self, a: f64, b: f64) -> bool {
        match self {
            RelOp::Equal => (a - b).abs() < 1e-12,
            RelOp::NotEqual => (a - b).abs() >= 1e-12,
            RelOp::LessThan => a < b,
            RelOp::LessEqual => a <= b,
            RelOp::GreaterThan => a > b,
            RelOp::GreaterEqual => a >= b,
        }
    }
}

/// Boolean logical operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogOp {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Not,
}

impl std::str::FromStr for LogOp {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let op = match s.trim().to_uppercase().as_str() {
            "OR" => LogOp::Or,
            "NAND" => LogOp::Nand,
            "NOR" => LogOp::Nor,
            "XOR" => LogOp::Xor,
            "NOT" => LogOp::Not,
            _ => LogOp::And,
        };
        Ok(op)
    }
}

/// Switch threshold comparison criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchCriteria {
    GreaterEqual,
    GreaterThan,
    NotZero,
}

impl std::str::FromStr for SwitchCriteria {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let crit = if trimmed == ">" || trimmed.contains("u2 >") {
            SwitchCriteria::GreaterThan
        } else if trimmed == "~=" || trimmed == "~=0" || trimmed == "!=0" || trimmed.contains("~=")
        {
            SwitchCriteria::NotZero
        } else {
            SwitchCriteria::GreaterEqual
        };
        Ok(crit)
    }
}

impl SwitchCriteria {
    #[inline]
    pub fn is_pass_first(&self, u2: f64, threshold: f64) -> bool {
        match self {
            SwitchCriteria::GreaterEqual => u2 >= threshold,
            SwitchCriteria::GreaterThan => u2 > threshold,
            SwitchCriteria::NotZero => u2.abs() >= 1e-12,
        }
    }
}

/// Mathematical functions supported by the MathFunction block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathFunc {
    Sqrt,
    Exp,
    Log,
    Log10,
    Square,
    Pow,
    Reciprocal,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
}

impl std::str::FromStr for MathFunc {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let func = match s.trim().to_lowercase().as_str() {
            "exp" => MathFunc::Exp,
            "log" | "ln" => MathFunc::Log,
            "log10" => MathFunc::Log10,
            "square" => MathFunc::Square,
            "pow" => MathFunc::Pow,
            "reciprocal" => MathFunc::Reciprocal,
            "sin" => MathFunc::Sin,
            "cos" => MathFunc::Cos,
            "tan" => MathFunc::Tan,
            "asin" => MathFunc::Asin,
            "acos" => MathFunc::Acos,
            "atan" => MathFunc::Atan,
            "sinh" => MathFunc::Sinh,
            "cosh" => MathFunc::Cosh,
            "tanh" => MathFunc::Tanh,
            _ => MathFunc::Sqrt,
        };
        Ok(func)
    }
}

impl MathFunc {
    #[inline]
    pub fn evaluate(&self, u: f64, u2: Option<f64>) -> f64 {
        match self {
            MathFunc::Sqrt => u.sqrt(),
            MathFunc::Exp => u.exp(),
            MathFunc::Log => u.ln(),
            MathFunc::Log10 => u.log10(),
            MathFunc::Square => u * u,
            MathFunc::Pow => u.powf(u2.unwrap_or(1.0)),
            MathFunc::Reciprocal => 1.0 / u,
            MathFunc::Sin => u.sin(),
            MathFunc::Cos => u.cos(),
            MathFunc::Tan => u.tan(),
            MathFunc::Asin => u.asin(),
            MathFunc::Acos => u.acos(),
            MathFunc::Atan => u.atan(),
            MathFunc::Sinh => u.sinh(),
            MathFunc::Cosh => u.cosh(),
            MathFunc::Tanh => u.tanh(),
        }
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
        freq_rad: f64,
        phase: f64,
        bias: f64,
    },
    Step {
        step_time: f64,
        before: f64,
        after: f64,
    },
    Sawtooth {
        amp: f64,
        freq_hz: f64,
        phase: f64,
        bias: f64,
    },
    Triangle {
        amp: f64,
        freq_hz: f64,
        phase: f64,
        bias: f64,
    },
    PulseGenerator {
        amp: f64,
        period: f64,
        duty: f64,
        delay: f64,
    },
    RepeatingSequence {
        time_values: Vec<f64>,
        output_values: Vec<f64>,
        period: f64,
    },
    RepeatingSequenceStair {
        time_values: Vec<f64>,
        output_values: Vec<f64>,
        period: f64,
    },
    Clock,
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
        sample_time: Option<f64>,
    },
    TransferFunction {
        state_idx: usize,
        num_states: usize,
        a_coeffs: Vec<f64>,
        c_coeffs: Vec<f64>,
        d_coeff: f64,
    },
    DiscreteTransferFunction {
        state_idx: usize,
        num_states: usize,
        a_coeffs: Vec<f64>,
        c_coeffs: Vec<f64>,
        d_coeff: f64,
        sample_time: Option<f64>,
    },
    Pid {
        is_discrete: bool,
        state_idx_i: usize,
        state_idx_d: usize,
        kp: f64,
        ki: f64,
        kd: f64,
        n: f64,
        upper_limit: Option<f64>,
        lower_limit: Option<f64>,
        clamping: bool,
        sample_time: Option<f64>,
    },
    ZeroOrderHold {
        state_idx: usize,
        sample_time: Option<f64>,
    },
    RelationalOperator {
        op: RelOp,
    },
    LogicalOperator {
        op: LogOp,
    },
    Switch {
        criteria: SwitchCriteria,
        threshold: f64,
    },
    Abs,
    MathFunction {
        func: MathFunc,
    },
    Saturation {
        lower: f64,
        upper: f64,
    },
    Scope,
    ToWorkspace,
    Outport,
    Inport,
    Sensor,
    Rms {
        buffer_idx: usize,
        sample_time: Option<f64>,
    },
    Mean {
        buffer_idx: usize,
        sample_time: Option<f64>,
    },
    PowerMeasurement {
        buffer_idx: usize,
        sample_time: Option<f64>,
    },
    Thd {
        buffer_idx: usize,
        sample_time: Option<f64>,
    },
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
    pub rms_buffers: Vec<ScalarWindowBuffer>,
    pub mean_buffers: Vec<ScalarWindowBuffer>,
    pub power_buffers: Vec<PowerMeasurementBuffer>,
    pub thd_buffers: Vec<ThdBuffer>,
}

impl SimWorkspace {
    pub fn new(num_blocks: usize, num_continuous_states: usize) -> Self {
        Self {
            outputs: vec![0.0; num_blocks],
            derivatives: vec![0.0; num_continuous_states],
            rms_buffers: Vec::new(),
            mean_buffers: Vec::new(),
            power_buffers: Vec::new(),
            thd_buffers: Vec::new(),
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
    pub block_output_offsets: Vec<usize>,
    pub total_output_slots: usize,
    pub initial_rms_buffers: Vec<ScalarWindowBuffer>,
    pub initial_mean_buffers: Vec<ScalarWindowBuffer>,
    pub initial_power_buffers: Vec<PowerMeasurementBuffer>,
    pub initial_thd_buffers: Vec<ThdBuffer>,
    pub warnings: Vec<crate::diagnostics::Diagnostic>,
}

impl ExecutionEngine {
    pub fn create_workspace(&self) -> SimWorkspace {
        SimWorkspace {
            outputs: vec![0.0; self.total_output_slots],
            derivatives: vec![0.0; self.layout.num_continuous_states],
            rms_buffers: self.initial_rms_buffers.clone(),
            mean_buffers: self.initial_mean_buffers.clone(),
            power_buffers: self.initial_power_buffers.clone(),
            thd_buffers: self.initial_thd_buffers.clone(),
        }
    }

    /// Compiles a block diagram into an execution engine: analyzes direct feedthrough,
    /// performs Tarjan SCC loop detection for E-ALG errors, validates block parameters,
    /// and schedules blocks topologically.
    pub fn compile(diagram: &BlockDiagram) -> Result<Self, SimError> {
        Self::compile_with_config(diagram, diagram.simulation_config.as_ref())
    }

    pub fn compile_with_config(
        diagram: &BlockDiagram,
        config: Option<&SimulationConfig>,
    ) -> Result<Self, SimError> {
        let num_blocks = diagram.blocks.len();

        // 1. Build SID/name to index lookup
        let mut block_indices: HashMap<String, usize> = HashMap::with_capacity(num_blocks * 2);
        for (i, b) in diagram.blocks.iter().enumerate() {
            block_indices.insert(b.sid.clone(), i);
            block_indices.insert(b.name.clone(), i);
        }

        // 2. Compute state offsets, initial conditions, and pre-validate stateful blocks
        let mut continuous_offsets = Vec::with_capacity(num_blocks);
        let mut discrete_offsets = Vec::with_capacity(num_blocks);
        let mut initial_continuous_states = Vec::new();
        let mut initial_discrete_states = Vec::new();

        for b in &diagram.blocks {
            match b.block_type {
                BlockType::Integrator => {
                    let init_cond = b
                        .get_param("InitialCondition")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let idx = initial_continuous_states.len();
                    continuous_offsets.push(Some(idx));
                    initial_continuous_states.push(init_cond);
                    discrete_offsets.push(None);
                }
                BlockType::UnitDelay => {
                    let init_cond = b
                        .get_param("InitialCondition")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let idx = initial_discrete_states.len();
                    discrete_offsets.push(Some(idx));
                    initial_discrete_states.push(init_cond);
                    continuous_offsets.push(None);
                }
                BlockType::ZeroOrderHold => {
                    let idx = initial_discrete_states.len();
                    discrete_offsets.push(Some(idx));
                    initial_discrete_states.push(0.0);
                    continuous_offsets.push(None);
                }
                BlockType::TransferFunction => {
                    let den_str = b.get_param("Denominator").unwrap_or("1, 1");
                    let den = parse_float_vector(den_str).map_err(SimError::parameter)?;
                    if den.is_empty() || den[0] == 0.0 {
                        return Err(SimError::parameter(format!(
                            "Denominator of transfer function '{}' cannot be empty or have zero leading coefficient",
                            b.name
                        )));
                    }
                    let num_str = b.get_param("Numerator").unwrap_or("1");
                    let num = parse_float_vector(num_str).map_err(SimError::parameter)?;
                    if num.len() > den.len() {
                        return Err(SimError::parameter(format!(
                            "Transfer function '{}' must be proper: degree of numerator ({}) must be <= degree of denominator ({})",
                            b.name,
                            num.len().saturating_sub(1),
                            den.len().saturating_sub(1)
                        )));
                    }

                    let n_states = den.len().saturating_sub(1);
                    if n_states > 0 {
                        let idx = initial_continuous_states.len();
                        continuous_offsets.push(Some(idx));
                        initial_continuous_states
                            .resize(initial_continuous_states.len() + n_states, 0.0);
                    } else {
                        continuous_offsets.push(None);
                    }
                    discrete_offsets.push(None);
                }
                BlockType::DiscreteTransferFunction => {
                    let den_str = b.get_param("Denominator").unwrap_or("1, -0.5");
                    let den = parse_float_vector(den_str).map_err(SimError::parameter)?;
                    if den.is_empty() || den[0] == 0.0 {
                        return Err(SimError::parameter(format!(
                            "Denominator of discrete transfer function '{}' cannot be empty or have zero leading coefficient",
                            b.name
                        )));
                    }
                    let num_str = b.get_param("Numerator").unwrap_or("1");
                    let num = parse_float_vector(num_str).map_err(SimError::parameter)?;
                    if num.len() > den.len() {
                        return Err(SimError::parameter(format!(
                            "Discrete transfer function '{}' must be proper: degree of numerator ({}) must be <= degree of denominator ({})",
                            b.name,
                            num.len().saturating_sub(1),
                            den.len().saturating_sub(1)
                        )));
                    }

                    let n_states = den.len().saturating_sub(1);
                    if n_states > 0 {
                        let idx = initial_discrete_states.len();
                        discrete_offsets.push(Some(idx));
                        initial_discrete_states
                            .resize(initial_discrete_states.len() + n_states, 0.0);
                    } else {
                        discrete_offsets.push(None);
                    }
                    continuous_offsets.push(None);
                }
                BlockType::Pid => {
                    let domain = b.get_param("TimeDomain").unwrap_or("Continuous");
                    let is_discrete = domain.eq_ignore_ascii_case("Discrete");
                    let init_i = b
                        .get_param("InitialConditionForIntegrator")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let init_d = b
                        .get_param("InitialConditionForFilter")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);

                    if is_discrete {
                        let idx = initial_discrete_states.len();
                        discrete_offsets.push(Some(idx));
                        initial_discrete_states.push(init_i);
                        initial_discrete_states.push(init_d);
                        continuous_offsets.push(None);
                    } else {
                        let idx = initial_continuous_states.len();
                        continuous_offsets.push(Some(idx));
                        initial_continuous_states.push(init_i);
                        initial_continuous_states.push(init_d);
                        discrete_offsets.push(None);
                    }
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

        // 3. Compile block configurations, parse parameters, and map input wiring
        let mut block_output_offsets = Vec::with_capacity(num_blocks);
        let mut total_output_slots = 0;
        for b in &diagram.blocks {
            block_output_offsets.push(total_output_slots);
            let n_out = b.output_ports().max(1);
            total_output_slots += n_out;
        }

        let mut slot_to_block = vec![0; total_output_slots];
        for (i, b) in diagram.blocks.iter().enumerate() {
            let offset = block_output_offsets[i];
            let n_out = b.output_ports().max(1);
            for item in slot_to_block.iter_mut().skip(offset).take(n_out) {
                *item = i;
            }
        }

        let mut initial_rms_buffers = Vec::new();
        let mut initial_mean_buffers = Vec::new();
        let mut initial_power_buffers = Vec::new();
        let mut initial_thd_buffers = Vec::new();
        let mut warnings = Vec::new();

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
                        let src_port = if line.src.port_index > 0 {
                            line.src.port_index - 1
                        } else {
                            0
                        };
                        let src_outs = diagram.blocks[src_idx].output_ports().max(1);
                        let src_slot = block_output_offsets[src_idx] + src_port.min(src_outs - 1);
                        if port_idx < num_inputs {
                            input_sources[port_idx] = Some(src_slot);
                        } else {
                            input_sources.push(Some(src_slot));
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
                    if freq <= 0.0 || !freq.is_finite() {
                        return Err(SimError::parameter(format!(
                            "Sine Wave block '{}' requires a positive finite frequency; got {freq}",
                            b.name
                        )));
                    }
                    let units = b.get_param("Units").unwrap_or("rad/s");
                    let freq_rad = if units.eq_ignore_ascii_case("Hz") {
                        freq * 2.0 * std::f64::consts::PI
                    } else {
                        freq
                    };
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
                        freq_rad,
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
                BlockType::Sawtooth => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let freq = b
                        .get_param("Frequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    if freq <= 0.0 || !freq.is_finite() {
                        return Err(SimError::parameter(format!(
                            "Sawtooth block '{}' requires a positive finite frequency; got {freq}",
                            b.name
                        )));
                    }
                    let units = b.get_param("Units").unwrap_or("Hz");
                    let freq_hz = if units.eq_ignore_ascii_case("rad/s") {
                        freq / (2.0 * std::f64::consts::PI)
                    } else {
                        freq
                    };
                    let phase = b
                        .get_param("Phase")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let bias = b
                        .get_param("Bias")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    CompiledKind::Sawtooth {
                        amp,
                        freq_hz,
                        phase,
                        bias,
                    }
                }
                BlockType::Triangle => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let freq = b
                        .get_param("Frequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    if freq <= 0.0 || !freq.is_finite() {
                        return Err(SimError::parameter(format!(
                            "Triangle block '{}' requires a positive finite frequency; got {freq}",
                            b.name
                        )));
                    }
                    let units = b.get_param("Units").unwrap_or("Hz");
                    let freq_hz = if units.eq_ignore_ascii_case("rad/s") {
                        freq / (2.0 * std::f64::consts::PI)
                    } else {
                        freq
                    };
                    let phase = b
                        .get_param("Phase")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let bias = b
                        .get_param("Bias")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    CompiledKind::Triangle {
                        amp,
                        freq_hz,
                        phase,
                        bias,
                    }
                }
                BlockType::PulseGenerator => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let period = b
                        .get_param("Period")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    if period <= 0.0 || !period.is_finite() {
                        return Err(SimError::parameter(format!(
                            "Pulse Generator '{}' requires a positive finite period; got {period}",
                            b.name
                        )));
                    }
                    let duty = b
                        .get_param("Duty")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(50.0);
                    if duty <= 0.0 || duty > 100.0 || !duty.is_finite() {
                        return Err(SimError::parameter(format!(
                            "Pulse Generator '{}' requires a duty cycle in (0, 100]; got {duty}",
                            b.name
                        )));
                    }
                    let delay = b
                        .get_param("Delay")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    CompiledKind::PulseGenerator {
                        amp,
                        period,
                        duty,
                        delay,
                    }
                }
                BlockType::RepeatingSequence => {
                    let t_str = b.get_param("TimeValues").unwrap_or("[0, 1, 2]");
                    let y_str = b.get_param("OutputValues").unwrap_or("[0, 2, 0]");
                    let time_values = parse_float_vector(t_str).map_err(SimError::parameter)?;
                    let output_values = parse_float_vector(y_str).map_err(SimError::parameter)?;
                    if time_values.len() < 2 || time_values.len() != output_values.len() {
                        return Err(SimError::parameter(format!(
                            "Repeating Sequence '{}' requires at least 2 matching time and output values; got times: {}, outputs: {}",
                            b.name, time_values.len(), output_values.len()
                        )));
                    }
                    for w in time_values.windows(2) {
                        if w[1] <= w[0] {
                            return Err(SimError::parameter(format!(
                                "Repeating Sequence '{}' TimeValues must be strictly monotonically increasing; got {} <= {}",
                                b.name, w[1], w[0]
                            )));
                        }
                    }
                    let period = time_values.last().copied().unwrap_or(0.0)
                        - time_values.first().copied().unwrap_or(0.0);
                    CompiledKind::RepeatingSequence {
                        time_values,
                        output_values,
                        period,
                    }
                }
                BlockType::RepeatingSequenceStair => {
                    let t_str = b.get_param("TimeValues").unwrap_or("[0, 1, 2]");
                    let y_str = b.get_param("OutputValues").unwrap_or("[0, 2, 0]");
                    let time_values = parse_float_vector(t_str).map_err(SimError::parameter)?;
                    let output_values = parse_float_vector(y_str).map_err(SimError::parameter)?;
                    if time_values.len() < 2 || time_values.len() != output_values.len() {
                        return Err(SimError::parameter(format!(
                            "Repeating Sequence Stair '{}' requires at least 2 matching time and output values; got times: {}, outputs: {}",
                            b.name, time_values.len(), output_values.len()
                        )));
                    }
                    for w in time_values.windows(2) {
                        if w[1] <= w[0] {
                            return Err(SimError::parameter(format!(
                                "Repeating Sequence Stair '{}' TimeValues must be strictly monotonically increasing; got {} <= {}",
                                b.name, w[1], w[0]
                            )));
                        }
                    }
                    let period = time_values.last().copied().unwrap_or(0.0)
                        - time_values.first().copied().unwrap_or(0.0);
                    CompiledKind::RepeatingSequenceStair {
                        time_values,
                        output_values,
                        period,
                    }
                }
                BlockType::Clock => CompiledKind::Clock,
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
                    let sample_time = b
                        .get_param("SampleTime")
                        .and_then(|s| s.parse::<f64>().ok());
                    CompiledKind::UnitDelay {
                        state_idx,
                        sample_time,
                    }
                }
                BlockType::TransferFunction => {
                    let den_str = b.get_param("Denominator").unwrap_or("1, 1");
                    let mut den = parse_float_vector(den_str).map_err(SimError::parameter)?;
                    let a_n = den[0];
                    for val in den.iter_mut() {
                        *val /= a_n;
                    }
                    let num_str = b.get_param("Numerator").unwrap_or("1");
                    let mut num = parse_float_vector(num_str).map_err(SimError::parameter)?;
                    for val in num.iter_mut() {
                        *val /= a_n;
                    }

                    let n = den.len().saturating_sub(1);
                    let mut num_padded = vec![0.0; n + 1];
                    let offset = (n + 1).saturating_sub(num.len());
                    for (k, &val) in num.iter().enumerate() {
                        num_padded[offset + k] = val;
                    }

                    let d_coeff = num_padded[0];
                    let mut a_coeffs = vec![0.0; n];
                    let mut c_coeffs = vec![0.0; n];
                    for j in 0..n {
                        let a_j = den[n - j];
                        let b_j = num_padded[n - j];
                        a_coeffs[j] = a_j;
                        c_coeffs[j] = b_j - d_coeff * a_j;
                    }

                    let state_idx = layout.continuous_offsets[i].unwrap_or(0);
                    CompiledKind::TransferFunction {
                        state_idx,
                        num_states: n,
                        a_coeffs,
                        c_coeffs,
                        d_coeff,
                    }
                }
                BlockType::DiscreteTransferFunction => {
                    let den_str = b.get_param("Denominator").unwrap_or("1, -0.5");
                    let mut den = parse_float_vector(den_str).map_err(SimError::parameter)?;
                    let a_n = den[0];
                    for val in den.iter_mut() {
                        *val /= a_n;
                    }
                    let num_str = b.get_param("Numerator").unwrap_or("1");
                    let mut num = parse_float_vector(num_str).map_err(SimError::parameter)?;
                    for val in num.iter_mut() {
                        *val /= a_n;
                    }

                    let n = den.len().saturating_sub(1);
                    let mut num_padded = vec![0.0; n + 1];
                    let offset = (n + 1).saturating_sub(num.len());
                    for (k, &val) in num.iter().enumerate() {
                        num_padded[offset + k] = val;
                    }

                    let d_coeff = num_padded[0];
                    let mut a_coeffs = vec![0.0; n];
                    let mut c_coeffs = vec![0.0; n];
                    for j in 0..n {
                        let a_j = den[n - j];
                        let b_j = num_padded[n - j];
                        a_coeffs[j] = a_j;
                        c_coeffs[j] = b_j - d_coeff * a_j;
                    }

                    let state_idx = layout.discrete_offsets[i].unwrap_or(0);
                    let sample_time = b
                        .get_param("SampleTime")
                        .and_then(|s| s.parse::<f64>().ok());
                    CompiledKind::DiscreteTransferFunction {
                        state_idx,
                        num_states: n,
                        a_coeffs,
                        c_coeffs,
                        d_coeff,
                        sample_time,
                    }
                }
                BlockType::Pid => {
                    let domain = b.get_param("TimeDomain").unwrap_or("Continuous");
                    let is_discrete = domain.eq_ignore_ascii_case("Discrete");
                    let kp = b
                        .get_param("P")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let ki = b
                        .get_param("I")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let kd = b
                        .get_param("D")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let n = b
                        .get_param("N")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(100.0);
                    let upper_limit = b
                        .get_param("UpperSaturationLimit")
                        .and_then(|s| s.parse::<f64>().ok());
                    let lower_limit = b
                        .get_param("LowerSaturationLimit")
                        .and_then(|s| s.parse::<f64>().ok());
                    let anti_windup = b.get_param("AntiWindupMode").unwrap_or("Clamping");
                    let clamping = anti_windup.eq_ignore_ascii_case("Clamping");
                    let sample_time = b
                        .get_param("SampleTime")
                        .and_then(|s| s.parse::<f64>().ok());

                    let (state_idx_i, state_idx_d) = if is_discrete {
                        let base = layout.discrete_offsets[i].unwrap_or(0);
                        (base, base + 1)
                    } else {
                        let base = layout.continuous_offsets[i].unwrap_or(0);
                        (base, base + 1)
                    };

                    CompiledKind::Pid {
                        is_discrete,
                        state_idx_i,
                        state_idx_d,
                        kp,
                        ki,
                        kd,
                        n,
                        upper_limit,
                        lower_limit,
                        clamping,
                        sample_time,
                    }
                }
                BlockType::ZeroOrderHold => {
                    let state_idx = layout.discrete_offsets[i].unwrap_or(0);
                    let sample_time = b
                        .get_param("SampleTime")
                        .and_then(|s| s.parse::<f64>().ok());
                    CompiledKind::ZeroOrderHold {
                        state_idx,
                        sample_time,
                    }
                }
                BlockType::RelationalOperator => {
                    let op_str = b.get_param("Operator").unwrap_or("<=");
                    CompiledKind::RelationalOperator {
                        op: op_str.parse().unwrap_or(RelOp::LessEqual),
                    }
                }
                BlockType::LogicalOperator => {
                    let op_str = b.get_param("Operator").unwrap_or("AND");
                    CompiledKind::LogicalOperator {
                        op: op_str.parse().unwrap_or(LogOp::And),
                    }
                }
                BlockType::Switch => {
                    let crit_str = b.get_param("Criteria").unwrap_or(">=");
                    let thresh = b
                        .get_param("Threshold")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    CompiledKind::Switch {
                        criteria: crit_str.parse().unwrap_or(SwitchCriteria::GreaterEqual),
                        threshold: thresh,
                    }
                }
                BlockType::Abs => CompiledKind::Abs,
                BlockType::MathFunction => {
                    let func_str = b.get_param("Function").unwrap_or("sqrt");
                    CompiledKind::MathFunction {
                        func: func_str.parse().unwrap_or(MathFunc::Sqrt),
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
                    if lower > upper {
                        return Err(SimError::parameter(format!(
                            "Saturation block '{}': lower limit ({lower}) must be <= upper limit ({upper})",
                            b.name
                        )));
                    }
                    CompiledKind::Saturation { lower, upper }
                }
                BlockType::Scope => CompiledKind::Scope,
                BlockType::ToWorkspace => CompiledKind::ToWorkspace,
                BlockType::Outport => CompiledKind::Outport,
                BlockType::Inport => CompiledKind::Inport,
                BlockType::VoltageSensor | BlockType::CurrentSensor => CompiledKind::Sensor,
                BlockType::Rms => {
                    let f_base = b
                        .get_param("FundamentalFrequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(50.0);
                    if f_base <= 0.0 {
                        return Err(SimError::parameter(format!(
                            "FundamentalFrequency on block '{}' ({f_base}) must be strictly positive.",
                            b.name
                        )));
                    }
                    let t_period = 1.0 / f_base;
                    let st_param = b.get_param("SampleTime").and_then(|s| {
                        let tr = s.trim();
                        if tr.is_empty() {
                            None
                        } else {
                            tr.parse::<f64>().ok()
                        }
                    });
                    let (ts, sample_time) = match st_param {
                        Some(ts_val) => {
                            if ts_val <= 0.0 {
                                return Err(SimError::parameter(format!(
                                    "SampleTime on block '{}' ({ts_val}) must be strictly positive.",
                                    b.name
                                )));
                            }
                            (ts_val, Some(ts_val))
                        }
                        None => {
                            if let Some(cfg) = config {
                                if cfg.solver.is_variable_step() {
                                    return Err(SimError::sample_time_missing(
                                        crate::solver::BlockRef {
                                            name: b.name.clone(),
                                            sid: b.sid.clone(),
                                        },
                                    ));
                                }
                                (cfg.step_size, None)
                            } else {
                                (0.001, None)
                            }
                        }
                    };
                    let nw = (t_period / ts).round() as usize;
                    if nw < 4 {
                        return Err(SimError::parameter(format!(
                            "Window sample count Nw ({nw}) is less than 4 on block '{}': FundamentalFrequency ({f_base} Hz) is too high for SampleTime ({ts} s) (E-PARAM).",
                            b.name
                        )));
                    }
                    if ((t_period / ts) - (nw as f64)).abs() > 1e-6 * (nw as f64) {
                        warnings.push(crate::diagnostics::Diagnostic::warning(
                            "W-WINDOW",
                            format!(
                                "Window sample count ({nw}) on block '{}' is not an integer number of samples per period ({:.4}): ripple expected in measurement outputs.",
                                b.name,
                                t_period / ts
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                    let init_val = b
                        .get_param("InitialValue")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let buf_idx = initial_rms_buffers.len();
                    initial_rms_buffers.push(ScalarWindowBuffer::new(nw, init_val, true));
                    CompiledKind::Rms {
                        buffer_idx: buf_idx,
                        sample_time,
                    }
                }
                BlockType::Mean => {
                    let f_base = b
                        .get_param("FundamentalFrequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(50.0);
                    if f_base <= 0.0 {
                        return Err(SimError::parameter(format!(
                            "FundamentalFrequency on block '{}' ({f_base}) must be strictly positive.",
                            b.name
                        )));
                    }
                    let t_period = 1.0 / f_base;
                    let st_param = b.get_param("SampleTime").and_then(|s| {
                        let tr = s.trim();
                        if tr.is_empty() {
                            None
                        } else {
                            tr.parse::<f64>().ok()
                        }
                    });
                    let (ts, sample_time) = match st_param {
                        Some(ts_val) => {
                            if ts_val <= 0.0 {
                                return Err(SimError::parameter(format!(
                                    "SampleTime on block '{}' ({ts_val}) must be strictly positive.",
                                    b.name
                                )));
                            }
                            (ts_val, Some(ts_val))
                        }
                        None => {
                            if let Some(cfg) = config {
                                if cfg.solver.is_variable_step() {
                                    return Err(SimError::sample_time_missing(
                                        crate::solver::BlockRef {
                                            name: b.name.clone(),
                                            sid: b.sid.clone(),
                                        },
                                    ));
                                }
                                (cfg.step_size, None)
                            } else {
                                (0.001, None)
                            }
                        }
                    };
                    let nw = (t_period / ts).round() as usize;
                    if nw < 4 {
                        return Err(SimError::parameter(format!(
                            "Window sample count Nw ({nw}) is less than 4 on block '{}': FundamentalFrequency ({f_base} Hz) is too high for SampleTime ({ts} s) (E-PARAM).",
                            b.name
                        )));
                    }
                    if ((t_period / ts) - (nw as f64)).abs() > 1e-6 * (nw as f64) {
                        warnings.push(crate::diagnostics::Diagnostic::warning(
                            "W-WINDOW",
                            format!(
                                "Window sample count ({nw}) on block '{}' is not an integer number of samples per period ({:.4}): ripple expected in measurement outputs.",
                                b.name,
                                t_period / ts
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                    let init_val = b
                        .get_param("InitialValue")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let buf_idx = initial_mean_buffers.len();
                    initial_mean_buffers.push(ScalarWindowBuffer::new(nw, init_val, false));
                    CompiledKind::Mean {
                        buffer_idx: buf_idx,
                        sample_time,
                    }
                }
                BlockType::PowerMeasurement => {
                    let f_base = b
                        .get_param("FundamentalFrequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(50.0);
                    if f_base <= 0.0 {
                        return Err(SimError::parameter(format!(
                            "FundamentalFrequency on block '{}' ({f_base}) must be strictly positive.",
                            b.name
                        )));
                    }
                    let t_period = 1.0 / f_base;
                    let st_param = b.get_param("SampleTime").and_then(|s| {
                        let tr = s.trim();
                        if tr.is_empty() {
                            None
                        } else {
                            tr.parse::<f64>().ok()
                        }
                    });
                    let (ts, sample_time) = match st_param {
                        Some(ts_val) => {
                            if ts_val <= 0.0 {
                                return Err(SimError::parameter(format!(
                                    "SampleTime on block '{}' ({ts_val}) must be strictly positive.",
                                    b.name
                                )));
                            }
                            (ts_val, Some(ts_val))
                        }
                        None => {
                            if let Some(cfg) = config {
                                if cfg.solver.is_variable_step() {
                                    return Err(SimError::sample_time_missing(
                                        crate::solver::BlockRef {
                                            name: b.name.clone(),
                                            sid: b.sid.clone(),
                                        },
                                    ));
                                }
                                (cfg.step_size, None)
                            } else {
                                (0.001, None)
                            }
                        }
                    };
                    let nw = (t_period / ts).round() as usize;
                    if nw < 4 {
                        return Err(SimError::parameter(format!(
                            "Window sample count Nw ({nw}) is less than 4 on block '{}': FundamentalFrequency ({f_base} Hz) is too high for SampleTime ({ts} s) (E-PARAM).",
                            b.name
                        )));
                    }
                    if ((t_period / ts) - (nw as f64)).abs() > 1e-6 * (nw as f64) {
                        warnings.push(crate::diagnostics::Diagnostic::warning(
                            "W-WINDOW",
                            format!(
                                "Window sample count ({nw}) on block '{}' is not an integer number of samples per period ({:.4}): ripple expected in measurement outputs.",
                                b.name,
                                t_period / ts
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                    let buf_idx = initial_power_buffers.len();
                    initial_power_buffers.push(PowerMeasurementBuffer::new(nw));
                    CompiledKind::PowerMeasurement {
                        buffer_idx: buf_idx,
                        sample_time,
                    }
                }
                BlockType::Thd => {
                    let f_base = b
                        .get_param("FundamentalFrequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(50.0);
                    if f_base <= 0.0 {
                        return Err(SimError::parameter(format!(
                            "FundamentalFrequency on block '{}' ({f_base}) must be strictly positive.",
                            b.name
                        )));
                    }
                    let t_period = 1.0 / f_base;
                    let st_param = b.get_param("SampleTime").and_then(|s| {
                        let tr = s.trim();
                        if tr.is_empty() {
                            None
                        } else {
                            tr.parse::<f64>().ok()
                        }
                    });
                    let (ts, sample_time) = match st_param {
                        Some(ts_val) => {
                            if ts_val <= 0.0 {
                                return Err(SimError::parameter(format!(
                                    "SampleTime on block '{}' ({ts_val}) must be strictly positive.",
                                    b.name
                                )));
                            }
                            (ts_val, Some(ts_val))
                        }
                        None => {
                            if let Some(cfg) = config {
                                if cfg.solver.is_variable_step() {
                                    return Err(SimError::sample_time_missing(
                                        crate::solver::BlockRef {
                                            name: b.name.clone(),
                                            sid: b.sid.clone(),
                                        },
                                    ));
                                }
                                (cfg.step_size, None)
                            } else {
                                (0.001, None)
                            }
                        }
                    };
                    let nw = (t_period / ts).round() as usize;
                    if nw < 4 {
                        return Err(SimError::parameter(format!(
                            "Window sample count Nw ({nw}) is less than 4 on block '{}': FundamentalFrequency ({f_base} Hz) is too high for SampleTime ({ts} s) (E-PARAM).",
                            b.name
                        )));
                    }
                    if ((t_period / ts) - (nw as f64)).abs() > 1e-6 * (nw as f64) {
                        warnings.push(crate::diagnostics::Diagnostic::warning(
                            "W-WINDOW",
                            format!(
                                "Window sample count ({nw}) on block '{}' is not an integer number of samples per period ({:.4}): ripple expected in measurement outputs.",
                                b.name,
                                t_period / ts
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                    let h_param = b
                        .get_param("MaxHarmonic")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(40);
                    let h_clamped = h_param.min((nw - 1) / 2).max(1);
                    if h_param > (nw - 1) / 2 {
                        warnings.push(crate::diagnostics::Diagnostic::warning(
                            "W-THD-LIMIT",
                            format!(
                                "MaxHarmonic ({h_param}) on block '{}' exceeds Nyquist limit (Nw - 1) / 2 = {h_clamped}: clamped to {h_clamped}.",
                                b.name
                            ),
                            vec![b.sid.clone()],
                        ));
                    }
                    let as_percent = b.get_param("Output").map(|s| s != "Ratio").unwrap_or(true);
                    let buf_idx = initial_thd_buffers.len();
                    initial_thd_buffers.push(ThdBuffer::new(nw, h_clamped, as_percent));
                    CompiledKind::Thd {
                        buffer_idx: buf_idx,
                        sample_time,
                    }
                }
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
                if let Some(src_slot) = src_opt {
                    let u = slot_to_block[src_slot];
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
            block_output_offsets,
            total_output_slots,
            initial_rms_buffers,
            initial_mean_buffers,
            initial_power_buffers,
            initial_thd_buffers,
            warnings,
        })
    }

    /// Pure evaluation function: computes algebraic outputs and continuous derivatives
    /// at time t without mutating engine or solver state. Returns an E-NAN error if any
    /// output or derivative is non-finite.
    pub fn eval(
        &self,
        t: f64,
        continuous_states: &[f64],
        discrete_states: &[f64],
        workspace: &mut SimWorkspace,
    ) -> Result<(), SimError> {
        let outputs = &mut workspace.outputs;

        // Evaluate all blocks in precomputed topological schedule order
        for &idx in &self.schedule {
            let b = &self.compiled_blocks[idx];
            let slot = self.block_output_offsets[idx];
            match &b.kind {
                CompiledKind::Constant { value } => {
                    outputs[slot] = *value;
                }
                CompiledKind::SineWave {
                    amp,
                    freq_rad,
                    phase,
                    bias,
                } => {
                    outputs[slot] = amp * (freq_rad * t + phase).sin() + bias;
                }
                CompiledKind::Step {
                    step_time,
                    before,
                    after,
                } => {
                    outputs[slot] = if t >= *step_time { *after } else { *before };
                }
                CompiledKind::Sawtooth {
                    amp,
                    freq_hz,
                    phase,
                    bias,
                } => {
                    let period = 1.0 / freq_hz;
                    let t_eff = t + phase / (2.0 * std::f64::consts::PI * freq_hz);
                    let tau = (t_eff.rem_euclid(period)) / period;
                    // Normalized ramp from -1 to +1
                    let saw = 2.0 * tau - 1.0;
                    outputs[slot] = amp * saw + bias;
                }
                CompiledKind::Triangle {
                    amp,
                    freq_hz,
                    phase,
                    bias,
                } => {
                    let period = 1.0 / freq_hz;
                    let t_eff = t + phase / (2.0 * std::f64::consts::PI * freq_hz);
                    let tau = (t_eff.rem_euclid(period)) / period;
                    let tri = if tau < 0.5 {
                        -1.0 + 4.0 * tau
                    } else {
                        3.0 - 4.0 * tau
                    };
                    outputs[slot] = amp * tri + bias;
                }
                CompiledKind::PulseGenerator {
                    amp,
                    period,
                    duty,
                    delay,
                } => {
                    if t < *delay {
                        outputs[slot] = 0.0;
                    } else {
                        let t_on = period * (duty / 100.0);
                        let tau = (t - delay).rem_euclid(*period);
                        outputs[slot] = if tau < t_on { *amp } else { 0.0 };
                    }
                }
                CompiledKind::RepeatingSequence {
                    time_values,
                    output_values,
                    period,
                } => {
                    let t0 = time_values[0];
                    let t_mod = (t - t0).rem_euclid(*period) + t0;
                    let mut y_val = output_values[0];
                    for k in 0..time_values.len() - 1 {
                        let t_a = time_values[k];
                        let t_b = time_values[k + 1];
                        if t_mod >= t_a && (t_mod <= t_b || k == time_values.len() - 2) {
                            let frac = if (t_b - t_a).abs() > 1e-12 {
                                (t_mod - t_a) / (t_b - t_a)
                            } else {
                                0.0
                            };
                            y_val =
                                output_values[k] + frac * (output_values[k + 1] - output_values[k]);
                            break;
                        }
                    }
                    outputs[slot] = y_val;
                }
                CompiledKind::RepeatingSequenceStair {
                    time_values,
                    output_values,
                    period,
                } => {
                    let t0 = time_values[0];
                    let t_mod = (t - t0).rem_euclid(*period) + t0;
                    let mut y_val = output_values[0];
                    for k in 0..time_values.len() - 1 {
                        let t_a = time_values[k];
                        let t_b = time_values[k + 1];
                        if t_mod >= t_a && t_mod < t_b {
                            y_val = output_values[k];
                            break;
                        }
                    }
                    if let (Some(&t_last), Some(&y_last)) =
                        (time_values.last(), output_values.last())
                    {
                        if t_mod >= t_last {
                            y_val = y_last;
                        }
                    }
                    outputs[slot] = y_val;
                }
                CompiledKind::Clock => {
                    outputs[slot] = t;
                }
                CompiledKind::Integrator { state_idx } => {
                    outputs[slot] = continuous_states[*state_idx];
                }
                CompiledKind::UnitDelay { state_idx, .. } => {
                    outputs[slot] = discrete_states[*state_idx];
                }
                CompiledKind::TransferFunction {
                    state_idx,
                    num_states,
                    c_coeffs,
                    d_coeff,
                    ..
                } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let mut y_val = d_coeff * u;
                    for j in 0..*num_states {
                        y_val += c_coeffs[j] * continuous_states[state_idx + j];
                    }
                    outputs[slot] = y_val;
                }
                CompiledKind::DiscreteTransferFunction {
                    state_idx,
                    num_states,
                    c_coeffs,
                    d_coeff,
                    ..
                } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let mut y_val = d_coeff * u;
                    for j in 0..*num_states {
                        y_val += c_coeffs[j] * discrete_states[state_idx + j];
                    }
                    outputs[slot] = y_val;
                }
                CompiledKind::Pid {
                    is_discrete,
                    state_idx_i,
                    state_idx_d,
                    kp,
                    kd,
                    n,
                    upper_limit,
                    lower_limit,
                    ..
                } => {
                    let e = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let (x_i, x_d) = if *is_discrete {
                        (discrete_states[*state_idx_i], discrete_states[*state_idx_d])
                    } else {
                        (
                            continuous_states[*state_idx_i],
                            continuous_states[*state_idx_d],
                        )
                    };
                    let p_term = kp * e;
                    let d_term = if *kd != 0.0 {
                        kd * n * (e - n * x_d)
                    } else {
                        0.0
                    };
                    let mut v = p_term + x_i + d_term;
                    if let Some(upper) = upper_limit {
                        v = v.min(*upper);
                    }
                    if let Some(lower) = lower_limit {
                        v = v.max(*lower);
                    }
                    outputs[slot] = v;
                }
                CompiledKind::ZeroOrderHold { state_idx, .. } => {
                    outputs[slot] = discrete_states[*state_idx];
                }
                CompiledKind::Gain { gain } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[slot] = gain * u;
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
                    outputs[slot] = sum;
                }
                CompiledKind::Product => {
                    let mut prod = 1.0;
                    let mut has_input = false;
                    for opt in &b.input_sources {
                        let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                        prod *= u;
                        has_input = true;
                    }
                    outputs[slot] = if has_input { prod } else { 0.0 };
                }
                CompiledKind::RelationalOperator { op } => {
                    let u1 = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let u2 = b
                        .input_sources
                        .get(1)
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[slot] = if op.evaluate(u1, u2) { 1.0 } else { 0.0 };
                }
                CompiledKind::LogicalOperator { op } => {
                    let res = match op {
                        LogOp::Not => {
                            let u = b
                                .input_sources
                                .first()
                                .and_then(|opt| *opt)
                                .map(|src| outputs[src])
                                .unwrap_or(0.0);
                            u.abs() < 1e-12
                        }
                        LogOp::And => {
                            let mut ok = true;
                            for opt in &b.input_sources {
                                let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                                if u.abs() < 1e-12 {
                                    ok = false;
                                    break;
                                }
                            }
                            ok
                        }
                        LogOp::Or => {
                            let mut ok = false;
                            for opt in &b.input_sources {
                                let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                                if u.abs() >= 1e-12 {
                                    ok = true;
                                    break;
                                }
                            }
                            ok
                        }
                        LogOp::Nand => {
                            let mut ok = true;
                            for opt in &b.input_sources {
                                let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                                if u.abs() < 1e-12 {
                                    ok = false;
                                    break;
                                }
                            }
                            !ok
                        }
                        LogOp::Nor => {
                            let mut ok = false;
                            for opt in &b.input_sources {
                                let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                                if u.abs() >= 1e-12 {
                                    ok = true;
                                    break;
                                }
                            }
                            !ok
                        }
                        LogOp::Xor => {
                            let mut true_count = 0;
                            for opt in &b.input_sources {
                                let u = opt.map(|src| outputs[src]).unwrap_or(0.0);
                                if u.abs() >= 1e-12 {
                                    true_count += 1;
                                }
                            }
                            true_count % 2 == 1
                        }
                    };
                    outputs[slot] = if res { 1.0 } else { 0.0 };
                }
                CompiledKind::Switch {
                    criteria,
                    threshold,
                } => {
                    let u1 = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let u2 = b
                        .input_sources
                        .get(1)
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let u3 = b
                        .input_sources
                        .get(2)
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[slot] = if criteria.is_pass_first(u2, *threshold) {
                        u1
                    } else {
                        u3
                    };
                }
                CompiledKind::Abs => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[slot] = u.abs();
                }
                CompiledKind::MathFunction { func } => {
                    let u1 = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let u2 = b
                        .input_sources
                        .get(1)
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src]);
                    outputs[slot] = func.evaluate(u1, u2);
                }
                CompiledKind::Saturation { lower, upper } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    outputs[slot] = u.clamp(*lower, *upper);
                }
                CompiledKind::Scope | CompiledKind::ToWorkspace | CompiledKind::Outport => {
                    outputs[slot] = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                }
                CompiledKind::Sensor => {
                    // Sensor measurement is computed and stored by the electrical circuit solver.
                }
                CompiledKind::Rms { buffer_idx, .. } => {
                    outputs[slot] = workspace.rms_buffers[*buffer_idx].eval_output();
                }
                CompiledKind::Mean { buffer_idx, .. } => {
                    outputs[slot] = workspace.mean_buffers[*buffer_idx].eval_output();
                }
                CompiledKind::PowerMeasurement { buffer_idx, .. } => {
                    for p in 0..5 {
                        outputs[slot + p] = workspace.power_buffers[*buffer_idx].eval_output(p);
                    }
                }
                CompiledKind::Thd { buffer_idx, .. } => {
                    outputs[slot] = workspace.thd_buffers[*buffer_idx].eval_output();
                }
                _ => {
                    outputs[slot] = 0.0;
                }
            }

            let num_outs = match &b.kind {
                CompiledKind::PowerMeasurement { .. } => 5,
                _ => 1,
            };
            for p in 0..num_outs {
                if !outputs[slot + p].is_finite() {
                    return Err(SimError::non_finite(
                        Some(BlockRef {
                            sid: b.sid.clone(),
                            name: b.name.clone(),
                        }),
                        outputs[slot + p],
                        t,
                        "output",
                    ));
                }
            }
        }

        // Evaluate continuous state derivatives
        let derivatives = &mut workspace.derivatives;
        for b in &self.compiled_blocks {
            match &b.kind {
                CompiledKind::Integrator { state_idx } => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    if !u.is_finite() {
                        return Err(SimError::non_finite(
                            Some(BlockRef {
                                sid: b.sid.clone(),
                                name: b.name.clone(),
                            }),
                            u,
                            t,
                            "derivative",
                        ));
                    }
                    derivatives[*state_idx] = u;
                }
                CompiledKind::TransferFunction {
                    state_idx,
                    num_states,
                    a_coeffs,
                    ..
                } => {
                    if *num_states > 0 {
                        let u = b
                            .input_sources
                            .first()
                            .and_then(|opt| *opt)
                            .map(|src| outputs[src])
                            .unwrap_or(0.0);
                        let x = &continuous_states[*state_idx..*state_idx + *num_states];
                        for j in 0..*num_states - 1 {
                            derivatives[*state_idx + j] = x[j + 1];
                        }
                        let mut dx_last = u;
                        for j in 0..*num_states {
                            dx_last -= a_coeffs[j] * x[j];
                        }
                        derivatives[*state_idx + *num_states - 1] = dx_last;
                    }
                }
                CompiledKind::Pid {
                    is_discrete: false,
                    state_idx_i,
                    state_idx_d,
                    kp,
                    ki,
                    kd,
                    n,
                    upper_limit,
                    lower_limit,
                    clamping,
                    ..
                } => {
                    let e = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let x_i = continuous_states[*state_idx_i];
                    let x_d = continuous_states[*state_idx_d];
                    let p_term = kp * e;
                    let d_term = if *kd != 0.0 {
                        kd * n * (e - n * x_d)
                    } else {
                        0.0
                    };
                    let v = p_term + x_i + d_term;

                    let allow_integrate = if *clamping {
                        let mut ok = true;
                        if let Some(upper) = upper_limit {
                            if v > *upper && e > 0.0 {
                                ok = false;
                            }
                        }
                        if let Some(lower) = lower_limit {
                            if v < *lower && e < 0.0 {
                                ok = false;
                            }
                        }
                        ok
                    } else {
                        true
                    };

                    derivatives[*state_idx_i] = if allow_integrate { ki * e } else { 0.0 };
                    derivatives[*state_idx_d] = if *kd != 0.0 { -n * x_d + e } else { 0.0 };
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Discrete state update: updates UnitDelay states unconditionally from current outputs.
    /// Called once per accepted major simulation step in fixed-step without sample times.
    pub fn update_discrete(&self, outputs: &[f64], discrete_states: &mut [f64]) {
        for b in &self.compiled_blocks {
            if let CompiledKind::UnitDelay { state_idx, .. } = b.kind {
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

    /// Finds the earliest scheduled breakpoint strictly after time t.
    pub fn next_breakpoint(&self, t: f64) -> Option<f64> {
        let mut next: Option<f64> = None;
        for b in &self.compiled_blocks {
            match &b.kind {
                CompiledKind::Step { step_time, .. } => {
                    if *step_time > t + 1e-12 {
                        next = Some(next.map_or(*step_time, |n| n.min(*step_time)));
                    }
                }
                CompiledKind::PulseGenerator {
                    period,
                    duty,
                    delay,
                    ..
                } => {
                    let t_on = period * (duty / 100.0);
                    // Check edges around t
                    if *delay > t + 1e-12 {
                        next = Some(next.map_or(*delay, |n| n.min(*delay)));
                    } else {
                        let k_est = ((t - delay) / period).floor() as i64;
                        for k in k_est..=(k_est + 2) {
                            if k < 0 {
                                continue;
                            }
                            let t_rise = delay + (k as f64) * period;
                            let t_fall = t_rise + t_on;
                            if t_rise > t + 1e-12 {
                                next = Some(next.map_or(t_rise, |n| n.min(t_rise)));
                            }
                            if t_fall > t + 1e-12 {
                                next = Some(next.map_or(t_fall, |n| n.min(t_fall)));
                            }
                        }
                    }
                }
                CompiledKind::RepeatingSequence {
                    time_values,
                    period,
                    ..
                }
                | CompiledKind::RepeatingSequenceStair {
                    time_values,
                    period,
                    ..
                } => {
                    let t0 = time_values[0];
                    if t0 > t + 1e-12 {
                        next = Some(next.map_or(t0, |n| n.min(t0)));
                    } else {
                        let k_est = ((t - t0) / period).floor() as i64;
                        for k in k_est..=(k_est + 2) {
                            if k < 0 {
                                continue;
                            }
                            let base = t0 + (k as f64) * period;
                            for &knot_dt in time_values {
                                let knot_t = base + (knot_dt - t0);
                                if knot_t > t + 1e-12 {
                                    next = Some(next.map_or(knot_t, |n| n.min(knot_t)));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        next
    }

    /// Validates discrete block sample times according to the active solver rules.
    pub fn validate_sample_times(&self, config: &SimulationConfig) -> Result<(), SimError> {
        for b in &self.compiled_blocks {
            let sample_time = match &b.kind {
                CompiledKind::UnitDelay { sample_time, .. } => *sample_time,
                CompiledKind::DiscreteTransferFunction { sample_time, .. } => *sample_time,
                CompiledKind::ZeroOrderHold { sample_time, .. } => *sample_time,
                CompiledKind::Pid {
                    is_discrete: true,
                    sample_time,
                    ..
                } => *sample_time,
                CompiledKind::Rms { sample_time, .. }
                | CompiledKind::Mean { sample_time, .. }
                | CompiledKind::PowerMeasurement { sample_time, .. }
                | CompiledKind::Thd { sample_time, .. } => *sample_time,
                _ => continue,
            };

            let blk = BlockRef {
                sid: b.sid.clone(),
                name: b.name.clone(),
            };

            match config.solver {
                SolverType::VariableStepDormandPrince => match sample_time {
                    None => return Err(SimError::sample_time_missing(blk)),
                    Some(ts) if ts <= 0.0 || !ts.is_finite() => {
                        return Err(SimError::sample_time_invalid(blk, ts));
                    }
                    _ => {}
                },
                SolverType::FixedStepEuler | SolverType::FixedStepRk4 => {
                    if let Some(ts) = sample_time {
                        if ts <= 0.0 || !ts.is_finite() {
                            return Err(SimError::sample_time_invalid(blk, ts));
                        }
                        let m = (ts / config.step_size).round();
                        if m < 1.0 || ((ts - m * config.step_size).abs() / config.step_size) > 1e-9
                        {
                            return Err(SimError::sample_time_not_multiple(
                                blk,
                                ts,
                                config.step_size,
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Finds the next discrete sample hit strictly after time t.
    pub fn next_sample_hit(&self, t: f64, t_start: f64, _config: &SimulationConfig) -> Option<f64> {
        let mut next_hit: Option<f64> = None;
        for b in &self.compiled_blocks {
            let sample_time = match &b.kind {
                CompiledKind::UnitDelay {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::DiscreteTransferFunction {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::ZeroOrderHold {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::Pid {
                    is_discrete: true,
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::Rms {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::Mean {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::PowerMeasurement {
                    sample_time: Some(ts),
                    ..
                }
                | CompiledKind::Thd {
                    sample_time: Some(ts),
                    ..
                } => *ts,
                _ => continue,
            };

            let k = ((t - t_start) / sample_time + 1e-9).floor() as i64 + 1;
            let tk = t_start + (k as f64) * sample_time;
            if tk > t + 1e-12 {
                next_hit = Some(next_hit.map_or(tk, |n| n.min(tk)));
            }
        }
        next_hit
    }

    /// Checks whether the given block with optional sample_time hits at time t.
    pub fn is_sample_hit(
        &self,
        t: f64,
        t_start: f64,
        sample_time: Option<f64>,
        _config: &SimulationConfig,
    ) -> bool {
        match sample_time {
            None => true,
            Some(ts) => {
                let k = ((t - t_start) / ts).round();
                ((t - t_start) - k * ts).abs() < 1e-8
            }
        }
    }

    /// Discrete state update with sample hit checking: updates UnitDelay, DiscreteTransferFunction,
    /// ZeroOrderHold, and discrete PID states only for blocks whose sample hit occurs at time t on accepted steps.
    pub fn update_discrete_at(
        &self,
        t: f64,
        t_start: f64,
        outputs: &[f64],
        discrete_states: &mut [f64],
        config: &SimulationConfig,
    ) {
        for b in &self.compiled_blocks {
            match &b.kind {
                CompiledKind::UnitDelay {
                    state_idx,
                    sample_time,
                } => {
                    if self.is_sample_hit(t, t_start, *sample_time, config) {
                        let u = b
                            .input_sources
                            .first()
                            .and_then(|opt| *opt)
                            .map(|src| outputs[src])
                            .unwrap_or(0.0);
                        discrete_states[*state_idx] = u;
                    }
                }
                CompiledKind::ZeroOrderHold {
                    state_idx,
                    sample_time,
                } => {
                    if self.is_sample_hit(t, t_start, *sample_time, config) {
                        let u = b
                            .input_sources
                            .first()
                            .and_then(|opt| *opt)
                            .map(|src| outputs[src])
                            .unwrap_or(0.0);
                        discrete_states[*state_idx] = u;
                    }
                }
                CompiledKind::DiscreteTransferFunction {
                    state_idx,
                    num_states,
                    a_coeffs,
                    sample_time,
                    ..
                } => {
                    if *num_states > 0 && self.is_sample_hit(t, t_start, *sample_time, config) {
                        let u = b
                            .input_sources
                            .first()
                            .and_then(|opt| *opt)
                            .map(|src| outputs[src])
                            .unwrap_or(0.0);
                        let mut dx_last = u;
                        for j in 0..*num_states {
                            dx_last -= a_coeffs[j] * discrete_states[*state_idx + j];
                        }
                        for j in 0..*num_states - 1 {
                            discrete_states[*state_idx + j] = discrete_states[*state_idx + j + 1];
                        }
                        discrete_states[*state_idx + *num_states - 1] = dx_last;
                    }
                }
                CompiledKind::Pid {
                    is_discrete: true,
                    state_idx_i,
                    state_idx_d,
                    kp,
                    ki,
                    kd,
                    n,
                    upper_limit,
                    lower_limit,
                    clamping,
                    sample_time,
                } if self.is_sample_hit(t, t_start, *sample_time, config) => {
                    let e = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|src| outputs[src])
                        .unwrap_or(0.0);
                    let dt = sample_time.unwrap_or(config.step_size);
                    let x_i = discrete_states[*state_idx_i];
                    let x_d = discrete_states[*state_idx_d];
                    let p_term = kp * e;
                    let d_term = if *kd != 0.0 {
                        kd * n * (e - n * x_d)
                    } else {
                        0.0
                    };
                    let v = p_term + x_i + d_term;

                    let allow_integrate = if *clamping {
                        let mut ok = true;
                        if let Some(upper) = upper_limit {
                            if v > *upper && e > 0.0 {
                                ok = false;
                            }
                        }
                        if let Some(lower) = lower_limit {
                            if v < *lower && e < 0.0 {
                                ok = false;
                            }
                        }
                        ok
                    } else {
                        true
                    };

                    let d_xi = if allow_integrate { ki * e } else { 0.0 };
                    let d_xd = if *kd != 0.0 { -n * x_d + e } else { 0.0 };

                    discrete_states[*state_idx_i] = x_i + dt * d_xi;
                    discrete_states[*state_idx_d] = x_d + dt * d_xd;
                }
                _ => {}
            }
        }
    }

    /// Streaming update for running measurement blocks (Rms, Mean, PowerMeasurement, Thd).
    pub fn update_measurements_at(
        &self,
        t: f64,
        t_start: f64,
        workspace: &mut SimWorkspace,
        config: &SimulationConfig,
    ) {
        let outputs = &workspace.outputs;
        for b in &self.compiled_blocks {
            match &b.kind {
                CompiledKind::Rms {
                    buffer_idx,
                    sample_time,
                } if self.is_sample_hit(t, t_start, *sample_time, config) => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|slot| outputs[slot])
                        .unwrap_or(0.0);
                    workspace.rms_buffers[*buffer_idx].update(u);
                }
                CompiledKind::Mean {
                    buffer_idx,
                    sample_time,
                } if self.is_sample_hit(t, t_start, *sample_time, config) => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|slot| outputs[slot])
                        .unwrap_or(0.0);
                    workspace.mean_buffers[*buffer_idx].update(u);
                }
                CompiledKind::PowerMeasurement {
                    buffer_idx,
                    sample_time,
                } if self.is_sample_hit(t, t_start, *sample_time, config) => {
                    let v = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|slot| outputs[slot])
                        .unwrap_or(0.0);
                    let i = b
                        .input_sources
                        .get(1)
                        .and_then(|opt| *opt)
                        .map(|slot| outputs[slot])
                        .unwrap_or(0.0);
                    workspace.power_buffers[*buffer_idx].update(v, i);
                }
                CompiledKind::Thd {
                    buffer_idx,
                    sample_time,
                } if self.is_sample_hit(t, t_start, *sample_time, config) => {
                    let u = b
                        .input_sources
                        .first()
                        .and_then(|opt| *opt)
                        .map(|slot| outputs[slot])
                        .unwrap_or(0.0);
                    workspace.thd_buffers[*buffer_idx].update(u);
                }
                _ => {}
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
