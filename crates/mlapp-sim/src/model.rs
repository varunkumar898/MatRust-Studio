//! Simulink block diagram data model: blocks, lines, parameters, and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard block types supported or recognized in Simulink block diagrams.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BlockType {
    Constant,
    SineWave,
    Step,
    Gain,
    Sum,
    Product,
    Integrator,
    UnitDelay,
    Saturation,
    Scope,
    ToWorkspace,
    Inport,
    Outport,
    Subsystem,
    Unknown(String),
}

impl BlockType {
    pub fn from_str_name(name: &str) -> Self {
        match name {
            "Constant" => BlockType::Constant,
            "Sin" | "SineWave" | "Sine Wave" => BlockType::SineWave,
            "Step" => BlockType::Step,
            "Gain" => BlockType::Gain,
            "Sum" | "Add" | "Subtract" => BlockType::Sum,
            "Product" => BlockType::Product,
            "Integrator" => BlockType::Integrator,
            "UnitDelay" | "Delay" => BlockType::UnitDelay,
            "Saturate" | "Saturation" => BlockType::Saturation,
            "Scope" => BlockType::Scope,
            "ToWorkspace" | "To Workspace" => BlockType::ToWorkspace,
            "Inport" => BlockType::Inport,
            "Outport" => BlockType::Outport,
            "SubSystem" | "Subsystem" => BlockType::Subsystem,
            other => BlockType::Unknown(other.to_string()),
        }
    }

    pub fn is_solver_supported(&self) -> bool {
        matches!(
            self,
            BlockType::Constant
                | BlockType::SineWave
                | BlockType::Step
                | BlockType::Gain
                | BlockType::Sum
                | BlockType::Product
                | BlockType::Integrator
                | BlockType::UnitDelay
                | BlockType::Saturation
                | BlockType::Scope
                | BlockType::ToWorkspace
                | BlockType::Inport
                | BlockType::Outport
        )
    }
}

/// A single block node inside a block diagram.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub sid: String,
    pub name: String,
    pub block_type: BlockType,
    pub parameters: HashMap<String, String>,
    pub position: Option<[f64; 4]>,
}

impl Block {
    pub fn new(sid: impl Into<String>, name: impl Into<String>, block_type: BlockType) -> Self {
        Self {
            sid: sid.into(),
            name: name.into(),
            block_type,
            parameters: HashMap::new(),
            position: None,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
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
}

/// A directed connection line between two block ports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub src: PortRef,
    pub dst: PortRef,
    pub name: Option<String>,
}

/// A parsed Simulink block diagram model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockDiagram {
    pub name: String,
    pub version: Option<String>,
    pub release: Option<String>,
    pub blocks: Vec<Block>,
    pub lines: Vec<Line>,
    pub toolboxes: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl BlockDiagram {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            release: None,
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
            },
            dst: PortRef {
                block_sid: dst_sid.to_string(),
                port_index: dst_port,
            },
            name: None,
        });
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
