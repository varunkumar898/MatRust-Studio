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
fn scalar_scientific_display_goldens_unverified() {
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

#[test]
fn test_scientific_notation_threshold_rule_cited_unverified() {
    // Citation: MathWorks MATLAB Documentation - "Set Output Display Format" (format short rule):
    // "If the largest element of a matrix is larger than 1000 or smaller than 0.001,
    //  MATLAB applies a common scale factor for the short and long formats."
    // URL: https://www.mathworks.com/help/matlab/ref/format.html

    // Value in [0.01, 1000) displays in standard fixed-point short format (4 decimals)
    assert_eq!(rendered("x = 0.01"), "x =\n\n    0.0100\n");
    assert_eq!(rendered("x = 999.9"), "x =\n\n    999.9000\n");

    // Value < 0.01 switches to signed zero-padded 2-digit exponent scientific notation (B2.1.1)
    assert_eq!(rendered("x = 0.001"), "x =\n\n   1.0000e-03\n");
    assert_eq!(rendered("x = 0.00099"), "x =\n\n   9.9000e-04\n");
    assert_eq!(rendered("x = 1e-16"), "x =\n\n   1.0000e-16\n");

    // Value >= 10000 non-integer switches to scientific notation
    assert_eq!(rendered("x = 12345.678"), "x =\n\n   1.2346e+04\n");
}

// A3.1.5: these two names were claimed by the A3.1.3 report but never existed
// in either git revision. They are re-created here as real tests.
#[test]
fn scalar_integer_display_goldens() {
    // Cited rule: integral doubles below 1e10 print full digits (format short).
    // https://www.mathworks.com/help/matlab/ref/format.html
    assert_eq!(rendered("x = 0"), "x =\n\n    0\n");
    assert_eq!(rendered("x = 1"), "x =\n\n    1\n");
    assert_eq!(rendered("x = -42"), "x =\n\n    -42\n");
    assert_eq!(rendered("x = 100000"), "x =\n\n    100000\n");
    assert_eq!(rendered("x = 1e9"), "x =\n\n    1000000000\n");
}

#[test]
fn non_integer_class_display_goldens_unverified() {
    // Scalar boundary rules (B2.1.1): 1234.5678 is in [1e3,1e4) and prints
    // scientific (1.2346e+03); 0.001 and 0.0012 print in e-notation.
    assert_eq!(rendered("x = 1e-16"), "x =\n\n   1.0000e-16\n");
    assert_eq!(rendered("x = 5e-4"), "x =\n\n   5.0000e-04\n");
    assert_eq!(rendered("x = 1234.5678"), "x =\n\n   1.2346e+03\n");
    assert_eq!(rendered("x = 0.001"), "x =\n\n   1.0000e-03\n");
    assert_eq!(rendered("x = 0.0012"), "x =\n\n   1.2000e-03\n");
    assert_eq!(rendered("x = 0.01"), "x =\n\n    0.0100\n");
    assert_eq!(rendered("x = 12345.678"), "x =\n\n   1.2346e+04\n");
}

#[test]
fn matrix_display_common_scale_boundary_unverified() {
    // [0.001 1000] has max=1000 and dyn_range=1e6 < 1e9, so it extracts a common scale factor 1.0e+03 * (B2.1.1)
    let s = rendered("m = [0.001 1000]");
    assert!(s.contains("1.0e+03 *"), "must have scale factor: {s}");
    assert!(
        s.contains("0.0000") && s.contains("1.0000"),
        "elements must be formatted with scale: {s}"
    );
}

#[test]
fn test_num2str_noninteger_matrix_g_style_unverified() {
    // Source (maximum 5 significant digits):
    // https://www.mathworks.com/help/matlab/ref/num2str.html
    // The precise %g rounding/alignment is unverified. A3.1.5 requires no
    // trailing zeros in non-integer num2str output.
    fn num2str_of(expr: &str) -> String {
        let mut interp = new_interpreter();
        let value = interp
            .eval_script(&format!("s = num2str({expr});"))
            .unwrap_or_else(|e| panic!("eval failed for num2str({expr}): {e}"));
        let _ = value;
        match interp.env.get("s") {
            Some(Value::Char(ca)) => ca.as_slice().iter().collect(),
            other => panic!("num2str did not return char: {other:?}"),
        }
    }

    assert_eq!(num2str_of("1234.5678"), "1234.6");
    assert_eq!(num2str_of("0.001"), "0.001");
    assert_eq!(num2str_of("0.0012"), "0.0012");
    assert_eq!(num2str_of("[1e-16 1]"), "1e-16  1");
    assert_eq!(
        num2str_of("[1.234 56.78; 9.1 0.2]"),
        "1.234  56.78\n  9.1    0.2"
    );
}

// ---------------------------------------------------------------------------
// 2. Matrix display parity: one shared column width + one common scale factor
// ---------------------------------------------------------------------------

#[test]
fn matrix_display_parity_goldens_unverified() {
    // [1e-16 1]: extreme dynamic range (1e16 >= 1e9), each entry in e-notation (B2.1.1)
    let s_extreme = rendered("m = [1e-16 1]");
    assert!(
        s_extreme.contains("1.0000e-16") && s_extreme.contains("1.0000e+00"),
        "{s_extreme}"
    );
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
    // integer elements in non-integer matrices format with decimals (1.0000).
    assert_eq!(rows[4].chars().count(), rows[5].chars().count());
    assert_eq!(rows[4], "     0.0000   1.0000");
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
    let name_set: std::collections::BTreeSet<_> = names.iter().cloned().collect();

    let docs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/functions");

    // 1. Forward check: every registered builtin must have a non-empty, non-TODO markdown doc and help text
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

    // 2. Reverse check: zero orphan doc files in docs/functions
    let mut orphan_docs = Vec::new();
    #[cfg(not(feature = "data"))]
    let data_builtins = [
        "median",
        "mode",
        "prctile",
        "quantile",
        "cov",
        "corrcoef",
        "histcounts",
        "movmean",
        "movmedian",
        "cummax",
        "cummin",
        "zscore",
        "sortrows",
        "readmatrix",
        "writematrix",
    ];
    for entry in std::fs::read_dir(&docs_dir).expect("failed to read docs_dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                #[cfg(not(feature = "data"))]
                if data_builtins.contains(&stem) {
                    continue;
                }
                if !name_set.contains(stem) {
                    orphan_docs.push(stem.to_string());
                }
            }
        }
    }
    assert!(
        orphan_docs.is_empty(),
        "Orphan markdown doc files found without a registered builtin: {:?}",
        orphan_docs
    );

    // Export sorted list to docs/builtins.txt
    let mut sorted_names = names;
    sorted_names.sort();
    if let Some(parent) = docs_dir.parent() {
        let builtins_txt = parent.join("builtins.txt");
        let _ = std::fs::write(builtins_txt, sorted_names.join("\n") + "\n");
    }
}
