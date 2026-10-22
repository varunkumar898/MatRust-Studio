use matlab_runtime::run_source;

#[test]
fn b17_prctile_vector() {
    let r = run_source("A = [1, 2, 3, 6, 10]; p = prctile(A, [25, 50, 75]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.data.len(), 3, "three percentiles requested");
}

#[test]
fn b17_quantile_vector() {
    let r = run_source("A = [1, 2, 3, 6, 10]; q = quantile(A, [0.25, 0.5, 0.75]);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert_eq!(q.data.len(), 3, "three probabilities requested");
}

#[test]
fn b17_quantile_integer_n() {
    let r = run_source("A = [1, 2, 3, 4, 5]; q = quantile(A, 3);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert_eq!(q.data.len(), 3, "n=3 -> 3 quantiles");
}

// Documented decision (MathWorks quantile): second arg = 1 is probability 1 (max),
// NOT integer-count form. Integer-count applies only when n > 1 and integer.
#[test]
fn b17_quantile_one_is_probability() {
    // Documented decision: quantile(A,1) means probability 1 (max), NOT count 1
    let r = run_source("A = [1, 2, 3]; q = quantile(A, 1);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!(
        (q.data[0] - 3.0).abs() < 1e-6,
        "quantile(A,1)=1 => max value 3"
    );
}

#[test]
fn b17_prctile_dim1() {
    let r = run_source("B = [1, 2; 3, 4; 5, 6]; p = prctile(B, [25, 50, 75], 1);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.rows, 3, "3 percentiles");
    assert_eq!(p.cols, 2, "2 columns");
}

#[test]
fn b17_quantile_dim2() {
    let r = run_source("B = [1, 2; 3, 4; 5, 6]; q = quantile(B, [0.25, 0.5, 0.75], 2);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert_eq!(q.rows, 3, "3 rows");
    assert_eq!(q.cols, 3, "3 percentiles -> 3 cols for dim=2");
}

#[test]
fn b17_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert_eq!(c.data.len(), 4, "2x2 matrix");
}

#[test]
fn b17_eig_success() {
    let r = run_source("d = eig([1, 2; 2, 1]);").unwrap();
    assert!(r.env.get("d").is_some());
}

#[test]
fn b17_invalid_pct() {
    let bad = run_source("p = prctile([1 2 3], 120);");
    assert!(bad.is_err(), "percentage > 100 should error");
}

#[test]
fn b17_invalid_prob() {
    let bad = run_source("q = quantile([1 2 3], 1.5);");
    assert!(bad.is_err(), "probability > 1 should error");
}

#[test]
fn b17_nan_sample() {
    let r = run_source("A = [1 NaN 3]; p = prctile(A, 50);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    // NaN removed => [1, 3] => midpoint = 2
    assert!((p.data[0] - 2.0).abs() < 1e-6, "NaN omitted");
}

#[test]
fn b17_single_obs() {
    let r = run_source("A = [42]; p = prctile(A, [0 50 100]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.data.len(), 3);
    assert!((p.data[0] - 42.0).abs() < 1e-6);
    assert!((p.data[2] - 42.0).abs() < 1e-6);
}
