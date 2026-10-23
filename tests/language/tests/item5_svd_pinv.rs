use matlab_runtime::{run_source, Session};

fn scalar(run: &matlab_runtime::RunResult, name: &str) -> f64 {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{} should be scalar", name))
}

#[test]
fn svd_orthogonality_reconstruction_real() {
    let r = run_source(
        "A = [1 2; 3 4; 5 6]; [U,S,V] = svd(A); \
         reconstruction = norm(A - U*S*V'); \
         left_orth = norm(U'*U - eye(3)); right_orth = norm(V'*V - eye(2));",
    )
    .unwrap();
    assert!(scalar(&r, "reconstruction") < 1e-9);
    assert!(scalar(&r, "left_orth") < 1e-9);
    assert!(scalar(&r, "right_orth") < 1e-9);

    let u = r.env.get("U").and_then(|v| v.as_matrix()).unwrap();
    let s = r.env.get("S").and_then(|v| v.as_matrix()).unwrap();
    let v = r.env.get("V").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((u.rows, u.cols), (3, 3));
    assert_eq!((s.rows, s.cols), (3, 2));
    assert_eq!((v.rows, v.cols), (2, 2));
}

#[test]
fn svd_pinv_moore_penrose_real() {
    let r = run_source(
        "A = [1 2; 3 4]; P = pinv(A); \
         left = norm(A*P*A - A); right = norm(P*A*P - P);",
    )
    .unwrap();
    assert!(scalar(&r, "left") < 1e-10);
    assert!(scalar(&r, "right") < 1e-10);

    let p = r.env.get("P").and_then(|v| v.as_matrix()).unwrap();
    let expected = [-2.0, 1.0, 1.5, -0.5];
    for (actual, expected) in p.data.iter().zip(expected.iter()) {
        assert!((actual - expected).abs() < 1e-9);
    }
}

#[test]
fn svd_orthogonality_complex() {
    let mut s = Session::new();
    s.exec(
        "A = complex([1 2; 3 4], [0 1; 2 1]); [U,S,V] = svd(A); reconstruction = norm(A - U*S*V');",
        None,
    )
    .unwrap();
    let reconstruction = s
        .env
        .get("reconstruction")
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .expect("reconstruction residual is real scalar");
    assert!(reconstruction < 1e-9);

    let u = s
        .env
        .get("U")
        .and_then(|v| v.as_complex_matrix())
        .expect("complex U");
    let v = s
        .env
        .get("V")
        .and_then(|v| v.as_complex_matrix())
        .expect("complex V");
    assert_eq!((u.rows, u.cols), (2, 2));
    assert_eq!((v.rows, v.cols), (2, 2));
}
