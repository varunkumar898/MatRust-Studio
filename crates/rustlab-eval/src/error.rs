//! Runtime error definitions for the interpreter.

use rustlab_math::MathError;
use rustlab_parse::{ParseError, Span};
use thiserror::Error;

/// Runtime evaluation error.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EvalError {
    #[error("Unrecognized function or variable '{name}' at {span}")]
    UndefinedSymbol { name: String, span: Span },

    #[error("Mathematical error at {span}: {error}")]
    Math { error: MathError, span: Span },

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Too many input arguments at {span}: expected at most {expected}, received {found}")]
    TooManyInputs {
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("Too many output arguments at {span}: function provides at most {expected}, requested {found}")]
    TooManyOutputs {
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("Invalid loop or branch condition at {span}: {message}")]
    InvalidCondition { message: String, span: Span },

    #[error("{identifier}: {message} at {span}")]
    UserError {
        identifier: String,
        message: String,
        span: Span,
    },
}

impl From<MathError> for EvalError {
    fn from(error: MathError) -> Self {
        Self::Math {
            error,
            span: Span::default(),
        }
    }
}
