//! Source span tracking for error diagnostics and AST locations.

use std::fmt;

/// Represents a byte range `[start, end)` in the source input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// 0-based byte offset where the span starts.
    pub start: usize,
    /// 0-based byte offset where the span ends (exclusive).
    pub end: usize,
}

impl Span {
    /// Creates a new `Span`.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Creates an empty span at a single point.
    pub const fn point(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Combines two spans into a single span enclosing both.
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
