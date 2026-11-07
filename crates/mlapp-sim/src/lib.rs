//! Simulink SLX package parser, block diagram inspection, and fixed-step simulation engine.

#![deny(clippy::unwrap_used)]

pub mod builder;
pub mod engine;
pub mod inspect;
pub mod model;
pub mod parser;
pub mod solver;
pub mod spec;

pub use builder::export_slx;
pub use engine::{
    feedthrough, CompiledBlock, CompiledKind, ExecutionEngine, ModelLayout, SimWorkspace,
};
pub use inspect::{inspect_slx, InspectError, SlxInspectReport};
pub use model::{Block, BlockDiagram, BlockType, Line, PortRef};
pub use parser::{parse_blockdiagram_xml, ParseError};
pub use solver::{
    simulate_diagram, BlockRef, DormandPrinceTableau, SimError, SimulationConfig, SimulationOutput,
    SolverType,
};
pub use spec::{
    all_block_specs, block_spec, param_specs_for, parse_float_vector, BlockSpec, ParamKind,
    ParamSpec,
};
