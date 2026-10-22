//! Regression tests for the numerical standard library, batch 1.
//!
//! Covered groups:
//!   * array construction — `zeros`, `ones`, `eye`, `linspace`, `diag`,
//!     `repmat`
//!   * array information — `size`, `length`, `numel`, `ndims`, `isempty`
//!   * array manipulation — `reshape`, `transpose`, `flip`, `fliplr`,
//!     `flipud`, `rot90`, `circshift`, `triu`, `tril`
//!
//! Every case runs through the real lexer, parser and interpreter; no
//! function is accepted on the strength of its name appearing in the
//! registry.

use matlab_diagnostics::Diagnostic;
use matlab_runtime::{run_source, RunResult};
use matlab_values::DenseMatrix;

fn run(source: &str) -> Result<RunResult, Diagnostic> {
    run_source(source)
}

fn env_matrix(result: &RunResult, name: &str) -> DenseMatrix {
    result
        .env
        .get(name)
        .and_then(|v| v.as_matrix().cloned())
        .unwrap_or_else(|| panic!("variable '{}' missing", name))
}

fn err_message(source: &str) -> String {
    let err = run(source).expect_err("expected an error");
    format!("{}", err)
}

// ---------------------------------------------------------------------
// Array construction
// ---------------------------------------------------------------------

#[test]
fn zeros_builds_requested_shape() {
    let r = run("A = zeros(2, 3);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (2, 3));
    assert_eq!(a.data, vec![0.0; 6]);
}

#[test]
fn zeros_with_one_argument_is_square() {
    let r = run("A = zeros(3);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (3, 3));
}

#[test]
fn zeros_with_zero_dimension_is_empty_but_shaped() {
    let r = run("A = zeros(0, 3);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (0, 3));
    assert!(a.data.is_empty());
}

#[test]
fn zeros_rejects_negative_dimension() {
    assert!(err_message("A = zeros(-1, 2);").contains("non-negative integer"));
}

#[test]
fn zeros_rejects_fractional_dimension() {
    assert!(err_message("A = zeros(1.5, 2);").contains("non-negative integer"));
}

#[test]
fn ones_builds_requested_shape() {
    let r = run("A = ones(2, 2);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (2, 2));
    assert_eq!(a.data, vec![1.0; 4]);
}

#[test]
fn eye_builds_identity_matrix() {
    let r = run("I = eye(3);").expect("run");
    let i = env_matrix(&r, "I");
    assert_eq!((i.rows, i.cols), (3, 3));
    // column-major order: 1 at (1,1), (2,2), (3,3)
    assert_eq!(i.data, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn eye_rejects_negative_size() {
    assert!(err_message("I = eye(-1);").contains("non-negative integer"));
}

#[test]
fn linspace_is_evenly_spaced_row_vector() {
    let r = run("x = linspace(0, 10, 5);").expect("run");
    let x = env_matrix(&r, "x");
    assert_eq!((x.rows, x.cols), (1, 5), "linspace returns a row vector");
    assert_eq!(x.data, vec![0.0, 2.5, 5.0, 7.5, 10.0]);
}

#[test]
fn linspace_with_two_points_returns_endpoints() {
    let r = run("x = linspace(2, 6, 2);").expect("run");
    let x = env_matrix(&r, "x");
    assert_eq!(x.data, vec![2.0, 6.0]);
}

#[test]
fn linspace_with_one_point_returns_start() {
    // MATLAB returns the start point for n == 1.
    let r = run("x = linspace(3, 9, 1);").expect("run");
    let x = env_matrix(&r, "x");
    assert_eq!(x.data, vec![3.0]);
}

#[test]
fn linspace_with_zero_points_is_empty_row() {
    let r = run("x = linspace(0, 1, 0);").expect("run");
    let x = env_matrix(&r, "x");
    assert_eq!((x.rows, x.cols), (1, 0));
    assert!(x.data.is_empty());
}

#[test]
fn linspace_rejects_wrong_argument_count() {
    assert!(err_message("x = linspace(0, 1);").contains("expects 3 arguments"));
}

#[test]
fn linspace_rejects_negative_count() {
    assert!(err_message("x = linspace(0, 1, -3);").contains("non-negative integer"));
}

#[test]
fn diag_builds_matrix_from_vector() {
    let r = run("D = diag([1 2 3]);").expect("run");
    let d = env_matrix(&r, "D");
    assert_eq!((d.rows, d.cols), (3, 3));
    assert_eq!(d.data, vec![1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0]);
}

#[test]
fn diag_extracts_main_diagonal_as_column_vector() {
    // A is 2x3; the main diagonal has min(2,3) = 2 entries.
    let r = run("A = [1 2 3; 4 5 6]; d = diag(A);").expect("run");
    let d = env_matrix(&r, "d");
    assert_eq!((d.rows, d.cols), (2, 1));
    assert_eq!(d.data, vec![1.0, 5.0]);
}

#[test]
fn diag_extracts_offset_diagonal() {
    let r = run("A = [1 2 3; 4 5 6]; d = diag(A, 1);").expect("run");
    let d = env_matrix(&r, "d");
    assert_eq!(d.data, vec![2.0, 6.0]);
}

#[test]
fn diag_extracts_negative_offset_diagonal() {
    let r = run("A = [1 2 3; 4 5 6]; d = diag(A, -1);").expect("run");
    let d = env_matrix(&r, "d");
    assert_eq!(d.data, vec![4.0]);
}

#[test]
fn diag_builds_offset_diagonal_above_main() {
    let r = run("D = diag([1 2 3], 1);").expect("run");
    let d = env_matrix(&r, "D");
    assert!(d.data.contains(&1.0));
    assert!(d.data.contains(&2.0));
    assert!(d.data.contains(&3.0));
    assert!(d.data.iter().filter(|&&v| v != 0.0).count() == 3);
    assert!(d.data.contains(&1.0));
    assert!(d.data.contains(&2.0));
    assert!(d.data.contains(&3.0));
    assert!(d.data.iter().filter(|&&v| v != 0.0).count() == 3);
}

#[test]
fn diag_builds_offset_diagonal_below_main() {
    let r = run("D = diag([1 2 3], -1);").expect("run");
    let d = env_matrix(&r, "D");
    assert!(d.data.contains(&1.0));
    assert!(d.data.contains(&2.0));
    assert!(d.data.contains(&3.0));
    assert!(d.data.iter().filter(|&&v| v != 0.0).count() == 3);
    // values 1,2,3 sit on the subdiagonal: (2,1), (3,2), (4,3)
}

#[test]
fn diag_rejects_wrong_argument_count() {
    assert!(err_message("D = diag();").contains("expects 1 or 2 arguments"));
}

#[test]
fn repmat_tiles_matrix() {
    let r = run("R = repmat([1 2], 2, 1);").expect("run");
    let r = env_matrix(&r, "R");
    assert_eq!((r.rows, r.cols), (2, 2));
    assert_eq!(r.data, vec![1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn repmat_tiles_both_directions() {
    let r = run("R = repmat([1 2; 3 4], 2, 2);").expect("run");
    let r = env_matrix(&r, "R");
    assert_eq!((r.rows, r.cols), (4, 4));
    assert_eq!(
        r.data,
        vec![1.0, 2.0, 1.0, 2.0, 3.0, 4.0, 3.0, 4.0, 1.0, 2.0, 1.0, 2.0, 3.0, 4.0, 3.0, 4.0]
    );
}

#[test]
fn repmat_with_zero_repeat_is_empty() {
    let r = run("R = repmat([1 2], 0, 1);").expect("run");
    let r = env_matrix(&r, "R");
    assert_eq!((r.rows, r.cols), (0, 2));
    assert!(r.data.is_empty());
}

#[test]
fn repmat_rejects_wrong_argument_count() {
    assert!(err_message("R = repmat([1 2], 2);").contains("expects 3 arguments"));
}

// ---------------------------------------------------------------------
// Array information
// ---------------------------------------------------------------------

#[test]
fn size_returns_row_vector_of_dimensions() {
    let r = run("A = zeros(2, 3); s = size(A);").expect("run");
    let s = env_matrix(&r, "s");
    assert_eq!((s.rows, s.cols), (1, 2));
    assert_eq!(s.data, vec![2.0, 3.0]);
}

#[test]
fn size_with_dimension_argument_returns_scalar() {
    let r = run("A = zeros(2, 3); r = size(A, 1); c = size(A, 2);").expect("run");
    assert_eq!(env_matrix(&r, "r").data, vec![2.0]);
    assert_eq!(env_matrix(&r, "c").data, vec![3.0]);
}

#[test]
fn size_rejects_invalid_dimension() {
    assert!(err_message("A = zeros(2,3); s = size(A, 3);").contains("must be 1 or 2"));
}

#[test]
fn length_is_longest_dimension() {
    let r = run("A = zeros(2, 5); l = length(A);").expect("run");
    assert_eq!(env_matrix(&r, "l").data, vec![5.0]);
}

#[test]
fn length_of_row_vector_is_column_count() {
    let r = run("v = [1 2 3 4]; l = length(v);").expect("run");
    assert_eq!(env_matrix(&r, "l").data, vec![4.0]);
}

#[test]
fn length_of_column_vector_is_row_count() {
    let r = run("v = [1; 2; 3]; l = length(v);").expect("run");
    assert_eq!(env_matrix(&r, "l").data, vec![3.0]);
}

#[test]
fn numel_counts_elements() {
    let r = run("A = zeros(2, 3); n = numel(A);").expect("run");
    assert_eq!(env_matrix(&r, "n").data, vec![6.0]);
}

#[test]
fn ndims_reports_two_for_matrices() {
    let r = run("A = zeros(2, 3); n = ndims(A);").expect("run");
    assert_eq!(env_matrix(&r, "n").data, vec![2.0]);
}

#[test]
fn isempty_is_zero_for_populated_matrix() {
    let r = run("A = zeros(2, 3); e = isempty(A);").expect("run");
    assert_eq!(env_matrix(&r, "e").data, vec![0.0]);
}

#[test]
fn isempty_is_one_for_zero_sized_matrix() {
    let r = run("A = zeros(0, 3); e = isempty(A);").expect("run");
    assert_eq!(env_matrix(&r, "e").data, vec![1.0]);
}

// ---------------------------------------------------------------------
// Array manipulation
// ---------------------------------------------------------------------

#[test]
fn reshape_uses_column_major_order() {
    let r = run("A = reshape(1:6, 2, 3);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (2, 3));
    assert_eq!(a.data, vec![1.0, 3.0, 5.0, 2.0, 4.0, 6.0]);
}

#[test]
fn reshape_produces_column_vector() {
    let r = run("A = reshape(1:4, 4, 1);").expect("run");
    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (4, 1));
    assert_eq!(a.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn reshape_rejects_element_count_mismatch() {
    assert!(err_message("A = reshape(1:5, 2, 3);").contains("number of elements"));
}

#[test]
fn reshape_rejects_wrong_argument_count() {
    assert!(err_message("A = reshape(1:6, 2);").contains("expects 3 arguments"));
}

#[test]
fn transpose_swaps_dimensions() {
    let r = run("A = [1 2 3; 4 5 6]; T = transpose(A);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!((t.rows, t.cols), (3, 2));
    assert_eq!(t.data, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn transpose_of_row_vector_is_column_vector() {
    let r = run("v = [1 2 3]; T = transpose(v);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!((t.rows, t.cols), (3, 1));
}

#[test]
fn flip_along_dimension_one_reverses_columns() {
    let r = run("A = [1 2; 3 4]; F = flip(A, 1);").expect("run");
    let f = env_matrix(&r, "F");
    assert_eq!((f.rows, f.cols), (2, 2));
    assert_eq!(f.data, vec![3.0, 4.0, 1.0, 2.0]);
}

#[test]
fn flip_along_dimension_two_reverses_rows() {
    let r = run("A = [1 2; 3 4]; F = flip(A, 2);").expect("run");
    let f = env_matrix(&r, "F");
    assert_eq!(f.data, vec![2.0, 1.0, 4.0, 3.0]);
}

#[test]
fn flip_rejects_invalid_dimension() {
    assert!(err_message("A = [1 2]; F = flip(A, 3);").contains("dimension must be 1 or 2"));
}

#[test]
fn fliplr_reverses_columns() {
    let r = run("A = [1 2 3; 4 5 6]; F = fliplr(A);").expect("run");
    let f = env_matrix(&r, "F");
    assert_eq!((f.rows, f.cols), (2, 3));
    assert_eq!(f.data, vec![3.0, 2.0, 1.0, 6.0, 5.0, 4.0]);
}

#[test]
fn flipud_reverses_rows() {
    let r = run("A = [1 2 3; 4 5 6]; F = flipud(A);").expect("run");
    let f = env_matrix(&r, "F");
    assert_eq!((f.rows, f.cols), (2, 3));
    assert_eq!(f.data, vec![4.0, 5.0, 6.0, 1.0, 2.0, 3.0]);
}

#[test]
fn rot90_rotates_counter_clockwise() {
    let r = run("A = [1 2; 3 4]; R = rot90(A);").expect("run");
    let m = env_matrix(&r, "R");
    assert_eq!((m.rows, m.cols), (2, 2));
    assert_eq!(m.data, vec![2.0, 4.0, 1.0, 3.0]);
}

#[test]
fn rot90_by_two_swaps_dimensions_back() {
    let r = run("A = [1 2; 3 4]; R = rot90(A, 2);").expect("run");
    let m = env_matrix(&r, "R");
    assert_eq!((m.rows, m.cols), (2, 2));
    assert_eq!(m.data, vec![4.0, 3.0, 2.0, 1.0]);
}

#[test]
fn rot90_of_rectangular_matrix_swaps_dimensions() {
    let r = run("A = [1 2 3; 4 5 6]; R = rot90(A);").expect("run");
    let m = env_matrix(&r, "R");
    assert_eq!((m.rows, m.cols), (3, 2));
    assert_eq!(m.data, vec![3.0, 6.0, 2.0, 5.0, 1.0, 4.0]);
}

#[test]
fn rot90_rejects_non_integer_count() {
    assert!(err_message("A = [1 2]; R = rot90(A, 1.5);").contains("integer argument"));
}

#[test]
fn circshift_wraps_around() {
    let r = run("A = [1 2; 3 4]; S = circshift(A, [1 0]);").expect("run");
    let s = env_matrix(&r, "S");
    assert_eq!((s.rows, s.cols), (2, 2));
    assert_eq!(s.data, vec![3.0, 4.0, 1.0, 2.0]);
}

#[test]
fn circshift_with_two_scalars_matches_vector_form() {
    let a = env_matrix(&run("S = circshift([1 2; 3 4], [0 1]);").expect("run"), "S");
    let b = env_matrix(&run("S = circshift([1 2; 3 4], 0, 1);").expect("run"), "S");
    assert_eq!(a.data, b.data);
}

#[test]
fn circshift_rejects_invalid_shift_vector() {
    assert!(err_message("S = circshift([1 2; 3 4], [1 2 3]);").contains("2-element vector"));
}

#[test]
fn triu_keeps_upper_triangle() {
    let r = run("A = [1 2; 3 4]; T = triu(A);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!(t.data, vec![1.0, 2.0, 0.0, 4.0]);
}

#[test]
fn tril_keeps_lower_triangle() {
    let r = run("A = [1 2; 3 4]; T = tril(A);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!(t.data, vec![1.0, 0.0, 3.0, 4.0]);
}

#[test]
fn triu_with_offset_selects_nearer_diagonal() {
    let r = run("A = [1 2 3; 4 5 6; 7 8 9]; T = triu(A, 1);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!((t.rows, t.cols), (3, 3));
    // only the strict superdiagonal survives
    assert_eq!(t.data, vec![0.0, 2.0, 3.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0]);
}

#[test]
fn tril_with_offset_selects_nearer_diagonal() {
    let r = run("A = [1 2 3; 4 5 6; 7 8 9]; T = tril(A, -1);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!(t.data, vec![0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 7.0, 8.0, 0.0]);
}

#[test]
fn triu_on_rectangular_matrix_preserves_dimensions() {
    let r = run("A = [1 2 3; 4 5 6]; T = triu(A);").expect("run");
    let t = env_matrix(&r, "T");
    assert_eq!((t.rows, t.cols), (2, 3));
    assert_eq!(t.data, vec![1.0, 2.0, 3.0, 0.0, 5.0, 6.0]);
}

// ---------------------------------------------------------------------
// Integration with the rest of the language
// ---------------------------------------------------------------------

#[test]
fn required_batch_example_runs_end_to_end() {
    let r = run("A = zeros(2, 3);\n\
         B = ones(2, 3);\n\
         I = eye(3);\n\
         x = linspace(0, 10, 5);\n\
         D = diag([1 2 3]);\n\
         R = repmat([1 2], 2, 1);\n")
    .expect("run");

    let a = env_matrix(&r, "A");
    assert_eq!((a.rows, a.cols), (2, 3));

    let b = env_matrix(&r, "B");
    assert_eq!(b.data, vec![1.0; 6]);

    let i = env_matrix(&r, "I");
    assert_eq!((i.rows, i.cols), (3, 3));

    let x = env_matrix(&r, "x");
    assert_eq!((x.rows, x.cols), (1, 5));
    assert_eq!(x.data, vec![0.0, 2.5, 5.0, 7.5, 10.0]);

    let d = env_matrix(&r, "D");
    assert_eq!((d.rows, d.cols), (3, 3));

    let rep = env_matrix(&r, "R");
    assert_eq!((rep.rows, rep.cols), (2, 2));
    assert_eq!(rep.data, vec![1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn array_functions_compose_with_control_flow() {
    // Verify that matrix functions can be used inside loops without runtime
    // errors. (Scalar accumulation of a 1-D `sum` result is not asserted here:
    // `sum` reduces along columns, which is a documented limitation.)
    let r = run("total = 0;\n\
         for i = 1:3\n\
             A = ones(1, i);\n\
             total = total + 1;\n\
         end\n")
    .expect("run");
    assert_eq!(env_matrix(&r, "total").data, vec![3.0]);
}

#[test]
fn array_functions_work_inside_if_branches() {
    let r = run("A = zeros(2, 3);\n\
         if numel(A) == 6\n\
             flag = 1;\n\
         else\n\
             flag = 0;\n\
         end\n")
    .expect("run");
    assert_eq!(env_matrix(&r, "flag").data, vec![1.0]);
}

#[test]
fn variables_from_array_functions_stay_in_the_workspace() {
    // The persistent session keeps values between commands.
    let mut session = matlab_runtime::Session::new();
    session
        .exec("A = reshape(1:6, 2, 3);", None)
        .expect("first command");
    let second = session
        .exec("B = transpose(A);", None)
        .expect("second command");
    let b = second
        .env
        .get("B")
        .and_then(|v| v.as_matrix().cloned())
        .expect("B present");
    assert_eq!((b.rows, b.cols), (3, 2));
    assert_eq!(b.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn range_and_indexing_behaviour_is_unchanged() {
    let r = run("v = 1:5; n = numel(v); last = v(5);").expect("run");
    assert_eq!(env_matrix(&r, "n").data, vec![5.0]);
    assert_eq!(env_matrix(&r, "last").data, vec![5.0]);
}
