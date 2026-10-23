use matlab_runtime::run_source;

fn matrix<'a>(run: &'a matlab_runtime::RunResult, name: &str) -> &'a matlab_values::DenseMatrix {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .unwrap_or_else(|| panic!("{} should be a real matrix", name))
}

fn scalar(run: &matlab_runtime::RunResult, name: &str) -> f64 {
    matrix(run, name)
        .as_scalar()
        .unwrap_or_else(|| panic!("{} should be scalar", name))
}

#[test]
fn num_svd_reconstruction() {
    let r = run_source("A = [3 2; 1 4]; [U,S,V] = svd(A); R = U * S * V'; diff = norm(A - R);")
        .unwrap();
    let diff = scalar(&r, "diff");
    assert!(
        diff < 1e-9,
        "SVD reconstruction residual too large: {}",
        diff
    );
    let s = matrix(&r, "S");
    assert_eq!((s.rows, s.cols), (2, 2));
    assert!((s.data[0] - 5.116672).abs() < 1e-5);
    assert!((s.data[3] - 1.954395).abs() < 1e-5);
}

#[test]
fn num_solve_residual() {
    let r =
        run_source("A = [2 1; 1 3]; B = [5; 8]; X = A \\ B; residual = norm(A*X - B);").unwrap();
    let x = matrix(&r, "X");
    assert_eq!((x.rows, x.cols), (2, 1));
    assert!((x.data[0] - 1.4).abs() < 1e-12);
    assert!((x.data[1] - 2.2).abs() < 1e-12);
    assert!(scalar(&r, "residual") < 1e-12);
}

#[test]
fn num_eig_symmetric_reconstruction() {
    let r = run_source("A = [2, -1; -1, 2]; [d, V] = eig(A); residual = norm(A*V - V*diag(d));")
        .unwrap();
    let d = matrix(&r, "d");
    assert_eq!((d.rows, d.cols), (2, 1));
    assert!((d.data[0] - 1.0).abs() < 1e-10);
    assert!((d.data[1] - 3.0).abs() < 1e-10);
    let residual = scalar(&r, "residual");
    assert!(
        residual < 1e-9,
        "eigenpair residual should be tiny, got {}",
        residual
    );
}

#[test]
fn num_complex_arithmetic() {
    let r = run_source("c = 3 + 2i; r = abs(c);").unwrap();
    let c = r
        .env
        .get("c")
        .and_then(|v| v.as_complex_matrix())
        .expect("c is complex");
    assert_eq!((c.rows, c.cols), (1, 1));
    assert!((c.data[0].re - 3.0).abs() < 1e-12);
    assert!((c.data[0].im - 2.0).abs() < 1e-12);
    assert!((scalar(&r, "r") - 13.0_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn num_det_and_inv() {
    let r = run_source("A = [4 2; 1 3]; d = det(A); invA = inv(A);").unwrap();
    assert!((scalar(&r, "d") - 10.0).abs() < 1e-12);
    let inv = matrix(&r, "invA");
    let expected = [0.3, -0.2, -0.1, 0.4];
    for (actual, expected) in inv.data.iter().zip(expected.iter()) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "inverse entry {}, expected {}",
            actual,
            expected
        );
    }
}

#[test]
fn num_pinv_condition() {
    let r = run_source(
        "A = [1 2; 3 4]; P = pinv(A); \
         left = norm(A*P*A - A); right = norm(P*A*P - P); shape = size(P);",
    )
    .unwrap();
    assert!(scalar(&r, "left") < 1e-10);
    assert!(scalar(&r, "right") < 1e-10);
    let p = matrix(&r, "P");
    assert_eq!((p.rows, p.cols), (2, 2));
    let expected = [-2.0, 1.0, 1.5, -0.5];
    for (actual, expected) in p.data.iter().zip(expected.iter()) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "pinv entry {}, expected {}",
            actual,
            expected
        );
    }
}
