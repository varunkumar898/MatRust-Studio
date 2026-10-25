//! # `rustlab-eval`
//!
//! Clean-room MATLAB interpreter and execution environment:
//! - Tree-walking AST evaluator
//! - Lexical scopes, global variables, and persistent variables
//! - Function call stack and multiple return argument binding (`[a, b] = func()`)
//! - 1-based dynamic indexing with `end` keyword resolution
//! - Control flow: `if/elseif/else`, `for` (column iteration), `while`, `switch`, `try-catch`

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod error;
pub mod eval;
pub mod scope;

pub use error::EvalError;
pub use eval::{BuiltinFn, Flow, IndexingContext, Interpreter};
pub use scope::Environment;

use rustlab_math::Value;

/// Evaluates a raw MATLAB script string, returning the final computed value.
pub fn eval_str(source: &str) -> Result<Option<Value>, EvalError> {
    let mut interp = Interpreter::new();
    interp.eval_script(source)
}
