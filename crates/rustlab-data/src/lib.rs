//! rustlab-data crate for statistics and tabular/matrix I/O.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod csv_io;
pub mod stats;
pub mod tables;

pub(crate) fn user_err(identifier: &str, message: &str) -> rustlab_eval::EvalError {
    rustlab_eval::EvalError::UserError {
        identifier: identifier.to_string(),
        message: message.to_string(),
        span: rustlab_parse::Span::default(),
    }
}
