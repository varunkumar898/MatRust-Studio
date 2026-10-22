use matlab_runtime::exec_source;
use matlab_values::{DenseMatrix, Value};

fn run(source: &str) -> Result<matlab_runtime::RunResult, matlab_diagnostics::Diagnostic> {
    matlab_runtime::run_source(source)
}

fn env_matrix(result: &matlab_runtime::RunResult, name: &str) -> DenseMatrix {
    result
        .env
        .get(name)
        .and_then(|v| v.as_matrix().cloned())
        .unwrap_or_else(|| panic!("variable '{}' missing", name))
}

#[test]
fn matrix_multiply_requires_matching_inner_dims() {
    let err = run("A = [1 2; 3 4];\nB = [1 2 3];\nC = A * B;\n").expect_err("2x2 * 1x3 must fail");
    let msg = format!("{}", err);
    assert!(
        msg.contains("Incorrect dimensions for matrix multiplication"),
        "unexpected message: {}",
        msg
    );
}

#[test]
fn matrix_multiply_is_not_elementwise() {
    // [1 2; 3 4] * [5 6; 7 8] = [19 22; 43 50]
    // [1 2; 3 4] .* [5 6; 7 8] = [5 12; 21 32]
    let result = run("A = [1 2; 3 4]; B = [5 6; 7 8]; C1 = A * B; C2 = A .* B;\n").expect("run");
    let c1 = env_matrix(&result, "C1");
    let c2 = env_matrix(&result, "C2");
    assert_eq!(&c1.data, &[19.0, 22.0, 43.0, 50.0]);
    assert_eq!(&c2.data, &[5.0, 12.0, 21.0, 32.0]);
    assert!(c1 != c2);
}

#[test]
fn elementwise_mismatch_is_an_error() {
    let err = run("A = [1 2; 3 4]; B = [1 2 3]; C = A .* B;\n").expect_err(".* size mismatch");
    assert!(format!("{}", err).contains("incompatible sizes"));
}

#[test]
fn indexing_is_one_based_and_column_major() {
    let result = run("A = [1 2; 3 4];\n");
    assert!(result.is_ok());
    let result = result.unwrap();
    let mut env = result.env;
    let mut get = |expr: &str| -> f64 {
        let r = exec_source(&mut env, expr).expect("index expression");
        match &r.outputs[0].value {
            Value::Matrix(m) => m.as_scalar().unwrap(),
            other => panic!("expected scalar, got {:?}", other),
        }
    };
    assert_eq!(get("A(1,1)"), 1.0);
    assert_eq!(get("A(1,2)"), 2.0);
    assert_eq!(get("A(2,1)"), 3.0);
    assert_eq!(get("A(2,2)"), 4.0);
    // Column-major linear indexing: A(2) == 3, A(3) == 2
    assert_eq!(get("A(2)"), 3.0);
    assert_eq!(get("A(3)"), 2.0);
}

#[test]
fn index_out_of_bounds_read_errors() {
    let err = run("A = [1 2; 3 4];\nA(3,1)\n").expect_err("OOB read");
    assert!(format!("{}", err).contains("Index exceeds matrix dimensions"));
}

#[test]
fn index_must_be_positive_integer() {
    let err = run("A = [1 2; 3 4];\nA(0.5, 1)\n").expect_err("fractional index");
    assert!(format!("{}", err).contains("positive integers"));

    // Zero indices are rejected too
    let err = run("A = [1 2; 3 4];\nA(0, 1)\n").expect_err("zero index");
    assert!(format!("{}", err).contains("positive integers"));
}

#[test]
fn assignment_out_of_bounds_grows_2d() {
    let result = run("A = [1 2; 3 4];\nA(2,3) = 9;\n").expect("run");
    let a = env_matrix(&result, "A");
    assert_eq!(a.rows, 2);
    assert_eq!(a.cols, 3);
    // zeros fill the new location; value sits at row 2, col 3
    assert_eq!(&a.data, &[1.0, 2.0, 0.0, 3.0, 4.0, 9.0]);
}

#[test]
fn assignment_out_of_bounds_linear_grows_row_vector() {
    // A(5) = 7 on a 2x2 matrix grows to 1x5 with column-major copy of old elements
    let result = run("A = [1 2; 3 4];\nA(5) = 7;\n").expect("run");
    let a = env_matrix(&result, "A");
    assert_eq!(a.rows, 1);
    assert_eq!(a.cols, 5);
    assert_eq!(&a.data, &[1.0, 3.0, 2.0, 4.0, 7.0]);
}

#[test]
fn transpose_apostrophe_vs_transpose_only() {
    // For real values ' and .' are identical; both must transpose.
    let result = run("A = [1 2; 3 4];\nB = A';\nC = A.';\n").expect("run");
    let b = env_matrix(&result, "B");
    let c = env_matrix(&result, "C");
    assert_eq!(b.rows, 2);
    assert_eq!(b.cols, 2);
    assert_eq!(&b.data, &[1.0, 3.0, 2.0, 4.0]);
    assert_eq!(b.data, c.data);
}

#[test]
fn matrix_power_and_inverse_division() {
    let result = run("A = [1 2; 3 4];\nB = A^2;\nC = A / [2 0; 0 4];\n").expect("run");
    let b = env_matrix(&result, "B");
    assert_eq!(&b.data, &[7.0, 10.0, 15.0, 22.0]);

    let c = env_matrix(&result, "C");
    assert_eq!(&c.data, &[0.5, 0.5, 1.5, 1.0]);
}

#[test]
fn elementwise_power_and_division() {
    let result = run("A = [1 2; 3 4];\nB = A .^ 2;\nC = A ./ 2;\n").expect("run");
    let b = env_matrix(&result, "B");
    assert_eq!(&b.data, &[1.0, 4.0, 9.0, 16.0]);
    let c = env_matrix(&result, "C");
    assert_eq!(&c.data, &[0.5, 1.0, 1.5, 2.0]);
}

#[test]
fn comparisons_produce_zero_one_matrix() {
    let result = run("A = [1 2; 3 4];\nB = A > 2;\nC = A == A;\n").expect("run");
    let b = env_matrix(&result, "B");
    assert_eq!(&b.data, &[0.0, 0.0, 1.0, 1.0]);
    let c = env_matrix(&result, "C");
    assert_eq!(&c.data, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn division_by_zero_yields_inf_like_matlab() {
    let result = run("A = [1 0];\nB = 1 ./ A;\n").expect("run");
    let b = env_matrix(&result, "B");
    assert!(b.data[0] == 1.0);
    assert!(b.data[1] == f64::INFINITY);
}
