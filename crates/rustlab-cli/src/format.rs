//! Formatting and diagnostic error reporting for RustLab CLI.

use rustlab_eval::EvalError;
use rustlab_math::Value;
use rustlab_parse::Span;

/// Converts a byte offset span into (line_number, column_number, width, line_text).
///
/// The scan is byte-based (column 1 is the first byte of the line). Both LF and CRLF
/// sources are supported: the `\r` immediately preceding a `\n` is counted as a source
/// byte for offset bookkeeping but stripped from the returned line text, so columns
/// no longer drift by one per preceding CRLF line on Windows files.
pub fn span_to_location(source: &str, span: Span) -> (usize, usize, usize, String) {
    if source.is_empty() {
        return (1, 1, 1, String::new());
    }

    let start = span.start.min(source.len());
    let mut offset = 0usize;

    for (line_idx, segment) in source.split('\n').enumerate() {
        // Position of the caret inside this line's text; the newline byte itself
        // (and its preceding `\r`) belongs to the NEXT line's bookkeeping.
        if start <= offset + segment.len() {
            let text = segment.strip_suffix('\r').unwrap_or(segment);
            let col = start.saturating_sub(offset) + 1;
            let width = span.end.saturating_sub(span.start).max(1);
            return (line_idx + 1, col, width, text.to_string());
        }
        offset += segment.len() + 1; // segment text plus its '\n'
    }

    // Unreachable for non-empty input (`start` is clamped to `source.len()`),
    // but the fallback keeps the signature total.
    (
        1,
        1,
        span.end.saturating_sub(span.start).max(1),
        source.split('\n').next().unwrap_or_default().to_string(),
    )
}

/// Formats an `EvalError` into a human-readable diagnostic message with line & column pointers.
pub fn format_eval_error(err: &EvalError, source: &str) -> String {
    let (span, msg) = match err {
        EvalError::UndefinedSymbol { name, span } => {
            (*span, format!("Undefined function or variable '{name}'."))
        }
        EvalError::Math { error, span } => (*span, format!("{error}")),
        EvalError::Parse(pe) => (pe.span(), format!("Syntax error: {pe}")),
        EvalError::TooManyInputs {
            expected,
            found,
            span,
        } => (
            *span,
            format!(
                "Too many input arguments: expected at most {expected}, received {found}."
            ),
        ),
        EvalError::TooManyOutputs {
            expected,
            found,
            span,
        } => (
            *span,
            format!(
                "Too many output arguments: function provides at most {expected}, requested {found}."
            ),
        ),
        EvalError::InvalidCondition { message, span } => {
            (*span, format!("Invalid condition: {message}"))
        }
        EvalError::UserError {
            identifier,
            message,
            span,
        } => (*span, format!("{identifier}: {message}")),
    };

    if span == Span::default() || source.trim().is_empty() {
        return format!("Error: {msg}");
    }

    let (line, col, width, line_text) = span_to_location(source, span);
    let padding = " ".repeat(col.saturating_sub(1));
    let underline = "^".repeat(width.min(line_text.len().saturating_sub(col - 1)).max(1));

    format!(
        "Error: {msg}\n  --> Line {line}, Column {col}:\n   |\n{:3} | {}\n   | {}{}",
        line, line_text, padding, underline
    )
}

/// Formats a runtime `Value` for display.
pub fn format_value(val: &Value) -> String {
    format!("{val}")
}
