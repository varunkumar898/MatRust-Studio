//! Interpreter runtime: evaluates a parsed program in an environment,
//! executing statements and producing MATLAB-style output records.
//!
//! This is intentionally a tree-walking interpreter (Phase E, milestone 1).
//! No JIT, no parallelism — see docs/architecture.md for rationale.

pub mod arrayfn;
pub mod builtins;
pub mod graphics;
pub mod interp;
pub mod rng;

pub use builtins::format_value;
pub use graphics::{FigureState, GraphicsState, PlotItem};
pub use interp::{
    exec_program_env, exec_source, run_source, variable_rows, Env, ExecIo, RunResult, Session,
    StatementOutput, VarRow,
};
