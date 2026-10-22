use matlab_runtime::run_source;

#[test]
fn batch12_eig_diagonal() {
    let r = run_source("A = [2 0; 0 3]; d = eig(A);").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 2 && d.cols == 1);
}

#[test]
fn batch12_eig_symmetric() {
    let r = run_source("A = [1 2; 2 4]; d = eig(A);").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 2 && d.cols == 1);
}

#[test]
fn batch12_prctile() {
    let r = run_source("A = [10 20 30 40 50]; p = prctile(A, 50);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert!((p.data[0] - 30.0).abs() < 1e-6);
}

#[test]
fn batch12_quantile() {
    let r = run_source("A = [1 3 5]; q = quantile(A, 0.5);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!((q.data[0] - 3.0).abs() < 1e-6);
}

#[test]
fn batch12_cov() {
    let r = run_source("A = [1 2; 3 4]; C = cov(A);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert!(c.rows == 2 && c.cols == 2);
}

#[test]
fn batch12_corrcoef() {
    let r = run_source("A = [1 2; 2 4]; R = corrcoef(A);").unwrap();
    let r_mat = r.env.get("R").unwrap().as_matrix().unwrap();
    assert!(r_mat.rows == 2 && r_mat.cols == 2);
    assert!((r_mat.data[0] - 1.0).abs() < 1e-6);
    assert!((r_mat.data[3] - 1.0).abs() < 1e-6);
}
