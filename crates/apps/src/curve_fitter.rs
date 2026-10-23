//! Curve Fitter backend — polynomial and linear least-squares fitting.
//!
//! The solve runs through the matlab_linalg QR path: the Vandermonde system is
//! handed to `matlab_linalg::decomposition::mldivide`, which routes every
//! overdetermined system through Householder QR least squares. No solver math
//! lives in this crate. Failures are `Diagnostic`s; there is no UI code here.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use matlab_diagnostics::Diagnostic;
use matlab_linalg::decomposition::mldivide;
use matlab_runtime::graphics::FigureState;
use matlab_runtime::graphics::PlotItem;
use matlab_runtime::Session;
use matlab_values::{DenseMatrix, Value};

use crate::data_analysis::ColumnKind;
use crate::data_analysis::CsvTable;

/// Result of a least-squares polynomial fit.
#[derive(Debug, Clone, PartialEq)]
pub struct FitResult {
    pub degree: usize,
    /// Ascending powers: `coefficients[0]` is the constant term.
    pub coefficients: Vec<f64>,
    pub r2: f64,
    pub rmse: f64,
    /// Fitted values in input order (observed best fit at each x).
    pub fitted: Vec<f64>,
    /// Residuals in input order: observed minus fitted.
    pub residuals: Vec<f64>,
    /// Column names from the table, when the fit came from a table.
    pub x_name: String,
    pub y_name: String,
    pub points: usize,
}

fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

/// Evaluate the fitted polynomial (ascending `coefficients`) at `x`.
fn eval_poly(coefficients: &[f64], x: f64) -> f64 {
    let mut acc = 0.0_f64;
    for &c in coefficients.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

/// Validate raw data slices and fit a degree-`degree` polynomial through the
/// linalg QR least-squares solver.
pub fn fit_polynomial(x: &[f64], y: &[f64], degree: usize) -> Result<FitResult, Diagnostic> {
    fit_polynomial_columns(x, y, degree, ("x", "y"), None)
}

fn fit_polynomial_columns(
    x: &[f64],
    y: &[f64],
    degree: usize,
    names: (&str, &str),
    cancel: Option<&AtomicBool>,
) -> Result<FitResult, Diagnostic> {
    if x.len() != y.len() {
        return Err(Diagnostic::error(
            format!(
                "x and y must have the same number of points ({} != {}).",
                x.len(),
                y.len()
            ),
            None,
        ));
    }
    if x.len() < 2 {
        return Err(Diagnostic::error(
            format!(
                "At least 2 points are required for a fit; got {} point(s).",
                x.len()
            ),
            None,
        ));
    }
    if degree + 1 > x.len() {
        return Err(Diagnostic::error(
            format!(
                "Degree {degree} needs {} points, but the data has {} point(s); underdetermined.",
                degree + 1,
                x.len()
            ),
            None,
        ));
    }
    for (idx, (xi, yi)) in x.iter().zip(y.iter()).enumerate() {
        if xi.is_nan() || yi.is_nan() {
            return Err(Diagnostic::error(
                format!(
                    "Row {} contains a NaN in ({}, {}); remove it before fitting.",
                    idx + 1,
                    names.0,
                    names.1
                ),
                None,
            ));
        }
        if xi.is_infinite() || yi.is_infinite() {
            return Err(Diagnostic::error(
                format!(
                    "Row {} contains a non-finite value in ({}, {}).",
                    idx + 1,
                    names.0,
                    names.1
                ),
                None,
            ));
        }
    }
    if let Some(flag) = cancel {
        if flag.load(Ordering::Relaxed) {
            return Err(Diagnostic::error("Fit cancelled before solving.", None));
        }
    }

    // Vandermonde matrix n x (degree+1), row-major: row r = [1, x_r, x_r^2, ...].
    let k = degree + 1;
    let mut vandermonde = vec![0.0; x.len() * k];
    for (r, &xr) in x.iter().enumerate() {
        let mut power = 1.0;
        for c in 0..k {
            vandermonde[r * k + c] = power;
            power *= xr;
        }
    }
    let design = DenseMatrix {
        rows: x.len(),
        cols: k,
        data: vandermonde,
    };
    let y_col = DenseMatrix {
        rows: y.len(),
        cols: 1,
        data: y.to_vec(),
    };
    // Overdetermined (n > degree+1 is guaranteed above): matlab_linalg solves
    // this with Householder QR least squares.
    let solve = mldivide(&design, &y_col, None)?;
    let coefficients: Vec<f64> = solve.data[..k].to_vec();
    if coefficients.iter().any(|c| !c.is_finite()) {
        return Err(Diagnostic::error(
            "The QR solve could not produce finite coefficients (rank-deficient design).",
            None,
        ));
    }

    let fitted: Vec<f64> = x.iter().map(|&xi| eval_poly(&coefficients, xi)).collect();
    let residuals: Vec<f64> = y
        .iter()
        .zip(fitted.iter())
        .map(|(&observed, &predicted)| observed - predicted)
        .collect();
    let ss_res: f64 = residuals.iter().map(|r| r * r).sum();
    let mean_y = y.iter().sum::<f64>() / y.len() as f64;
    let ss_tot: f64 = y.iter().map(|v| (v - mean_y) * (v - mean_y)).sum();
    // R^2 is NaN for a constant y column (0/0 fit quality); documented.
    let r2 = if ss_tot.abs() < f64::EPSILON {
        f64::NAN
    } else {
        1.0 - ss_res / ss_tot
    };
    let rmse = (ss_res / y.len() as f64).sqrt();

    Ok(FitResult {
        degree,
        coefficients,
        r2,
        rmse,
        fitted,
        residuals,
        x_name: names.0.to_string(),
        y_name: names.1.to_string(),
        points: x.len(),
    })
}

/// Convenience: linear least squares (degree-1 polynomial). Same QR path.
pub fn fit_linear(x: &[f64], y: &[f64]) -> Result<FitResult, Diagnostic> {
    fit_polynomial(x, y, 1)
}

/// Fit selected table columns. Non-numeric columns, missing cells and
/// out-of-range selections produce explicit `Diagnostic`s; nothing is dropped
/// silently.
pub fn fit_from_table(
    table: &CsvTable,
    x_col: usize,
    y_col: usize,
    degree: usize,
    cancel: Option<&AtomicBool>,
) -> Result<FitResult, Diagnostic> {
    if x_col >= table.col_count {
        return Err(Diagnostic::error(
            format!("x column index {} is out of range.", x_col),
            None,
        ));
    }
    if y_col >= table.col_count {
        return Err(Diagnostic::error(
            format!("y column index {} is out of range.", y_col),
            None,
        ));
    }
    for (label, col) in [("x", x_col), ("y", y_col)] {
        if table.columns[col].kind == ColumnKind::Text {
            return Err(Diagnostic::error(
                format!(
                    "Column '{}' (selected as {}) is not numeric.",
                    table.columns[col].name, label
                ),
                None,
            ));
        }
    }
    if cancelled(cancel) {
        return Err(Diagnostic::error("Fit cancelled before parsing.", None));
    }
    let mut xs = Vec::with_capacity(table.row_count);
    let mut ys = Vec::with_capacity(table.row_count);
    for (row_idx, row) in table.numeric.iter().enumerate() {
        match (row[x_col], row[y_col]) {
            (Some(x), Some(y)) => {
                xs.push(x);
                ys.push(y);
            }
            (Some(_), None) => {
                return Err(Diagnostic::error(
                    format!(
                        "Row {} has a missing value in column '{}'; remove it before fitting.",
                        row_idx + 1,
                        table.columns[y_col].name
                    ),
                    None,
                ));
            }
            (None, Some(_)) => {
                return Err(Diagnostic::error(
                    format!(
                        "Row {} has a missing value in column '{}'; remove it before fitting.",
                        row_idx + 1,
                        table.columns[x_col].name
                    ),
                    None,
                ));
            }
            (None, None) => {
                return Err(Diagnostic::error(
                    format!("Row {} has missing values in both selected columns; remove it before fitting.", row_idx + 1),
                    None,
                ));
            }
        }
    }
    fit_polynomial_columns(
        &xs,
        &ys,
        degree,
        (&table.columns[x_col].name, &table.columns[y_col].name),
        cancel,
    )
}

/// Fit against an open session: computes through `fit_from_table`, writes the
/// results to workspace variables, and pushes a data+picsum figure and a
/// residual figure. The UI never does the math itself.
pub fn run_fit(
    session: &mut Session,
    table: &CsvTable,
    x_col: usize,
    y_col: usize,
    degree: usize,
    cancel: Option<&AtomicBool>,
) -> Result<FitResult, Diagnostic> {
    let result = fit_from_table(table, x_col, y_col, degree, cancel)?;

    // Workspace variables (1 x (degree+1) coefficient row, column vectors).
    session.env.set(
        "curve_fit_coefficients",
        Value::Matrix(DenseMatrix {
            rows: 1,
            cols: result.coefficients.len(),
            data: result.coefficients.clone(),
        }),
    );
    session.env.set(
        "curve_fit_fitted",
        Value::Matrix(DenseMatrix {
            rows: result.fitted.len(),
            cols: 1,
            data: result.fitted.clone(),
        }),
    );
    session.env.set(
        "curve_fit_residuals",
        Value::Matrix(DenseMatrix {
            rows: result.residuals.len(),
            cols: 1,
            data: result.residuals.clone(),
        }),
    );
    session.env.set(
        "curve_fit_r2",
        Value::Matrix(DenseMatrix::scalar(result.r2)),
    );
    session.env.set(
        "curve_fit_rmse",
        Value::Matrix(DenseMatrix::scalar(result.rmse)),
    );
    session.env.set(
        "curve_fit_degree",
        Value::Matrix(DenseMatrix::scalar(result.degree as f64)),
    );

    // Figure 1: data scatter plus the fitted curve (sorted by x so the
    // polyline follows the polynomial, not the input order).
    let xs: Vec<f64> = (0..result.points)
        .map(|i| table.numeric[i][x_col].unwrap_or(f64::NAN))
        .collect();
    let ys: Vec<f64> = (0..result.points)
        .map(|i| table.numeric[i][y_col].unwrap_or(f64::NAN))
        .collect();
    let mut order: Vec<usize> = (0..result.points).collect();
    order.sort_by(|&a, &b| {
        xs[a]
            .partial_cmp(&xs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let fit_line_x: Vec<f64> = order.iter().map(|&i| xs[i]).collect();
    let fit_line_y: Vec<f64> = order.iter().map(|&i| result.fitted[i]).collect();

    let handle = session
        .graphics
        .figures
        .iter()
        .map(|f| f.handle)
        .max()
        .unwrap_or(0)
        + 1;
    let mut figure = FigureState::fresh(handle);
    figure.title = Some(format!(
        "Curve Fitter — {} vs {}",
        result.y_name, result.x_name
    ));
    figure.xlabel = Some(result.x_name.clone());
    figure.ylabel = Some(result.y_name.clone());
    figure.grid = true;
    figure.items.push(PlotItem::Scatter {
        label: "data".to_string(),
        x: xs.clone(),
        y: ys.clone(),
    });
    figure.items.push(PlotItem::Line {
        label: format!("degree {} fit", result.degree),
        x: fit_line_x,
        y: fit_line_y,
    });
    session.graphics.figures.push(figure);
    session.graphics.current = Some(session.graphics.figures.len() - 1);

    // Figure 2: residuals against x with the two-point zero baseline
    // (`points >= 2` is guaranteed by the fit validation).
    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let handle_zero = session
        .graphics
        .figures
        .iter()
        .map(|f| f.handle)
        .max()
        .unwrap_or(0)
        + 1;
    let mut residual_figure = FigureState::fresh(handle_zero);
    residual_figure.title = Some("Curve Fitter — residuals".to_string());
    residual_figure.xlabel = Some(result.x_name.clone());
    residual_figure.ylabel = Some("residual".to_string());
    residual_figure.grid = true;
    residual_figure.items.push(PlotItem::Scatter {
        label: "residuals".to_string(),
        x: xs,
        y: result.residuals.clone(),
    });
    residual_figure.items.push(PlotItem::Line {
        label: "zero".to_string(),
        x: vec![min_x, max_x],
        y: vec![0.0, 0.0],
    });
    session.graphics.figures.push(residual_figure);
    session.graphics.current = Some(session.graphics.figures.len() - 1);

    Ok(result)
}

fn guard_overwrite(path: &Path, overwrite: bool, what: &str) -> Result<(), Diagnostic> {
    if path.exists() && !overwrite {
        return Err(Diagnostic::error(
            format!(
                "Refusing to overwrite existing file {} without confirmation ({}).",
                path.display(),
                what
            ),
            None,
        ));
    }
    Ok(())
}

fn coefficients_csv(result: &FitResult) -> String {
    let mut out = String::from("coefficient,value\n");
    for (i, c) in result.coefficients.iter().enumerate() {
        out.push_str(&format!("c{},{}\n", i, c));
    }
    out
}

fn fitted_csv(result: &FitResult, xs: &[f64]) -> Result<String, Diagnostic> {
    if xs.len() != result.fitted.len() {
        return Err(Diagnostic::error(
            "x values and fitted values must have the same length to export.",
            None,
        ));
    }
    let mut out = String::from("index,x,fitted,residual\n");
    for (i, ((x, fitted), residual)) in xs
        .iter()
        .zip(result.fitted.iter())
        .zip(result.residuals.iter())
        .enumerate()
    {
        out.push_str(&format!("{},{},{},{}\n", i + 1, x, fitted, residual));
    }
    Ok(out)
}

/// Export the coefficients as `coefficient,value` CSV rows (c0..cd ascending).
pub fn export_coefficients_csv(
    path: &Path,
    result: &FitResult,
    overwrite: bool,
) -> Result<(), Diagnostic> {
    guard_overwrite(path, overwrite, "coefficients")?;
    std::fs::write(path, coefficients_csv(result)).map_err(|err| {
        Diagnostic::error(format!("Could not write {}: {}", path.display(), err), None)
    })
}

/// Export the data x values with the fitted values and residuals, one row per
/// data point: `index,x,fitted,residual`.
pub fn export_fitted_csv(
    path: &Path,
    result: &FitResult,
    xs: &[f64],
    overwrite: bool,
) -> Result<(), Diagnostic> {
    guard_overwrite(path, overwrite, "fitted values")?;
    let out = fitted_csv(result, xs)?;
    std::fs::write(path, out).map_err(|err| {
        Diagnostic::error(format!("Could not write {}: {}", path.display(), err), None)
    })
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_sibling(path: &Path, tag: &str) -> PathBuf {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("curve_fit.csv");
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{name}.{tag}.{}.{}.tmp",
        std::process::id(),
        nonce
    ))
}

fn write_temp(path: &Path, tag: &str, contents: &str) -> Result<PathBuf, Diagnostic> {
    let temp = temp_sibling(path, tag);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|err| {
            Diagnostic::error(
                format!(
                    "Could not create temporary export for {}: {}",
                    path.display(),
                    err
                ),
                None,
            )
        })?;
    if let Err(err) = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
    {
        let _ = std::fs::remove_file(&temp);
        return Err(Diagnostic::error(
            format!(
                "Could not write temporary export for {}: {}",
                path.display(),
                err
            ),
            None,
        ));
    }
    Ok(temp)
}

fn backup_existing(path: &Path, tag: &str) -> Result<Option<PathBuf>, Diagnostic> {
    if !path.exists() {
        return Ok(None);
    }
    let backup = temp_sibling(path, tag);
    std::fs::rename(path, &backup).map_err(|err| {
        Diagnostic::error(
            format!("Could not prepare {} for export: {}", path.display(), err),
            None,
        )
    })?;
    Ok(Some(backup))
}

fn restore_backup(path: &Path, backup: Option<PathBuf>) {
    if let Some(backup) = backup {
        let _ = std::fs::rename(backup, path);
    }
}

fn remove_if_present(path: &Path) {
    let _ = std::fs::remove_file(path);
}

/// Export both Curve Fitter files transactionally.
///
/// Both CSV bodies are written to temporary files next to their destinations
/// before either destination is touched.  Only after both writes succeed are
/// the temporary files renamed into place.  If the second write fails, the
/// first temporary file is removed and neither final path is created.
pub fn export_fit_csv(
    coeff_path: &Path,
    fitted_path: &Path,
    result: &FitResult,
    xs: &[f64],
    overwrite: bool,
) -> Result<(), Diagnostic> {
    if coeff_path == fitted_path {
        return Err(Diagnostic::error(
            "Coefficient and fitted-value exports must use different paths.",
            None,
        ));
    }
    guard_overwrite(coeff_path, overwrite, "coefficients")?;
    guard_overwrite(fitted_path, overwrite, "fitted values")?;

    let coeff_contents = coefficients_csv(result);
    let fitted_contents = fitted_csv(result, xs)?;

    let coeff_temp = write_temp(coeff_path, "coeff", &coeff_contents)?;
    let fitted_temp = match write_temp(fitted_path, "fitted", &fitted_contents) {
        Ok(path) => path,
        Err(err) => {
            remove_if_present(&coeff_temp);
            return Err(err);
        }
    };

    // Keep the previous files recoverable until both renames have succeeded.
    let coeff_backup = match backup_existing(coeff_path, "coeff-backup") {
        Ok(backup) => backup,
        Err(err) => {
            remove_if_present(&coeff_temp);
            remove_if_present(&fitted_temp);
            return Err(err);
        }
    };
    let fitted_backup = match backup_existing(fitted_path, "fitted-backup") {
        Ok(backup) => backup,
        Err(err) => {
            restore_backup(coeff_path, coeff_backup);
            remove_if_present(&coeff_temp);
            remove_if_present(&fitted_temp);
            return Err(err);
        }
    };

    if let Err(err) = std::fs::rename(&coeff_temp, coeff_path) {
        restore_backup(coeff_path, coeff_backup);
        restore_backup(fitted_path, fitted_backup);
        remove_if_present(&coeff_temp);
        remove_if_present(&fitted_temp);
        return Err(Diagnostic::error(
            format!("Could not finalize {}: {}", coeff_path.display(), err),
            None,
        ));
    }
    if let Err(err) = std::fs::rename(&fitted_temp, fitted_path) {
        remove_if_present(coeff_path);
        restore_backup(coeff_path, coeff_backup);
        restore_backup(fitted_path, fitted_backup);
        remove_if_present(&fitted_temp);
        return Err(Diagnostic::error(
            format!("Could not finalize {}: {}", fitted_path.display(), err),
            None,
        ));
    }

    if let Some(backup) = coeff_backup {
        remove_if_present(&backup);
    }
    if let Some(backup) = fitted_backup {
        remove_if_present(&backup);
    }
    Ok(())
}
