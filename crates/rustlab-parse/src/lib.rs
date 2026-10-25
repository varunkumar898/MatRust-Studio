//! # `rustlab-parse`
//!
//! Clean-room lexical analyzer, parser, and abstract syntax tree (AST)
//! for MATLAB-compatible numerical computing scripts and functions.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use ast::*;
pub use error::ParseError;
pub use lexer::Lexer;
pub use parser::Parser;
pub use span::Span;
pub use token::{Token, TokenKind};

/// Convenience function to tokenize a raw MATLAB source string.
pub fn tokenize(source: &str) -> Result<Vec<Token>, ParseError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

/// Convenience function to parse a raw MATLAB source string into a sequence of AST statements.
pub fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
