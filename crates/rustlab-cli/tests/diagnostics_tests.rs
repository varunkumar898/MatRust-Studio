//! Milestone A3.1.2 regression tests for the CLI diagnostics caret.
//!
//! Locks the CRLF-aware `span_to_location` fix: a `\r` immediately preceding
//! `\n` counts as a source byte for span bookkeeping but is stripped from the
//! rendered line text, so caret positions no longer drift by one per preceding
//! CRLF line on Windows files.
//!
//! Every expected string below was verified against the live RustLab CLI
//! binary (release build) before being pinned.

use rustlab_cli::format::{format_eval_error, span_to_location};
use rustlab_eval::EvalError;
use rustlab_parse::Span;
use rustlab_stdlib::new_interpreter;

const DIMAGREE_LF: &str = "x = [1 2] + [1 2 3]";
const UNDEF_LF: &str = "x = 1 + y";
const PARSE_LF: &str = "x = [1 2";
const DIMAGREE_LF_MULTI: &str = "a = 1;\nb = 2;\nx = [1 2] + [1 2 3]";
const UNDEF_LF_MULTI: &str = "a = 1;\nb = 2;\nx = 1 + y";
const DIMAGREE_CRLF: &str = "% CRLF probe\r\nb = 2;\r\nx = [1 2] + [1 2 3]";
const UNDEF_CRLF: &str = "% three lines\r\nb = 2;\r\nx = 1 + y";

/// Evaluates `code` and returns the runtime `EvalError`, panicking if the code
/// unexpectedly succeeds.
fn eval_err(code: &str) -> EvalError {
    let mut interp = new_interpreter();
    interp
        .eval_script(code)
        .err()
        .unwrap_or_else(|| panic!("`{code}` did not fail"))
}

// ---------------------------------------------------------------------------
// Exact end-to-end diagnostic rendering (LF sources)
// ---------------------------------------------------------------------------

#[test]
fn binary_op_error_underlines_whole_expression_lf() {
    let err = eval_err(DIMAGREE_LF);
    let expected = concat!(
        "Error: Arrays have incompatible sizes for this operation",
        " (identifier: MATLAB:dimagree)\n",
        "  --> Line 1, Column 5:\n",
        "   |\n",
        "  1 | x = [1 2] + [1 2 3]\n",
        "   |     ^^^^^^^^^^^^^^^",
    );
    assert_eq!(format_eval_error(&err, DIMAGREE_LF), expected);
}

#[test]
fn undefined_symbol_gets_a_single_caret_lf() {
    let err = eval_err(UNDEF_LF);
    let expected = concat!(
        "Error: Undefined function or variable 'y'.\n",
        "  --> Line 1, Column 9:\n",
        "   |\n",
        "  1 | x = 1 + y\n",
        "   |         ^",
    );
    assert_eq!(format_eval_error(&err, UNDEF_LF), expected);
}

#[test]
fn parse_error_keeps_location_and_caret() {
    let err = eval_err(PARSE_LF);
    let expected = concat!(
        "Error: Syntax error: Unexpected end of input: expected ']' at 8..8\n",
        "  --> Line 1, Column 9:\n",
        "   |\n",
        "  1 | x = [1 2\n",
        "   |         ^",
    );
    assert_eq!(format_eval_error(&err, PARSE_LF), expected);
}

// ---------------------------------------------------------------------------
// Multi-line positioning (LF sources)
// ---------------------------------------------------------------------------

#[test]
fn binary_op_caret_lands_on_spanning_line_lf() {
    let err = eval_err(DIMAGREE_LF_MULTI);
    let formatted = format_eval_error(&err, DIMAGREE_LF_MULTI);
    assert!(
        formatted.contains("  --> Line 3, Column 5:"),
        "got: {formatted}"
    );
    // 15 carets under the full expression `[1 2] + [1 2 3]`.
    let caret_line = formatted
        .lines()
        .find(|l| l.contains('^'))
        .unwrap_or_else(|| panic!("no caret line in {formatted:?}"));
    let carets = caret_line.matches('^').count();
    assert_eq!(carets, 15, "whole-expression underline, got {caret_line:?}");
}

#[test]
fn undefined_symbol_caret_lands_on_symbol_line_lf() {
    let err = eval_err(UNDEF_LF_MULTI);
    let formatted = format_eval_error(&err, UNDEF_LF_MULTI);
    assert!(
        formatted.contains("  --> Line 3, Column 9:"),
        "got: {formatted}"
    );
}

// ---------------------------------------------------------------------------
// CRLF sources behave identically (the A3.1.2 caret fix)
// ---------------------------------------------------------------------------

#[test]
fn binary_op_caret_is_crlf_insensitive() {
    let err = eval_err(DIMAGREE_CRLF);
    let formatted = format_eval_error(&err, DIMAGREE_CRLF);
    // Before the fix, this drifted to Column 6 (one extra offset per CRLF line).
    assert!(
        formatted.contains("  --> Line 3, Column 5:"),
        "CRLF caret must not drift, got: {formatted}"
    );
    let caret_line = formatted
        .lines()
        .find(|l| l.contains('^'))
        .unwrap_or_else(|| panic!("no caret line in {formatted:?}"));
    assert_eq!(
        caret_line.matches('^').count(),
        15,
        "15 carets, got {caret_line:?}"
    );
    assert_eq!(caret_line, "   |     ^^^^^^^^^^^^^^^");
}

#[test]
fn undefined_symbol_caret_survives_two_crlf_lines() {
    // Before the fix the two preceding CRLF lines pushed this to Column 11.
    let err = eval_err(UNDEF_CRLF);
    let expected = concat!(
        "Error: Undefined function or variable 'y'.\n",
        "  --> Line 3, Column 9:\n",
        "   |\n",
        "  3 | x = 1 + y\n",
        "   |         ^",
    );
    assert_eq!(format_eval_error(&err, UNDEF_CRLF), expected);
}

#[test]
fn rendered_line_text_never_contains_carriage_return() {
    let formatted = format_eval_error(&eval_err(UNDEF_CRLF), UNDEF_CRLF);
    assert!(
        !formatted.contains('\r'),
        "stripped \\r from display: {formatted:?}"
    );
}

// ---------------------------------------------------------------------------
// span_to_location unit table: LF bookkeeping
// ---------------------------------------------------------------------------

#[test]
fn span_to_location_lf_bookkeeping() {
    let src = "a = 1;\nb = 2;\ncde"; // 'c' sits at byte offset 14
                                     // Span inside a line: Column 5 in Line 1.
    assert_eq!(
        span_to_location(src, Span::new(4, 5)),
        (1, 5, 1, "a = 1;".into())
    );
    // Span covering a full line: Line 2, Columns 1..7.
    assert_eq!(
        span_to_location(src, Span::new(7, 13)),
        (2, 1, 6, "b = 2;".into())
    );
    // Span on the last line.
    assert_eq!(
        span_to_location(src, Span::new(14, 16)),
        (3, 1, 2, "cde".into())
    );
}

// ---------------------------------------------------------------------------
// span_to_location unit table: CRLF bookkeeping (regression on the fix)
// ---------------------------------------------------------------------------

#[test]
fn span_to_location_crlf_bookkeeping() {
    let crlf = "a = 1;\r\nb = 2;\r\ncde"; // 'c' sits at byte offset 16
                                          // Old buggy bookkeeping reported Line 3, Column 2 here.
    assert_eq!(
        span_to_location(crlf, Span::new(16, 18)),
        (3, 1, 2, "cde".into())
    );
    // CRLF and LF layouts must resolve to identical (line, column).
    let lf = "a = 1;\nb = 2;\ncde";
    assert_eq!(
        span_to_location(lf, Span::new(14, 16)),
        (3, 1, 2, "cde".into())
    );
}

#[test]
fn span_to_location_single_crlf_line() {
    let src = "x = 1 + y\r\n"; // 'y' at byte 8, '\r' at 9, '\n' at 10
    assert_eq!(
        span_to_location(src, Span::new(8, 9)),
        (1, 9, 1, "x = 1 + y".into())
    );
}

#[test]
fn span_to_location_edge_cases() {
    // Empty source is total.
    assert_eq!(span_to_location("", Span::new(0, 0)), (1, 1, 1, "".into()));
    // Offsets clamp into the existing last line.
    let src = "abc";
    assert_eq!(
        span_to_location(src, Span::new(2, 3)),
        (1, 3, 1, "abc".into())
    );
    // Point span (zero width) still reports one caret-width.
    assert_eq!(
        span_to_location(src, Span::new(1, 1)),
        (1, 2, 1, "abc".into())
    );
}
