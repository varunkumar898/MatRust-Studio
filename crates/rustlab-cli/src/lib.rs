//! # `rustlab-cli`
//!
//! Interactive REPL, formatted diagnostic reporter, CLI runner, and benchmark suite.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod bench;
pub mod format;
pub mod repl;
