//! Simulink SLX package parser, block diagram inspection, and fixed-step simulation engine.

#![deny(clippy::unwrap_used)]

pub mod builder;
pub mod diagnostics;
pub mod electrical;
pub mod engine;
pub mod inspect;
pub mod measurements;
pub mod model;
pub mod netlist;
pub mod parser;
pub mod solver;
pub mod spec;

pub use builder::export_slx;
pub use diagnostics::{
    apply_fix, check_diagram, lookup_diagnostic, Diagnostic, DiagnosticInfo, DiagnosticSeverity,
    FixAction, DIAGNOSTIC_REGISTRY,
};
pub use electrical::{LinearCircuitSolver, LuDecomposition};
pub use engine::{
    feedthrough, CompiledBlock, CompiledKind, ExecutionEngine, ModelLayout, SimWorkspace,
};
pub use inspect::{inspect_slx, InspectError, SlxInspectReport};
pub use measurements::{
    dft_direct, fft_radix2, spectrum, spectrum_with_window, NeumaierSum, PowerMeasurementBuffer,
    ScalarWindowBuffer, SpectrumResult, ThdBuffer, WindowFunction,
};
pub use model::{
    Block, BlockDiagram, BlockRotation, BlockType, Line, PortDesc, PortKind, PortRef, PortRole,
};
pub use netlist::{resolve_electrical_nets, ElectricalTopology, NetlistResult};
pub use parser::{parse_blockdiagram_xml, ParseError};
pub use solver::{
    simulate_diagram, BlockRef, DormandPrinceTableau, SimError, SimulationConfig, SimulationOutput,
    SolverType,
};
pub use spec::{
    all_block_specs, block_spec, param_specs_for, parse_float_vector, pin_geometry,
    validate_block_params, BlockSpec, ParamKind, ParamSpec,
};
