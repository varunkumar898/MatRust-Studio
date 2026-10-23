//! Layout lock-in tests for the storage contract documented on `DenseMatrix`
//! (crates/values/src/matrix.rs): `data` is row-major internally, and MATLAB's
//! column-major order is reached only through `get_linear` / `to_col_major`.
//!
//! Ground truth here is MATLAB reference semantics for A = [1 2 3; 4 5 6]:
//!   A(:)'                = [1 4 2 5 3 6]  (A(:) is a 6x1 column)
//!   reshape(1:6,2,3)     = [1 3 5; 2 4 6] (columns are filled first)
//!   prctile(A,p,1)       = 3x3, row i = p(i) percentile of every column
//!   prctile(A,p,2)       = 2x3, row r = the three quantiles of row r
//!
//! Hand computation for the 2x3 matrix (n = 2 per column, n = 3 per row),
//! MATLAB rank rule r = 0.5 + n*p:
//!   dim 1, per column:  p25 -> j=1.0 -> 1; p50 -> j=1.5 -> 2.5; p75 -> j=2.0 -> 4
//!   dim 2, per row:     p25 -> j=1.25; p50 -> j=2.0; p75 -> j=2.75

use matlab_runtime::run_source;

fn matrix<'a>(run: &'a matlab_runtime::RunResult, name: &str) -> &'a matlab_values::DenseMatrix {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .unwrap_or_else(|| panic!("{} should be a real matrix", name))
}

/// Row-major element access, matching the DenseMatrix storage contract.
fn at(m: &matlab_values::DenseMatrix, r: usize, c: usize) -> f64 {
    m.data[r * m.cols + c]
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "{}: expected {}, got {}",
        context,
        expected,
        actual
    );
}

#[test]
fn layout_colon_is_lineared_column_vector() {
    // MATLAB: A(:) is always a column vector in column-major element order,
    // so A(:) = [1;4;2;5;3;6] and A(:)' = [1 4 2 5 3 6] (a 1x6 row).
    let r = run_source("A = [1 2 3; 4 5 6]; s = size(A(:)); w = A(:)';").unwrap();
    let s = matrix(&r, "s");
    assert_eq!(s.data, vec![6.0, 1.0], "size(A(:)) = [6 1]");

    let w = matrix(&r, "w");
    assert_eq!((w.rows, w.cols), (1, 6), "A(:)' is a 1x6 row");
    assert_eq!(w.data, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn layout_colon_flips_row_vector_to_column() {
    // MATLAB: [1 2 3](:) is a 3x1 column, not a row.
    let r = run_source("v = [1 2 3]; s = size(v(:)); c = v(:);").unwrap();
    let s = matrix(&r, "s");
    assert_eq!(s.data, vec![3.0, 1.0], "size(v(:)) = [3 1]");

    let c = matrix(&r, "c");
    assert_eq!((c.rows, c.cols), (3, 1));
    assert_eq!(c.data, vec![1.0, 2.0, 3.0]);
}

#[test]
fn layout_reshape_fills_column_major() {
    // MATLAB: reshape fills columns first, so reshape(1:6,2,3) = [1 3 5; 2 4 6].
    // Row-major storage of that displayed matrix is exactly [1,2,3,4,5,6].
    let r = run_source("R = reshape(1:6, 2, 3); s = size(R);").unwrap();
    let s = matrix(&r, "s");
    assert_eq!(s.data, vec![2.0, 3.0], "reshape(1:6,2,3) is 2x3");

    let m = matrix(&r, "R");
    assert_eq!((m.rows, m.cols), (2, 3));
    // Row-major storage of the displayed matrix [1 3 5; 2 4 6].
    assert_eq!(m.data, vec![1.0, 3.0, 5.0, 2.0, 4.0, 6.0]);

    // Display-level rows must be [1 3 5] and [2 4 6].
    for c in 0..3 {
        assert_close(at(m, 0, c), (2 * c + 1) as f64, "top row");
        assert_close(at(m, 1, c), (2 * c + 2) as f64, "bottom row");
    }
}

#[test]
fn layout_prctile_dim1_hand_computed() {
    // A = [1 2 3; 4 5 6], percentiles along columns (n = 2, r = 0.5 + 2p):
    //   p25 = 1.0 (j=1, first element), p50 = 2.5 (j=1.5), p75 = 4.0 (j=2)
    // Column results: [1 2 3; 2.5 3.5 4.5; 4 5 6] (3x3), row-major stored.
    let r = run_source("A = [1 2 3; 4 5 6]; p = prctile(A, [25 50 75], 1);").unwrap();
    let p = matrix(&r, "p");
    assert_eq!((p.rows, p.cols), (3, 3));

    let expected = [[1.0, 2.0, 3.0], [2.5, 3.5, 4.5], [4.0, 5.0, 6.0]];
    for (i, row) in expected.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            assert_close(at(p, i, c), *value, "prctile(A,p,1)");
        }
    }
}

#[test]
fn layout_prctile_dim2_hand_computed() {
    // A = [1 2 3; 4 5 6], percentiles along rows (n = 3, r = 0.5 + 3p):
    //   row 1 = [1 2 3] -> p25 j=1.25 -> 1.25; p50 j=2.0 -> 2.0; p75 j=2.75 -> 2.75
    //   row 2 = [4 5 6] -> [4.25, 5, 5.75]
    // Result is 2x3 (one row per data row, one column per percentile).
    let r = run_source("A = [1 2 3; 4 5 6]; q = prctile(A, [25 50 75], 2);").unwrap();
    let q = matrix(&r, "q");
    assert_eq!((q.rows, q.cols), (2, 3));

    let expected = [[1.25, 2.0, 2.75], [4.25, 5.0, 5.75]];
    for (r_i, row) in expected.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            assert_close(at(q, r_i, c), *value, "prctile(A,p,2)");
        }
    }
}

#[test]
fn layout_prctile_scalar_default_is_row_vector() {
    // MATLAB prctile(matrix, scalar) with unspecified dim reduces along the
    // first non-singleton dimension and returns a 1xncols row vector.
    let r = run_source("A = [1 2 3; 4 5 6]; m = prctile(A, 50);").unwrap();
    let m = matrix(&r, "m");
    assert_eq!((m.rows, m.cols), (1, 3), "scalar p on a matrix -> 1x3 row");
    assert_close(at(m, 0, 0), 2.5, "50th of column 1");
    assert_close(at(m, 0, 1), 3.5, "50th of column 2");
    assert_close(at(m, 0, 2), 4.5, "50th of column 3");
}

/// MATLAB bracketConcatenation ground-truth table.
///
/// Rule (parser.rs `in_bracket_literal`): inside `[ ]`, a `+`/`-` that is
/// detached from the previous element by whitespace but attached to its
/// operand starts a NEW element with a unary sign. With no whitespace, or
/// whitespace on both sides, it is subtraction. The rule is scoped to `[ ]`:
/// parentheses and call-argument contexts evaluate as ordinary expressions.
/// Each row asserts the element count (size) AND the stored values
/// (row-major), per the DenseMatrix storage contract.
#[test]
fn bracket_concatenation_table() {
    #[rustfmt::skip]
    let table: &[(&str, usize, usize, &[f64])] = &[
        // (source, rows, cols, row-major values)
        ("x = [1 2 3]",        1, 3, &[1.0, 2.0, 3.0]),
        ("x = [1 -2 3]",       1, 3, &[1.0, -2.0, 3.0]),   // " -x" -> new element
        ("x = [1 -2]",         1, 2, &[1.0, -2.0]),
        ("x = [1, -2]",        1, 2, &[1.0, -2.0]),        // comma plus detached unary minus
        ("x = [1 -2 -3]",      1, 3, &[1.0, -2.0, -3.0]),
        ("x = [0-1]",          1, 1, &[-1.0]),             // no space -> subtraction
        ("x = [1 - 2]",        1, 1, &[-1.0]),             // spaces both sides -> subtraction
        ("x = [1+2]",          1, 1, &[3.0]),
        ("x = [1 + 2]",        1, 1, &[3.0]),
        ("x = [1 +2]",         1, 2, &[1.0, 2.0]),         // " +x" -> new element
        ("x = [1 - 2 -3]",     1, 2, &[-1.0, -3.0]),       // mixed: binary, then unary
        ("x = [0 -1; 1 0]",    2, 2, &[0.0, -1.0, 1.0, 0.0]),
        ("x = [(0 - 1) 2]",    1, 2, &[-1.0, 2.0]),        // parentheses reset the rule
        ("x = 2; x = [x -2]",  1, 2, &[2.0, -2.0]),
        ("a = 5; b = 3; x = [a -b]", 1, 2, &[5.0, -3.0]),
        ("a = 5; b = 3; x = [a - b]", 1, 1, &[2.0]),
        ("a = 5; b = 3; x = [a' b']", 1, 2, &[5.0, 3.0]),
        ("x = [10 20]; x = [x(1) -x(2)]", 1, 2, &[10.0, -20.0]),
        ("x = max(1 - 2)",     1, 1, &[-1.0]),             // call args: ordinary expression
        ("x = abs([1 -2])",    1, 2, &[1.0, 2.0]),         // bracket inside call: rule applies
    ];

    for (source, rows, cols, values) in table {
        let run = run_source(source)
            .unwrap_or_else(|e| panic!("source must parse and run: {source:?}\n{e}"));
        let x = matrix(&run, "x");
        assert_eq!(
            (x.rows, x.cols),
            (*rows, *cols),
            "size mismatch for {source:?}: got {}x{}, want {rows}x{cols}",
            x.rows,
            x.cols
        );
        assert_eq!(
            x.data.len(),
            values.len(),
            "element count mismatch for {source:?}"
        );
        for (r, c) in (0..*rows).flat_map(|r| (0..*cols).map(move |c| (r, c))) {
            assert_close(
                at(x, r, c),
                values[r * cols + c],
                &format!("{source:?} element ({r},{c})"),
            );
        }
    }
}
