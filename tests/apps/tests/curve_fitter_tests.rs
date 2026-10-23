//! Curve Fitter integration tests: CSV -> parse -> fit -> workspace
//! variables -> figures -> export round trip, error paths, cancellation and
//! the overwrite guard.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use matlabr_apps::curve_fitter;
use matlabr_apps::data_analysis;

fn assert_close(actual: f64, expected: f64, tolerance: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: expected {expected}, got {actual}"
    );
}

/// Write a uniqueness-safe temp CSV fixture under std::env::temp_dir().
fn write_fixture(tag: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "matrust_curve_fixture_{}_{}_{:?}.csv",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&path, content).expect("write fixture");
    path
}

fn scalar(session: &matlab_runtime::Session, name: &str) -> f64 {
    session
        .env
        .get(name)
        .and_then(|v| v.as_matrix())
        .and_then(|m| m.as_scalar())
        .unwrap_or_else(|| panic!("{name} should be a scalar"))
}

fn matrix(session: &matlab_runtime::Session, name: &str) -> matlab_values::DenseMatrix {
    session
        .env
        .get(name)
        .and_then(|v| v.as_matrix())
        .unwrap_or_else(|| panic!("{name} should be a matrix"))
        .clone()
}

const QUIET_LINE: &str = "x,y\n1,5\n2,8\n3,11\n4,14\n5,17\n";

#[test]
fn curve_fitter_rejects_non_numeric_column() {
    let path = write_fixture(
        "nonnumeric",
        "x,label,noise\n1,alpha,0.2\n2,beta,0.3\n3,gamma,0.4\n",
    );
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let err = curve_fitter::fit_from_table(&table, 0, 1, 1, None)
        .expect_err("text column must be rejected");
    assert!(
        err.message.contains("not numeric"),
        "unexpected: {}",
        err.message
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_rejects_missing_value_in_selected_columns() {
    let path = write_fixture("missing_y", "x,y\n1,5\n2,\n3,11\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let err = curve_fitter::fit_from_table(&table, 0, 1, 1, None)
        .expect_err("missing y must be rejected");
    assert!(
        err.message.contains("Row 2") && err.message.contains("missing value"),
        "unexpected: {}",
        err.message
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_rejects_out_of_range_columns() {
    let path = write_fixture("oor", "x,y\n1,5\n2,8\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let err = curve_fitter::fit_from_table(&table, 0, 7, 1, None)
        .expect_err("out-of-range y must be rejected");
    assert!(
        err.message.contains("out of range"),
        "unexpected: {}",
        err.message
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_csv_fit_workspace_export_round_trip() {
    let path = write_fixture("line", QUIET_LINE);
    let mut session = matlab_runtime::Session::new();
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let result = curve_fitter::run_fit(&mut session, &table, 0, 1, 1, None)
        .expect("run fit on the worker-visible backend");
    assert_close(result.coefficients[0], 2.0, 1e-10, "intercept");
    assert_close(result.coefficients[1], 3.0, 1e-10, "slope");
    assert_close(result.r2, 1.0, 1e-10, "R^2");
    assert!(result.rmse < 1e-10, "RMSE = {}", result.rmse);

    // Workspace variables.
    let coeffs = matrix(&session, "curve_fit_coefficients");
    assert_eq!((coeffs.rows, coeffs.cols), (1, 2));
    assert_close(coeffs.data[0], 2.0, 1e-10, "workspace intercept");
    assert_close(coeffs.data[1], 3.0, 1e-10, "workspace slope");
    let fitted = matrix(&session, "curve_fit_fitted");
    assert_eq!((fitted.rows, fitted.cols), (5, 1));
    assert_close(fitted.data[3], 14.0, 1e-10, "fitted[4]");
    let residuals = matrix(&session, "curve_fit_residuals");
    assert_eq!(residuals.data.len(), 5);
    for r in &residuals.data {
        assert!(r.abs() < 1e-10);
    }
    assert_close(scalar(&session, "curve_fit_r2"), 1.0, 1e-10, "r2 var");
    assert_close(
        scalar(&session, "curve_fit_degree"),
        1.0,
        1e-10,
        "degree var",
    );

    // Figures: data+fit and residuals.
    let figures = &session.graphics.figures;
    assert_eq!(figures.len(), 2);
    assert_eq!(figures[0].items.len(), 2, "scatter + fit line");

    // Export coefficients and fitted values, then re-read and verify.
    let coeff_path = write_fixture("coeff_out", "");
    let fitted_path = write_fixture("fitted_out", "");
    let xs: Vec<f64> = table
        .numeric
        .iter()
        .map(|row| row[0].expect("numeric x"))
        .collect();
    curve_fitter::export_fit_csv(&coeff_path, &fitted_path, &result, &xs, true)
        .expect("export coefficients and fitted values together");
    let coeff_csv = std::fs::read_to_string(&coeff_path).expect("read coeffs");
    assert!(coeff_csv.starts_with("coefficient,value\n"));
    assert!(coeff_csv.contains("c0,") && coeff_csv.contains("c1,"));
    let fitted_csv = std::fs::read_to_string(&fitted_path).expect("read fitted");
    assert!(fitted_csv.starts_with("index,x,fitted,residual\n"));
    for (index, line) in fitted_csv.lines().skip(1).enumerate() {
        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(parts.len(), 4, "four columns per row");
        let fitted_value: f64 = parts[2].parse().expect("fitted parses");
        let expected_x = (index + 1) as f64;
        let residual_value: f64 = parts[3].parse().expect("residual parses");
        assert!(
            (fitted_value - (2.0 + 3.0 * expected_x)).abs() < 1e-9,
            "fitted at row {index}"
        );
        assert!(residual_value.abs() < 1e-9, "residual at row {index}");
    }

    // The workspace variables survive a Session-based MATLAB readback.
    session
        .exec("b = curve_fit_coefficients; s = b(1,2);", None)
        .expect("exec");
    let s = scalar(&session, "s");
    assert_close(s, 3.0, 1e-10, "round trip through the interpreter");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&coeff_path);
    let _ = std::fs::remove_file(&fitted_path);
}

#[test]
fn curve_fitter_second_write_failure_leaves_no_final_files() {
    // The coefficients destination is a valid free path; the fitted destination
    // has a missing parent directory, so the second temporary write fails.
    let coeff_path = write_fixture("second_write_coeff", "");
    std::fs::remove_file(&coeff_path).expect("remove coefficient placeholder");
    let missing_dir = std::env::temp_dir().join(format!(
        "matrust_curve_missing_dir_{}_{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    assert!(!missing_dir.exists(), "missing-dir precondition");
    let fitted_path = missing_dir.join("fitted.csv");

    let result = curve_fitter::fit_polynomial(&[0.0, 1.0, 2.0], &[1.0, 3.0, 5.0], 1).expect("fit");
    let xs = vec![0.0, 1.0, 2.0];
    let err = curve_fitter::export_fit_csv(&coeff_path, &fitted_path, &result, &xs, true)
        .expect_err("second temporary write must fail");
    assert!(
        err.message.contains("Could not create temporary export"),
        "unexpected diagnostic: {}",
        err.message
    );

    assert!(
        !coeff_path.exists(),
        "coefficient final must not be created when the second write fails"
    );
    assert!(
        !fitted_path.exists(),
        "fitted final must not be created when the second write fails"
    );
    assert!(
        !missing_dir.exists(),
        "failed export must not create the directory"
    );

    let coeff_name = coeff_path
        .file_name()
        .expect("coefficient file name")
        .to_string_lossy()
        .to_string();
    let prefix = format!(".{coeff_name}.");
    let leftovers: Vec<_> = std::fs::read_dir(coeff_path.parent().expect("temp dir"))
        .expect("read temp dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temporary coefficient file was not cleaned up: {leftovers:?}"
    );
}

#[test]
fn curve_fitter_export_refuses_overwrite() {
    let path = write_fixture("guarded", "pre-existing,guard\n");
    let result = curve_fitter::fit_polynomial(&[0.0, 1.0, 2.0], &[0.0, 1.0, 2.0], 1).expect("fit");
    let err = curve_fitter::export_coefficients_csv(&path, &result, false)
        .expect_err("must refuse without confirmation");
    assert!(
        err.message.contains("Refusing to overwrite"),
        "unexpected: {}",
        err.message
    );
    // The pre-existing guard file must be untouched.
    let content = std::fs::read_to_string(&path).expect("read guard file");
    assert_eq!(content, "pre-existing,guard\n", "file must not be modified");

    // Explicit overwrite is allowed.
    curve_fitter::export_coefficients_csv(&path, &result, true).expect("overwrite with confirm");
    let content = std::fs::read_to_string(&path).expect("read overwritten");
    assert!(content.starts_with("coefficient,value\n"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_cancellation_flag_stops_the_fit() {
    let path = write_fixture("cancel", QUIET_LINE);
    let cancel = AtomicBool::new(true);
    let mut session = matlab_runtime::Session::new();
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let err = curve_fitter::run_fit(&mut session, &table, 0, 1, 1, Some(&cancel))
        .expect_err("pre-cancelled fit must report cancellation");
    assert!(
        err.message.contains("cancelled"),
        "unexpected: {}",
        err.message
    );
    assert!(
        session.env.get("curve_fit_coefficients").is_none(),
        "no results may be written when cancelled"
    );
    assert!(
        session.graphics.figures.is_empty(),
        "no figures may be pushed when cancelled"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_residuals_figure_has_zero_baseline() {
    let path = write_fixture("resid", QUIET_LINE);
    let mut session = matlab_runtime::Session::new();
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    curve_fitter::run_fit(&mut session, &table, 0, 1, 1, None).expect("fit");
    let figures = &session.graphics.figures;
    let residual_fig = figures.last().expect("residual figure pushed");
    assert_eq!(
        residual_fig.title.as_deref(),
        Some("Curve Fitter — residuals")
    );
    assert_eq!(residual_fig.items.len(), 2, "scatter + zero baseline");
    // Zero baseline endpoints.
    let zero_baseline = match &residual_fig.items[1] {
        matlab_runtime::PlotItem::Line { x, y, .. } => {
            assert_eq!(y, &vec![0.0, 0.0]);
            x.clone()
        }
        other => panic!("expected zero line, got {other:?}"),
    };
    assert_eq!(zero_baseline, vec![1.0, 5.0], "baseline spans the x range");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn curve_fitter_polyfit_degree_two_through_table() {
    // y = 2x^2 - 3x + 1 with x = 0..5, sampled exactly, degree 2.
    let path = write_fixture("quad", "x,y\n0,1\n1,0\n2,3\n3,10\n4,21\n5,36\n");
    let mut session = matlab_runtime::Session::new();
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let result = curve_fitter::run_fit(&mut session, &table, 0, 1, 2, None).expect("quadratic fit");
    assert_close(result.coefficients[0], 1.0, 1e-10, "c0");
    assert_close(result.coefficients[1], -3.0, 1e-10, "c1");
    assert_close(result.coefficients[2], 2.0, 1e-10, "c2");
    assert_close(scalar(&session, "curve_fit_r2"), 1.0, 1e-10, "r2");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn registry_marks_curve_fitter_ready_once_ui_exists() {
    let curve = matlabr_apps::find_by_id("curve_fitter").expect("registry entry");
    assert!(
        curve.launchable(),
        "Curve Fitter must be launchable once its UI exists"
    );
    let data = matlabr_apps::find_by_id("data_analysis").expect("registry entry");
    assert!(data.launchable(), "data analysis stays Ready");
    for app in matlabr_apps::REGISTRY {
        if app.id != "data_analysis" && app.id != "curve_fitter" {
            assert!(!app.launchable(), "{} must not be launchable", app.id);
        }
    }
}
