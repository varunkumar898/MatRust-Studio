use std::fs;
use matlab_runtime::run_source;

#[test]
fn stage6_csv_roundtrip() {
    let r = run_source("A = [1 2; 3 4]; csvwrite('/tmp/stage6.csv', A);").unwrap();
    assert!(!r.figures.is_empty() || r.env.get("A").is_some());
    let r2 = run_source("A2 = csvread('/tmp/stage6.csv');").unwrap();
    assert!(r2.env.get("A2").is_some());
    let a2 = r2.env.get("A2").unwrap().as_matrix().unwrap();
    assert_eq!(a2.rows, 2);
    assert_eq!(a2.cols, 2);
    assert!((a2.data[0] - 1.0).abs() < 1e-6);
    assert!((a2.data[3] - 4.0).abs() < 1e-6);
}

#[test]
fn stage6_save_creates_file() {
    let r = run_source("x = [10 20]; save('/tmp/stage6_save.csv', x);").unwrap();
    assert!(std::path::Path::new("/tmp/stage6_save.csv").exists());
}
