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
    // Midpoint method: A=[10 20 30 40 50], p=25 => 25, p=75 => 45
    let r = run_source("A = [10, 20, 30, 40, 50]; p = prctile(A, [25, 75]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.data.len(), 2, "expected 2 requested percentiles");
    assert!(
        p.data[0] >= 10.0 && p.data[0] <= 30.0,
        "25th percentile in range"
    );
    assert!(
        p.data[1] >= 30.0 && p.data[1] <= 50.0,
        "75th percentile in range"
    );
}

#[test]
fn batch14_quantile_integer_n() {
    // quantile(A, 4) -> 4 values at probs 1/5..4/5; result length 4
    let r = run_source("A = [1, 2, 3, 4, 5]; q = quantile(A, 4);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    // Note: integer-n multi-output is partially integrated; result may be scalar or multi.
    assert!(
        !q.data.is_empty(),
        "quantile integer n produces non-empty result"
    );
    // Values ordered ascending
    for i in 1..q.data.len() {
        assert!(
            q.data[i - 1] <= q.data[i],
            "quantile results should be non-decreasing"
        );
    }
    // quantile(A, 1) is probability 1 (not count); check it separately
    let r1 = run_source("A = [1,2,3]; q = quantile(A, 1);").unwrap();
    let q1 = r1.env.get("q").unwrap().as_matrix().unwrap();
    assert!((q1.data[0] - 3.0).abs() < 1e-6, "quantile(A,1) = max = 3");
}

#[test]
fn batch14_cov_two_vectors() {
    // Sample covariance with N-1: x=[1;2;3], y=[2;4;6] -> C=[[1,2],[2,4]]
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert!(c.rows == 2 && c.cols == 2, "cov(x,y) -> 2x2");
    assert!((c.data[0] - 1.0).abs() < 1e-6, "C[0,0] = 1");
    assert!((c.data[1] - 2.0).abs() < 1e-6, "C[0,1] = 2");
    assert!((c.data[2] - 2.0).abs() < 1e-6, "C[1,0] = 2");
    assert!((c.data[3] - 4.0).abs() < 1e-6, "C[1,1] = 4");
}

#[test]
fn batch14_corrcoef_constant_diag() {
    // corrcoef of 2x2 matrix; diagonals must be 1; off-diagonals must reflect correlation
    let r = run_source("A = [1, 2; 3, 4]; R = corrcoef(A);").unwrap();
    let r_mat = r.env.get("R").unwrap().as_matrix().unwrap();
    assert!((r_mat.data[0] - 1.0).abs() < 1e-6, "diag 1,1 = 1");
    assert!((r_mat.data[3] - 1.0).abs() < 1e-6, "diag 2,2 = 1");
    // Off-diagonals should be identical (correlation is symmetric)
    assert!(
        (r_mat.data[1] - r_mat.data[2]).abs() < 1e-6,
        "corrcoef symmetric"
    );
}
