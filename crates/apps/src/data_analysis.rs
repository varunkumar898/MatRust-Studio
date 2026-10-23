//! Data Analysis backend.
//!
//! Plain Rust API: paths and options in, owned results out, failures as
//! `Diagnostic`. The UI never parses or computes statistics itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use matlab_diagnostics::Diagnostic;
use matlab_runtime::graphics::{FigureState, PlotItem};
use matlab_runtime::Session;
use matlab_values::{DenseMatrix, Value};

const MISSING_TOKENS: &[&str] = &["", "na", "nan", "null", "n/a"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnKind {
    Numeric,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    pub name: String,
    pub kind: ColumnKind,
    pub missing: usize,
}

/// Parsed CSV table. `rows` contains the raw fields (header excluded);
/// `numeric[row][col]` is `Some` only for parseable values in numeric columns.
#[derive(Debug, Clone, PartialEq)]
pub struct CsvTable {
    pub path: PathBuf,
    pub delimiter: char,
    pub has_header: bool,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
    pub numeric: Vec<Vec<Option<f64>>>,
    pub ragged: bool,
    pub row_count: usize,
    pub col_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub missing: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q50: f64,
    pub q75: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotKind {
    Column,
    Scatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotRequest {
    pub kind: PlotKind,
    /// Column plotted against the row index for `Column`, or the x axis for
    /// `Scatter`.
    pub x: usize,
    /// Column plotted on the y axis.
    pub y: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisOutcome {
    pub table: CsvTable,
    pub stats: Vec<ColumnStats>,
    pub correlation: Option<f64>,
    pub plotted: bool,
}

fn cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

fn is_missing(field: &str) -> bool {
    let trimmed = field.trim();
    MISSING_TOKENS
        .iter()
        .any(|token| trimmed.eq_ignore_ascii_case(token))
}

/// Split one CSV record, supporting quoted fields and doubled quotes.
/// Embedded newlines inside quotes are not supported (one record per line).
fn split_fields(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' && current.trim().is_empty() {
            current.clear();
            in_quotes = true;
        } else if ch == delimiter {
            fields.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn detect_delimiter(content: &str) -> char {
    let first = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default();
    let mut best = ',';
    let mut best_count = 0usize;
    for candidate in [',', '\t', ';'] {
        let count = split_fields(first, candidate).len().saturating_sub(1);
        if count > best_count {
            best = candidate;
            best_count = count;
        }
    }
    best
}

fn looks_like_header(record: &[String]) -> bool {
    record
        .iter()
        .any(|field| !field.trim().is_empty() && field.chars().any(|c| c.is_alphabetic()))
}

fn parse_numeric(field: &str) -> Option<f64> {
    field.trim().parse::<f64>().ok().filter(|v| !v.is_nan())
}

/// Parse a CSV file. Missing cells become `None`; text values are retained in
/// `rows` and make the column `Text` (they are not misreported as missing).
pub fn parse_csv(path: &Path, cancel: Option<&AtomicBool>) -> Result<CsvTable, Diagnostic> {
    let content = std::fs::read_to_string(path).map_err(|err| {
        Diagnostic::error(format!("Could not read {}: {}", path.display(), err), None)
    })?;
    if content.trim().is_empty() {
        return Err(Diagnostic::error("CSV file is empty.", None));
    }

    let delimiter = detect_delimiter(&content);
    let records: Vec<Vec<String>> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| split_fields(line, delimiter))
        .collect();
    if records.is_empty() {
        return Err(Diagnostic::error("CSV file has no records.", None));
    }

    if cancelled(cancel) {
        return Err(Diagnostic::error("Data Analysis cancelled.", None));
    }

    let has_header = looks_like_header(&records[0]);
    let col_count = records.iter().map(Vec::len).max().unwrap_or(0);
    if col_count == 0 {
        return Err(Diagnostic::error("CSV file has no columns.", None));
    }

    let header = if has_header {
        records[0].clone()
    } else {
        Vec::new()
    };
    let data_records = if has_header {
        &records[1..]
    } else {
        &records[..]
    };
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(data_records.len());
    let mut ragged = false;
    for record in data_records {
        if cancelled(cancel) {
            return Err(Diagnostic::error("Data Analysis cancelled.", None));
        }
        if record.len() != col_count {
            ragged = true;
        }
        let mut row = record.clone();
        row.resize(col_count, String::new());
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(Diagnostic::error(
            "CSV file has a header but no data rows.",
            None,
        ));
    }

    let mut names = Vec::with_capacity(col_count);
    for i in 0..col_count {
        let name = header
            .get(i)
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| format!("col{}", i + 1));
        names.push(name);
    }

    let mut columns = Vec::with_capacity(col_count);
    let mut numeric = vec![vec![None; col_count]; rows.len()];
    for col in 0..col_count {
        let mut missing = 0usize;
        let mut text = false;
        for (row_idx, row) in rows.iter().enumerate() {
            let field = &row[col];
            if is_missing(field) {
                missing += 1;
            } else if let Some(value) = parse_numeric(field) {
                numeric[row_idx][col] = Some(value);
            } else {
                text = true;
            }
        }
        columns.push(ColumnInfo {
            name: names[col].clone(),
            kind: if text {
                ColumnKind::Text
            } else {
                ColumnKind::Numeric
            },
            missing,
        });
    }

    Ok(CsvTable {
        path: path.to_path_buf(),
        delimiter,
        has_header,
        columns,
        rows: rows.clone(),
        numeric,
        ragged,
        row_count: rows.len(),
        col_count,
    })
}

fn numeric_values(table: &CsvTable, col: usize) -> Result<Vec<f64>, Diagnostic> {
    let info = table
        .columns
        .get(col)
        .ok_or_else(|| Diagnostic::error(format!("Column index {} is out of range.", col), None))?;
    if info.kind == ColumnKind::Text {
        return Err(Diagnostic::error(
            format!(
                "Column '{}' contains text and has no numeric statistics.",
                info.name
            ),
            None,
        ));
    }
    let values: Vec<f64> = table.numeric.iter().filter_map(|row| row[col]).collect();
    if values.is_empty() {
        return Err(Diagnostic::error(
            format!("Column '{}' has no numeric values.", info.name),
            None,
        ));
    }
    Ok(values)
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn sample_std(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let sum_sq: f64 = values.iter().map(|v| (v - m) * (v - m)).sum();
    (sum_sq / (values.len() - 1) as f64).sqrt()
}

fn quantile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    // Median-unbiased plotting position, matching the interpreter's stats.
    let r = 0.5 + sorted.len() as f64 * p;
    if r <= 1.0 {
        sorted[0]
    } else if r >= sorted.len() as f64 {
        sorted[sorted.len() - 1]
    } else {
        let k = r.floor() as usize;
        let f = r - k as f64;
        (1.0 - f) * sorted[k - 1] + f * sorted[k]
    }
}

/// Compute all requested statistics for a numeric column.
pub fn column_stats(
    table: &CsvTable,
    col: usize,
    cancel: Option<&AtomicBool>,
) -> Result<ColumnStats, Diagnostic> {
    let values = numeric_values(table, col)?;
    if cancelled(cancel) {
        return Err(Diagnostic::error("Data Analysis cancelled.", None));
    }
    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let info = &table.columns[col];
    Ok(ColumnStats {
        name: info.name.clone(),
        count: values.len(),
        missing: info.missing,
        mean: mean(&values),
        median: quantile(&sorted, 0.5),
        std_dev: sample_std(&values),
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        q25: quantile(&sorted, 0.25),
        q50: quantile(&sorted, 0.5),
        q75: quantile(&sorted, 0.75),
    })
}

/// Pearson correlation over rows where both columns have numeric values.
pub fn correlation(table: &CsvTable, x_col: usize, y_col: usize) -> Result<f64, Diagnostic> {
    numeric_values(table, x_col)?;
    numeric_values(table, y_col)?;
    let mut pairs = Vec::new();
    for row in &table.numeric {
        if let (Some(a), Some(b)) = (row[x_col], row[y_col]) {
            pairs.push((a, b));
        }
    }
    let x_values: Vec<f64> = pairs.iter().map(|p| p.0).collect();
    let y_values: Vec<f64> = pairs.iter().map(|p| p.1).collect();
    if pairs.len() < 2 {
        return Err(Diagnostic::error(
            "Correlation requires at least two complete row pairs.",
            None,
        ));
    }
    let mx = mean(&x_values);
    let my = mean(&y_values);
    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    for (a, b) in &pairs {
        let da = a - mx;
        let db = b - my;
        num += da * db;
        dx += da * da;
        dy += db * db;
    }
    if dx == 0.0 || dy == 0.0 {
        return Err(Diagnostic::error(
            "Correlation is undefined for a constant column.",
            None,
        ));
    }
    Ok(num / (dx.sqrt() * dy.sqrt()))
}

/// Run the analysis, write results into the session workspace and push a
/// figure into the session graphics state.
pub fn analyze(
    session: &mut Session,
    path: &Path,
    selected: &[usize],
    plot: PlotRequest,
    cancel: Option<&AtomicBool>,
) -> Result<AnalysisOutcome, Diagnostic> {
    let table = parse_csv(path, cancel)?;
    if selected.is_empty() {
        return Err(Diagnostic::error(
            "Select at least one numeric column to analyze.",
            None,
        ));
    }
    for &col in selected {
        if col >= table.col_count {
            return Err(Diagnostic::error(
                format!("Column index {} is out of range.", col),
                None,
            ));
        }
    }
    let mut stats = Vec::with_capacity(selected.len());
    for &col in selected {
        stats.push(column_stats(&table, col, cancel)?);
    }

    if plot.x >= table.col_count || plot.y >= table.col_count {
        return Err(Diagnostic::error(
            "Plot column selection is out of range.",
            None,
        ));
    }
    if table.columns[plot.x].kind == ColumnKind::Text
        || table.columns[plot.y].kind == ColumnKind::Text
    {
        return Err(Diagnostic::error("Plot columns must be numeric.", None));
    }
    let corr = if selected.contains(&plot.x) && selected.contains(&plot.y) && plot.x != plot.y {
        Some(correlation(&table, plot.x, plot.y)?)
    } else {
        None
    };

    // Workspace: one row per CSV row, one column per selected numeric column.
    let selected_numeric: Vec<usize> = selected
        .iter()
        .copied()
        .filter(|&c| table.columns[c].kind == ColumnKind::Numeric)
        .collect();
    if !selected_numeric.is_empty() {
        let ncols = selected_numeric.len();
        // DenseMatrix stores row-major: element (r, c) lives at r * ncols + c.
        let mut data = vec![f64::NAN; table.row_count * ncols];
        for (r, row) in table.numeric.iter().enumerate() {
            for (c, &col) in selected_numeric.iter().enumerate() {
                data[r * ncols + c] = row[col].unwrap_or(f64::NAN);
            }
        }
        session.env.set(
            "data_analysis_data",
            Value::Matrix(DenseMatrix {
                rows: table.row_count,
                cols: ncols,
                data,
            }),
        );
        let mean_vec: Vec<f64> = stats.iter().map(|s| s.mean).collect();
        let median_vec: Vec<f64> = stats.iter().map(|s| s.median).collect();
        let std_vec: Vec<f64> = stats.iter().map(|s| s.std_dev).collect();
        let min_vec: Vec<f64> = stats.iter().map(|s| s.min).collect();
        let max_vec: Vec<f64> = stats.iter().map(|s| s.max).collect();
        let q25_vec: Vec<f64> = stats.iter().map(|s| s.q25).collect();
        let q50_vec: Vec<f64> = stats.iter().map(|s| s.q50).collect();
        let q75_vec: Vec<f64> = stats.iter().map(|s| s.q75).collect();
        let n = stats.len();
        for (name, values) in [
            ("data_analysis_mean", mean_vec),
            ("data_analysis_median", median_vec),
            ("data_analysis_std", std_vec),
            ("data_analysis_min", min_vec),
            ("data_analysis_max", max_vec),
            ("data_analysis_q25", q25_vec),
            ("data_analysis_q50", q50_vec),
            ("data_analysis_q75", q75_vec),
        ] {
            session.env.set(
                name,
                Value::Matrix(DenseMatrix {
                    rows: n,
                    cols: 1,
                    data: values,
                }),
            );
        }
    }

    // Figure: a new figure, not appended to an unrelated current figure.
    let handle = session
        .graphics
        .figures
        .iter()
        .map(|f| f.handle)
        .max()
        .unwrap_or(0)
        + 1;
    let mut figure = FigureState::fresh(handle);
    let x_name = table.columns[plot.x].name.clone();
    let y_name = table.columns[plot.y].name.clone();
    figure.title = Some(format!("Data Analysis — {}", y_name));
    figure.xlabel = Some(x_name.clone());
    figure.ylabel = Some(y_name.clone());
    figure.grid = true;
    let x_values: Vec<f64> = match plot.kind {
        PlotKind::Column => (1..=table.row_count).map(|i| i as f64).collect(),
        PlotKind::Scatter => table
            .numeric
            .iter()
            .map(|row| row[plot.x].unwrap_or(f64::NAN))
            .collect(),
    };
    let y_values: Vec<f64> = table
        .numeric
        .iter()
        .map(|row| row[plot.y].unwrap_or(f64::NAN))
        .collect();
    let item = match plot.kind {
        PlotKind::Column => PlotItem::Line {
            label: y_name.clone(),
            x: x_values,
            y: y_values,
        },
        PlotKind::Scatter => PlotItem::Scatter {
            label: format!("{} vs {}", y_name, x_name),
            x: x_values,
            y: y_values,
        },
    };
    figure.items.push(item);
    session.graphics.figures.push(figure);
    session.graphics.current = Some(session.graphics.figures.len() - 1);

    Ok(AnalysisOutcome {
        table,
        stats,
        correlation: corr,
        plotted: true,
    })
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Export statistics (and correlation when present) as a long-format CSV.
/// Refuses to overwrite an existing file unless `overwrite` is true.
pub fn export_stats_csv(
    path: &Path,
    stats: &[ColumnStats],
    correlation: Option<(&str, &str, f64)>,
    overwrite: bool,
) -> Result<(), Diagnostic> {
    if stats.is_empty() {
        return Err(Diagnostic::error("No statistics to export.", None));
    }
    if path.exists() && !overwrite {
        return Err(Diagnostic::error(
            format!(
                "Refusing to overwrite existing file {} without confirmation.",
                path.display()
            ),
            None,
        ));
    }
    let mut out = String::from("column,statistic,value\n");
    for s in stats {
        for (stat, value) in [
            ("count", s.count as f64),
            ("missing", s.missing as f64),
            ("mean", s.mean),
            ("median", s.median),
            ("std", s.std_dev),
            ("min", s.min),
            ("max", s.max),
            ("q25", s.q25),
            ("q50", s.q50),
            ("q75", s.q75),
        ] {
            out.push_str(&format!("{},{},{}\n", csv_escape(&s.name), stat, value));
        }
    }
    if let Some((x, y, r)) = correlation {
        out.push_str(&format!(
            "{},{},{}\n",
            csv_escape(&format!("{} vs {}", x, y)),
            "pearson",
            r
        ));
    }
    std::fs::write(path, out).map_err(|err| {
        Diagnostic::error(format!("Could not write {}: {}", path.display(), err), None)
    })
}
