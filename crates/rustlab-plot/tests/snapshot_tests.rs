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

#[test]
fn test_independent_axes_properties_different_limits_and_scale() {
    let mut fig = Figure::new(201);
    fig.title = "Independent Axes Properties".to_string();

    let ax1 = fig.subplot(1, 2, 1);
    ax1.set_xlim(0.0, 10.0);
    ax1.set_ylim(-5.0, 5.0);
    ax1.set_xscale(rustlab_plot::Scale::Linear);
    ax1.set_yscale(rustlab_plot::Scale::Linear);
    ax1.set_xticks(vec![0.0, 5.0, 10.0]);
    ax1.set_xticklabels(vec![
        "start".to_string(),
        "mid".to_string(),
        "end".to_string(),
    ]);
    ax1.set_title("Linear Time Domain");
    ax1.set_xlabel("Time (s)");
    ax1.set_ylabel("Amplitude");
    ax1.set_grid(true);
    ax1.set_legend(rustlab_plot::LegendLocation::NorthEast);
    ax1.push_series(Series::Line(Line::new(
        "signal",
        vec![0.0, 5.0, 10.0],
        vec![0.0, 4.0, -2.0],
    )));

    let ax2 = fig.subplot(1, 2, 2);
    ax2.set_xlim(1.0, 1000.0);
    ax2.set_ylim(0.1, 100.0);
    ax2.set_xscale(rustlab_plot::Scale::Log);
    ax2.set_yscale(rustlab_plot::Scale::Log);
    ax2.set_xticks(vec![1.0, 10.0, 100.0, 1000.0]);
    ax2.set_xticklabels(vec!["1".into(), "10".into(), "100".into(), "1000".into()]);
    ax2.set_title("Log Frequency Domain");
    ax2.set_xlabel("Frequency (Hz)");
    ax2.set_ylabel("Magnitude");
    ax2.set_grid(false);
    ax2.set_legend(rustlab_plot::LegendLocation::SouthWest);
    ax2.push_series(Series::Line(Line::new(
        "response",
        vec![1.0, 10.0, 100.0, 1000.0],
        vec![100.0, 10.0, 1.0, 0.1],
    )));

    // Verify distinct independent properties
    let ax1_ref = fig.axes.iter().find(|a| a.id == 1).expect("ax1");
    let ax2_ref = fig.axes.iter().find(|a| a.id == 2).expect("ax2");

    assert_eq!(ax1_ref.xlim, Some([0.0, 10.0]));
    assert_eq!(ax2_ref.xlim, Some([1.0, 1000.0]));
    assert_eq!(ax1_ref.ylim, Some([-5.0, 5.0]));
    assert_eq!(ax2_ref.ylim, Some([0.1, 100.0]));
    assert_eq!(ax1_ref.xscale, rustlab_plot::Scale::Linear);
    assert_eq!(ax2_ref.xscale, rustlab_plot::Scale::Log);
    assert_eq!(ax1_ref.yscale, rustlab_plot::Scale::Linear);
    assert_eq!(ax2_ref.yscale, rustlab_plot::Scale::Log);
    assert!(ax1_ref.grid);
    assert!(!ax2_ref.grid);
    assert_eq!(ax1_ref.title, "Linear Time Domain");
    assert_eq!(ax2_ref.title, "Log Frequency Domain");
    assert_eq!(
        ax1_ref.legend.as_ref().map(|l| l.location),
        Some(rustlab_plot::LegendLocation::NorthEast)
    );
    assert_eq!(
        ax2_ref.legend.as_ref().map(|l| l.location),
        Some(rustlab_plot::LegendLocation::SouthWest)
    );

    let svg = to_svg(&fig, 800.0, 400.0);
    assert!(svg.contains("Linear Time Domain"));
    assert!(svg.contains("Log Frequency Domain"));
    assert!(svg.contains("legend"));
}

#[test]
fn test_axes_serialization_roundtrip_preserves_independent_state() {
    let mut fig = Figure::new(202);
    let ax1 = fig.subplot(1, 2, 1);
    ax1.set_xlim(0.0, 50.0);
    ax1.set_ylim(-1.0, 1.0);
    ax1.set_xscale(rustlab_plot::Scale::Linear);
    ax1.set_xticks(vec![0.0, 25.0, 50.0]);
    ax1.set_xticklabels(vec!["A".into(), "B".into(), "C".into()]);
    ax1.set_title("Plot A");
    ax1.set_grid(true);

    let ax2 = fig.subplot(1, 2, 2);
    ax2.set_xlim(1.0, 100.0);
    ax2.set_ylim(10.0, 1000.0);
    ax2.set_xscale(rustlab_plot::Scale::Log);
    ax2.set_yscale(rustlab_plot::Scale::Log);
    ax2.set_title("Plot B");
    ax2.set_grid(false);
    ax2.set_legend(rustlab_plot::LegendLocation::NorthWest);

    let serialized = serde_json::to_string(&fig).expect("serialize figure to JSON");
    let deserialized: Figure =
        serde_json::from_str(&serialized).expect("deserialize figure from JSON");

    assert_eq!(fig, deserialized);

    let d_ax1 = &deserialized.axes[0];
    let d_ax2 = &deserialized.axes[1];

    assert_eq!(d_ax1.xlim, Some([0.0, 50.0]));
    assert_eq!(d_ax2.xlim, Some([1.0, 100.0]));
    assert_eq!(d_ax1.xscale, rustlab_plot::Scale::Linear);
    assert_eq!(d_ax2.xscale, rustlab_plot::Scale::Log);
    assert!(d_ax1.grid);
    assert!(!d_ax2.grid);
    assert_eq!(d_ax1.title, "Plot A");
    assert_eq!(d_ax2.title, "Plot B");
    assert_eq!(
        d_ax2.legend.as_ref().map(|l| l.location),
        Some(rustlab_plot::LegendLocation::NorthWest)
    );
}

#[test]
fn test_axes_regression_no_crosstalk() {
    let mut fig = Figure::new(203);
    let _ax1 = fig.subplot(1, 2, 1);
    let ax2 = fig.subplot(1, 2, 2);

    // Baseline config for ax2
    ax2.set_xlim(5.0, 15.0);
    ax2.set_ylim(20.0, 30.0);
    ax2.set_xscale(rustlab_plot::Scale::Linear);
    ax2.set_grid(false);
    ax2.set_title("Unchanged Subplot 2");
    ax2.set_xlabel("X2");
    ax2.set_ylabel("Y2");

    let ax2_snapshot = fig.axes[1].clone();

    // Now aggressively mutate ax1
    let ax1 = &mut fig.axes[0];
    ax1.set_xlim(-100.0, 100.0);
    ax1.set_ylim(-200.0, 200.0);
    ax1.set_xscale(rustlab_plot::Scale::Log);
    ax1.set_yscale(rustlab_plot::Scale::Log);
    ax1.set_xticks(vec![1.0, 10.0]);
    ax1.set_xticklabels(vec!["one".into(), "ten".into()]);
    ax1.set_yticks(vec![2.0, 20.0]);
    ax1.set_yticklabels(vec!["two".into(), "twenty".into()]);
    ax1.set_grid(true);
    ax1.set_title("Mutated Subplot 1");
    ax1.set_xlabel("Mutated X1");
    ax1.set_ylabel("Mutated Y1");
    ax1.set_legend(rustlab_plot::LegendLocation::SouthEast);
    ax1.hold = true;

    // Verify ax2 is completely untouched
    assert_eq!(
        fig.axes[1], ax2_snapshot,
        "Mutations on ax1 caused crosstalk in ax2"
    );

    // Now aggressively mutate ax2
    let ax1_snapshot = fig.axes[0].clone();
    let ax2 = &mut fig.axes[1];
    ax2.set_xlim(500.0, 1500.0);
    ax2.set_title("Mutated Subplot 2");
    ax2.set_legend(rustlab_plot::LegendLocation::NorthEast);

    // Verify ax1 is completely untouched
    assert_eq!(
        fig.axes[0], ax1_snapshot,
        "Mutations on ax2 caused crosstalk in ax1"
    );
}

#[test]
fn test_shared_axes_synchronization_and_unlinking() {
    let mut fig = Figure::new(204);
    let ax1 = fig.subplot(2, 1, 1);
    ax1.push_series(Series::Line(Line::new(
        "s1",
        vec![0.0, 5.0],
        vec![10.0, 20.0],
    )));

    let ax2 = fig.subplot(2, 1, 2);
    ax2.push_series(Series::Line(Line::new(
        "s2",
        vec![2.0, 10.0],
        vec![5.0, 15.0],
    )));

    // Link X axes across subplots
    fig.link_axes(&[1, 2], "x");
    assert_eq!(fig.axes[0].shared_x_with, Some(1));
    assert_eq!(fig.axes[1].shared_x_with, Some(1));

    // Both subplots render without error and synchronize
    let svg = to_svg(&fig, 600.0, 600.0);
    assert!(svg.contains(r#"id="axes-1""#));
    assert!(svg.contains(r#"id="axes-2""#));

    // Unlink axes
    fig.unlink_axes(&[1, 2]);
    assert_eq!(fig.axes[0].shared_x_with, None);
    assert_eq!(fig.axes[1].shared_x_with, None);

    // Direct axes linking helper test
    fig.axes[1].link_x_with(1);
    fig.axes[1].link_y_with(1);
    assert_eq!(fig.axes[1].shared_x_with, Some(1));
    assert_eq!(fig.axes[1].shared_y_with, Some(1));

    fig.axes[1].unlink();
    assert_eq!(fig.axes[1].shared_x_with, None);
    assert_eq!(fig.axes[1].shared_y_with, None);
}
