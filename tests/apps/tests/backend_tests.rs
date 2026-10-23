//! Curve Fitter backend tests. The solver is matlab_linalg itself:
//! `mldivide` solves the overdetermined Vandermonde system with Householder
//! QR least squares (and the exact square interpolating system with pivoted
//! LU); this crate hand-rolls no solver math.

fn assert_close(actual: f64, expected: f64, tolerance: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: expected {expected}, got {actual}"
    );
}

#[test]
fn curve_fitter_recovers_all_three_polynomial_coefficients() {
    let x = vec![0.0f64, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y: Vec<f64> = x.iter().map(|&xi| 2.0 * xi * xi - 3.0 * xi + 1.0).collect();
    let res = matlabr_apps::curve_fitter::fit_polynomial(&x, &y, 2).expect("exact fit");
    assert_eq!(res.coefficients.len(), 3);
    assert_close(res.coefficients[0], 1.0, 1e-10, "c0 (constant)");
    assert_close(res.coefficients[1], -3.0, 1e-10, "c1 (linear)");
    assert_close(res.coefficients[2], 2.0, 1e-10, "c2 (quadratic)");
    assert_close(res.r2, 1.0, 1e-10, "R^2");
    assert!(res.rmse < 1e-9, "RMSE = {}", res.rmse);
    assert_eq!(res.points, y.len());
    // Exact fit: fitted values equal the data and every residual is zero.
    for (i, (&fitted, &observed)) in res.fitted.iter().zip(y.iter()).enumerate() {
        assert_close(fitted, observed, 1e-9, &format!("fitted[{i}]"));
        assert_close(res.residuals[i], 0.0, 1e-9, &format!("residual[{i}]"));
    }
}

#[test]
fn curve_fitter_solves_square_exact_system() {
    // Three points determine a quadratic exactly; the Vandermonde system is
    // square and mldivide solves it with pivoted LU inside matlab_linalg.
    let x = vec![0.0f64, 1.0, 2.0];
    let y = vec![1.0f64, 0.0, 3.0];
    let res = matlabr_apps::curve_fitter::fit_polynomial(&x, &y, 2).expect("square exact fit");
    assert_close(res.coefficients[0], 1.0, 1e-10, "c0");
    assert_close(res.coefficients[1], -3.0, 1e-10, "c1");
    assert_close(res.coefficients[2], 2.0, 1e-10, "c2");
    assert!(res.rmse < 1e-10, "RMSE = {}", res.rmse);
}

#[test]
fn curve_fitter_exact_line_coefficients_and_quality() {
    // y = 4x - 2 sampled exactly; linear least squares must recover it.
    let x = vec![-2.0f64, -1.0, 0.0, 1.0, 2.0];
    let y: Vec<f64> = x.iter().map(|&xi| 4.0 * xi - 2.0).collect();
    let res = matlabr_apps::curve_fitter::fit_linear(&x, &y).expect("exact line");
    assert_eq!(res.degree, 1);
    assert_close(res.coefficients[0], -2.0, 1e-10, "intercept");
    assert_close(res.coefficients[1], 4.0, 1e-10, "slope");
    assert_close(res.r2, 1.0, 1e-12, "R^2");
    assert!(res.rmse < 1e-12, "RMSE = {}", res.rmse);
}

#[test]
fn curve_fitter_noisy_line_matches_closed_form_normal_equations() {
    // Closed-form ordinary least squares for a line:
    //   slope   b1 = (n*sum(xy) - sum(x)*sum(y)) / (n*sum(x^2) - (sum(x))^2)
    //   offset  b0 = (sum(y) - b1*sum(x)) / n
    // The QR path must agree with the exact algebra within 1e-9.
    let x = vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0];
    let noise = [-0.1f64, 0.2, -0.15, 0.05, 0.1, -0.05];
    let y: Vec<f64> = x
        .iter()
        .zip(noise.iter())
        .map(|(&xi, &n)| 3.0 * xi + 2.0 + n)
        .collect();

    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f64 = x.iter().map(|v| v * v).sum();
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    let res = matlabr_apps::curve_fitter::fit_polynomial(&x, &y, 1).expect("noisy line");
    assert_close(res.coefficients[1], slope, 1e-9, "slope vs closed form");
    assert_close(
        res.coefficients[0],
        intercept,
        1e-9,
        "intercept vs closed form",
    );

    // Closed-form residuals must match the reported ones too.
    for (i, &xi) in x.iter().enumerate() {
        let expected_resid = y[i] - (intercept + slope * xi);
        assert_close(
            res.residuals[i],
            expected_resid,
            1e-9,
            &format!("residual[{i}]"),
        );
        assert_close(
            res.fitted[i],
            intercept + slope * xi,
            1e-9,
            &format!("fitted[{i}]"),
        );
    }
    let ss_res: f64 = res.residuals.iter().map(|r| r * r).sum();
    assert_close(res.rmse, (ss_res / n).sqrt(), 1e-9, "RMSE");
}

#[test]
fn curve_fitter_rejects_mismatched_vectors() {
    let err = matlabr_apps::curve_fitter::fit_polynomial(&[0.0, 1.0, 2.0], &[1.0, 2.0], 1)
        .expect_err("mismatched x/y lengths must be rejected");
    assert!(
        err.message
            .starts_with("x and y must have the same number of points"),
        "unexpected message: {}",
        err.message
    );
}

#[test]
fn curve_fitter_rejects_nan_samples() {
    let err = matlabr_apps::curve_fitter::fit_polynomial(
        &[0.0, 1.0, f64::NAN, 3.0],
        &[1.0, 2.0, 3.0, 4.0],
        1,
    )
    .expect_err("NaN samples must be rejected");
    assert!(
        err.message.contains("Row 3 contains a NaN"),
        "unexpected message: {}",
        err.message
    );
}

#[test]
fn curve_fitter_rejects_underdetermined_degree() {
    let err = matlabr_apps::curve_fitter::fit_polynomial(&[0.0, 1.0], &[1.0, 2.0], 5)
        .expect_err("must reject underdetermined degree");
    assert!(
        err.message.contains("Degree 5 needs 6 points"),
        "unexpected message: {}",
        err.message
    );
}

#[test]
fn curve_fitter_rejects_fewer_than_two_points() {
    let err = matlabr_apps::curve_fitter::fit_polynomial(&[1.0], &[2.0], 0)
        .expect_err("one point cannot be fitted");
    assert!(
        err.message.contains("At least 2 points are required"),
        "unexpected message: {}",
        err.message
    );
}
