//! Backward-error residual contract for `[V, D] = eig(A)`.
//!
//! For each case the test asserts
//! `||A*V - V*D||_F <= c * eps * n * ||A||_F` with `c = 100`.  The
//! multiplication and Frobenius norm below are deliberately test-local and
//! naive so the assertion does not reuse the implementation it is checking.
//!
//! The 20 seeded 4x4 cases must converge.  A non-convergence Diagnostic is a
//! test failure here, not an accepted outcome.  The fixed asymmetric 5x5 is
//! the matrix from the old complex-pair probe, and the last case is the
//! companion matrix of (x-1)(x-2)(x-3)(x-4)(x-5).

use matlab_linalg::complex::{
    eig_complex, inverse_iteration_eigenvectors, refine_complex_eigenpairs,
};
use matlab_linalg::decomposition::{det, trace};
use matlab_runtime::run_source;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix, Value};

/// Deterministic xorshift* generator, identical to the old probe's generator.
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        let frac = (x >> 11) as f64 / (1u64 << 53) as f64;
        (frac - 0.5) * 10.0 // uniform in [-5, 5)
    }
}

fn real_or_complex(value: &Value, name: &str) -> ComplexDenseMatrix {
    if let Some(c) = value.as_complex_matrix() {
        c.clone()
    } else if let Some(m) = value.as_matrix() {
        ComplexDenseMatrix::from_real(m)
    } else {
        panic!("{name} must be a real or complex matrix");
    }
}

/// Naive row-major complex matrix product: C = A*B.
fn naive_complex_mul(a: &ComplexDenseMatrix, b: &ComplexDenseMatrix) -> ComplexDenseMatrix {
    assert_eq!(a.cols, b.rows, "naive_complex_mul dimension mismatch");
    let mut data = vec![Complex64::new(0.0, 0.0); a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut acc = Complex64::new(0.0, 0.0);
            for k in 0..a.cols {
                acc = acc + a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
            data[i * b.cols + j] = acc;
        }
    }
    ComplexDenseMatrix {
        rows: a.rows,
        cols: b.cols,
        data,
    }
}

fn frobenius(m: &ComplexDenseMatrix) -> f64 {
    m.data
        .iter()
        .map(|z| z.re * z.re + z.im * z.im)
        .sum::<f64>()
        .sqrt()
}

fn matrix_source(entries: &[f64], n: usize) -> String {
    let mut source = String::from("A = [");
    for row in 0..n {
        if row > 0 {
            source.push_str("; ");
        }
        for col in 0..n {
            if col > 0 {
                source.push(' ');
            }
            source.push_str(&format!("{:.17e}", entries[row * n + col]));
        }
    }
    // This runtime's two-output contract returns eigenvalues first, then
    // eigenvectors (the `d = eig(A)` and `[d, v] = eig(A)` convention used by
    // the existing language tests).  The residual below is still the MATLAB
    // `A*V = V*D` identity; D is assembled from the eigenvalue column.
    source.push_str("]; [d, v] = eig(A);");
    source
}

fn diagonalize(d: &ComplexDenseMatrix, n: usize) -> ComplexDenseMatrix {
    if d.rows == n && d.cols == n {
        return d.clone();
    }
    assert_eq!(
        (d.rows, d.cols),
        (n, 1),
        "eigenvalue output must be n x 1 or n x n"
    );
    let mut data = vec![Complex64::new(0.0, 0.0); n * n];
    for i in 0..n {
        data[i * n + i] = d.data[i];
    }
    ComplexDenseMatrix {
        rows: n,
        cols: n,
        data,
    }
}

fn assert_eig_residual(label: &str, a: &DenseMatrix) {
    let n = a.rows;
    let run = run_source(&matrix_source(&a.data, n))
        .unwrap_or_else(|e| panic!("{label}: eig must converge, got: {}", e.message));
    let v = run
        .env
        .get("v")
        .map(|value| real_or_complex(value, "v"))
        .expect("v must be in the workspace");
    let d = run
        .env
        .get("d")
        .map(|value| real_or_complex(value, "d"))
        .expect("d must be in the workspace");
    assert_eq!((v.rows, v.cols), (n, n), "{label}: V shape");
    let dmat = diagonalize(&d, n);

    let av = naive_complex_mul(&ComplexDenseMatrix::from_real(a), &v);
    let vd = naive_complex_mul(&v, &dmat);
    let residual = frobenius(&ComplexDenseMatrix {
        rows: n,
        cols: n,
        data: av
            .data
            .iter()
            .zip(vd.data.iter())
            .map(|(x, y)| *x - *y)
            .collect(),
    });

    // c = 100 justification: the eigensolver is built from unitary
    // similarity transforms (Hessenberg reduction, shifted QR) and inverse
    // iteration for the eigenvectors.  A backward-stable solver has a
    // residual of order n*eps*||A||; converting the 2-norm to the Frobenius
    // norm can add a factor sqrt(n), and the test-local naive accumulation
    // adds a small constant.  c = 100 leaves a wide margin for those
    // constants while remaining tight enough that a wrong eigenvector column
    // (residual order ||A||) cannot pass.
    const C: f64 = 100.0;
    let bound = C * f64::EPSILON * n as f64 * frobenius(&ComplexDenseMatrix::from_real(a));
    assert!(
        residual <= bound,
        "{label}: ||A*V - V*D||_F = {residual:.6e} exceeds \
c*eps*n*||A||_F = {bound:.6e} (c = {C}, n = {n})"
    );
}

#[test]
fn eig_residual_twenty_seeded_random_4x4() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for case in 0..20 {
        let entries: Vec<f64> = (0..16).map(|_| rng.next_f64()).collect();
        let a = DenseMatrix {
            rows: 4,
            cols: 4,
            data: entries.clone(),
        };
        let label = format!("seeded-4x4-#{case}");
        assert_eig_residual(&label, &a);

        // Keep the trace/determinant identities beside the residual check.
        // These are independent of the eigenvector residual and catch a
        // solver that returns a correct-looking V for the wrong eigenvalues.
        let run = run_source(&matrix_source(&entries, 4))
            .unwrap_or_else(|e| panic!("{label}: eig must converge, got: {}", e.message));
        let d = run
            .env
            .get("d")
            .map(|value| real_or_complex(value, "d"))
            .expect("d must be in the workspace");
        let sum = d
            .data
            .iter()
            .copied()
            .fold(Complex64::new(0.0, 0.0), |acc, z| acc + z);
        let product = d
            .data
            .iter()
            .copied()
            .fold(Complex64::new(1.0, 0.0), |acc, z| acc * z);
        let frob_a = entries.iter().map(|x| x * x).sum::<f64>().sqrt();
        let trace_a = trace(&a, None).expect("trace");
        let det_a = det(&a, None).expect("determinant");
        let trace_tol = 1e-10 * (1.0 + frob_a);
        let det_tol = 1e-10 * (1.0 + frob_a).powi(4);
        assert!(
            (sum.re - trace_a).abs() <= trace_tol && sum.im.abs() <= trace_tol,
            "{label}: sum(eigenvalues) = {sum} must equal trace {trace_a}"
        );
        assert!(
            (product.re - det_a).abs() <= det_tol && product.im.abs() <= det_tol,
            "{label}: product(eigenvalues) = {product} must equal determinant {det_a}"
        );
    }
}

#[test]
fn eig_residual_old_probe_asymmetric_5x5() {
    // The fixed 5x5 asymmetric matrix carried by the old complex-pair probe.
    let a = DenseMatrix {
        rows: 5,
        cols: 5,
        data: vec![
            3.0, 1.0, 0.0, -2.0, 1.0, //
            4.0, -1.0, 2.0, 0.0, 3.0, //
            -1.5, 2.0, 5.0, 1.0, 0.0, //
            0.5, -3.0, 0.0, 4.0, 2.0, //
            1.0, 0.0, -1.0, 2.5, -2.0,
        ],
    };
    assert_eig_residual("old-probe-asymmetric-5x5", &a);
}

#[test]
fn eig_residual_companion_of_x_minus_1_to_5() {
    // Companion matrix of (x-1)(x-2)(x-3)(x-4)(x-5).
    let a = DenseMatrix {
        rows: 5,
        cols: 5,
        data: vec![
            15.0, 1.0, 0.0, 0.0, 0.0, //
            -85.0, 0.0, 1.0, 0.0, 0.0, //
            225.0, 0.0, 0.0, 1.0, 0.0, //
            -274.0, 0.0, 0.0, 0.0, 1.0, //
            120.0, 0.0, 0.0, 0.0, 0.0,
        ],
    };
    assert_eig_residual("companion-(x-1)..(x-5)", &a);
}

#[test]
fn eig_eigenvector_only_refinement_seeded_4x4_18() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut entries_18 = Vec::new();
    for case in 0..20 {
        let entries: Vec<f64> = (0..16).map(|_| rng.next_f64()).collect();
        if case == 18 {
            entries_18 = entries;
            break;
        }
    }
    let a = DenseMatrix {
        rows: 4,
        cols: 4,
        data: entries_18,
    };
    let ca = ComplexDenseMatrix::from_real(&a);

    // Compute raw eigenvalues and initial eigenvectors before refinement.
    let (d, _) = eig_complex(&ca, None).expect("eig_complex converges");
    let v_before = inverse_iteration_eigenvectors(&ca, &d.data);

    // Eigenvector-only refinement: eigenvalues must be returned unchanged.
    let (d_refined, v_after) = refine_complex_eigenpairs(&ca, d.data.clone(), v_before.clone());
    assert_eq!(
        d_refined, d.data,
        "eigenvector-only refinement must return eigenvalues unchanged"
    );

    let dmat = diagonalize(&d, 4);

    let av_before = naive_complex_mul(&ca, &v_before);
    let vd_before = naive_complex_mul(&v_before, &dmat);
    let residual_before = frobenius(&ComplexDenseMatrix {
        rows: 4,
        cols: 4,
        data: av_before
            .data
            .iter()
            .zip(vd_before.data.iter())
            .map(|(x, y)| *x - *y)
            .collect(),
    });

    let av_after = naive_complex_mul(&ca, &v_after);
    let vd_after = naive_complex_mul(&v_after, &dmat);
    let residual_after = frobenius(&ComplexDenseMatrix {
        rows: 4,
        cols: 4,
        data: av_after
            .data
            .iter()
            .zip(vd_after.data.iter())
            .map(|(x, y)| *x - *y)
            .collect(),
    });

    println!(
        "seeded-4x4-#18 residual before refinement = {residual_before:.6e}, after = {residual_after:.6e}"
    );

    const C: f64 = 100.0;
    let bound = C * f64::EPSILON * 4.0 * frobenius(&ca);
    assert!(
        residual_after <= bound,
        "seeded-4x4-#18: after refinement residual {residual_after:.6e} exceeds c*eps*n*||A||_F = {bound:.6e} (c = {C})"
    );
}
