//! Milestone A3.1.2 regression tests.
//!
//! Locks every behavioral fix landed since Milestone A3's acceptance turn:
//!   1. Scalar scientific display under `format short` (2-digit signed exponent,
//!      the 1e10 threshold, full-digit integers below the threshold).
//!   2. Matrix display parity: ONE shared column width and ONE common scale
//!      factor decision for the whole matrix, even with mixed magnitudes.
//!   3. reshape size arguments: `[]` auto-dimension, `[]` vs `-1`, multiple `[]`,
//!      zero, and numel-mismatch variants keep MATLAB's exact identifiers/messages.
//!   4. Mixed-integer-class operations keep MATLAB's exact refusal text.
//!   5. Diagnostics prerequisite: binary expressions own a whole-expression span
//!      (drives the CLI caret underline).
//!   6. Docs coverage: every registered builtin has a doc page and help text.

use rustlab_eval::EvalError;
use rustlab_math::Value;
use rustlab_parse::{parse, Expr, Span, Stmt};
use rustlab_stdlib::{eval_matlab, help, new_interpreter};
use std::path::Path;

/// Runs `code` and returns the join of every REPL log entry, i.e. exactly what
/// the CLI would print (`name =\n\n<Display of the value>\n` per statement).
fn rendered(code: &str) -> String {
    let mut interp = new_interpreter();
    interp
        .eval_script(code)
        .unwrap_or_else(|e| panic!("eval failed for `{code}`: {e}"));
    interp.output_log.join("")
}

/// Runs `code`, then renders variable `name` from the environment.
fn rendered_var(code: &str, name: &str) -> String {
    let mut interp = new_interpreter();
    interp
        .eval_script(code)
        .unwrap_or_else(|e| panic!("eval failed for `{code}`: {e}"));
    let value = interp
        .env
        .get(name)
        .unwrap_or_else(|| panic!("`{name}` was not defined by `{code}`"));
    format!("{value}")
}

// ---------------------------------------------------------------------------
// 1. Scalar scientific display (format short)
// ---------------------------------------------------------------------------

#[test]
fn scalar_scientific_display_goldens() {
    // Values far below display resolution -> exact MATLAB exponential layout.
    assert_eq!(rendered("x = 1e-16"), "x =\n\n   1.0000e-16\n");
    // Exponent is 2-digit zero-padded and SIGNED (-04, not 04, not 4).
    assert_eq!(rendered("x = 5e-4"), "x =\n\n   5.0000e-04\n");
    // 12345.678 (>= 1e4) switches to scientific with a +04 exponent.
    assert_eq!(rendered("x = 12345.678"), "x =\n\n   1.2346e+04\n");
    // 100000 still prints full integer digits below the 1e10 threshold.
    assert_eq!(rendered("x = 100000"), "x =\n\n    100000\n");
    // 1e10 crosses the fixed threshold (was 1e12 before A3.1.2) -> scientific.
    assert_eq!(rendered("x = 1e10"), "x =\n\n   1.0000e+10\n");
    // Negatives keep sign placement and exponent padding.
    assert_eq!(rendered("x = -12345.678"), "x =\n\n   -1.2346e+04\n");
    assert_eq!(rendered("x = -0.00025"), "x =\n\n   -2.5000e-04\n");
    // Plain small integers keep the ordinary fixed-point gutter.
    assert_eq!(rendered("x = 1"), "x =\n\n    1\n");
}

#[test]
fn scalar_display_via_env_matches_output_log() {
    // Whatever path renders a value (REPL log, CLI, env Display), it must agree.
    assert_eq!(rendered_var("x = 5e-4", "x"), "   5.0000e-04");
    assert_eq!(rendered_var("x = 12345.678", "x"), "   1.2346e+04");
    assert_eq!(rendered_var("x = 100000", "x"), "    100000");
}

// ---------------------------------------------------------------------------
// 2. Matrix display parity: one shared column width + one common scale factor
// ---------------------------------------------------------------------------

#[test]
fn matrix_display_parity_goldens() {
    // [1e-16 1]: the scale decision comes from the WHOLE-matrix maximum. The max
    // (1.0) is inside [1e-3, 1e4), so no multiplier is applied: the tiny cell
    // clips to 0.0000 and BOTH columns share ONE width.
    assert_eq!(rendered("m = [1e-16 1]"), "m =\n\n    0.0000       1\n");
    // [1 1000000]: integer magnitudes always print full digits in one width.
    assert_eq!(rendered("m = [1 1000000]"), "m =\n\n          1  1000000\n");
    // [12345.678 1]: mixed magnitudes get ONE common scale factor for the whole
    // matrix (never per-column) and cells keep a single common width.
    assert_eq!(
        rendered("m = [12345.678 1]"),
        "m =\n\n  1.0e+04 *\n\n    1.2346  0.0001\n"
    );
}

#[test]
fn matrix_display_width_is_uniform_across_rows() {
    let out = rendered("m = [0.5 100000; -2.25 3]");
    let rows: Vec<&str> = out.lines().collect();
    // "m =", "", "  1.0e+05 *", "", data row 1, data row 2.
    assert_eq!(rows.len(), 6, "unexpected rendering: {out:?}");
    assert_eq!(rows[2], "  1.0e+05 *", "whole-matrix common scale header");
    // Exactly one shared column width: both data rows carry the same length, and
    // the tiny clipped cells (0.0000) sit inside the same columns as raw ones.
    assert_eq!(rows[4].chars().count(), rows[5].chars().count());
    assert_eq!(rows[4], "     0.0000        1");
    assert_eq!(rows[5], "    -0.0000   0.0000");
}

// ---------------------------------------------------------------------------
// 3. reshape: `[]` auto-dimension vs `-1` and other invalid size arguments
// ---------------------------------------------------------------------------

#[test]
fn reshape_empty_bracket_autodim_matches_explicit_dims() {
    let auto = rendered_var("a = reshape(1:6, [], 2)", "a");
    let explicit = rendered_var("b = reshape(1:6, 3, 2)", "b");
    assert_eq!(auto, explicit, "reshape(A, [], 2) == reshape(A, 3, 2)");
    // Column-major identity result, pinned as display regression lock.
    assert_eq!(auto, "       1     4\n       2     5\n       3     6");

    let mut interp = new_interpreter();
    interp.eval_script("a = reshape(1:6, [], 2)").unwrap();
    match interp.env.get("a") {
        Some(Value::Double(d)) => assert_eq!(d.shape(), &[3, 2], "[] filled in as 3x2"),
        other => panic!("expected 3x2 double, got {other:?}"),
    }
}

#[test]
fn reshape_empty_bracket_in_higher_dimension() {
    let mut interp = new_interpreter();
    interp
        .eval_script("c = reshape(1:12, 2, 2, [])")
        .expect("eval failed");
    match interp.env.get("c") {
        Some(Value::Double(d)) => assert_eq!(d.shape(), &[2, 2, 3], "12/4 = 3 inferred"),
        other => panic!("expected 2x2x3 double, got {other:?}"),
    }
}

#[test]
fn reshape_invalid_size_arguments_keep_matlab_diagnostics() {
    let cases: &[(&str, &str, &str)] = &[
        // (code, identifier, message)
        (
            "reshape(1:6, [], -1)",
            "MATLAB:checkdim:badSizeArg",
            "Size arguments must be integer scalars or [].",
        ),
        (
            "reshape(1:6, -1, 2)",
            "MATLAB:checkdim:badSizeArg",
            "Size arguments must be integer scalars or [].",
        ),
        (
            "reshape(1:6, 2, -1, 3)",
            "MATLAB:checkdim:badSizeArg",
            "Size arguments must be integer scalars or [].",
        ),
        (
            "reshape(1:8, [], [])",
            "MATLAB:getReshapeDims:multipleNegative",
            "Only one dimension can be unknown ([]).",
        ),
        (
            "reshape(1:6, [], 0)",
            "MATLAB:getReshapeDims:notSameNumel",
            "Dimensions must evaluate to the same number of elements.",
        ),
        (
            "reshape(1:6, 2, 2)",
            "MATLAB:getReshapeDims:notSameNumel",
            "To RESHAPE the number of elements must not change.",
        ),
    ];

    for (code, identifier, message) in cases {
        match eval_matlab(code) {
            Err(EvalError::UserError {
                identifier: id,
                message: msg,
                ..
            }) => {
                assert_eq!(&id, identifier, "error identifier for `{code}`");
                assert_eq!(&msg, message, "error message for `{code}`");
            }
            Err(other) => panic!("expected UserError for `{code}`, got {other:?}"),
            Ok(value) => panic!("`{code}` unexpectedly succeeded with {value:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Mixed-integer-class operation error text (exact MATLAB wording)
// ---------------------------------------------------------------------------

#[test]
fn mixed_integer_class_binary_ops_use_exact_matlab_text() {
    let expected =
        "Integers can only be combined with integers of the same class, or scalar doubles.";

    for code in ["x = int8(1) + int16(2)", "x = uint32(3) * int32(4)"] {
        match eval_matlab(code) {
            Err(EvalError::Math { error, .. }) => {
                assert_eq!(
                    error.to_string(),
                    expected,
                    "mixed-class message for `{code}`"
                );
            }
            Err(other) => panic!("expected EvalError::Math for `{code}`, got {other:?}"),
            Ok(value) => panic!("`{code}` unexpectedly succeeded with {value:?}"),
        }
    }
}

#[test]
fn mixed_integer_assignment_refuses_inconsistent_class() {
    let err = eval_matlab("a = int8([1 2; 3 4]); a(1, 1) = int16(5);")
        .err()
        .unwrap_or_else(|| panic!("mixed-class indexed assignment must fail"));
    match err {
        EvalError::Math { error, .. } => assert_eq!(
            error.to_string(),
            "Integers can only be combined with integers of the same class, or scalar doubles.",
        ),
        other => panic!("expected EvalError::Math, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 5. Binary expressions own a whole-expression span (drives the caret)
// ---------------------------------------------------------------------------

#[test]
fn binary_expression_span_covers_full_expression() {
    let stmts = parse("x = 1 + 2").unwrap();
    assert_eq!(stmts.len(), 1, "single statement expected");
    let value: &Expr = match &stmts[0] {
        Stmt::Assignment { value, .. } => value,
        other => panic!("expected Stmt::Assignment, got {other:?}"),
    };
    let span = value.span();
    // "1 + 2" occupies byte offsets 4..9, so the caret underline produced from
    // this span starts at Column 5 and runs the full expression width.
    assert_eq!(span, Span::new(4, 9), "binary span must merge the operands");
}

// ---------------------------------------------------------------------------
// 6. Docs coverage: every registered builtin has docs and help
// ---------------------------------------------------------------------------

#[test]
fn every_registered_builtin_has_docs_and_help() {
    let interp = new_interpreter();
    let names = interp.builtin_names();
    assert!(
        names.len() >= 160,
        "expected >= 160 registered builtins, got {}",
        names.len()
    );

    let docs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/functions");

    for name in &names {
        let doc_path = docs_dir.join(format!("{name}.md"));
        assert!(
            doc_path.is_file(),
            "docs/functions/{name}.md is missing for registered builtin"
        );
        let body = std::fs::read_to_string(&doc_path)
            .unwrap_or_else(|e| panic!("failed to read docs/functions/{name}.md: {e}"));
        assert!(!body.trim().is_empty(), "docs/functions/{name}.md is empty");
        assert!(
            !body.to_lowercase().contains("todo"),
            "docs/functions/{name}.md still contains a TODO"
        );
        let help_text = help::get_help_text(name)
            .unwrap_or_else(|| panic!("help text for `{name}` is missing"));
        assert!(
            !help_text.trim().is_empty(),
            "help text for `{name}` is empty"
        );
        assert!(
            help_text.contains(name),
            "help text for `{name}` lacks its own name"
        );
    }
}
