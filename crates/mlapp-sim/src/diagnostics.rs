//! Simulation and modeling diagnostics: errors, warnings, and suggested fixes.

use serde::{Deserialize, Serialize};

/// Severity of a simulation or validation diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Suggested action to resolve a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixAction {
    Message(String),
}

/// A structured diagnostic message adhering to Charter Principle P4:
/// "what happened -> why -> how to fix".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub block_sids: Vec<String>,
    pub fix: Option<FixAction>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        block_sids: Vec<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            block_sids,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: FixAction) -> Self {
        self.fix = Some(fix);
        self
    }
}
