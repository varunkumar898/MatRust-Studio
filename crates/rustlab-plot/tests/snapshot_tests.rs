use rustlab_plot::Figure;
use rustlab_plot::Axes;
use rustlab_plot::to_svg;
use rustlab_plot::Series;
use rustlab_plot::Line;

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
    assert_eq!(
        rendered, expected,
        "snapshot mismatch for {name}.svg"
    );
}

#[test]
fn snapshot_single_line_plot() {
    let mut fig = Figure::new(1);
    fig.title = "Simple Line".to_string();
    let mut ax = Axes::new(1);
    ax.title = "y = 2x".to_string();
    ax.xlabel = "x".to_string();
    ax.ylabel = "y".to_string();
    ax.push_series(Series::Line(Line::new("2x", vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 2.0, 4.0, 6.0])));
    fig.add_axes(ax);
    assert_snapshot("single_line_plot", &fig, 400.0, 300.0);
}

#[test]
fn snapshot_scatter_plot() {
    let mut fig = Figure::new(2);
    fig.title = "Scatter Example".to_string();
    let mut ax = Axes::new(1);
    ax.push_series(Series::Line(Line::new("points", vec![1.0, 2.0, 3.0, 4.0], vec![1.0, 4.0, 9.0, 16.0])));
    fig.add_axes(ax);
    assert_snapshot("scatter_plot", &fig, 500.0, 400.0);
}

#[test]
fn snapshot_deterministic_across_runs() {
    let mut fig = Figure::new(3);
    let mut ax = Axes::new(1);
    ax.push_series(Series::Line(Line::new("s", vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 0.0])));
    fig.add_axes(ax);
    let a = to_svg(&fig, 300.0, 200.0);
    let b = to_svg(&fig, 300.0, 200.0);
    assert_eq!(a, b, "SVG output must be deterministic");
    // Also match the snapshot
    assert_snapshot("deterministic_line", &fig, 300.0, 200.0);
}
