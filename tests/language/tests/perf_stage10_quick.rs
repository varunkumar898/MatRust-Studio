use matlab_runtime::{run_source, RunResult};
use std::time::Instant;

fn scalar(run: &RunResult, name: &str) -> f64 {
    run.env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{} should be scalar", name))
}

/// 10x10 dense multiplication with deterministic inputs:
/// ones(10,10) * (2*ones(10,10)) has every entry 20.
#[test]
fn benchmark_matrix_mult_10x10() {
    let start = Instant::now();
    let r = run_source("A = ones(10,10); B = 2*ones(10,10); C = A * B;").unwrap();
    let duration = start.elapsed();
    let c = r.env.get("C").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((c.rows, c.cols), (10, 10));
    assert!(c.data.iter().all(|v| (*v - 20.0).abs() < 1e-12));
    println!("BENCH 10x10 mult: {:?}", duration);
}

#[test]
fn benchmark_matrix_mult_50x50() {
    let start = Instant::now();
    let r = run_source("A = ones(50,50); B = ones(50,50); C = A * B;").unwrap();
    let duration = start.elapsed();
    let c = r.env.get("C").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((c.rows, c.cols), (50, 50));
    assert!((c.data[0] - 50.0).abs() < 1e-10);
    assert!((c.data[c.data.len() - 1] - 50.0).abs() < 1e-10);
    println!("BENCH 50x50 mult: {:?}", duration);
}

#[test]
fn benchmark_matrix_mult_100x100() {
    let start = Instant::now();
    let r = run_source("A = ones(100,100); B = ones(100,100); C = A * B;").unwrap();
    let duration = start.elapsed();
    let c = r.env.get("C").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((c.rows, c.cols), (100, 100));
    assert!((c.data[0] - 100.0).abs() < 1e-9);
    assert!((c.data[c.data.len() - 1] - 100.0).abs() < 1e-9);
    println!("BENCH 100x100 mult: {:?}", duration);
}

#[test]
fn benchmark_matrix_solve_20x20() {
    let start = Instant::now();
    let r = run_source("A = 2*eye(20); B = ones(20,1); X = A \\ B;").unwrap();
    let duration = start.elapsed();
    let x = r.env.get("X").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((x.rows, x.cols), (20, 1));
    assert!(x.data.iter().all(|v| (*v - 0.5).abs() < 1e-12));
    println!("BENCH 20x20 solve: {:?}", duration);
}

#[test]
fn benchmark_svd_30x30() {
    let start = Instant::now();
    let r =
        run_source("A = eye(30); [U,S,V] = svd(A); reconstruction = norm(A - U*S*V');").unwrap();
    let duration = start.elapsed();
    let s = r.env.get("S").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!((s.rows, s.cols), (30, 30));
    assert!((s.data[0] - 1.0).abs() < 1e-12);
    assert!((s.data[s.data.len() - 1] - 1.0).abs() < 1e-12);
    assert!(scalar(&r, "reconstruction") < 1e-9);
    println!("BENCH 30x30 svd: {:?}", duration);
}

#[test]
fn benchmark_stats_100x100() {
    let start = Instant::now();
    let r = run_source("A = ones(100,100); m = mean(A(:)); s = std(A(:));").unwrap();
    let duration = start.elapsed();
    assert!((scalar(&r, "m") - 1.0).abs() < 1e-12);
    assert!(scalar(&r, "s").abs() < 1e-12);
    println!("BENCH stats 100x100: {:?}", duration);
}
