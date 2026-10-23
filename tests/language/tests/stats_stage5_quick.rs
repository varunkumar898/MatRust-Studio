use matlab_runtime::run_source;

fn matrix<'a>(run: &'a matlab_runtime::RunResult, name: &str) -> &'a matlab_values::DenseMatrix {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .unwrap_or_else(|| panic!("{} should be a real matrix", name))
}

#[test]
fn stats_mean_median_std_var() {
    let r = run_source("A = [1 2 3 4 5]; m = mean(A); med = median(A); s = std(A); v = var(A);")
        .unwrap();
    assert!((matrix(&r, "m").data[0] - 3.0).abs() < 1e-12);
    assert!((matrix(&r, "med").data[0] - 3.0).abs() < 1e-12);
    assert!((matrix(&r, "s").data[0] - 2.5_f64.sqrt()).abs() < 1e-12);
    assert!((matrix(&r, "v").data[0] - 2.5).abs() < 1e-12);
}

#[test]
fn stats_prctile_multi_output() {
    let r = run_source("A = [1 2 3 6 10]; p = prctile(A, [25 50 75]);").unwrap();
    let p = matrix(&r, "p");
    assert_eq!((p.rows, p.cols), (1, 3));
    // Median-unbiased linear interpolation used by stats::quantile:
    // r = 0.5 + n*p, interpolating between sorted samples.
    assert!((p.data[0] - 1.75).abs() < 1e-12);
    assert!((p.data[1] - 3.0).abs() < 1e-12);
    assert!((p.data[2] - 7.0).abs() < 1e-12);
}

#[test]
fn stats_quantile_integer_n() {
    let r = run_source("A = [1 2 3 4 5]; q = quantile(A, 3);").unwrap();
    let q = matrix(&r, "q");
    assert_eq!(q.data.len(), 3, "integer count 3 => 3 quantiles");
    assert!((q.data[0] - 1.75).abs() < 1e-12);
    assert!((q.data[1] - 3.0).abs() < 1e-12);
    assert!((q.data[2] - 4.25).abs() < 1e-12);
}

#[test]
fn stats_quantile_endpoint_1() {
    let r = run_source("A = [1 2 3]; q = quantile(A, 1);").unwrap();
    assert!((matrix(&r, "q").data[0] - 3.0).abs() < 1e-12);
}

#[test]
fn stats_cov_two_vectors() {
    let r = run_source("x = [1; 2; 3]; y = [2; 4; 6]; C = cov(x, y);").unwrap();
    let c = matrix(&r, "C");
    assert_eq!((c.rows, c.cols), (2, 2));
    assert_eq!(c.data, vec![1.0, 2.0, 2.0, 4.0]);
}
