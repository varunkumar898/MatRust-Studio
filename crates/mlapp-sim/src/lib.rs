//! Simulink SLX package parser, block diagram inspection, and fixed-step simulation engine.

#![deny(clippy::unwrap_used)]

pub mod builder;
pub mod inspect;
pub mod model;
pub mod parser;
pub mod solver;

pub use builder::export_slx;
pub use inspect::{inspect_slx, InspectError, SlxInspectReport};
pub use model::{Block, BlockDiagram, BlockType, Line, PortRef};
pub use parser::{parse_blockdiagram_xml, ParseError};
pub use solver::{simulate_diagram, SimError, SimulationConfig, SimulationOutput};
