//! Error types for lexical analysis and parsing.

use crate::span::Span;
use thiserror::Error;

/// Parse error with exact source span and contextual explanation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at {span}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Unexpected end of input: expected {expected} at {span}")]
    UnexpectedEof { expected: String, span: Span },

    #[error("Unclosed delimiter '{delimiter}' starting at {span}")]
    UnclosedDelimiter { delimiter: char, span: Span },

    #[error("Invalid numeric literal '{literal}' at {span}")]
    InvalidNumberLiteral { literal: String, span: Span },

    #[error("Invalid assignment target at {span}")]
    InvalidAssignmentTarget { span: Span },

    #[error("Unrecognized character '{ch}' at {span}")]
    UnrecognizedCharacter { ch: char, span: Span },

    #[error("Empty matrix or cell row at {span}")]
    EmptyRow { span: Span },
}

impl ParseError {
    /// Returns the source span of the error.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::UnexpectedEof { span, .. } => *span,
            Self::UnclosedDelimiter { span, .. } => *span,
            Self::InvalidNumberLiteral { span, .. } => *span,
            Self::InvalidAssignmentTarget { span } => *span,
            Self::UnrecognizedCharacter { span, .. } => *span,
            Self::EmptyRow { span } => *span,
        }
    }
}
