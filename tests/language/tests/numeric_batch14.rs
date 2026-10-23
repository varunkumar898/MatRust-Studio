use matlab_runtime::run_source;

fn matrix<'a>(run: &'a matlab_runtime::RunResult, name: &str) -> &'a matlab_values::DenseMatrix {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .unwrap_or_else(|| panic!("{} should be a real matrix", name))
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() < 1e-10,
        "{}: expected {}, got {}",
        context,
        expected,
        actual
    );
}

#[test]
fn batch14_eig_convergence_contract() {
    let r = run_source("d = eig([1, 2; 3, 4]);").unwrap();
    let d = matrix(&r, "d");
    assert_eq!((d.rows, d.cols), (2, 1));
    // Eigenvalues of [[1,2],[3,4]]: (5 +/- sqrt(33))/2, returned ascending.
    let root = 33.0_f64.sqrt();
    assert_close(d.data[0], (5.0 - root) / 2.0, "small eigenvalue");
    assert_close(d.data[1], (5.0 + root) / 2.0, "large eigenvalue");

    let err = run_source("d = eig([1, 2, 3]);").expect_err("non-square eig must fail");
    assert!(err
        .message
        .starts_with("Matrix must be square to compute eigenvalues:"));
}

#[test]
fn batch14_prctile_vector_pct() {
    let r = run_source("A = [10, 20, 30, 40, 50]; p = prctile(A, [25, 75]);").unwrap();
    let p = matrix(&r, "p");
    assert_eq!(p.data.len(), 2);
    assert_close(p.data[0], 17.5, "25th percentile");
    assert_close(p.data[1], 42.5, "75th percentile");
}

#[test]
fn batch14_quantile_integer_n() {
    let r = run_source("A = [1, 2, 3, 4, 5]; q = quantile(A, 4);").unwrap();
    let q = matrix(&r, "q");
    assert_eq!(q.data, vec![1.5, 2.5, 3.5, 4.5]);

    let r1 = run_source("A = [1,2,3]; q = quantile(A, 1);").unwrap();
    assert_close(matrix(&r1, "q").data[0], 3.0, "quantile(A,1) maximum");
}

#[test]
fn batch14_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = matrix(&r, "C");
    assert_eq!((c.rows, c.cols), (2, 2));
    assert_eq!(c.data, vec![1.0, 2.0, 2.0, 4.0]);
}

#[test]
fn batch14_corrcoef_constant_diag() {
    // The two columns of [1 2; 3 4] are perfectly linearly related.
    let r = run_source("A = [1, 2; 3, 4]; R = corrcoef(A);").unwrap();
    let c = matrix(&r, "R");
    assert_eq!((c.rows, c.cols), (2, 2));
    for value in &c.data {
        assert!((value - 1.0).abs() < 1e-12, "corrcoef entry {}", value);
    }
}
