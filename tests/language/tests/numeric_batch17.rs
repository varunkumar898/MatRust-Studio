use matlab_runtime::run_source;

#[test]
fn b17_prctile_vector() {
    let r = run_source("A = [1, 2, 3, 6, 10]; p = prctile(A, [25, 50, 75]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert!(p.data.len() == 3 || p.rows == 3 || p.cols == 3);
}

#[test]
fn b17_quantile_vector() {
    let r = run_source("A = [1, 2, 3, 6, 10]; q = quantile(A, [0.25, 0.5, 0.75]);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!(q.data.len() == 3 || q.rows == 3 || q.cols == 3);
}

#[test]
fn b17_quantile_integer_n() {
    let r = run_source("A = [1, 2, 3, 4, 5]; q = quantile(A, 3);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!(q.data.len() == 3);
}

#[test]
fn b17_prctile_dim1() {
    let r = run_source("B = [1, 2; 3, 4; 5, 6]; p = prctile(B, [25, 50, 75], 1);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert!(p.rows == 3 && p.cols == 2);
}

#[test]
fn b17_quantile_dim2() {
    let r = run_source("B = [1, 2; 3, 4; 5, 6]; q = quantile(B, [0.25, 0.5, 0.75], 2);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!(q.rows == 3 && q.cols == 3);
}

#[test]
fn b17_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert!(c.rows == 2 && c.cols == 2);
}

#[test]
fn b17_eig_success() {
    let r = run_source("d = eig([1, 2; 2, 1]);").unwrap();
    assert!(r.env.get("d").is_some());
}
