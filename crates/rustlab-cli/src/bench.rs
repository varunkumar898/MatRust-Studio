//! Performance benchmark suite for RustLab's numerical engine.
//! Uses a statistical harness with warmup iterations, 20+ samples, median time, and spread.

use rustlab_math::{matmul_2d, DenseArray};
use rustlab_stdlib::new_interpreter;
use std::time::Instant;

/// Runs a benchmark function with warmup, collects multiple samples,
/// and computes the median time and spread (in milliseconds).
fn run_benchmark_samples<F: FnMut()>(mut op: F, warmup: usize, samples: usize) -> (f64, f64) {
    for _ in 0..warmup {
        op();
    }
    let mut times_ms = Vec::with_capacity(samples);
    for _ in 0..samples {
        let t0 = Instant::now();
        op();
        times_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = times_ms[times_ms.len() / 2];
    let spread = (times_ms[times_ms.len() - 1] - times_ms[0]) / 2.0;
    (median, spread)
}

/// Runs the complete benchmark suite and prints formatted results.
pub fn run_benchmarks() {
    println!();
    println!("===============================================================");
    println!("              RustLab Numerical Performance Suite              ");
    println!("===============================================================");
    println!();

    bench_matmul();
    bench_linear_solve();
    bench_fft();
    bench_elementwise();
    bench_eig_comparison();
    bench_fft_architectures();
    bench_sparse_vs_dense();

    println!();
    println!("All benchmarks completed successfully.");
    println!("===============================================================");
}

fn bench_matmul() {
    println!("--- 1. Dense Matrix Multiplication (2D Gemm) ---");
    println!(
        "{:<12} {:<15} {:<18} {:<15}",
        "Size", "Samples", "Median Time (ms)", "Throughput (GFLOPS)"
    );
    println!("{:-<62}", "");

    let sizes = [64, 128, 256, 384];
    for &n in &sizes {
        let a = DenseArray::from_vec(vec![n, n], vec![1.05; n * n])
            .unwrap_or_else(|_| DenseArray::empty());
        let b = DenseArray::from_vec(vec![n, n], vec![0.95; n * n])
            .unwrap_or_else(|_| DenseArray::empty());

        let samples = 20;
        let warmup = 5;
        let (median_ms, spread_ms) = run_benchmark_samples(
            || {
                let _ = matmul_2d(&a, &b);
            },
            warmup,
            samples,
        );

        let median_secs = median_ms / 1000.0;
        let flops = 2.0 * (n as f64).powi(3);
        let gflops = if median_secs > 0.0 {
            (flops / median_secs) / 1e9
        } else {
            0.0
        };

        println!(
            "{:<12} {:<15} {:<18.3} {:<15.2}",
            format!("{n}x{n}"),
            format!("{samples} (±{spread_ms:.3})"),
            median_ms,
            gflops
        );
    }
    println!();
}

fn bench_linear_solve() {
    println!("--- 2. Linear System Solver (A \\ b with LU / Gaussian Elim) ---");
    println!(
        "{:<12} {:<18} {:<18}",
        "Size", "Samples", "Median Time (ms)"
    );
    println!("{:-<50}", "");

    let sizes = [50, 100, 200];
    for &n in &sizes {
        let mut interp = new_interpreter();
        let setup = format!("A = eye({n}) + 0.1 * ones({n}, {n}); b = ones({n}, 1);");
        let _ = interp.eval_script(&setup);

        let samples = 20;
        let warmup = 5;
        let (median_ms, spread_ms) = run_benchmark_samples(
            || {
                let _ = interp.eval_script("x = A \\ b;");
            },
            warmup,
            samples,
        );

        println!(
            "{:<12} {:<18} {:<18.3}",
            format!("{n}x{n}"),
            format!("{samples} (±{spread_ms:.3})"),
            median_ms
        );
    }
    println!();
}

fn bench_fft() {
    println!("--- 3. 1D FFT Scaling Across Problem Sizes (RustFFT Backend) ---");
    println!(
        "{:<12} {:<18} {:<18}",
        "Points", "Samples", "Median Time (µs)"
    );
    println!("{:-<50}", "");

    let sizes = [1024, 4096, 16384, 65536];
    for &n in &sizes {
        let mut interp = new_interpreter();
        let setup = format!("x = linspace(0, 1, {n});");
        let _ = interp.eval_script(&setup);

        let samples = 20;
        let warmup = 5;
        let (median_ms, spread_ms) = run_benchmark_samples(
            || {
                let _ = interp.eval_script("X = fft(x);");
            },
            warmup,
            samples,
        );
        let median_us = median_ms * 1000.0;
        let spread_us = spread_ms * 1000.0;

        println!(
            "{:<12} {:<18} {:<18.2}",
            n,
            format!("{samples} (±{spread_us:.2})"),
            median_us
        );
    }
    println!();
}

fn bench_elementwise() {
    println!("--- 4. Elementwise & Broadcasting Operations (1,000,000 Elements) ---");
    println!(
        "{:<20} {:<18} {:<18} {:<18}",
        "Operation", "Samples", "Median Time (ms)", "Throughput (MEl/s)"
    );
    println!("{:-<76}", "");

    let n = 1_000_000usize;
    let mut interp = new_interpreter();
    let _ = interp.eval_script("A = ones(1000, 1000); B = 2 * ones(1000, 1000);");

    let samples = 20;
    let warmup = 5;

    // 1. Matrix Addition
    let add_stmts = rustlab_parse::parse("C = A + B;").unwrap_or_default();
    let (add_ms, add_spread) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&add_stmts);
        },
        warmup,
        samples,
    );
    let add_mels = if add_ms > 0.0 {
        (n as f64 / (add_ms / 1000.0)) / 1e6
    } else {
        0.0
    };
    println!(
        "{:<20} {:<18} {:<18.3} {:<18.1}",
        "A + B",
        format!("{samples} (±{add_spread:.3})"),
        add_ms,
        add_mels
    );

    // 2. Elementwise Multiplication
    let mul_stmts = rustlab_parse::parse("C = A .* B;").unwrap_or_default();
    let (mul_ms, mul_spread) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&mul_stmts);
        },
        warmup,
        samples,
    );
    let mul_mels = if mul_ms > 0.0 {
        (n as f64 / (mul_ms / 1000.0)) / 1e6
    } else {
        0.0
    };
    println!(
        "{:<20} {:<18} {:<18.3} {:<18.1}",
        "A .* B",
        format!("{samples} (±{mul_spread:.3})"),
        mul_ms,
        mul_mels
    );

    // 3. Trigonometric Sin
    let sin_stmts = rustlab_parse::parse("C = sin(A);").unwrap_or_default();
    let (sin_ms, sin_spread) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&sin_stmts);
        },
        warmup,
        samples,
    );
    let sin_mels = if sin_ms > 0.0 {
        (n as f64 / (sin_ms / 1000.0)) / 1e6
    } else {
        0.0
    };
    println!(
        "{:<20} {:<18} {:<18.3} {:<18.1}",
        "sin(A)",
        format!("{samples} (±{sin_spread:.3})"),
        sin_ms,
        sin_mels
    );
    println!();
}

#[allow(clippy::needless_range_loop)]
fn bench_eig_comparison() {
    println!("--- 5. Symmetric Eigenvalue: Native Householder+QL vs Faer Self-Adjoint ---");
    println!(
        "{:<10} {:<12} {:<22} {:<22}",
        "Size", "Samples", "Native QL (ms)", "Faer Self-Adjoint (ms)"
    );
    println!("{:-<68}", "");

    let sizes = [30, 60, 100];
    for &n in &sizes {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let val = ((i * 37 + j * 17) % 100) as f64 * 0.01;
                data[i + j * n] = val;
            }
        }
        for i in 0..n {
            for j in 0..n {
                let sym = 0.5 * (data[i + j * n] + data[j + i * n]);
                data[i + j * n] = sym;
            }
        }
        let da = DenseArray::from_vec(vec![n, n], data).unwrap_or_else(|_| DenseArray::empty());

        let samples = 20;
        let warmup = 5;

        // 1. Native symmetric eig (Householder tridiag + QL with Wilkinson shifts)
        let (native_ms, native_spread) = run_benchmark_samples(
            || {
                let _ = rustlab_stdlib::linalg::eig_sym::eig_sym(&da);
            },
            warmup,
            samples,
        );

        // 2. Faer self-adjoint solver
        #[cfg(feature = "faer")]
        let (faer_ms, faer_spread) = run_benchmark_samples(
            || {
                let _ = rustlab_stdlib::linalg::eig_sym::faer_self_adjoint_eig(&da);
            },
            warmup,
            samples,
        );

        #[cfg(not(feature = "faer"))]
        let (faer_ms, faer_spread) = (0.0, 0.0);

        println!(
            "{:<10} {:<12} {:<22.3} {:<22.3}",
            format!("{n}x{n}"),
            format!("{samples}"),
            native_ms,
            faer_ms
        );
        let _ = (native_spread, faer_spread);
    }
    println!();
}

fn bench_fft_architectures() {
    println!("--- 6. Architectural FFT Comparison (RustFFT vs Radix-2 Ref vs Naive O(N^2)) ---");
    println!(
        "{:<15} {:<18} {:<18} {:<18}",
        "Length (N)", "RustFFT (µs)", "Radix-2 Ref (µs)", "Naive O(N^2) (µs)"
    );
    println!("{:-<72}", "");

    use num_complex::Complex64;
    use rustlab_stdlib::signal::radix2::fft_radix2_reference;

    #[allow(clippy::needless_range_loop)]
    fn naive_dft(input: &[Complex64]) -> Vec<Complex64> {
        let n = input.len();
        let mut out = vec![Complex64::new(0.0, 0.0); n];
        for k in 0..n {
            let mut sum = Complex64::new(0.0, 0.0);
            for j in 0..n {
                let angle = -2.0 * std::f64::consts::PI * ((k * j) % n) as f64 / (n as f64);
                let twiddle = Complex64::new(angle.cos(), angle.sin());
                sum += input[j] * twiddle;
            }
            out[k] = sum;
        }
        out
    }

    let cases: [(usize, &str); 4] = [
        (1024, "1024 (pow2)"),
        (1000, "1000 (non-pow2)"),
        (1009, "1009 (prime)"),
        (65536, "65536 (pow2)"),
    ];

    for &(n, label) in &cases {
        let input: Vec<Complex64> = (0..n).map(|i| Complex64::new(i as f64, 0.0)).collect();

        // 1. RustFFT (via interpreter)
        let mut interp = new_interpreter();
        let _ = interp.eval_script(&format!("x = (1:{n})';"));
        let (rustfft_ms, _) = run_benchmark_samples(
            || {
                let _ = interp.eval_script("Y = fft(x);");
            },
            3,
            20,
        );
        let rustfft_us = rustfft_ms * 1000.0;

        // 2. Radix-2 Reference (pow2 only)
        let radix2_us = if n.is_power_of_two() {
            let (r2_ms, _) = run_benchmark_samples(
                || {
                    let _ = fft_radix2_reference(&input, false);
                },
                3,
                20,
            );
            format!("{:.1}", r2_ms * 1000.0)
        } else {
            "N/A (non-pow2)".to_string()
        };

        // 3. Naive DFT (small sizes only)
        let naive_us = if n <= 1024 {
            let (nd_ms, _) = run_benchmark_samples(
                || {
                    let _ = naive_dft(&input);
                },
                2,
                10,
            );
            format!("{:.1}", nd_ms * 1000.0)
        } else {
            "Omitted (>10^8 ops)".to_string()
        };

        println!(
            "{:<15} {:<18.1} {:<18} {:<18}",
            label, rustfft_us, radix2_us, naive_us
        );
    }
    println!();
}

fn bench_sparse_vs_dense() {
    println!("--- 7. Sparse vs Dense Scaling (N = 1,000, Density 1%) ---");
    println!(
        "{:<20} {:<18} {:<18} {:<18}",
        "Operation", "Sparse Time (ms)", "Dense Time (ms)", "Speedup / Ratio"
    );
    println!("{:-<76}", "");

    let mut interp = new_interpreter();
    let setup = "
        n = 1000;
        density = 0.01;
        S = sprand(n, n, density);
        D = full(S);
        b = ones(n, 1);
    ";
    let _ = interp.eval_script(setup);

    let (sp_mul_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_script("c = S * b;");
        },
        3,
        20,
    );
    let (dn_mul_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_script("c = D * b;");
        },
        3,
        20,
    );
    let mul_ratio = if sp_mul_ms > 0.0 {
        dn_mul_ms / sp_mul_ms
    } else {
        0.0
    };

    println!(
        "{:<20} {:<18.3} {:<18.3} {:<18.1}x faster",
        "Matvec (A * b)", sp_mul_ms, dn_mul_ms, mul_ratio
    );

    let (sp_add_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_script("C = S + S;");
        },
        3,
        20,
    );
    let (dn_add_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_script("C = D + D;");
        },
        3,
        20,
    );
    let add_ratio = if sp_add_ms > 0.0 {
        dn_add_ms / sp_add_ms
    } else {
        0.0
    };

    println!(
        "{:<20} {:<18.3} {:<18.3} {:<18.1}x faster",
        "Matrix Add (A + A)", sp_add_ms, dn_add_ms, add_ratio
    );
    println!();
}
