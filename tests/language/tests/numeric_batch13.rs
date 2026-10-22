use matlab_runtime::run_source;

// --- Real-valued matrices ---
#[test]
fn batch13_real_diagonal() {
    let r = run_source("d = eig([2, 0; 0, -1]);").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 2 && d.cols == 1);
}

#[test]
fn batch13_real_symmetric() {
    let r = run_source("d = eig([1 2; 2 4]);").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 2 && d.cols == 1);
}

#[test]
fn batch13_real_identity() {
    let r = run_source("d = eig(eye(3));").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 3 && d.cols == 1);
    for v in d.data.iter() {
        assert!((*v - 1.0).abs() < 1e-6);
    }
}

#[test]
fn batch13_real_zero() {
    let r = run_source("d = eig(zeros(2));").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 2 && d.cols == 1);
}

#[test]
fn batch13_real_repeated() {
    let r = run_source("d = eig([3, 0, 1; 0, 3, 0; 1, 0, 3]);").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert!(d.rows == 3 && d.cols == 1);
}

#[test]
fn batch13_real_nonsquare_fails() {
    let r = run_source("d = eig([1 2 3]);");
    assert!(r.is_err());
}

// --- Complex-valued matrices ---
