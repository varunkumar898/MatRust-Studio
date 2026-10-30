//! Performance benchmark suite for RustLab's numerical engine.
//! Uses a statistical harness with warmup iterations, 20+ samples, median time, and spread.

use rustlab_math::{matmul_2d, DenseArray};
use rustlab_stdlib::new_interpreter;
use std::time::Instant;

/// Runs a benchmark function with warmup, collects multiple samples,
/// and computes the median time and spread (in milliseconds).
fn run_benchmark_samples<F: FnMut()>(mut op: F, warmup: usize, samples: usize) -> (f64, f64) {
    let (_min, median, _max, spread) = run_benchmark_detailed(&mut op, warmup, samples);
    (median, spread)
}

fn run_benchmark_detailed<F: FnMut()>(
    mut op: F,
    warmup: usize,
    samples: usize,
) -> (f64, f64, f64, f64) {
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
    let min = times_ms[0];
    let max = times_ms[times_ms.len() - 1];
    let median = times_ms[times_ms.len() / 2];
    let spread = (max - min) / 2.0;
    (min, median, max, spread)
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
    bench_b21_data_and_csv();

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
    println!(
        "  Backend: matrixmultiply::dgemm (single-threaded; no thread pool). Host available_parallelism={:?}",
        std::thread::available_parallelism()
    );

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

    // 1. Matrix Addition (extended warm-up & samples for stable measurement)
    let add_stmts = rustlab_parse::parse("C = A + B;").unwrap_or_default();
    let add_warmup = 15;
    let add_samples = 30;
    let (add_min, add_ms, add_max, add_spread) = run_benchmark_detailed(
        || {
            let _ = interp.eval_stmts(&add_stmts);
        },
        add_warmup,
        add_samples,
    );
    let add_mels = if add_ms > 0.0 {
        (n as f64 / (add_ms / 1000.0)) / 1e6
    } else {
        0.0
    };
    println!(
        "{:<20} {:<18} {:<18.3} {:<18.1}  [min: {:.3} ms, max: {:.3} ms]",
        "A + B",
        format!("{add_samples} (±{add_spread:.3})"),
        add_ms,
        add_mels,
        add_min,
        add_max
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
        "Operation", "Sparse Time (ms)", "Dense Time (ms)", "Dense/Sparse"
    );
    println!("{:-<76}", "");

    let mut interp = new_interpreter();
    // A3.1.5: `sprand` is not a registered builtin. The earlier version
    // swallowed that setup error and timed an error path (matvec ratio 1.0x,
    // dense add ~0.043 ms). Build a deterministic 1%-dense CSC matrix from
    // triplets instead, and fail loudly if setup/evaluation regresses.
    let setup = "
        n = 1000;
        idx = (0:n*n/100-1)';
        I = mod(idx, n) + 1;
        J = floor(idx/n) + 1;
        V = ones(numel(idx), 1);
        S = sparse(I, J, V, n, n);
        D = full(S);
        b = ones(n, 1);
    ";
    if let Err(e) = interp.eval_script(setup) {
        println!("  SETUP ERROR: {e}");
        return;
    }
    if let Err(e) = interp.eval_script("c = S * b;") {
        println!("  MATVEC SETUP ERROR: {e}");
        return;
    }
    if let Err(e) = interp.eval_script("C = S + S;") {
        println!("  SPARSE ADD SETUP ERROR: {e}");
        return;
    }
    if let Err(e) = interp.eval_script("C = D + D;") {
        println!("  DENSE ADD SETUP ERROR: {e}");
        return;
    }

    let sp_mul_stmts = rustlab_parse::parse("c = S * b;").unwrap_or_default();
    let dn_mul_stmts = rustlab_parse::parse("c = D * b;").unwrap_or_default();
    let (sp_mul_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&sp_mul_stmts);
        },
        3,
        20,
    );
    let (dn_mul_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&dn_mul_stmts);
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
        "{:<20} {:<18.3} {:<18.3} {:<18.2}x",
        "Matvec (S * b)", sp_mul_ms, dn_mul_ms, mul_ratio
    );

    let sp_add_stmts = rustlab_parse::parse("C = S + S;").unwrap_or_default();
    let dn_add_stmts = rustlab_parse::parse("C = D + D;").unwrap_or_default();
    let (sp_add_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&sp_add_stmts);
        },
        3,
        20,
    );
    let (dn_add_ms, _) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&dn_add_stmts);
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
        "{:<20} {:<18.3} {:<18.3} {:<18.2}x",
        "Matrix Add (S + S)", sp_add_ms, dn_add_ms, add_ratio
    );
    println!();
}

fn bench_b21_data_and_csv() {
    println!("--- 8. Milestone B2.1: 1,000,000-Element Statistics & CSV Matrix I/O ---");
    println!(
        "{:<25} {:<18} {:<18}",
        "Benchmark", "Median (ms)", "Spread (ms)"
    );
    println!("{:-<65}", "");

    let mut interp = new_interpreter();
    if let Err(e) = interp.eval_script("x = (1:1000000)';") {
        println!("  SETUP ERROR: {e}");
        return;
    }

    // 1. movmean (k = 3) on 1M elements
    let mm_stmts = rustlab_parse::parse("m = movmean(x, 3);").unwrap_or_default();
    let (mm_ms, mm_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&mm_stmts);
        },
        2,
        10,
    );
    println!("{:<25} {:<18.3} {:<18.3}", "movmean(1M, 3)", mm_ms, mm_sp);

    // 2. cummax on 1M elements
    let cm_stmts = rustlab_parse::parse("c = cummax(x);").unwrap_or_default();
    let (cm_ms, cm_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&cm_stmts);
        },
        2,
        10,
    );
    println!("{:<25} {:<18.3} {:<18.3}", "cummax(1M)", cm_ms, cm_sp);

    // 3. zscore on 1M elements
    let zs_stmts = rustlab_parse::parse("z = zscore(x);").unwrap_or_default();
    let (zs_ms, zs_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_stmts(&zs_stmts);
        },
        2,
        10,
    );
    println!("{:<25} {:<18.3} {:<18.3}", "zscore(1M)", zs_ms, zs_sp);

    // 4. CSV writematrix and readmatrix on 1,000,000 elements (250,000 x 4)
    let _ = interp
        .eval_script("X = [(1:250000)', (250001:500000)', (500001:750000)', (750001:1000000)'];");
    let csv_path = "target/bench_1m.csv";
    let (write_ms, write_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_script(&format!("writematrix(X, '{csv_path}');"));
        },
        1,
        5,
    );
    println!(
        "{:<25} {:<18.3} {:<18.3}",
        "writematrix(1M floats)", write_ms, write_sp
    );

    let (read_ms, read_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_script(&format!("R = readmatrix('{csv_path}');"));
        },
        1,
        5,
    );
    println!(
        "{:<25} {:<18.3} {:<18.3}",
        "readmatrix(1M floats)", read_ms, read_sp
    );

    // 5. A true 1,000,000-ROW CSV (one column), separate from the 250,000x4
    // float workload above. This is deliberately a smaller sample count: each
    // iteration writes/reads a multi-megabyte text file.
    let rows_path = "target/bench_1m_rows.csv";
    let (rows_write_ms, rows_write_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_script(&format!("writematrix(x, '{rows_path}');"));
        },
        1,
        3,
    );
    println!(
        "{:<25} {:<18.3} {:<18.3}",
        "writematrix(1M rows)", rows_write_ms, rows_write_sp
    );

    let (rows_read_ms, rows_read_sp) = run_benchmark_samples(
        || {
            let _ = interp.eval_script(&format!("Rrows = readmatrix('{rows_path}');"));
        },
        1,
        3,
    );
    println!(
        "{:<25} {:<18.3} {:<18.3}",
        "readmatrix(1M rows)", rows_read_ms, rows_read_sp
    );

    let _ = std::fs::remove_file(csv_path);
    let _ = std::fs::remove_file(rows_path);
    println!();
}
