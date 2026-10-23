//! Data Analysis backend and integration tests.
//!
//! Every assertion compares a concrete value, a mathematical identity, or an
//! exported file's contents.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};

use matlabr_apps::data_analysis::{self, ColumnKind, PlotKind, PlotRequest};
use matlabr_apps::{find_by_id, Status, Verification, REGISTRY};

fn temp_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "matrust_data_analysis_{}_{}_{}.csv",
        label,
        std::process::id(),
        nonce
    ))
}

fn write_fixture(label: &str, contents: &str) -> PathBuf {
    let path = temp_path(label);
    std::fs::write(&path, contents).expect("write fixture");
    path
}

#[test]
fn registry_marks_app2_ready_after_ui() {
    assert_eq!(REGISTRY.len(), 8);
    assert_eq!(REGISTRY[0].id, "data_analysis");
    assert_eq!(REGISTRY[0].status, Status::Ready);
    assert_eq!(REGISTRY[0].verification, Verification::Unit);
    assert_eq!(REGISTRY[1].id, "curve_fitter");
    // Curve Fitter became Ready in the same ordering slot once its UI was
    // wired; it is still Unit-verified and not reference-verified.
    assert_eq!(REGISTRY[1].status, Status::Ready);
    assert_eq!(REGISTRY[1].verification, Verification::Unit);
    assert_eq!(REGISTRY[2].id, "simulation");
    assert_eq!(REGISTRY[2].status, Status::Ready);
    assert_eq!(REGISTRY[2].verification, Verification::Unit);
    for app in &REGISTRY[3..] {
        assert_eq!(app.status, Status::Planned, "{} must be Planned", app.name);
        assert!(!app.launchable(), "{} must be disabled", app.name);
    }

    assert_eq!(
        find_by_id("curve_fitter").map(|a| a.name),
        Some("Curve Fitter")
    );
    assert_eq!(find_by_id("unknown-app"), None);
}

#[test]
fn parses_header_quoted_fields_and_text_columns() {
    let path = write_fixture(
        "quoted",
        "name,score\n\"Smith, John\",10\n\"A \"\"B\"\"\",20\nCarol,30\n",
    );
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    assert_eq!(table.delimiter, ',');
    assert!(table.has_header);
    assert_eq!(table.col_count, 2);
    assert_eq!(table.row_count, 3);
    assert_eq!(table.columns[0].name, "name");
    assert_eq!(table.columns[0].kind, ColumnKind::Text);
    assert_eq!(table.columns[1].name, "score");
    assert_eq!(table.columns[1].kind, ColumnKind::Numeric);
    assert_eq!(table.rows[0][0], "Smith, John");
    assert_eq!(table.rows[1][0], "A \"B\"");
    assert_eq!(table.numeric[2][1], Some(30.0));
    assert!(!table.ragged);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn detects_no_header_and_numeric_columns() {
    let path = write_fixture("no_header", "1,2\n3,4\n5,6\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    assert!(!table.has_header);
    assert_eq!(table.columns[0].name, "col1");
    assert_eq!(table.columns[1].name, "col2");
    assert_eq!(table.numeric[0], vec![Some(1.0), Some(2.0)]);
    assert_eq!(table.numeric[2], vec![Some(5.0), Some(6.0)]);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn counts_missing_values_without_counting_text_as_missing() {
    let path = write_fixture("missing", "name,x,y\nA,1,\nB,,4\nC,3,6\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    assert_eq!(table.columns[0].kind, ColumnKind::Text);
    assert_eq!(table.columns[0].missing, 0);
    assert_eq!(table.columns[1].missing, 1);
    assert_eq!(table.columns[2].missing, 1);
    assert_eq!(table.numeric[0], vec![None, Some(1.0), None]);
    assert_eq!(table.numeric[1], vec![None, None, Some(4.0)]);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn flags_ragged_rows_and_pads_columns() {
    let path = write_fixture("ragged", "a,b\n1,2\n3,4,5,6\n7\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    assert!(table.ragged);
    assert_eq!(table.col_count, 4);
    assert_eq!(table.columns.len(), 4);
    assert_eq!(table.columns[3].name, "col4");
    assert_eq!(table.rows[1].len(), 4);
    assert_eq!(table.rows[1][3], "6");
    assert_eq!(table.rows[2][1], "");
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn column_statistics_match_hand_computed_values() {
    let path = write_fixture("stats", "v\n1\n2\n3\n6\n10\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let stats = data_analysis::column_stats(&table, 0, None).expect("stats");
    assert_eq!(stats.name, "v");
    assert_eq!(stats.count, 5);
    assert_eq!(stats.missing, 0);
    assert!((stats.mean - 4.4).abs() < 1e-12);
    assert!((stats.median - 3.0).abs() < 1e-12);
    assert!((stats.std_dev - 13.3_f64.sqrt()).abs() < 1e-12);
    assert_eq!(stats.min, 1.0);
    assert_eq!(stats.max, 10.0);
    assert!((stats.q25 - 1.75).abs() < 1e-12);
    assert!((stats.q50 - 3.0).abs() < 1e-12);
    assert!((stats.q75 - 7.0).abs() < 1e-12);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn statistics_skip_missing_values() {
    let path = write_fixture("stats_missing", "v,label\n1,a\n,b\n3,c\n,d\n5,e\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let stats = data_analysis::column_stats(&table, 0, None).expect("stats");
    assert_eq!(stats.count, 3);
    assert_eq!(stats.missing, 2);
    assert!((stats.mean - 3.0).abs() < 1e-12);
    assert!((stats.std_dev - 2.0).abs() < 1e-12);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn pearson_correlation_is_exact_for_linear_data() {
    let path = write_fixture("corr", "x,y,z\n1,2,6\n2,4,4\n3,6,2\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    assert!((data_analysis::correlation(&table, 0, 1).unwrap() - 1.0).abs() < 1e-12);
    assert!((data_analysis::correlation(&table, 0, 2).unwrap() + 1.0).abs() < 1e-12);
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn text_statistics_produce_a_clear_error() {
    let path = write_fixture("text_error", "name,value\nA,1\nB,2\n");
    let table = data_analysis::parse_csv(&path, None).expect("parse");
    let err = data_analysis::column_stats(&table, 0, None).expect_err("text has no stats");
    assert_eq!(
        err.message,
        "Column 'name' contains text and has no numeric statistics."
    );
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn export_round_trip_and_overwrite_refusal() {
    let csv = write_fixture("export_source", "v\n1\n2\n3\n");
    let table = data_analysis::parse_csv(&csv, None).expect("parse");
    let stats = data_analysis::column_stats(&table, 0, None).expect("stats");
    let out = temp_path("export_out");

    data_analysis::export_stats_csv(
        &out,
        std::slice::from_ref(&stats),
        Some(("v", "v", 1.0)),
        false,
    )
    .expect("export creates the file");
    let text = std::fs::read_to_string(&out).expect("read export");
    assert!(text.starts_with("column,statistic,value\n"));
    assert!(text.contains("v,count,3\n"));
    assert!(text.contains("v,mean,2\n"));
    assert!(text.contains("v,std,1\n"));
    assert!(text.contains("v vs v,pearson,1\n"));

    // Existing file must be protected unless overwrite is explicitly requested.
    let refused = data_analysis::export_stats_csv(&out, std::slice::from_ref(&stats), None, false)
        .expect_err("overwrite must be refused");
    assert!(refused
        .message
        .starts_with("Refusing to overwrite existing file"));

    data_analysis::export_stats_csv(&out, std::slice::from_ref(&stats), None, true)
        .expect("explicit overwrite");
    std::fs::remove_file(&out).expect("cleanup export");
    std::fs::remove_file(&csv).expect("cleanup fixture");
}

#[test]
fn analyze_writes_workspace_variables_and_a_figure() {
    let path = write_fixture("analyze", "x,y\n1,2\n2,4\n3,6\n");
    let mut session = matlab_runtime::Session::new();
    let outcome = data_analysis::analyze(
        &mut session,
        &path,
        &[0, 1],
        PlotRequest {
            kind: PlotKind::Scatter,
            x: 0,
            y: 1,
        },
        None,
    )
    .expect("analysis");

    assert_eq!(outcome.stats.len(), 2);
    let corr = outcome.correlation.expect("correlation computed");
    assert!((corr - 1.0).abs() < 1e-12, "corr = {corr}");
    let data = session
        .env
        .get("data_analysis_data")
        .and_then(|v| v.as_matrix())
        .expect("workspace matrix");
    assert_eq!((data.rows, data.cols), (3, 2));
    // Row-major storage of the 3x2 table [[1, 2], [2, 4], [3, 6]].
    assert_eq!(data.data, vec![1.0, 2.0, 2.0, 4.0, 3.0, 6.0]);
    let means = session
        .env
        .get("data_analysis_mean")
        .and_then(|v| v.as_matrix())
        .expect("mean vector");
    assert_eq!(means.data, vec![2.0, 4.0]);

    let figure = session.graphics.figures.last().expect("figure pushed");
    assert_eq!(figure.items.len(), 1);
    match &figure.items[0] {
        matlab_runtime::PlotItem::Scatter { x, y, .. } => {
            assert_eq!(x, &vec![1.0, 2.0, 3.0]);
            assert_eq!(y, &vec![2.0, 4.0, 6.0]);
        }
        other => panic!("expected scatter, got {:?}", other),
    }
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn integration_analysis_then_export_round_trip() {
    let source = write_fixture("integration", "x,y\n1,2\n2,4\n3,6\n4,8\n");
    let export = temp_path("integration_out");
    let mut session = matlab_runtime::Session::new();
    let outcome = data_analysis::analyze(
        &mut session,
        &source,
        &[0, 1],
        PlotRequest {
            kind: PlotKind::Column,
            x: 1,
            y: 1,
        },
        None,
    )
    .expect("analysis");
    assert_eq!(outcome.stats[0].mean, 2.5);
    assert_eq!(outcome.stats[1].mean, 5.0);
    assert_eq!(session.graphics.figures.len(), 1);
    data_analysis::export_stats_csv(
        &export,
        &outcome.stats,
        outcome.correlation.map(|r| ("x", "y", r)),
        false,
    )
    .expect("export");

    let text = std::fs::read_to_string(&export).expect("read export");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "column,statistic,value");
    assert!(
        lines.contains(&"x,mean,2.5"),
        "exported mean missing: {lines:?}"
    );
    assert!(
        lines.contains(&"y,mean,5"),
        "exported mean missing: {lines:?}"
    );
    std::fs::remove_file(&export).expect("cleanup export");
    std::fs::remove_file(&source).expect("cleanup fixture");
}

#[test]
fn cancellation_is_reported_before_work() {
    let path = write_fixture("cancel", "v\n1\n2\n3\n");
    let cancel = AtomicBool::new(true);
    let err = data_analysis::parse_csv(&path, Some(&cancel)).expect_err("must cancel");
    assert_eq!(err.message, "Data Analysis cancelled.");
    std::fs::remove_file(&path).expect("cleanup");
}

#[test]
fn missing_file_reports_the_path() {
    let path = Path::new("/nonexistent/matrust_data_analysis_does_not_exist.csv");
    let err = data_analysis::parse_csv(path, None).expect_err("missing file");
    assert!(err.message.contains("Could not read"));
    assert!(err.message.contains("matrust_data_analysis_does_not_exist"));
}
