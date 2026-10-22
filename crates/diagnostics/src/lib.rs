use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn merge(self, other: Span) -> Self {
        let (start, line, column) = if self.start <= other.start {
            (self.start, self.line, self.column)
        } else {
            (other.start, other.line, other.column)
        };
        let end = self.end.max(other.end);
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Option<Span>,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();
        let prefix = match self.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Note => "note",
        };

        if let Some(span) = self.span {
            out.push_str(&format!(
                "{}:{}: {}: {}\n",
                span.line, span.column, prefix, self.message
            ));

            // Extract the relevant line from source
            if let Some(line_str) = source.lines().nth(span.line.saturating_sub(1)) {
                out.push_str(&format!("  |\n"));
                out.push_str(&format!("{:3} | {}\n", span.line, line_str));
                let indent = " ".repeat(span.column.saturating_sub(1));
                let width = (span.end.saturating_sub(span.start)).max(1);
                let pointer = "^".repeat(width);
                out.push_str(&format!("  | {}{}\n", indent, pointer));
            }
        } else {
            out.push_str(&format!("{}: {}\n", prefix, self.message));
        }

        if let Some(hint) = &self.hint {
            out.push_str(&format!("  = hint: {}\n", hint));
        }

        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span {
            write!(
                f,
                "line {}, col {}: {}",
                span.line, span.column, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for Diagnostic {}
