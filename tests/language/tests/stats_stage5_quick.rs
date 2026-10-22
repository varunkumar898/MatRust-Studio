use matlab_runtime::run_source;

#[test]
fn stats_mean_median_std_var() {
    let r = run_source("A = [1 2 3 4 5]; m = mean(A); med = median(A); s = std(A); v = var(A);").unwrap();
    assert!(r.env.get("m").is_some());
    assert!(r.env.get("med").is_some());
    assert!(r.env.get("s").is_some());
    assert!(r.env.get("v").is_some());
}

#[test]
fn stats_prctile_multi_output() {
    let r = run_source("A = [1 2 3 6 10]; p = prctile(A, [25 50 75]);").unwrap();
    let p = r.env.get("p").unwrap().as_matrix().unwrap();
    assert_eq!(p.data.len(), 3, "three percentiles");
}

#[test]
fn stats_quantile_integer_n() {
    let r = run_source("A = [1 2 3 4 5]; q = quantile(A, 3);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert_eq!(q.data.len(), 3, "integer count 3 => 3 quantiles");
}

#[test]
fn stats_quantile_endpoint_1() {
    let r = run_source("A = [1 2 3]; q = quantile(A, 1);").unwrap();
    let q = r.env.get("q").unwrap().as_matrix().unwrap();
    assert!((q.data[0] - 3.0).abs() < 1e-6, "probability 1 => max");
}

#[test]
fn stats_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = r.env.get("C").unwrap().as_matrix().unwrap();
    assert_eq!(c.rows, 2);
    assert_eq!(c.cols, 2);
}
