use rustlab_plot::to_svg;
use rustlab_plot::Axes;
use rustlab_plot::Figure;
use rustlab_plot::Line;
use rustlab_plot::Series;

/// Load a reference SVG snapshot from the snapshots directory.
fn reference(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.svg"));
    let msg = format!("missing snapshot {path:?}");
    std::fs::read_to_string(&path).expect(&msg)
}

/// Render a figure and compare byte-for-byte against a stored snapshot.
fn assert_snapshot(name: &str, fig: &Figure, width: f64, height: f64) {
    let rendered = to_svg(fig, width, height);
    let expected = reference(name);
    assert_eq!(rendered, expected, "snapshot mismatch for {name}.svg");
}

#[test]
fn snapshot_single_line_plot() {
    let mut fig = Figure::new(1);
    fig.title = "Simple Line".to_string();
    let mut ax = Axes::new(1);
    ax.title = "y = 2x".to_string();
    ax.xlabel = "x".to_string();
    ax.ylabel = "y".to_string();
    ax.push_series(Series::Line(Line::new(
        "2x",
        vec![0.0, 1.0, 2.0, 3.0],
        vec![0.0, 2.0, 4.0, 6.0],
    )));
    fig.add_axes(ax);
    assert_snapshot("single_line_plot", &fig, 400.0, 300.0);
}

#[test]
fn snapshot_scatter_plot() {
    let mut fig = Figure::new(2);
    fig.title = "Scatter Example".to_string();
    let mut ax = Axes::new(1);
    ax.push_series(Series::Line(Line::new(
        "points",
        vec![1.0, 2.0, 3.0, 4.0],
        vec![1.0, 4.0, 9.0, 16.0],
    )));
    fig.add_axes(ax);
    assert_snapshot("scatter_plot", &fig, 500.0, 400.0);
}

#[test]
fn snapshot_deterministic_across_runs() {
    let mut fig = Figure::new(3);
    let mut ax = Axes::new(1);
    ax.push_series(Series::Line(Line::new(
        "s",
        vec![0.0, 1.0, 2.0],
        vec![0.0, 1.0, 0.0],
    )));
    fig.add_axes(ax);
    let a = to_svg(&fig, 300.0, 200.0);
    let b = to_svg(&fig, 300.0, 200.0);
    assert_eq!(a, b, "SVG output must be deterministic");
    // Also match the snapshot
    assert_snapshot("deterministic_line", &fig, 300.0, 200.0);
}

#[test]
fn test_subplot_223_places_axes_in_bottom_left_cell() {
    let mut fig = Figure::new(10);
    let ax = fig.subplot(2, 2, 3);
    let pos = ax.subplot_pos.as_ref().expect("subplot pos set");
    let (min_c, max_c, min_r, max_r) = pos.grid_bounds();
    assert_eq!(
        (min_c, max_c, min_r, max_r),
        (0, 0, 1, 1),
        "cell 3 is col 0, row 1 (bottom-left)"
    );
    let (x0, y0, w, h) = pos.compute_rect(60.0, 60.0, 1170.0, 690.0, 60.0, 60.0);
    assert!((x0 - 60.0).abs() < 1e-6);
    assert!((y0 - 435.0).abs() < 1e-6);
    assert!((w - 555.0).abs() < 1e-6);
    assert!((h - 315.0).abs() < 1e-6);
}

#[test]
fn test_empty_subplot_no_bleeding_or_collapsing() {
    let mut fig = Figure::new(11);
    fig.title = "Empty Subplot Layout".to_string();
    let ax1 = fig.subplot(2, 1, 1);
    ax1.push_series(Series::Line(Line::new(
        "s1",
        vec![0.0, 1.0],
        vec![0.0, 1.0],
    )));
    let _ax2 = fig.subplot(2, 1, 2); // empty
    let svg = to_svg(&fig, 1280.0, 800.0);
    assert!(svg.contains(r#"id="axes-1""#));
    assert!(svg.contains(r#"id="axes-2""#));
    assert!(svg.contains(r#"width="1170.00" height="315.00""#));
}

#[test]
fn snapshot_subplot_1x1_at_1280x800() {
    let mut fig = Figure::new(101);
    fig.title = "Subplot 1x1".to_string();
    let ax = fig.subplot(1, 1, 1);
    ax.push_series(Series::Line(Line::new(
        "s",
        vec![0.0, 1.0, 2.0, 3.0],
        vec![0.0, 3.0, 2.0, 5.0],
    )));
    assert_snapshot("subplot_1x1", &fig, 1280.0, 800.0);
}

#[test]
fn snapshot_subplot_2x1_at_1280x800() {
    let mut fig = Figure::new(102);
    fig.title = "Subplot 2x1 Grid".to_string();
    let ax1 = fig.subplot(2, 1, 1);
    ax1.push_series(Series::Line(Line::new(
        "top",
        vec![0.0, 1.0, 2.0],
        vec![1.0, 4.0, 9.0],
    )));
    let ax2 = fig.subplot(2, 1, 2);
    ax2.push_series(Series::Line(Line::new(
        "bottom",
        vec![0.0, 1.0, 2.0],
        vec![9.0, 4.0, 1.0],
    )));
    assert_snapshot("subplot_2x1", &fig, 1280.0, 800.0);
}

#[test]
fn snapshot_subplot_2x2_at_1280x800() {
    let mut fig = Figure::new(103);
    fig.title = "Subplot 2x2 Grid".to_string();
    let ax1 = fig.subplot(2, 2, 1);
    ax1.push_series(Series::Line(Line::new(
        "s1",
        vec![0.0, 1.0],
        vec![1.0, 2.0],
    )));
    let ax2 = fig.subplot(2, 2, 2);
    ax2.push_series(Series::Scatter(rustlab_plot::Scatter::new(
        "s2",
        vec![0.0, 1.0, 2.0],
        vec![2.0, 0.0, 1.0],
    )));
    let ax3 = fig.subplot(2, 2, 3);
    ax3.push_series(Series::Bar(rustlab_plot::Bar::new(
        "s3",
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
    )));
    let ax4 = fig.subplot(2, 2, 4);
    ax4.push_series(Series::Line(Line::new(
        "s4",
        vec![0.0, 1.0],
        vec![3.0, 0.0],
    )));
    assert_snapshot("subplot_2x2", &fig, 1280.0, 800.0);
}

#[test]
fn snapshot_subplot_spanning_at_1280x800() {
    let mut fig = Figure::new(104);
    fig.title = "Subplot Spanning Layout".to_string();
    let ax_top = fig.subplot_spanning(2, 2, vec![1, 2]);
    ax_top.push_series(Series::Line(Line::new(
        "wide",
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        vec![0.0, 2.0, 1.0, 4.0, 3.0],
    )));
    let ax_bl = fig.subplot(2, 2, 3);
    ax_bl.push_series(Series::Bar(rustlab_plot::Bar::new(
        "b",
        vec![1.0, 2.0],
        vec![3.0, 5.0],
    )));
    let ax_br = fig.subplot(2, 2, 4);
    ax_br.push_series(Series::Scatter(rustlab_plot::Scatter::new(
        "sc",
        vec![0.0, 1.0],
        vec![1.0, 2.0],
    )));
    assert_snapshot("subplot_spanning", &fig, 1280.0, 800.0);
}

#[test]
fn snapshot_subplot_empty_at_1280x800() {
    let mut fig = Figure::new(105);
    fig.title = "Subplot Empty Cell".to_string();
    let ax1 = fig.subplot(2, 1, 1);
    ax1.push_series(Series::Line(Line::new(
        "data",
        vec![0.0, 1.0, 2.0],
        vec![2.0, 4.0, 1.0],
    )));
    let _ax2 = fig.subplot(2, 1, 2); // empty subplot
    assert_snapshot("subplot_empty", &fig, 1280.0, 800.0);
}
