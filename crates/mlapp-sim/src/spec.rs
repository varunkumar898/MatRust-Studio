//! Single source of truth: per-block-type specification table.
//!
//! Drives the editor, validator, documentation, and code generator without duplicated lists.

use crate::diagnostics::Diagnostic;
use crate::model::{BlockRotation, BlockType, PortKind};
use std::collections::HashMap;

/// Parameter data type and editing representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Numeric,
    Text,
    Choice(&'static [&'static str]),
    Vector,
    SumInputs,
}

/// Specification for a single block parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub default_val: &'static str,
    pub kind: ParamKind,
    pub doc: &'static str,
}

/// Full specification for a block type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpec {
    pub block_type: BlockType,
    pub display_name: &'static str,
    pub category: &'static str,
    pub params: &'static [ParamSpec],
    pub min_inputs: usize,
    pub max_inputs: usize,
    pub outputs: usize,
    pub default_feedthrough: bool,
    pub doc: &'static str,
}

/// Parses a comma- or whitespace-separated list of floating-point numbers.
pub fn parse_float_vector(s: &str) -> Result<Vec<f64>, String> {
    let trimmed = s
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut values = Vec::new();
    let parts: Vec<&str> = if trimmed.contains(',') {
        trimmed.split(',').collect()
    } else {
        trimmed.split_whitespace().collect()
    };

    for part in parts {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let val = p
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse float '{p}': {e}"))?;
        values.push(val);
    }
    Ok(values)
}

/// Master block specification table.
static BLOCK_SPECS: &[BlockSpec] = &[
    // --- Sources ---
    BlockSpec {
        block_type: BlockType::Constant,
        display_name: "Constant",
        category: "Sources",
        params: &[ParamSpec {
            key: "Value",
            label: "Constant Value",
            unit: "",
            default_val: "1.0",
            kind: ParamKind::Numeric,
            doc: "Numeric output scalar constant value.",
        }],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs a constant scalar value.",
    },
    BlockSpec {
        block_type: BlockType::SineWave,
        display_name: "Sine Wave",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "Amplitude",
                label: "Amplitude",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Peak amplitude of sinusoidal waveform.",
            },
            ParamSpec {
                key: "Frequency",
                label: "Frequency",
                unit: "rad/s",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Frequency of sinusoidal waveform.",
            },
            ParamSpec {
                key: "Units",
                label: "Frequency Units",
                unit: "",
                default_val: "rad/s",
                kind: ParamKind::Choice(&["rad/s", "Hz"]),
                doc: "Units for frequency: 'rad/s' or 'Hz'.",
            },
            ParamSpec {
                key: "Phase",
                label: "Phase",
                unit: "rad",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Phase shift in radians.",
            },
            ParamSpec {
                key: "Bias",
                label: "Bias",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "DC bias offset added to waveform.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs a sinusoidal waveform.",
    },
    BlockSpec {
        block_type: BlockType::Step,
        display_name: "Step",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "Time",
                label: "Step Time",
                unit: "s",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Time in seconds at which the step transition occurs.",
            },
            ParamSpec {
                key: "Before",
                label: "Initial Value",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Output value before step time.",
            },
            ParamSpec {
                key: "After",
                label: "Final Value",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Output value at and after step time.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs a step transition at the specified time.",
    },
    BlockSpec {
        block_type: BlockType::Sawtooth,
        display_name: "Sawtooth",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "Amplitude",
                label: "Amplitude",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Peak amplitude of the sawtooth wave.",
            },
            ParamSpec {
                key: "Frequency",
                label: "Frequency",
                unit: "Hz",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Frequency in Hertz.",
            },
            ParamSpec {
                key: "Phase",
                label: "Phase",
                unit: "rad",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Phase shift in radians.",
            },
            ParamSpec {
                key: "Bias",
                label: "Bias",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "DC bias offset.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Generates a periodic sawtooth wave.",
    },
    BlockSpec {
        block_type: BlockType::Triangle,
        display_name: "Triangle",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "Amplitude",
                label: "Amplitude",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Peak amplitude of triangle wave.",
            },
            ParamSpec {
                key: "Frequency",
                label: "Frequency",
                unit: "Hz",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Frequency in Hertz.",
            },
            ParamSpec {
                key: "Phase",
                label: "Phase",
                unit: "rad",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Phase shift in radians.",
            },
            ParamSpec {
                key: "Bias",
                label: "Bias",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "DC bias offset.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Generates a periodic triangle wave.",
    },
    BlockSpec {
        block_type: BlockType::PulseGenerator,
        display_name: "Pulse Generator",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "Amplitude",
                label: "Amplitude",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Pulse height.",
            },
            ParamSpec {
                key: "Period",
                label: "Period",
                unit: "s",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Pulse period in seconds.",
            },
            ParamSpec {
                key: "Duty",
                label: "Duty Cycle",
                unit: "%",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Percentage of period pulse is high (0 to 100).",
            },
            ParamSpec {
                key: "Delay",
                label: "Phase Delay",
                unit: "s",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Delay time before first pulse starts.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Generates periodic square pulses.",
    },
    BlockSpec {
        block_type: BlockType::RepeatingSequence,
        display_name: "Repeating Sequence",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "TimeValues",
                label: "Time Values",
                unit: "s",
                default_val: "[0, 1, 2]",
                kind: ParamKind::Vector,
                doc: "Monotonically increasing time values in seconds.",
            },
            ParamSpec {
                key: "OutputValues",
                label: "Output Values",
                unit: "",
                default_val: "[0, 2, 0]",
                kind: ParamKind::Vector,
                doc: "Signal values corresponding to time points (linearly interpolated).",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs a periodically repeating piecewise linear waveform.",
    },
    BlockSpec {
        block_type: BlockType::RepeatingSequenceStair,
        display_name: "Repeating Sequence Stair",
        category: "Sources",
        params: &[
            ParamSpec {
                key: "TimeValues",
                label: "Time Values",
                unit: "s",
                default_val: "[0, 1, 2]",
                kind: ParamKind::Vector,
                doc: "Monotonically increasing time values in seconds.",
            },
            ParamSpec {
                key: "OutputValues",
                label: "Output Values",
                unit: "",
                default_val: "[0, 2, 0]",
                kind: ParamKind::Vector,
                doc: "Signal values held constant between time points.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs a periodically repeating staircase waveform.",
    },
    BlockSpec {
        block_type: BlockType::Clock,
        display_name: "Clock",
        category: "Sources",
        params: &[],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Outputs current simulation time.",
    },
    // --- Sinks ---
    BlockSpec {
        block_type: BlockType::Scope,
        display_name: "Scope",
        category: "Sinks",
        params: &[],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: true,
        doc: "Visual scope display and time-domain signal logging.",
    },
    BlockSpec {
        block_type: BlockType::ToWorkspace,
        display_name: "To Workspace",
        category: "Sinks",
        params: &[ParamSpec {
            key: "VariableName",
            label: "Variable Name",
            unit: "",
            default_val: "simout",
            kind: ParamKind::Text,
            doc: "Name of target workspace variable for logged signal array.",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: true,
        doc: "Logs time-series signals directly to workspace variables.",
    },
    BlockSpec {
        block_type: BlockType::Outport,
        display_name: "Outport",
        category: "Sinks",
        params: &[ParamSpec {
            key: "Port",
            label: "Port Number",
            unit: "",
            default_val: "1",
            kind: ParamKind::Numeric,
            doc: "Index of output port in subsystem or top-level interface.",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: true,
        doc: "Subsystem external output connection terminal.",
    },
    // --- Math ---
    BlockSpec {
        block_type: BlockType::Gain,
        display_name: "Gain",
        category: "Math",
        params: &[ParamSpec {
            key: "Gain",
            label: "Gain Multiplier",
            unit: "",
            default_val: "1.0",
            kind: ParamKind::Numeric,
            doc: "Element-wise scalar gain factor: y = K * u.",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: true,
        doc: "Multiplies input by a constant gain factor.",
    },
    BlockSpec {
        block_type: BlockType::Sum,
        display_name: "Sum",
        category: "Math",
        params: &[ParamSpec {
            key: "Inputs",
            label: "List of Signs (+/-)",
            unit: "",
            default_val: "+-",
            kind: ParamKind::SumInputs,
            doc: "Sequence of '+' and '-' characters defining input port signs and count.",
        }],
        min_inputs: 1,
        max_inputs: 16,
        outputs: 1,
        default_feedthrough: true,
        doc: "Adds or subtracts inputs according to port signs.",
    },
    BlockSpec {
        block_type: BlockType::Product,
        display_name: "Product",
        category: "Math",
        params: &[],
        min_inputs: 2,
        max_inputs: 16,
        outputs: 1,
        default_feedthrough: true,
        doc: "Multiplies multiple input signals together.",
    },
    BlockSpec {
        block_type: BlockType::Abs,
        display_name: "Abs",
        category: "Math",
        params: &[],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: true,
        doc: "Outputs the absolute value of the input: y = |u|.",
    },
    BlockSpec {
        block_type: BlockType::MathFunction,
        display_name: "Math Function",
        category: "Math",
        params: &[ParamSpec {
            key: "Function",
            label: "Function",
            unit: "",
            default_val: "sqrt",
            kind: ParamKind::Choice(&["sqrt", "sin", "cos", "exp", "log", "square", "pow"]),
            doc: "Mathematical function to evaluate on input signal.",
        }],
        min_inputs: 1,
        max_inputs: 2,
        outputs: 1,
        default_feedthrough: true,
        doc: "Applies a standard mathematical function to the input signal.",
    },
    BlockSpec {
        block_type: BlockType::Saturation,
        display_name: "Saturation",
        category: "Discontinuities",
        params: &[
            ParamSpec {
                key: "UpperLimit",
                label: "Upper Limit",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Upper saturation limit (must be >= Lower Limit).",
            },
            ParamSpec {
                key: "LowerLimit",
                label: "Lower Limit",
                unit: "",
                default_val: "-1.0",
                kind: ParamKind::Numeric,
                doc: "Lower saturation limit (must be <= Upper Limit).",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: true,
        doc: "Clamps input signal within upper and lower limits.",
    },
    // --- Continuous & Control ---
    BlockSpec {
        block_type: BlockType::Integrator,
        display_name: "Integrator",
        category: "Continuous",
        params: &[ParamSpec {
            key: "InitialCondition",
            label: "Initial Condition",
            unit: "",
            default_val: "0.0",
            kind: ParamKind::Numeric,
            doc: "Initial continuous state value at t = t_start.",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Integrates input signal with respect to time: dx/dt = u.",
    },
    BlockSpec {
        block_type: BlockType::TransferFunction,
        display_name: "Transfer Fcn",
        category: "Continuous",
        params: &[
            ParamSpec {
                key: "Numerator",
                label: "Numerator",
                unit: "",
                default_val: "1",
                kind: ParamKind::Vector,
                doc: "Numerator polynomial coefficients in descending powers of s.",
            },
            ParamSpec {
                key: "Denominator",
                label: "Denominator",
                unit: "",
                default_val: "1, 1",
                kind: ParamKind::Vector,
                doc: "Denominator polynomial coefficients in descending powers of s.",
            },
            ParamSpec {
                key: "InitialCondition",
                label: "Initial Condition",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Initial condition for internal state variables.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false, // Strictly proper is false; proper with deg(num)==deg(den) is true
        doc: "Continuous linear transfer function modeled in controllable canonical form.",
    },
    BlockSpec {
        block_type: BlockType::Pid,
        display_name: "PID Controller",
        category: "Continuous",
        params: &[
            ParamSpec {
                key: "P",
                label: "Proportional (P)",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Proportional gain Kp.",
            },
            ParamSpec {
                key: "I",
                label: "Integral (I)",
                unit: "",
                default_val: "1.0",
                kind: ParamKind::Numeric,
                doc: "Integral gain Ki.",
            },
            ParamSpec {
                key: "D",
                label: "Derivative (D)",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Derivative gain Kd.",
            },
            ParamSpec {
                key: "N",
                label: "Filter Coefficient (N)",
                unit: "",
                default_val: "100.0",
                kind: ParamKind::Numeric,
                doc: "Derivative low-pass filter pole coefficient N.",
            },
            ParamSpec {
                key: "TimeDomain",
                label: "Time Domain",
                unit: "",
                default_val: "Continuous",
                kind: ParamKind::Choice(&["Continuous", "Discrete"]),
                doc: "Operating domain: 'Continuous' or 'Discrete'.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "0.01",
                kind: ParamKind::Numeric,
                doc: "Sample time in seconds (required if Discrete).",
            },
            ParamSpec {
                key: "UpperSaturationLimit",
                label: "Upper Saturation Limit",
                unit: "",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Upper output limit (leave blank for none).",
            },
            ParamSpec {
                key: "LowerSaturationLimit",
                label: "Lower Saturation Limit",
                unit: "",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Lower output limit (leave blank for none).",
            },
            ParamSpec {
                key: "AntiWindupMode",
                label: "Anti-Windup Mode",
                unit: "",
                default_val: "Clamping",
                kind: ParamKind::Choice(&["Clamping", "None"]),
                doc: "Integrator anti-windup method.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: true,
        doc: "Proportional-Integral-Derivative controller with anti-windup clamping.",
    },
    // --- Discrete ---
    BlockSpec {
        block_type: BlockType::UnitDelay,
        display_name: "Unit Delay",
        category: "Discrete",
        params: &[
            ParamSpec {
                key: "InitialCondition",
                label: "Initial Condition",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Initial discrete delay state value at t = 0.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Sample period in seconds (required under variable-step solvers).",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Delays signal by one sample period.",
    },
    BlockSpec {
        block_type: BlockType::DiscreteTransferFunction,
        display_name: "Discrete Transfer Fcn",
        category: "Discrete",
        params: &[
            ParamSpec {
                key: "Numerator",
                label: "Numerator",
                unit: "",
                default_val: "1",
                kind: ParamKind::Vector,
                doc: "Numerator polynomial coefficients in descending powers of z.",
            },
            ParamSpec {
                key: "Denominator",
                label: "Denominator",
                unit: "",
                default_val: "1, -0.5",
                kind: ParamKind::Vector,
                doc: "Denominator polynomial coefficients in descending powers of z.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "0.01",
                kind: ParamKind::Numeric,
                doc: "Discrete sample period in seconds.",
            },
            ParamSpec {
                key: "InitialCondition",
                label: "Initial Condition",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Initial state value.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Discrete linear transfer function in z.",
    },
    BlockSpec {
        block_type: BlockType::ZeroOrderHold,
        display_name: "Zero-Order Hold",
        category: "Discrete",
        params: &[ParamSpec {
            key: "SampleTime",
            label: "Sample Time",
            unit: "s",
            default_val: "0.01",
            kind: ParamKind::Numeric,
            doc: "Sample period in seconds.",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Samples and holds input value for the duration of the sample period.",
    },
    // --- Logic & Switching ---
    BlockSpec {
        block_type: BlockType::RelationalOperator,
        display_name: "Relational Operator",
        category: "Logic",
        params: &[ParamSpec {
            key: "Operator",
            label: "Operator",
            unit: "",
            default_val: "<=",
            kind: ParamKind::Choice(&["==", "~=", "<", "<=", ">=", ">"]),
            doc: "Relational comparison condition.",
        }],
        min_inputs: 2,
        max_inputs: 2,
        outputs: 1,
        default_feedthrough: true,
        doc: "Compares two inputs relationally: y = (u1 op u2) ? 1.0 : 0.0.",
    },
    BlockSpec {
        block_type: BlockType::LogicalOperator,
        display_name: "Logical Operator",
        category: "Logic",
        params: &[
            ParamSpec {
                key: "Operator",
                label: "Operator",
                unit: "",
                default_val: "AND",
                kind: ParamKind::Choice(&["AND", "OR", "NAND", "NOR", "XOR", "NOT"]),
                doc: "Boolean logical operator to apply.",
            },
            ParamSpec {
                key: "Inputs",
                label: "Number of Inputs",
                unit: "",
                default_val: "2",
                kind: ParamKind::Numeric,
                doc: "Number of input ports (fixed to 1 for NOT).",
            },
        ],
        min_inputs: 1,
        max_inputs: 16,
        outputs: 1,
        default_feedthrough: true,
        doc: "Performs logical operations on input signals.",
    },
    BlockSpec {
        block_type: BlockType::Switch,
        display_name: "Switch",
        category: "Logic",
        params: &[
            ParamSpec {
                key: "Criteria",
                label: "Criteria",
                unit: "",
                default_val: ">=",
                kind: ParamKind::Choice(&[">=", ">", "~="]),
                doc: "Criteria evaluated against threshold on input port 2.",
            },
            ParamSpec {
                key: "Threshold",
                label: "Threshold",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Threshold value tested against input 2.",
            },
        ],
        min_inputs: 3,
        max_inputs: 3,
        outputs: 1,
        default_feedthrough: true,
        doc: "Passes input 1 when input 2 meets threshold criteria, otherwise passes input 3.",
    },
    // --- Interface & Subsystems ---
    BlockSpec {
        block_type: BlockType::Inport,
        display_name: "Inport",
        category: "Ports",
        params: &[ParamSpec {
            key: "Port",
            label: "Port Number",
            unit: "",
            default_val: "1",
            kind: ParamKind::Numeric,
            doc: "Index of input port in subsystem interface.",
        }],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Subsystem external input connection terminal.",
    },
    BlockSpec {
        block_type: BlockType::Subsystem,
        display_name: "Subsystem",
        category: "Subsystems",
        params: &[],
        min_inputs: 1,
        max_inputs: 16,
        outputs: 1,
        default_feedthrough: true,
        doc: "Encapsulated sub-model diagram.",
    },
    // --- Electrical (Milestone M4) ---
    BlockSpec {
        block_type: BlockType::Resistor,
        display_name: "Resistor",
        category: "Electrical",
        params: &[ParamSpec {
            key: "Resistance",
            label: "Resistance",
            unit: "Ohm",
            default_val: "1000",
            kind: ParamKind::Numeric,
            doc: "Linear resistance value in Ohms (must be > 0).",
        }],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Linear electrical resistor adhering to Ohm's law V = I * R.",
    },
    BlockSpec {
        block_type: BlockType::Capacitor,
        display_name: "Capacitor",
        category: "Electrical",
        params: &[
            ParamSpec {
                key: "Capacitance",
                label: "Capacitance",
                unit: "F",
                default_val: "1e-6",
                kind: ParamKind::Numeric,
                doc: "Capacitance in Farads (must be > 0).",
            },
            ParamSpec {
                key: "InitialVoltage",
                label: "Initial Voltage",
                unit: "V",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Initial capacitor potential difference at t = 0.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Linear electrical capacitor adhering to I = C * dV/dt.",
    },
    BlockSpec {
        block_type: BlockType::Inductor,
        display_name: "Inductor",
        category: "Electrical",
        params: &[
            ParamSpec {
                key: "Inductance",
                label: "Inductance",
                unit: "H",
                default_val: "1e-3",
                kind: ParamKind::Numeric,
                doc: "Inductance in Henries (must be > 0).",
            },
            ParamSpec {
                key: "InitialCurrent",
                label: "Initial Current",
                unit: "A",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Initial inductor branch current at t = 0.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Linear electrical inductor adhering to V = L * dI/dt.",
    },
    BlockSpec {
        block_type: BlockType::DcVoltageSource,
        display_name: "DC Voltage Source",
        category: "Electrical",
        params: &[ParamSpec {
            key: "Voltage",
            label: "Constant Voltage",
            unit: "V",
            default_val: "5.0",
            kind: ParamKind::Numeric,
            doc: "DC output potential difference between positive and negative terminals.",
        }],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Ideal constant DC voltage source.",
    },
    BlockSpec {
        block_type: BlockType::AcVoltageSource,
        display_name: "AC Voltage Source",
        category: "Electrical",
        params: &[
            ParamSpec {
                key: "Amplitude",
                label: "Amplitude",
                unit: "V",
                default_val: "10.0",
                kind: ParamKind::Numeric,
                doc: "Peak sinusoidal voltage amplitude (must be >= 0).",
            },
            ParamSpec {
                key: "Frequency",
                label: "Frequency",
                unit: "Hz",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Sinusoidal frequency in Hertz (must be > 0; use DC source for 0 Hz).",
            },
            ParamSpec {
                key: "Phase",
                label: "Phase",
                unit: "deg",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Phase shift angle in degrees.",
            },
            ParamSpec {
                key: "Offset",
                label: "Offset",
                unit: "V",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "DC bias voltage offset.",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Ideal sinusoidal AC voltage source.",
    },
    BlockSpec {
        block_type: BlockType::DcCurrentSource,
        display_name: "DC Current Source",
        category: "Electrical",
        params: &[ParamSpec {
            key: "Current",
            label: "Constant Current",
            unit: "A",
            default_val: "1.0",
            kind: ParamKind::Numeric,
            doc: "Constant branch current flowing from positive to negative terminal.",
        }],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Ideal constant DC current source.",
    },
    BlockSpec {
        block_type: BlockType::Ground,
        display_name: "Ground",
        category: "Electrical",
        params: &[],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Circuit reference ground node establishing 0 V potential.",
    },
    BlockSpec {
        block_type: BlockType::Diode,
        display_name: "Diode",
        category: "Electrical",
        params: &[
            ParamSpec {
                key: "ForwardVoltage",
                label: "Forward Voltage",
                unit: "V",
                default_val: "0.7",
                kind: ParamKind::Numeric,
                doc: "Forward diode threshold voltage Vf (must be >= 0).",
            },
            ParamSpec {
                key: "OnResistance",
                label: "On Resistance",
                unit: "Ohm",
                default_val: "0.01",
                kind: ParamKind::Numeric,
                doc: "Forward conduction on-resistance Ron (must be > 0).",
            },
            ParamSpec {
                key: "OffResistance",
                label: "Off Resistance",
                unit: "Ohm",
                default_val: "1e6",
                kind: ParamKind::Numeric,
                doc: "Reverse blocking off-resistance Roff (must be > Ron).",
            },
        ],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 0,
        default_feedthrough: false,
        doc: "Piecewise linear electrical diode with forward voltage Vf, Ron, and Roff.",
    },
    BlockSpec {
        block_type: BlockType::IdealSwitch,
        display_name: "Ideal Switch",
        category: "Electrical",
        params: &[
            ParamSpec {
                key: "OnResistance",
                label: "On Resistance",
                unit: "Ohm",
                default_val: "0.01",
                kind: ParamKind::Numeric,
                doc: "Closed switch on-resistance Ron (must be > 0).",
            },
            ParamSpec {
                key: "OffResistance",
                label: "Off Resistance",
                unit: "Ohm",
                default_val: "1e6",
                kind: ParamKind::Numeric,
                doc: "Open switch off-resistance Roff (must be > Ron).",
            },
            ParamSpec {
                key: "Threshold",
                label: "Threshold",
                unit: "",
                default_val: "0.5",
                kind: ParamKind::Numeric,
                doc: "Gate switching control threshold (closed when control input v >= Threshold).",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: false,
        doc: "Controlled bilateral switch operated by signal control input v.",
    },
    BlockSpec {
        block_type: BlockType::VoltageSensor,
        display_name: "Voltage Sensor",
        category: "Electrical",
        params: &[],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Ideal voltmeter with infinite input impedance measuring potential difference Vp - Vn.",
    },
    BlockSpec {
        block_type: BlockType::CurrentSensor,
        display_name: "Current Sensor",
        category: "Electrical",
        params: &[],
        min_inputs: 0,
        max_inputs: 0,
        outputs: 1,
        default_feedthrough: false,
        doc: "Ideal ammeter with zero internal resistance measuring branch current entering terminal p and leaving terminal n.",
    },
    BlockSpec {
        block_type: BlockType::VoltageControlledVoltageSource,
        display_name: "Controlled Voltage Source",
        category: "Electrical",
        params: &[ParamSpec {
            key: "Gain",
            label: "Gain",
            unit: "V/V",
            default_val: "1.0",
            kind: ParamKind::Numeric,
            doc: "Voltage amplification gain K (finite).",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: false,
        doc: "Voltage source whose terminal potential difference Vp - Vn = Gain * in(t).",
    },
    BlockSpec {
        block_type: BlockType::VoltageControlledCurrentSource,
        display_name: "Controlled Current Source",
        category: "Electrical",
        params: &[ParamSpec {
            key: "Transconductance",
            label: "Transconductance",
            unit: "A/V",
            default_val: "1.0",
            kind: ParamKind::Numeric,
            doc: "Transconductance Gm (finite).",
        }],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 0,
        default_feedthrough: false,
        doc: "Current source whose injected current from n to p = Transconductance * in(t).",
    },
    // --- Measurements (Milestone M8) ---
    BlockSpec {
        block_type: BlockType::Rms,
        display_name: "RMS",
        category: "Measurements",
        params: &[
            ParamSpec {
                key: "FundamentalFrequency",
                label: "Fundamental Frequency",
                unit: "Hz",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Base fundamental frequency f_base (> 0) defining averaging window T = 1/f.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Discrete sample period Ts. If empty, inherits model fixed step size.",
            },
            ParamSpec {
                key: "InitialValue",
                label: "Initial Value",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Output value held until first complete window of samples is acquired.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Computes true running Root-Mean-Square (RMS) over a sliding one-period window.",
    },
    BlockSpec {
        block_type: BlockType::Mean,
        display_name: "Mean",
        category: "Measurements",
        params: &[
            ParamSpec {
                key: "FundamentalFrequency",
                label: "Fundamental Frequency",
                unit: "Hz",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Base fundamental frequency f_base (> 0) defining averaging window T = 1/f.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Discrete sample period Ts. If empty, inherits model fixed step size.",
            },
            ParamSpec {
                key: "InitialValue",
                label: "Initial Value",
                unit: "",
                default_val: "0.0",
                kind: ParamKind::Numeric,
                doc: "Output value held until first complete window of samples is acquired.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Computes running average (DC component) over a sliding one-period window.",
    },
    BlockSpec {
        block_type: BlockType::PowerMeasurement,
        display_name: "Power Measurement",
        category: "Measurements",
        params: &[
            ParamSpec {
                key: "FundamentalFrequency",
                label: "Fundamental Frequency",
                unit: "Hz",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Base fundamental frequency f_base (> 0) defining averaging window T = 1/f.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Discrete sample period Ts. If empty, inherits model fixed step size.",
            },
        ],
        min_inputs: 2,
        max_inputs: 2,
        outputs: 5,
        default_feedthrough: false,
        doc: "Measures active power P, fundamental reactive power Q1, apparent power S, power factor PF, and displacement power factor DPF.",
    },
    BlockSpec {
        block_type: BlockType::Thd,
        display_name: "Total Harmonic Distortion",
        category: "Measurements",
        params: &[
            ParamSpec {
                key: "FundamentalFrequency",
                label: "Fundamental Frequency",
                unit: "Hz",
                default_val: "50.0",
                kind: ParamKind::Numeric,
                doc: "Base fundamental frequency f_base (> 0) defining window T = 1/f.",
            },
            ParamSpec {
                key: "SampleTime",
                label: "Sample Time",
                unit: "s",
                default_val: "",
                kind: ParamKind::Numeric,
                doc: "Discrete sample period Ts. If empty, inherits model fixed step size.",
            },
            ParamSpec {
                key: "MaxHarmonic",
                label: "Max Harmonic",
                unit: "",
                default_val: "40",
                kind: ParamKind::Numeric,
                doc: "Maximum harmonic order H included in distortion calculation.",
            },
            ParamSpec {
                key: "Output",
                label: "Output Format",
                unit: "",
                default_val: "Percent",
                kind: ParamKind::Choice(&["Percent", "Ratio"]),
                doc: "Output units: Percent (%) or decimal ratio.",
            },
        ],
        min_inputs: 1,
        max_inputs: 1,
        outputs: 1,
        default_feedthrough: false,
        doc: "Calculates Total Harmonic Distortion (THD) of input signal up to harmonic H relative to fundamental.",
    },
];

/// Returns the full list of supported block specifications.
pub fn all_block_specs() -> &'static [BlockSpec] {
    BLOCK_SPECS
}

/// Returns the specification for a block type, if registered.
pub fn block_spec(block_type: &BlockType) -> Option<&'static BlockSpec> {
    BLOCK_SPECS.iter().find(|s| &s.block_type == block_type)
}

/// Returns the parameter specifications for a block type.
pub fn param_specs_for(block_type: &BlockType) -> &'static [ParamSpec] {
    block_spec(block_type).map(|s| s.params).unwrap_or(&[])
}

/// Determines whether input_port of block_type has direct feedthrough to its output.
pub fn spec_feedthrough(
    block_type: &BlockType,
    parameters: &HashMap<String, String>,
    _input_port: usize,
) -> bool {
    match block_type {
        BlockType::TransferFunction | BlockType::DiscreteTransferFunction => {
            let num_str = parameters
                .get("Numerator")
                .map(|s| s.as_str())
                .unwrap_or("1");
            let den_str = parameters
                .get("Denominator")
                .map(|s| s.as_str())
                .unwrap_or("1, 1");
            let num_len = parse_float_vector(num_str).map(|v| v.len()).unwrap_or(1);
            let den_len = parse_float_vector(den_str).map(|v| v.len()).unwrap_or(2);
            // Strictly proper: deg(num) < deg(den) <=> num_len < den_len -> false
            // Proper: deg(num) == deg(den) <=> num_len == den_len -> true
            num_len >= den_len
        }
        BlockType::Pid => true,
        BlockType::ZeroOrderHold => false,
        _ => block_spec(block_type)
            .map(|s| s.default_feedthrough)
            .unwrap_or(true),
    }
}

/// Returns the normalized relative pin position [0.0..1.0, 0.0..1.0] for a given port/terminal and rotation.
pub fn pin_geometry(
    block_type: &BlockType,
    port_kind: PortKind,
    port_index: usize,
    rotation: BlockRotation,
) -> (f32, f32) {
    let (u, v) = match block_type {
        BlockType::Ground => (0.5, 0.0),
        BlockType::IdealSwitch
        | BlockType::VoltageSensor
        | BlockType::CurrentSensor
        | BlockType::VoltageControlledVoltageSource
        | BlockType::VoltageControlledCurrentSource => {
            if port_kind == PortKind::Signal {
                (0.5, 0.0) // Signal port on top
            } else if port_index == 0 {
                (0.0, 0.5) // Terminal p on left
            } else {
                (1.0, 0.5) // Terminal n on right
            }
        }
        BlockType::Resistor
        | BlockType::Capacitor
        | BlockType::Inductor
        | BlockType::DcVoltageSource
        | BlockType::AcVoltageSource
        | BlockType::DcCurrentSource
        | BlockType::Diode => {
            if port_index == 0 {
                (0.0, 0.5) // Terminal p on left
            } else {
                (1.0, 0.5) // Terminal n on right
            }
        }
        _ => (0.0, 0.5),
    };

    // Rotate point (u, v) clockwise around (0.5, 0.5)
    match rotation {
        BlockRotation::R0 => (u, v),
        BlockRotation::R90 => (1.0 - v, u),
        BlockRotation::R180 => (1.0 - u, 1.0 - v),
        BlockRotation::R270 => (v, 1.0 - u),
    }
}

/// Validates parameters for an electrical or signal block and returns any diagnostics.
pub fn validate_block_params(
    block_type: &BlockType,
    block_sid: &str,
    params: &HashMap<String, String>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let parse_num = |key: &str, default_val: f64| -> Option<f64> {
        params
            .get(key)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<f64>().ok())
            .unwrap_or(Some(default_val))
    };

    match block_type {
        BlockType::Resistor => {
            let r = parse_num("Resistance", 1000.0);
            match r {
                Some(val) if val.is_finite() && val > 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Resistance on block '{block_sid}': value must be a positive finite number (> 0) -> parsed value <= 0 or invalid -> specify a positive resistance in Ohms (e.g. 1000)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::Capacitor => {
            let c = parse_num("Capacitance", 1e-6);
            match c {
                Some(val) if val.is_finite() && val > 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Capacitance on block '{block_sid}': value must be a positive finite number (> 0) -> parsed value <= 0 or invalid -> specify a positive capacitance in Farads (e.g. 1e-6)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let v0 = parse_num("InitialVoltage", 0.0);
            match v0 {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter InitialVoltage on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite initial voltage in Volts"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::Inductor => {
            let l = parse_num("Inductance", 1e-3);
            match l {
                Some(val) if val.is_finite() && val > 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Inductance on block '{block_sid}': value must be a positive finite number (> 0) -> parsed value <= 0 or invalid -> specify a positive inductance in Henries (e.g. 1e-3)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let i0 = parse_num("InitialCurrent", 0.0);
            match i0 {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter InitialCurrent on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite initial current in Amperes"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::DcVoltageSource => {
            let v = parse_num("Voltage", 5.0);
            match v {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Voltage on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite voltage in Volts"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::AcVoltageSource => {
            let amp = parse_num("Amplitude", 10.0);
            match amp {
                Some(val) if val.is_finite() && val >= 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Amplitude on block '{block_sid}': value must be a non-negative finite number (>= 0) -> parsed value < 0 or invalid -> specify a non-negative amplitude in Volts"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let freq = parse_num("Frequency", 50.0);
            match freq {
                Some(val) if val.is_finite() && val > 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Frequency on block '{block_sid}': frequency must be strictly positive (> 0) -> parsed frequency <= 0 -> specify a positive frequency in Hz, or use a DC Voltage Source for 0 Hz"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let phase = parse_num("Phase", 0.0);
            match phase {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Phase on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite phase angle in degrees"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let offset = parse_num("Offset", 0.0);
            match offset {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Offset on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite bias offset in Volts"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::DcCurrentSource => {
            let i = parse_num("Current", 1.0);
            match i {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Current on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite current in Amperes"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::Diode => {
            let vf = parse_num("ForwardVoltage", 0.7);
            match vf {
                Some(val) if val.is_finite() && val >= 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter ForwardVoltage on block '{block_sid}': value must be non-negative (>= 0) -> parsed value < 0 -> specify a non-negative forward voltage in Volts (e.g. 0.7)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let ron = parse_num("OnResistance", 0.01);
            let ron_val = match ron {
                Some(val) if val.is_finite() && val > 0.0 => Some(val),
                _ => {
                    diags.push(Diagnostic::error(
                        "E-PARAM",
                        format!(
                            "Invalid parameter OnResistance on block '{block_sid}': value must be strictly positive (> 0) -> parsed value <= 0 -> specify a positive on-resistance in Ohms (e.g. 0.01)"
                        ),
                        vec![block_sid.to_string()],
                    ));
                    None
                }
            };
            let roff = parse_num("OffResistance", 1e6);
            match (ron_val, roff) {
                (Some(ron_v), Some(roff_v)) if roff_v.is_finite() && roff_v > ron_v => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter OffResistance on block '{block_sid}': off-resistance must be strictly greater than OnResistance -> parsed value <= OnResistance -> specify an off-resistance larger than OnResistance in Ohms (e.g. 1e6)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::IdealSwitch => {
            let ron = parse_num("OnResistance", 0.01);
            let ron_val = match ron {
                Some(val) if val.is_finite() && val > 0.0 => Some(val),
                _ => {
                    diags.push(Diagnostic::error(
                        "E-PARAM",
                        format!(
                            "Invalid parameter OnResistance on block '{block_sid}': value must be strictly positive (> 0) -> parsed value <= 0 -> specify a positive on-resistance in Ohms (e.g. 0.01)"
                        ),
                        vec![block_sid.to_string()],
                    ));
                    None
                }
            };
            let roff = parse_num("OffResistance", 1e6);
            match (ron_val, roff) {
                (Some(ron_v), Some(roff_v)) if roff_v.is_finite() && roff_v > ron_v => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter OffResistance on block '{block_sid}': off-resistance must be strictly greater than OnResistance -> parsed value <= OnResistance -> specify an off-resistance larger than OnResistance in Ohms (e.g. 1e6)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            let th = parse_num("Threshold", 0.5);
            match th {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Threshold on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite switching threshold"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::VoltageControlledVoltageSource => {
            let gain = parse_num("Gain", 1.0);
            match gain {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Gain on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite voltage gain"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::VoltageControlledCurrentSource => {
            let gm = parse_num("Transconductance", 1.0).or_else(|| parse_num("Gain", 1.0));
            match gm {
                Some(val) if val.is_finite() => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter Transconductance on block '{block_sid}': value must be finite -> parsed value is non-finite -> specify a finite transconductance in A/V"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
        }
        BlockType::Rms | BlockType::Mean | BlockType::PowerMeasurement | BlockType::Thd => {
            let f = parse_num("FundamentalFrequency", 50.0);
            match f {
                Some(val) if val.is_finite() && val > 0.0 => {}
                _ => diags.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Invalid parameter FundamentalFrequency on block '{block_sid}': value must be a positive finite number (> 0) -> parsed value <= 0 or invalid -> specify a positive fundamental frequency in Hz (e.g. 50.0)"
                    ),
                    vec![block_sid.to_string()],
                )),
            }
            if let Some(st_str) = params.get("SampleTime") {
                let st_trimmed = st_str.trim();
                if !st_trimmed.is_empty() {
                    match st_trimmed.parse::<f64>() {
                        Ok(st_val) if st_val.is_finite() && st_val > 0.0 => {}
                        _ => diags.push(Diagnostic::error(
                            "E-PARAM",
                            format!(
                                "Invalid parameter SampleTime on block '{block_sid}': value must be a positive finite number (> 0) -> specify a positive sample time in seconds"
                            ),
                            vec![block_sid.to_string()],
                        )),
                    }
                }
            }
            if *block_type == BlockType::Thd {
                if let Some(h_str) = params.get("MaxHarmonic") {
                    let h_trimmed = h_str.trim();
                    if !h_trimmed.is_empty() {
                        match h_trimmed.parse::<usize>() {
                            Ok(h_val) if h_val >= 1 => {}
                            _ => diags.push(Diagnostic::error(
                                "E-PARAM",
                                format!(
                                    "Invalid parameter MaxHarmonic on block '{block_sid}': value must be an integer >= 1 -> specify a valid harmonic order (e.g. 40)"
                                ),
                                vec![block_sid.to_string()],
                            )),
                        }
                    }
                }
                if let Some(out_str) = params.get("Output") {
                    let out_trimmed = out_str.trim();
                    if !out_trimmed.is_empty() && out_trimmed != "Percent" && out_trimmed != "Ratio"
                    {
                        diags.push(Diagnostic::error(
                            "E-PARAM",
                            format!(
                                "Invalid parameter Output on block '{block_sid}': must be 'Percent' or 'Ratio' -> got '{out_trimmed}'"
                            ),
                            vec![block_sid.to_string()],
                        ));
                    }
                }
            }
        }
        _ => {}
    }

    diags
}
