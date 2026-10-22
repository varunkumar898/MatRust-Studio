use matlab_runtime::run_source;

#[test]
fn batch14_eig_convergence_contract() {
    // Valid cases remain successful; non-square is rejected clearly
    let ok = run_source("d = eig([1, 2; 3, 4]);").unwrap();
    assert!(ok.env.get("d").is_some());
    let bad = run_source("d = eig([1, 2, 3]);");
    assert!(bad.is_err());
}

#[test]
fn batch14_prctile_vector_pct() {
    let r = run_source("A = [10 20 30 40 50]; p = prctile(A, [25, 75]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert!(p.rows == 1 || p.cols == 2);
}

#[test]
fn batch14_quantile_integer_n() {
    let r = run_source("A = [1, 2, 3, 4, 5]; q = quantile(A, 4);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!(q.data.len() == 4 || q.data.len() == 5);
}

#[test]
fn batch14_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert!(c.rows == 2 && c.cols == 2);
}

#[test]
fn batch14_corrcoef_constant_diag() {
    let r = run_source("A = [1 2; 3 4]; R = corrcoef(A);").unwrap();
    let r_mat = r.env.get("R").unwrap().as_matrix().unwrap();
    assert!((r_mat.data[0] - 1.0).abs() < 1e-6);
}
