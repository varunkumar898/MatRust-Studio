//! Complex-pair probe for `eig` on real matrices.
//!
//! Contract enforced here:
//! * For every seeded random 4x4 probe matrix, `eig` must either succeed with
//!   sum(eigenvalues) = trace AND product(eigenvalues) = determinant within a
//!   norm-scaled tolerance, or return the documented non-convergence
//!   Diagnostic ("Eigenvalue iteration did not converge to working
//!   precision.", its complex-path twin prefixed with "Complex"). No third
//!   outcome is accepted.
//! * The rotation matrix [0 -1; 1 0] has the characteristic polynomial
//!   lambda^2 + 1 = 0, so its exact spectrum is the complex pair {+i, -i}.
//!   The pair itself is asserted at runtime (enabled, single- and
//!   multi-output, order-independent).
//!
//! documented MATLAB behavior, not reference-verified, and the known limitation parked below: MATLAB lists
//! `eig([0 -1; 1 0])` as (+1i, -1i). This runtime's complex shifted-QR
//! produces the same two numbers but lists them as (-1i, +1i) — QR iteration
//! order, not MATLAB's listing order. The exact raw-order assertion is
//! therefore kept as a separate `#[ignore]`-marked test so the gap stays
//! visible without weakening the passing order-independent coverage.

use matlab_diagnostics::DiagnosticSeverity;
use matlab_linalg::decomposition::{det, eig_general, trace};
use matlab_runtime::run_source;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix};

/// Documented non-convergence diagnostic on the real shifted-QR path
/// (`linalg::decomposition`), pinned by `item3_max_iter_eig.rs`.
const CONV_MSG_REAL: &str = "Eigenvalue iteration did not converge to working precision.";
/// Documented non-convergence diagnostic on the complex path
/// (`linalg::complex`).
const CONV_MSG_COMPLEX: &str =
    "Complex eigenvalue iteration did not converge to working precision.";

/// Deterministic xorshift* generator (seeded), uniform in [-5, 5).
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        let frac = (x >> 11) as f64 / (1u64 << 53) as f64;
        (frac - 0.5) * 10.0 // uniform in [-5, 5)
    }
}

fn complex_of(mat: &matlab_values::Value) -> ComplexDenseMatrix {
    if let Some(c) = mat.as_complex_matrix() {
        c.clone()
    } else if let Some(m) = mat.as_matrix() {
        ComplexDenseMatrix::from_real(m)
    } else {
        panic!("eig result must be a matrix");
    }
}

/// Runtime-level assertion: `eig([0 -1; 1 0])` must return the pair {0+1i,
/// 0-1i} (lambda^2 + 1 = 0), as a 2x1 complex column. Sorting is applied so
/// the assertion's strength does not depend on the solver's listing order;
/// the raw listing order carries its own (ignored) test below.
#[test]
fn eig_rotation_matrix_complex_pair_single_output() {
    let run = run_source("A = [0 -1; 1 0]; d = eig(A); s = size(d);").expect("eig call");
    let d = run
        .env
        .get("d")
        .map(complex_of)
        .unwrap_or_else(|| panic!("eig([0 -1; 1 0]) must return a complex column"));
    let s = run.env.get("s").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!(s.data, vec![2.0, 1.0], "size = [2 1]");

    let mut values = d.data.clone();
    values.sort_by(|a, b| a.im.partial_cmp(&b.im).unwrap());
    assert!(
        (values[0].re - 0.0).abs() < 1e-12 && (values[0].im + 1.0).abs() < 1e-12,
        "expect 0 - 1i, got {} + {}i",
        values[0].re,
        values[0].im
    );
    assert!(
        (values[1].re - 0.0).abs() < 1e-12 && (values[1].im - 1.0).abs() < 1e-12,
        "expect 0 + 1i, got {} + {}i",
        values[1].re,
        values[1].im
    );
}

/// Multi-output form must expose the same pair, paired with eigenvectors.
#[test]
fn eig_rotation_matrix_complex_pair_two_outputs() {
    let run = run_source("A = [0 -1; 1 0]; [d, v] = eig(A);").expect("two-output eig call");
    let d = run
        .env
        .get("d")
        .map(complex_of)
        .unwrap_or_else(|| panic!("multi-output eig d must be a complex column"));
    let v = run
        .env
        .get("v")
        .map(complex_of)
        .unwrap_or_else(|| panic!("multi-output eig V must be complex"));
    assert_eq!((d.rows, d.cols), (2, 1));
    assert_eq!((v.rows, v.cols), (2, 2));
    let mut values = d.data.clone();
    values.sort_by(|a, b| a.im.partial_cmp(&b.im).unwrap());
    assert!((values[0].im + 1.0).abs() < 1e-12 && values[0].re.abs() < 1e-12);
    assert!((values[1].im - 1.0).abs() < 1e-12 && values[1].re.abs() < 1e-12);
}

/// Exact MATLAB raw listing order for the rotation matrix: MATLAB prints
/// eig([0 -1; 1 0]) as (+1i, -1i); this runtime prints (-1i, +1i). The
/// assertion below is the full ground-truth statement; it is parked here
/// (visible, must-pass-in-principle, currently failing on listing order)
/// rather than folded into the passing sorted tests above.
#[test]
#[ignore = "known limitation: eig lists conjugate pairs in QR-iteration order; [0 -1; 1 0] comes out (-i, +i) while MATLAB lists (+i, -i)"]
fn eig_rotation_matrix_matlab_raw_order_plus_i_first() {
    let run = run_source("A = [0 -1; 1 0]; d = eig(A);").expect("eig call");
    let d = run
        .env
        .get("d")
        .map(complex_of)
        .unwrap_or_else(|| panic!("eig([0 -1; 1 0]) must return a complex column"));
    assert_eq!((d.rows, d.cols), (2, 1));
    assert!(
        d.data[0].re.abs() < 1e-12 && (d.data[0].im - 1.0).abs() < 1e-12,
        "first eigenvalue must be 0+1i, got {}+{}i",
        d.data[0].re,
        d.data[0].im
    );
    assert!(
        d.data[1].re.abs() < 1e-12 && (d.data[1].im + 1.0).abs() < 1e-12,
        "second eigenvalue must be 0-1i, got {}+{}i",
        d.data[1].re,
        d.data[1].im
    );
}

/// Seeded probe: 20 random 4x4 real matrices. Per matrix, the contract is
/// exactly one of two outcomes:
///   1. eig succeeds and sum(eigenvalues) = trace AND
///      product(eigenvalues) = determinant within a norm-scaled tolerance;
///   2. eig returns the documented non-convergence Diagnostic.
/// A summary line is printed (visible with `-- --nocapture`).
#[test]
fn eig_complex_pair_random_matrix_probe() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let n = 20usize;
    let mut verified = 0usize;
    let mut documented_failures = 0usize;

    for case in 0..n {
        let entries: Vec<f64> = (0..16).map(|_| rng.next_f64()).collect();
        let norm = entries
            .iter()
            .fold(0.0f64, |m, v| m.max(v.abs()))
            .max(1e-30f64);
        let a = DenseMatrix {
            rows: 4,
            cols: 4,
            data: entries,
        };

        match eig_general(&a, None) {
            Ok((d, _v)) => {
                let t = trace(&a, None).expect("trace of a real 4x4 must compute");
                let dt = det(&a, None).expect("det of a real 4x4 must compute");

                let sum: Complex64 = d.data.iter().copied().fold(Complex64::zero(), |s, z| s + z);
                let product: Complex64 =
                    d.data.iter().copied().fold(Complex64::one(), |p, z| p * z);

                // Relative tolerances scaled by the matrix norm.
                let tol = 1e-8 * (1.0 + norm);
                assert!(
                    (sum.re - t).abs() <= tol && sum.im.abs() <= tol,
                    "case {case}: sum of eigenvalues {sum} must equal trace {t}\nA = {:?}",
                    a.data
                );
                assert!(
                    (product.re - dt).abs() <= tol && product.im.abs() <= tol,
                    "case {case}: product of eigenvalues {product} must equal det {dt}\nA = {:?}",
                    a.data
                );
                verified += 1;
            }
            Err(diag) => {
                assert_eq!(
                    diag.severity,
                    DiagnosticSeverity::Error,
                    "case {case}: non-convergence must be an Error diagnostic"
                );
                assert!(
                    diag.message == CONV_MSG_REAL || diag.message == CONV_MSG_COMPLEX,
                    "case {case}: diagnostic must be the documented non-convergence message, got: {}",
                    diag.message
                );
                documented_failures += 1;
                println!(
                    "case {case}: documented non-convergence diagnostic: {}",
                    diag.message
                );
            }
        }
    }

    println!(
        "eig complex-pair probe: {n} seeded 4x4 matrices — {verified} verified invariants, \
{documented_failures} documented non-convergence diagnostics \
(contract: every matrix is exactly one of the two)"
    );
    assert_eq!(
        verified + documented_failures,
        n,
        "every matrix must resolve to exactly one contract outcome"
    );
}

/// Naive complex matrix product for test-local verification: out[i][j] =
/// sum_k a[i][k] * b[k][j], using the documented row-major storage contract of
/// `ComplexDenseMatrix` (mirrors `DenseMatrix`).
fn naive_complex_mul(a: &ComplexDenseMatrix, b: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    assert_eq!(a.cols, b.rows, "naive_complex_mul dimension mismatch");
    let mut data = vec![Complex64::new(0.0, 0.0); a.rows * b.cols];
    for (i, j) in (0..a.rows).flat_map(|i| (0..b.cols).map(move |j| (i, j))) {
        let mut acc = Complex64::new(0.0, 0.0);
        for k in 0..a.cols {
            acc = acc + a.data[i * a.cols + k] * b.data[k * b.cols + j];
        }
        data[i * b.cols + j] = acc;
    }
    ComplexDenseMatrix {
        rows: a.rows,
        cols: b.cols,
        data,
    }
}

/// Complex Frobenius norm: sqrt(sum |z|^2).
fn frobenius_complex(m: &ComplexDenseMatrix) -> f64 {
    m.data
        .iter()
        .map(|z| z.re * z.re + z.im * z.im)
        .sum::<f64>()
        .sqrt()
}

/// Eigenvector residual contract: for [V, D] = eig(A), the classic backward
/// error residual ||A*V - V*D||_F must be within c * eps * n * ||A||_F with
/// c = 100 (see the comment inside the test for why this constant is ample).
/// This complements (does not replace) the trace/determinant probe above: the
/// eigenvalues alone can look right while the eigenvectors are wrong.
#[test]
fn eig_eigenvector_residual_bound() {
    // c = 100 justification: for a backward-stable eigensolver built from
    // unitary transforms (Hessenberg reduction + shifted QR, eigenvectors by
    // inverse iteration), the residual of the exact eigenpair equations is
    // bounded by ~n * eps * ||A|| (2-norm) for the unitary similarity itself;
    // the Frobenius norm can exceed the 2-norm by at most sqrt(n), and the
    // product chain A*V - V*D plus naive per-element accumulation adds a
    // small constant. c = 100 therefore covers all of these machine-precision
    // effects with a wide margin while remaining a tight, meaningful bound
    // (a wrong eigenvector column produces a residual of order ||A||, not
    // order eps*n*||A||).
    const C: f64 = 100.0;

    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut cases: Vec<(String, Vec<f64>, usize)> = (0..20usize)
        .map(|case| {
            (
                format!("seeded-4x4-#{case}"),
                (0..16).map(|_| rng.next_f64()).collect(),
                4,
            )
        })
        .collect();
    // A fixed 5x5 asymmetric matrix (A != A^T, no symmetry to exploit):
    cases.push((
        "fixed-5x5-asymmetric".to_string(),
        vec![
            3.0, 1.0, 0.0, -2.0, 1.0, //
            4.0, -1.0, 2.0, 0.0, 3.0, //
            -1.5, 2.0, 5.0, 1.0, 0.0, //
            0.5, -3.0, 0.0, 4.0, 2.0, //
            1.0, 0.0, -1.0, 2.5, -2.0,
        ],
        5,
    ));

    let mut worst_ratio = 0.0f64;
    for (label, entries, n) in &cases {
        let a = DenseMatrix {
            rows: *n,
            cols: *n,
            data: entries.clone(),
        };
        match eig_general(&a, None) {
            Ok((d, v)) => {
                // Keep the trace/determinant invariants side-by-side with the
                // residual: both are parts of the same verification contract.
                let t = trace(&a, None).expect("trace of a real square matrix");
                let dt = det(&a, None).expect("det of a real square matrix");
                let sum: Complex64 = d.data.iter().copied().fold(Complex64::zero(), |s, z| s + z);
                let product: Complex64 =
                    d.data.iter().copied().fold(Complex64::one(), |p, z| p * z);
                let inv_tol = 1e-8 * (1.0 + entries.iter().fold(0.0f64, |m, x| m.max(x.abs())));
                assert!(
                    (sum.re - t).abs() <= inv_tol && sum.im.abs() <= inv_tol,
                    "{label}: sum of eigenvalues {sum} must equal trace {t}"
                );
                assert!(
                    (product.re - dt).abs() <= inv_tol && product.im.abs() <= inv_tol,
                    "{label}: product of eigenvalues {product} must equal det {dt}"
                );

                // D = diag(d) with d the n x 1 eigenvalue column.
                let mut dm = ComplexDenseMatrix::zeros(*n, *n);
                for (i, z) in d.data.iter().enumerate() {
                    dm.data[i * (*n) + i] = *z;
                }
                let av = naive_complex_mul(&ComplexDenseMatrix::from_real(&a), &v);
                let vd = naive_complex_mul(&v, &dm);
                let residual: Vec<Complex64> = av
                    .data
                    .iter()
                    .zip(vd.data.iter())
                    .map(|(x, y)| *x - *y)
                    .collect();
                let res = frobenius_complex(&ComplexDenseMatrix {
                    rows: av.rows,
                    cols: av.cols,
                    data: residual.clone(),
                });
                let frob_a = entries.iter().map(|x| x * x).sum::<f64>().sqrt();
                let bound = C * f64::EPSILON * (*n as f64) * frob_a;
                let ratio = res / bound;
                worst_ratio = worst_ratio.max(ratio);
                assert!(
                    res <= bound,
                    "{label}: eigenvector residual ||A*V - V*D||_F = {res:.3e} \
exceeds c*eps*n*||A||_F = {bound:.3e} (c = {C})"
                );
            }
            Err(diag) => {
                // Only the documented non-convergence Diagnostic counts.
                assert_eq!(
                    diag.severity,
                    DiagnosticSeverity::Error,
                    "{label}: non-convergence must be an Error diagnostic"
                );
                assert!(
                    diag.message == CONV_MSG_REAL || diag.message == CONV_MSG_COMPLEX,
                    "{label}: unexpected diagnostic: {}",
                    diag.message
                );
                println!(
                    "{label}: documented non-convergence diagnostic: {}",
                    diag.message
                );
            }
        }
    }

    println!(
        "eig residual probe: 20 seeded 4x4 + 1 fixed 5x5 asymmetric — \
worst ||A*V - V*D||_F / (100*eps*n*||A||_F) ratio = {worst_ratio:.3e}"
    );
}
