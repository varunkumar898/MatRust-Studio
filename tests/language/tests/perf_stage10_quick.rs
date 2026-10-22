use std::time::Instant;
use matlab_runtime::{run_source, Session};

/// Existing baseline: 10x10 dense matrix multiplication.
/// Actual measurement captured live; not synthesized.
#[test]
fn benchmark_matrix_mult_10x10() {
    let start = Instant::now();
    let r = run_source("A = rand(10,10); B = rand(10,10); C = A * B;").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("C").is_some());
    println!("BENCH 10x10 mult: {:?}", duration);
}

/// Expanded M4 workload: 50x50 dense matrix multiplication.
#[test]
fn benchmark_matrix_mult_50x50() {
    let start = Instant::now();
    let r = run_source("A = rand(50,50); B = rand(50,50); C = A * B;").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("C").is_some());
    println!("BENCH 50x50 mult: {:?}", duration);
}

/// Expanded M4 workload: 100x100 dense matrix multiplication.
/// Note: larger sizes not systematically profiled; this is one representative point.
#[test]
fn benchmark_matrix_mult_100x100() {
    let start = Instant::now();
    let r = run_source("A = rand(100,100); B = rand(100,100); C = A * B;").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("C").is_some());
    println!("BENCH 100x100 mult: {:?}", duration);
}

/// Expanded M4 workload: LU-style solving (matrix division).
#[test]
fn benchmark_matrix_solve_20x20() {
    let start = Instant::now();
    let r = run_source("A = rand(20,20); B = rand(20,1); X = A \\ B;").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("X").is_some());
    println!("BENCH 20x20 solve: {:?}", duration);
}

/// Expanded M4 workload: SVD on 30x30 matrix.
#[test]
fn benchmark_svd_30x30() {
    let start = Instant::now();
    let r = run_source("A = rand(30,30); [U,S,V] = svd(A);").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("U").is_some());
    println!("BENCH 30x30 svd: {:?}", duration);
}

/// Expanded M4 workload: statistics reduction on 100x100.
#[test]
fn benchmark_stats_100x100() {
    let start = Instant::now();
    let r = run_source("A = rand(100,100); m = mean(A(:)); s = std(A(:));").unwrap();
    let duration = start.elapsed();
    assert!(r.env.get("m").is_some());
    println!("BENCH stats 100x100: {:?}", duration);
}
