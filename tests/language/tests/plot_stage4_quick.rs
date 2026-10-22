use matlab_runtime::run_source;

#[test]
fn plot_single_line_pair() {
    let r = run_source("x = [1 2 3]; y = [1 4 9]; plot(x, y);").unwrap();
    assert!(!r.figures.is_empty());
    let item = &r.figures[0].items[0];
    match item {
        matlab_runtime::graphics::PlotItem::Line { x, y, .. } => {
            assert_eq!(x.len(), 3);
            assert_eq!(y.len(), 3);
        }
        _ => panic!("expected Line"),
    }
}

#[test]
fn plot_single_input_default_x() {
    let r = run_source("plot([10 20 30]);").unwrap();
    let item = &r.figures[0].items[0];
    match item {
        matlab_runtime::graphics::PlotItem::Line { x, y, .. } => {
            assert_eq!(y.len(), 3);
            assert!((x[0] - 1.0).abs() < 1e-6);
        }
        _ => panic!("expected Line"),
    }
}

#[test]
fn plot_mismatched_lengths_errors() {
    assert!(run_source("plot([1 2], [1 2 3])").is_err());
}

#[test]
fn plot_negative_and_constant() {
    let r = run_source("plot([-2, -1, 0], [-3, -3, -3]);").unwrap();
    assert!(!r.figures.is_empty());
}

#[test]
fn plot_scatter() {
    let r = run_source("scatter([1 2 3], [3 2 1]);").unwrap();
    assert!(!r.figures.is_empty());
    match &r.figures[0].items[0] {
        matlab_runtime::graphics::PlotItem::Scatter { .. } => {},
        _ => panic!("expected Scatter"),
    }
}

#[test]
fn plot_with_title_axis() {
    let r = run_source("plot([1 2],[2 4]); title('A'); xlabel('X'); ylabel('Y'); grid on;").unwrap();
    assert!(!r.figures.is_empty());
    let fig = &r.figures[0];
    assert!(fig.title.as_ref().unwrap().contains("A"));
    assert!(fig.grid);
}

#[test]
fn plot_empty_rejected_or_empty() {
    // Empty array: plot produces clear diagnostic rather than crash.
    assert!(run_source("plot([]);").is_err());
}

#[test]
fn plot_nan_in_sample() {
    // NaN literal works; stats remove NaN for percentile; plot just renders points
    let r = run_source("plot([1 NaN 3], [2 4 6]);").unwrap();
    assert!(!r.figures.is_empty());
}
