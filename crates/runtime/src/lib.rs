//! Interpreter runtime: evaluates a parsed program in an environment,
//! executing statements and producing MATLAB-style output records.
//!
//! This is intentionally a tree-walking interpreter (Phase E, milestone 1).
//! No JIT, no parallelism — see docs/architecture.md for rationale.

pub mod builtins;
pub mod interp;

pub use builtins::format_value;
pub use interp::{exec_program_env, exec_source, run_source, Env, RunResult, StatementOutput};
