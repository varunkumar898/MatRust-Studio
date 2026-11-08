//! Simulink block diagram data model: blocks, lines, parameters, and metadata.

pub use crate::spec::{
    all_block_specs, block_spec, param_specs_for, parse_float_vector, BlockSpec, ParamKind,
    ParamSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domain kind of a block port or line connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortKind {
    Signal,
    Electrical,
}

/// Role of a port on a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortRole {
    Input,
    Output,
    Terminal,
}

/// Description of a block port: domain kind, direction/role, label, and index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDesc {
    pub kind: PortKind,
    pub role: PortRole,
    pub label: String,
    pub index: usize,
}

/// Clockwise block rotation in 90-degree increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlockRotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl BlockRotation {
    pub fn rotated_cw(&self) -> Self {
        match self {
            BlockRotation::R0 => BlockRotation::R90,
            BlockRotation::R90 => BlockRotation::R180,
            BlockRotation::R180 => BlockRotation::R270,
            BlockRotation::R270 => BlockRotation::R0,
        }
    }

    pub fn angle_degrees(&self) -> f32 {
        match self {
            BlockRotation::R0 => 0.0,
            BlockRotation::R90 => 90.0,
            BlockRotation::R180 => 180.0,
            BlockRotation::R270 => 270.0,
        }
    }
}

/// Standard block types supported or recognized in Simulink block diagrams.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BlockType {
    Constant,
    SineWave,
    Step,
    Sawtooth,
    Triangle,
    PulseGenerator,
    RepeatingSequence,
    RepeatingSequenceStair,
    Clock,
    Gain,
    Sum,
    Product,
    Integrator,
    UnitDelay,
    TransferFunction,
    DiscreteTransferFunction,
    Pid,
    ZeroOrderHold,
    RelationalOperator,
    LogicalOperator,
    Switch,
    Abs,
    MathFunction,
    Saturation,
    Scope,
    ToWorkspace,
    Inport,
    Outport,
    Subsystem,
    // Electrical blocks (Milestone M4)
    Resistor,
    Capacitor,
    Inductor,
    DcVoltageSource,
    AcVoltageSource,
    DcCurrentSource,
    Ground,
    Diode,
    IdealSwitch,
    // Sensors and Controlled Sources (Milestone M7)
    VoltageSensor,
    CurrentSensor,
    VoltageControlledVoltageSource,
    VoltageControlledCurrentSource,
    // Measurement blocks (Milestone M8)
    Rms,
    Mean,
    PowerMeasurement,
    Thd,
    Unknown(String),
}

impl BlockType {
    pub fn from_str_name(name: &str) -> Self {
        match name {
            "Constant" => BlockType::Constant,
            "Sin" | "SineWave" | "Sine Wave" => BlockType::SineWave,
            "Step" => BlockType::Step,
            "Sawtooth" => BlockType::Sawtooth,
            "Triangle" => BlockType::Triangle,
            "PulseGenerator" | "Pulse Generator" => BlockType::PulseGenerator,
            "RepeatingSequence" | "Repeating Sequence" => BlockType::RepeatingSequence,
            "RepeatingSequenceStair" | "Repeating Sequence Stair" | "RepeatingSequenceStairs" => {
                BlockType::RepeatingSequenceStair
            }
            "Clock" => BlockType::Clock,
            "Gain" => BlockType::Gain,
            "Sum" | "Add" | "Subtract" => BlockType::Sum,
            "Product" => BlockType::Product,
            "Integrator" => BlockType::Integrator,
            "UnitDelay" | "Delay" => BlockType::UnitDelay,
            "TransferFunction" | "Transfer Function" | "Transfer Fcn" => {
                BlockType::TransferFunction
            }
            "DiscreteTransferFunction" | "Discrete Transfer Function" | "Discrete Transfer Fcn" => {
                BlockType::DiscreteTransferFunction
            }
            "PID" | "Pid" | "PID Controller" => BlockType::Pid,
            "ZeroOrderHold" | "Zero-Order Hold" | "Zero Order Hold" | "ZOH" => {
                BlockType::ZeroOrderHold
            }
            "RelationalOperator" | "Relational Operator" => BlockType::RelationalOperator,
            "LogicalOperator" | "Logical Operator" => BlockType::LogicalOperator,
            "Switch" => BlockType::Switch,
            "Abs" => BlockType::Abs,
            "MathFunction" | "Math Function" => BlockType::MathFunction,
            "Saturate" | "Saturation" => BlockType::Saturation,
            "Scope" => BlockType::Scope,
            "ToWorkspace" | "To Workspace" => BlockType::ToWorkspace,
            "Inport" => BlockType::Inport,
            "Outport" => BlockType::Outport,
            "SubSystem" | "Subsystem" => BlockType::Subsystem,
            "Rms" | "RMS" => BlockType::Rms,
            "Mean" | "RunningMean" | "Average" => BlockType::Mean,
            "PowerMeasurement" | "Power Measurement" | "Power" => BlockType::PowerMeasurement,
            "Thd" | "THD" => BlockType::Thd,
            other => BlockType::Unknown(other.to_string()),
        }
    }

    pub fn is_solver_supported(&self) -> bool {
        matches!(
            self,
            BlockType::Constant
                | BlockType::SineWave
                | BlockType::Step
                | BlockType::Sawtooth
                | BlockType::Triangle
                | BlockType::PulseGenerator
                | BlockType::RepeatingSequence
                | BlockType::RepeatingSequenceStair
                | BlockType::Clock
                | BlockType::Gain
                | BlockType::Sum
                | BlockType::Product
                | BlockType::Integrator
                | BlockType::UnitDelay
                | BlockType::TransferFunction
                | BlockType::DiscreteTransferFunction
                | BlockType::Pid
                | BlockType::ZeroOrderHold
                | BlockType::RelationalOperator
                | BlockType::LogicalOperator
                | BlockType::Switch
                | BlockType::Abs
                | BlockType::MathFunction
                | BlockType::Saturation
                | BlockType::Scope
                | BlockType::ToWorkspace
                | BlockType::Inport
                | BlockType::Outport
                | BlockType::Rms
                | BlockType::Mean
                | BlockType::PowerMeasurement
                | BlockType::Thd
        )
    }

    pub fn is_electrical(&self) -> bool {
        matches!(
            self,
            BlockType::Resistor
                | BlockType::Capacitor
                | BlockType::Inductor
                | BlockType::DcVoltageSource
                | BlockType::AcVoltageSource
                | BlockType::DcCurrentSource
                | BlockType::Ground
                | BlockType::Diode
                | BlockType::IdealSwitch
                | BlockType::VoltageSensor
                | BlockType::CurrentSensor
                | BlockType::VoltageControlledVoltageSource
                | BlockType::VoltageControlledCurrentSource
        )
    }

    /// Number of physical electrical terminals on this block.
    pub fn terminal_count(&self) -> usize {
        match self {
            BlockType::Ground => 1,
            BlockType::Resistor
            | BlockType::Capacitor
            | BlockType::Inductor
            | BlockType::DcVoltageSource
            | BlockType::AcVoltageSource
            | BlockType::DcCurrentSource
            | BlockType::Diode
            | BlockType::IdealSwitch
            | BlockType::VoltageSensor
            | BlockType::CurrentSensor
            | BlockType::VoltageControlledVoltageSource
            | BlockType::VoltageControlledCurrentSource => 2,
            _ => 0,
        }
    }

    /// Returns the number of signal input ports for this block type given its configuration parameters.
    pub fn input_ports(&self, parameters: &HashMap<String, String>) -> usize {
        match self {
            BlockType::Constant
            | BlockType::SineWave
            | BlockType::Step
            | BlockType::Sawtooth
            | BlockType::Triangle
            | BlockType::PulseGenerator
            | BlockType::RepeatingSequence
            | BlockType::RepeatingSequenceStair
            | BlockType::Clock
            | BlockType::Inport
            | BlockType::Resistor
            | BlockType::Capacitor
            | BlockType::Inductor
            | BlockType::DcVoltageSource
            | BlockType::AcVoltageSource
            | BlockType::DcCurrentSource
            | BlockType::Ground
            | BlockType::Diode
            | BlockType::VoltageSensor
            | BlockType::CurrentSensor => 0,
            BlockType::IdealSwitch
            | BlockType::VoltageControlledVoltageSource
            | BlockType::VoltageControlledCurrentSource => 1,
            BlockType::Sum => {
                let signs = parameters.get("Inputs").map(|s| s.as_str()).unwrap_or("++");
                let count = signs.chars().filter(|&c| c == '+' || c == '-').count();
                if count == 0 {
                    2
                } else {
                    count
                }
            }
            BlockType::RelationalOperator | BlockType::Product | BlockType::PowerMeasurement => 2,
            BlockType::LogicalOperator => {
                let op = parameters
                    .get("Operator")
                    .map(|s| s.as_str())
                    .unwrap_or("AND");
                if op == "NOT" {
                    1
                } else {
                    parameters
                        .get("Inputs")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(2)
                        .max(1)
                }
            }
            BlockType::Switch => 3,
            BlockType::MathFunction => {
                let func = parameters
                    .get("Function")
                    .map(|s| s.as_str())
                    .unwrap_or("sqrt");
                if func == "pow" {
                    2
                } else {
                    1
                }
            }
            BlockType::Gain
            | BlockType::Integrator
            | BlockType::UnitDelay
            | BlockType::TransferFunction
            | BlockType::DiscreteTransferFunction
            | BlockType::Pid
            | BlockType::ZeroOrderHold
            | BlockType::Abs
            | BlockType::Saturation
            | BlockType::Scope
            | BlockType::ToWorkspace
            | BlockType::Outport
            | BlockType::Rms
            | BlockType::Mean
            | BlockType::Thd => 1,
            BlockType::Subsystem | BlockType::Unknown(_) => 1,
        }
    }

    /// Returns the number of signal output ports for this block type.
    pub fn output_ports(&self) -> usize {
        match self {
            BlockType::Scope
            | BlockType::ToWorkspace
            | BlockType::Outport
            | BlockType::Resistor
            | BlockType::Capacitor
            | BlockType::Inductor
            | BlockType::DcVoltageSource
            | BlockType::AcVoltageSource
            | BlockType::DcCurrentSource
            | BlockType::Ground
            | BlockType::Diode
            | BlockType::IdealSwitch
            | BlockType::VoltageControlledVoltageSource
            | BlockType::VoltageControlledCurrentSource => 0,
            BlockType::VoltageSensor | BlockType::CurrentSensor => 1,
            BlockType::PowerMeasurement => 5,
            _ => 1,
        }
    }

    /// Returns a list of all port descriptions (kind, role, label, and index) for this block type.
    pub fn ports(&self, parameters: &HashMap<String, String>) -> Vec<PortDesc> {
        let mut list = Vec::new();
        match self {
            BlockType::Ground => {
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "gnd".to_string(),
                    index: 0,
                });
            }
            BlockType::Resistor
            | BlockType::Capacitor
            | BlockType::Inductor
            | BlockType::DcVoltageSource
            | BlockType::AcVoltageSource
            | BlockType::DcCurrentSource
            | BlockType::Diode => {
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "p".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "n".to_string(),
                    index: 1,
                });
            }
            BlockType::IdealSwitch => {
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Input,
                    label: "v".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "p".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "n".to_string(),
                    index: 1,
                });
            }
            BlockType::VoltageSensor => {
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Output,
                    label: "V".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "p".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "n".to_string(),
                    index: 1,
                });
            }
            BlockType::CurrentSensor => {
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Output,
                    label: "I".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "p".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "n".to_string(),
                    index: 1,
                });
            }
            BlockType::VoltageControlledVoltageSource
            | BlockType::VoltageControlledCurrentSource => {
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Input,
                    label: "in".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "p".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Electrical,
                    role: PortRole::Terminal,
                    label: "n".to_string(),
                    index: 1,
                });
            }
            BlockType::PowerMeasurement => {
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Input,
                    label: "v".to_string(),
                    index: 0,
                });
                list.push(PortDesc {
                    kind: PortKind::Signal,
                    role: PortRole::Input,
                    label: "i".to_string(),
                    index: 1,
                });
                let labels = ["P", "Q1", "S", "PF", "DPF"];
                for (j, lbl) in labels.iter().enumerate() {
                    list.push(PortDesc {
                        kind: PortKind::Signal,
                        role: PortRole::Output,
                        label: lbl.to_string(),
                        index: j,
                    });
                }
            }
            _ => {
                let in_count = self.input_ports(parameters);
                for i in 0..in_count {
                    list.push(PortDesc {
                        kind: PortKind::Signal,
                        role: PortRole::Input,
                        label: format!("in{}", i + 1),
                        index: i,
                    });
                }
                let out_count = self.output_ports();
                for j in 0..out_count {
                    list.push(PortDesc {
                        kind: PortKind::Signal,
                        role: PortRole::Output,
                        label: if out_count == 1 {
                            "out".to_string()
                        } else {
                            format!("out{}", j + 1)
                        },
                        index: j,
                    });
                }
            }
        }
        list
    }
}

/// A single block node inside a block diagram.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub sid: String,
    pub name: String,
    pub block_type: BlockType,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
    #[serde(default)]
    pub position: Option<[f64; 4]>,
    #[serde(default)]
    pub rotation: BlockRotation,
}

impl Block {
    pub fn new(sid: impl Into<String>, name: impl Into<String>, block_type: BlockType) -> Self {
        Self {
            sid: sid.into(),
            name: name.into(),
            block_type,
            parameters: HashMap::new(),
            position: None,
            rotation: BlockRotation::R0,
        }
    }

    pub fn input_ports(&self) -> usize {
        self.block_type.input_ports(&self.parameters)
    }

    pub fn output_ports(&self) -> usize {
        self.block_type.output_ports()
    }

    pub fn terminal_count(&self) -> usize {
        self.block_type.terminal_count()
    }

    pub fn ports(&self) -> Vec<PortDesc> {
        self.block_type.ports(&self.parameters)
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    pub fn with_rotation(mut self, rotation: BlockRotation) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn get_param(&self, key: &str) -> Option<&str> {
        self.parameters.get(key).map(|s| s.as_str())
    }

    pub fn set_param(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.parameters.insert(key.into(), value.into());
    }
}

/// Port reference in a connection line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortRef {
    pub block_sid: String,
    pub port_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port_kind: Option<PortKind>,
}

impl PortRef {
    pub fn new(block_sid: impl Into<String>, port_index: usize) -> Self {
        Self {
            block_sid: block_sid.into(),
            port_index,
            port_kind: None,
        }
    }

    pub fn signal(block_sid: impl Into<String>, port_index: usize) -> Self {
        Self {
            block_sid: block_sid.into(),
            port_index,
            port_kind: Some(PortKind::Signal),
        }
    }

    pub fn electrical(block_sid: impl Into<String>, terminal_index: usize) -> Self {
        Self {
            block_sid: block_sid.into(),
            port_index: terminal_index,
            port_kind: Some(PortKind::Electrical),
        }
    }
}

/// A connection line between two block ports or terminals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub src: PortRef,
    pub dst: PortRef,
    #[serde(default)]
    pub name: Option<String>,
}

fn default_schema_version() -> u32 {
    1
}

/// A parsed Simulink block diagram model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockDiagram {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub simulation_config: Option<crate::solver::SimulationConfig>,
    pub blocks: Vec<Block>,
    pub lines: Vec<Line>,
    #[serde(default)]
    pub toolboxes: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl BlockDiagram {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            release: None,
            schema_version: 3,
            simulation_config: None,
            blocks: Vec::new(),
            lines: Vec::new(),
            toolboxes: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn add_line(&mut self, src_sid: &str, src_port: usize, dst_sid: &str, dst_port: usize) {
        self.lines.push(Line {
            src: PortRef {
                block_sid: src_sid.to_string(),
                port_index: src_port,
                port_kind: None,
            },
            dst: PortRef {
                block_sid: dst_sid.to_string(),
                port_index: dst_port,
                port_kind: None,
            },
            name: None,
        });
    }

    pub fn add_electrical_line(
        &mut self,
        src_sid: &str,
        src_term: usize,
        dst_sid: &str,
        dst_term: usize,
    ) {
        self.lines.push(Line {
            src: PortRef::electrical(src_sid, src_term),
            dst: PortRef::electrical(dst_sid, dst_term),
            name: None,
        });
    }

    pub fn migrate_schema(&mut self) {
        if self.schema_version < 3 {
            self.schema_version = 3;
        }
    }

    pub fn find_block(&self, name_or_sid: &str) -> Option<&Block> {
        self.blocks
            .iter()
            .find(|b| b.name == name_or_sid || b.sid == name_or_sid)
    }

    pub fn find_block_mut(&mut self, name_or_sid: &str) -> Option<&mut Block> {
        self.blocks
            .iter_mut()
            .find(|b| b.name == name_or_sid || b.sid == name_or_sid)
    }
}
