use matlab_runtime::{run_source, Session};

fn scalar(run: &matlab_runtime::RunResult, name: &str) -> f64 {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{} should be scalar", name))
}

#[test]
fn item4_cov_reference() {
    let r = run_source("X = [1;2;3]; Y = [2;4;6]; C = cov(X,Y);").unwrap();
    let c = r.env.get("C").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!(c.data, vec![1.0, 2.0, 2.0, 4.0]);
}

#[test]
fn item4_prctile_reference() {
    // stats::quantile uses the median-unbiased rank r = 0.5 + n*p.
    let r = run_source("v = [1 2 3 6 10]; p = prctile(v, [25 50 75]);").unwrap();
    let p = r.env.get("p").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!(p.data.len(), 3);
    assert!((p.data[0] - 1.75).abs() < 1e-12);
    assert!((p.data[1] - 3.0).abs() < 1e-12);
    assert!((p.data[2] - 7.0).abs() < 1e-12);
}

#[test]
fn item4_first_order_step_reference() {
    let mut s = Session::new();
    s.exec("t = 0:1:5; tau = 1.0; y = 1 - exp(-t / tau);", None)
        .unwrap();
    let y = s.env.get("y").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((y.rows, y.cols), (1, 6));
    for (i, actual) in y.data.iter().enumerate() {
        let expected = 1.0 - (-(i as f64)).exp();
        assert!(
            (actual - expected).abs() < 1e-12,
            "y({}) = {}, expected {}",
            i,
            actual,
            expected
        );
    }
}

#[test]
fn item4_stage_c_stats_reference() {
    let r = run_source("A = [1 2; 3 4]; m = mean(A(:)); s = std(A(:));").unwrap();
    assert!((scalar(&r, "m") - 2.5).abs() < 1e-12);
    assert!((scalar(&r, "s") - (5.0_f64 / 3.0).sqrt()).abs() < 1e-12);
}
