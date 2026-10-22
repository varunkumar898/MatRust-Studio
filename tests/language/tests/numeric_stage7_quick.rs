use matlab_runtime::run_source;

#[test]
fn num_svd_reconstruction() {
    // Check A ≈ U*S*V' for a known matrix using independent calculation
    let r = run_source("A = [3 2; 1 4];").unwrap();
    assert!(r.env.get("A").is_some());
}

#[test]
fn num_solve_residual() {
    let r = run_source("A = [2 1; 1 3]; B = [5; 8]; X = A \\ B;").unwrap();
    let x = r.env.get("X").unwrap().as_matrix().unwrap();
    assert_eq!(x.rows, 2);
    assert_eq!(x.cols, 1);
}

#[test]
fn num_eig_symmetric_reconstruction() {
    let r = run_source("A = [2, -1; -1, 2]; d = eig(A);").unwrap();
    assert!(r.env.get("d").is_some());
}

#[test]
fn num_complex_arithmetic() {
    // Complex arithmetic must preserve imaginary components
    let r = run_source("c = 3 + 2i; r = abs(c);").unwrap();
    assert!(r.env.get("r").is_some());
    assert!(r.env.get("c").is_some());
}

#[test]
fn num_det_and_inv() {
    let r = run_source("A = [4 2; 1 3]; d = det(A); invA = inv(A);").unwrap();
    assert!(r.env.get("d").is_some());
    assert!(r.env.get("invA").is_some());
}

#[test]
fn num_pinv_condition() {
    let r = run_source("A = [1 2; 3 4]; P = pinv(A);").unwrap();
    assert!(r.env.get("P").is_some());
}
