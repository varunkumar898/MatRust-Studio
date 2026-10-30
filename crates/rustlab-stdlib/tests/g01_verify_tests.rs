//! G0.1-VERIFY regression tests.
//!
//! These tests were added by the verification pass because Milestone G0
//! changed LU/linear-solve code, core `omitnan` handling, error identifiers,
//! and display formatting without adding regressions.
//!
//! Sources:
//! * LU factorization / solve: internal contract from the G0 review.
//! * `omitnan`: MathWorks `sum`, `mean`, `min`, `max` documentation rule.
//! * Display goldens: `docs/spec/display-rules.md`; rules marked unverified
//!   there remain unverified here.
//! * Error identifiers: `docs/spec/error-catalog.md` (identifier strings are
//!   marked UNVERIFIED unless a MathWorks page is cited).

use rustlab_eval::EvalError;
use rustlab_math::{matmul_2d, DenseArray, MathError, Value};
use rustlab_stdlib::linalg::{lu_fn, mldivide_fn};
use rustlab_stdlib::new_interpreter;
use rustlab_stdlib::stats::{max_fn, mean_fn, min_fn, sum_fn};

/// Deterministic 64-bit LCG used only to build reproducible test matrices.
/// This is not a claim about MATLAB's random stream.
fn next_unit(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let top = (*seed >> 11) as f64;
    top / ((1u64 << 53) as f64)
}

/// Diagonally dominant matrix: stable under partial pivoting for the sizes
/// used here, with non-trivial off-diagonal entries.
fn well_conditioned_matrix(n: usize, mut seed: u64) -> DenseArray<f64> {
    let mut data = vec![0.0; n * n];
    for j in 0..n {
        for i in 0..n {
            let value = if i == j {
                10.0
            } else {
                (next_unit(&mut seed) * 2.0 - 1.0) * 0.1
            };
            data[i + j * n] = value;
        }
    }
    DenseArray::from_vec(vec![n, n], data).expect("test matrix shape is valid")
}

fn max_abs_diff(a: &DenseArray<f64>, b: &DenseArray<f64>) -> f64 {
    assert_eq!(a.shape(), b.shape(), "test arrays must have equal shapes");
    a.as_slice()
        .iter()
        .zip(b.as_slice().iter())
        .map(|(&x, &y)| (x - y).abs())
        .fold(0.0f64, f64::max)
}

fn as_double(value: &Value) -> &DenseArray<f64> {
    match value {
        Value::Double(array) => array,
        other => panic!("expected double array, got {}", other.type_name()),
    }
}

fn check_lu_and_solve_residual(n: usize, seed: u64) {
    let a = well_conditioned_matrix(n, seed);
    let x_true = DenseArray::from_vec(
        vec![n, 1],
        (0..n).map(|i| 0.5 + (i as f64) * 0.25).collect(),
    )
    .expect("right-hand-side shape is valid");
    let b = matmul_2d(&a, &x_true).expect("A*x_true dimensions are valid");

    // 1. Factorization identity: P*A = L*U.
    let factors = lu_fn(&[Value::Double(a.clone())], 3).expect("lu must succeed");
    assert_eq!(factors.len(), 3);
    let l = as_double(&factors[0]);
    let u = as_double(&factors[1]);
    let p = as_double(&factors[2]);
    let pa = matmul_2d(p, &a).expect("P*A dimensions are valid");
    let lu = matmul_2d(l, u).expect("L*U dimensions are valid");
    let factorization_residual = max_abs_diff(&pa, &lu);
    assert!(
        factorization_residual < 1e-10,
        "{n}x{n} LU factorization residual {factorization_residual:e} is not < 1e-10"
    );

    // 2. Solve residual: norm(A*x - b) < 1e-10.
    let solution = mldivide_fn(&[Value::Double(a.clone()), Value::Double(b.clone())], 1)
        .expect("well-conditioned solve must succeed");
    let x = as_double(&solution[0]);
    let ax = matmul_2d(&a, x).expect("A*x dimensions are valid");
    let solve_residual = max_abs_diff(&ax, &b);
    assert!(
        solve_residual < 1e-10,
        "{n}x{n} solve residual {solve_residual:e} is not < 1e-10"
    );
}

#[test]
fn test_g01_lu_residuals_random_5x5() {
    check_lu_and_solve_residual(5, 0x5eed_0005);
}

#[test]
fn test_g01_lu_residuals_random_50x50() {
    check_lu_and_solve_residual(50, 0x5eed_0050);
}

#[test]
fn test_g01_mldivide_singular_returns_inf() {
    // Column-major: [1 2; 2 4].
    let a = DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 2.0, 4.0])
        .expect("singular matrix shape is valid");
    let b =
        DenseArray::from_vec(vec![2, 1], vec![1.0, 2.0]).expect("right-hand-side shape is valid");
    let solution =
        mldivide_fn(&[Value::Double(a), Value::Double(b)], 1).expect("mldivide returns a value");
    let x = as_double(&solution[0]);
    assert!(
        x.as_slice().iter().all(|v| v.is_infinite()),
        "singular solve must return Inf, got {:?}",
        x.as_slice()
    );
}

fn row(vector: Vec<f64>) -> Value {
    Value::Double(DenseArray::row_vector(vector))
}

fn option(name: &str) -> Value {
    Value::char_vector(name)
}

fn scalar(value: &Value) -> f64 {
    value
        .as_scalar_f64()
        .expect("reduction must produce a scalar")
}

#[test]
fn test_g01_core_omitnan_sum_mean_min_max() {
    // 1 + 3 + (-2) = 2; mean = 2/3; min = -2; max = 3.
    let data = row(vec![1.0, f64::NAN, 3.0, -2.0]);
    let omitnan = option("omitnan");

    let sum = sum_fn(&[data.clone(), omitnan.clone()], 1).expect("sum omitnan");
    assert_eq!(scalar(&sum[0]), 2.0);

    let mean = mean_fn(&[data.clone(), omitnan.clone()], 1).expect("mean omitnan");
    assert!((scalar(&mean[0]) - (2.0 / 3.0)).abs() < 1e-15);

    let min = min_fn(&[data.clone(), omitnan.clone()], 1).expect("min omitnan");
    assert_eq!(scalar(&min[0]), -2.0);

    let max = max_fn(&[data.clone(), omitnan.clone()], 1).expect("max omitnan");
    assert_eq!(scalar(&max[0]), 3.0);

    // Default and explicit 'includenan' propagate NaN for sum and mean. The
    // default/includenan NaN rules for min/max are not asserted here; their
    // exact MATLAB behaviour is not verified in this environment.
    let include = option("includenan");
    for value in [
        sum_fn(std::slice::from_ref(&data), 1).expect("sum default")[0].clone(),
        mean_fn(std::slice::from_ref(&data), 1).expect("mean default")[0].clone(),
        sum_fn(&[data.clone(), include.clone()], 1).expect("sum includenan")[0].clone(),
        mean_fn(&[data.clone(), include], 1).expect("mean includenan")[0].clone(),
    ] {
        assert!(
            scalar(&value).is_nan(),
            "sum/mean includenan reduction must propagate NaN"
        );
    }
}

#[test]
fn test_g01_mask_count_and_size_identifiers_settled() {
    let mut interp = new_interpreter();

    // Mask-count mismatch: one id and one message.
    let count_err = interp
        .eval_script("A = 1:5; A([true false true false true]) = [10 20];")
        .expect_err("mask-count mismatch must fail");
    match count_err {
        EvalError::Math {
            error: MathError::SubscriptedAssignmentCountMismatch { identifier },
            ..
        } => assert_eq!(identifier, "MATLAB:subsassigndimmismatch"),
        other => panic!("unexpected mask-count error: {other}"),
    }

    // Mask-shape mismatch is a different error with a single settled message.
    let size_err = interp
        .eval_script("B = 1:5; B(zeros(2, 2) == 1) = 5;")
        .expect_err("mask-size mismatch must fail");
    match size_err {
        EvalError::Math {
            error: MathError::LogicalMaskSizeMismatch { identifier },
            ..
        } => {
            assert_eq!(identifier, "MATLAB:dimagree");
            let message = format!(
                "{}",
                MathError::LogicalMaskSizeMismatch {
                    identifier: "MATLAB:dimagree"
                }
            );
            assert_eq!(
                message,
                "Array dimensions must match for binary array op or logical indexing."
            );
        }
        other => panic!("unexpected mask-size error: {other}"),
    }
}

fn rendered(code: &str) -> String {
    let mut interp = new_interpreter();
    interp
        .eval_script(code)
        .unwrap_or_else(|e| panic!("eval failed for `{code}`: {e}"));
    interp.output_log.join("")
}

#[test]
fn test_g01_display_scale_factor_0_001_1000_golden_unverified() {
    assert_eq!(
        rendered("m = [0.001 1000]"),
        "m =\n\n  1.0e+03 *\n\n    0.0000  1.0000\n"
    );
}

#[test]
fn test_g01_display_extreme_range_1e_16_1_golden_unverified() {
    assert_eq!(
        rendered("m = [1e-16 1]"),
        "m =\n\n    1.0000e-16  1.0000e+00\n"
    );
}
