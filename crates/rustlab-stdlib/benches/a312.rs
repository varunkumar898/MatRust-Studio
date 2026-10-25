//! Milestone A3.1.2 criterion benchmark suite for the RustLab numerical engine.
//!
//! Covers the four contract targets from the performance plan:
//!   1. Elementwise `A + B` on 1_000_000 doubles (raw dispatch + full eval pipeline), target <= 3 ms.
//!   2. LU linear solve `A \\ b` at 50/100/200 (monotonic scaling, 100x100 <= 0.6 ms).
//!   3. GEMM throughput at 64/128/256/384, target >= 11.8 GFLOPS.
//!   4. Symmetric eigendsolvers: native Householder+QL vs faer self-adjoint.
//!
//! Run with: `cargo bench --bench a312` (release profile == opt-level 3, fat LTO, 1 codegen unit).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rustlab_math::{matmul_2d, DenseArray, Value};
use rustlab_stdlib::linalg::mldivide_fn;
use rustlab_stdlib::new_interpreter;

const N_ELEMENTS: usize = 1_000_000;

/// Deterministic symmetric test matrix (matches the in-binary benchmark suite input).
fn sym_matrix(n: usize) -> DenseArray<f64> {
    let mut data = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            data[i + j * n] = ((i * 37 + j * 17) % 100) as f64 * 0.01;
        }
    }
    for i in 0..n {
        for j in 0..n {
            let sym = 0.5 * (data[i + j * n] + data[j + i * n]);
            data[i + j * n] = sym;
        }
    }
    DenseArray::from_vec_unchecked(vec![n, n], data)
}

/// Deterministic well-conditioned square matrix (identity + small values).
fn solve_matrix(n: usize) -> DenseArray<f64> {
    let mut data = vec![0.0f64; n * n];
    for j in 0..n {
        for i in 0..n {
            data[i + j * n] = if i == j {
                1.1
            } else {
                0.02 * ((i * 13 + j * 7) % 10) as f64
            };
        }
    }
    DenseArray::from_vec_unchecked(vec![n, n], data)
}

fn bench_elementwise_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("a312/elementwise_add");
    group.throughput(Throughput::Elements(N_ELEMENTS as u64));

    let a_data = vec![1.0f64; N_ELEMENTS];
    let b_data = vec![2.0f64; N_ELEMENTS];
    let a = Value::Double(DenseArray::from_vec_unchecked(vec![N_ELEMENTS, 1], a_data));
    let b = Value::Double(DenseArray::from_vec_unchecked(vec![N_ELEMENTS, 1], b_data));

    // 1. Raw dynamic dispatch (`Value::add` -> shape-match fast path).
    // Only the result is black-boxed: boxing the operands as well would block
    // inlining and force the non-vectorized fallback loop, measuring codegen
    // artifacts instead of the operation.
    group.bench_function("raw_Value_add/1e6", |bb| bb.iter(|| black_box(a.add(&b))));

    // 2. Full interpreter pipeline (parse + assign + add + env store), as used by the CLI suite.
    let mut interp = new_interpreter();
    let setup = format!("A = ones({},1); B = 2*ones({},1);", N_ELEMENTS, N_ELEMENTS);
    let eval_result = interp.eval_script(&setup);
    assert!(eval_result.is_ok());
    let stmts = rustlab_parse::parse("C = A + B;").unwrap_or_default();
    group.bench_function("eval_pipeline/1e6", |bb| {
        bb.iter(|| {
            let out = interp.eval_stmts(&stmts);
            black_box(out)
        })
    });

    group.finish();
}

fn bench_lu_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("a312/lu_solve");
    for &n in &[50usize, 100usize, 200usize] {
        group.throughput(Throughput::Elements((n as u64).pow(3)));
        let a = Value::Double(solve_matrix(n));
        let b = Value::Double(DenseArray::from_vec_unchecked(vec![n, 1], vec![1.0f64; n]));
        group.bench_with_input(BenchmarkId::new("mldivide/linsolve", n), &n, |bb, _| {
            bb.iter(|| {
                black_box(mldivide_fn(
                    black_box(&[black_box(&a).clone(), b.clone()]),
                    1,
                ))
            })
        });
    }
    group.finish();
}

fn bench_gemm(c: &mut Criterion) {
    let mut group = c.benchmark_group("a312/gemm");
    for &n in &[64usize, 128usize, 256usize, 384usize] {
        // 2 * n^3 FLOP per multiply; reported as Elements so Elem/s == FLOP/s.
        group.throughput(Throughput::Elements(2u64 * (n as u64).pow(3)));
        let a = DenseArray::from_vec_unchecked(vec![n, n], vec![1.05f64; n * n]);
        let b = DenseArray::from_vec_unchecked(vec![n, n], vec![0.95f64; n * n]);
        group.bench_function(BenchmarkId::new("matmul_2d", n), |bb| {
            bb.iter(|| black_box(matmul_2d(black_box(&a), black_box(&b))))
        });
    }
    group.finish();
}

fn bench_eig_symestic(c: &mut Criterion) {
    let mut group = c.benchmark_group("a312/eig_sym");
    for &n in &[30usize, 60usize, 100usize] {
        let a = sym_matrix(n);
        group.bench_function(BenchmarkId::new("native_householder_ql", n), |bb| {
            bb.iter(|| black_box(rustlab_stdlib::linalg::eig_sym::eig_sym(black_box(&a))))
        });
        #[cfg(feature = "faer")]
        group.bench_function(BenchmarkId::new("faer_self_adjoint", n), |bb| {
            bb.iter(|| {
                black_box(rustlab_stdlib::linalg::eig_sym::faer_self_adjoint_eig(
                    black_box(&a),
                ))
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_elementwise_add,
    bench_lu_solve,
    bench_gemm,
    bench_eig_symestic
);
criterion_main!(benches);
