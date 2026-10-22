//! Integration tests for the plotting builtins and graphics state.

use matlab_runtime::{run_source, PlotItem};

fn figures_of(source: &str) -> Vec<matlab_runtime::FigureState> {
    run_source(source).expect("source should execute").figures
}

#[test]
fn plot_one_and_two_args() {
    let figs = figures_of("y = [3 1 4]; plot(y);");
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].items.len(), 1);
    match &figs[0].items[0] {
        PlotItem::Line { label, x, y } => {
            assert_eq!(label, "data 1");
            assert_eq!(x, &[1.0, 2.0, 3.0]);
            assert_eq!(y, &[3.0, 1.0, 4.0]);
        }
        other => panic!("expected Line, got {:?}", other),
    }
}

#[test]
fn plot_column_vector_rows() {
    // Column vectors (Nx1) are also acceptable inputs.
    let figs = figures_of("v = [1; 2; 3]; plot(v, v.^2);");
    match &figs[0].items[0] {
        PlotItem::Line { x, y, .. } => {
            assert_eq!(x, &[1.0, 2.0, 3.0]);
            assert_eq!(y, &[1.0, 4.0, 9.0]);
        }
        other => panic!("expected Line, got {:?}", other),
    }
}

#[test]
fn plot_and_scatter_two_args() {
    let figs = figures_of("plot([0 1 2], [0 1 4]); scatter([0 1 2], [1 2 3]);");
    assert_eq!(figs[0].items.len(), 2);
    assert!(matches!(figs[0].items[0], PlotItem::Line { .. }));
    assert!(matches!(figs[0].items[1], PlotItem::Scatter { .. }));
}

#[test]
fn length_mismatch_is_an_error() {
    let result = run_source("plot([1 2], [1 2 3]);");
    let d = result.err().expect("should fail");
    assert!(
        d.message.contains("Vectors must be the same length"),
        "{}",
        d.message
    );
}

#[test]
fn non_vector_argument_is_an_error() {
    let result = run_source("plot([1 2; 3 4]);");
    let d = result.err().expect("should fail");
    assert!(d.message.starts_with("plot:"), "{}", d.message);
}

#[test]
fn figure_handles_and_current_selection() {
    let figs = figures_of("figure(); figure(3); plot([1 2], [3 4]);");
    assert_eq!(figs.len(), 2);
    assert_eq!(figs[0].handle, 1);
    assert_eq!(figs[1].handle, 3);
    assert_eq!(figs[1].items.len(), 1, "plot goes to figure 3");

    // Selecting an existing handle reuses it.
    let figs = figures_of("figure(1); figure(1); plot([1], [1]);");
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].items.len(), 1);
}

#[test]
fn titles_labels_grid_legend() {
    let figs = figures_of(
        "plot([1 2], [3 4]); scatter([1 2], [5 6]); \
         title('My Title'); xlabel('x'); ylabel('y'); \
         legend('line', 'points'); grid on;",
    );
    let fig = &figs[0];
    assert_eq!(fig.title.as_deref(), Some("My Title"));
    assert_eq!(fig.xlabel.as_deref(), Some("x"));
    assert_eq!(fig.ylabel.as_deref(), Some("y"));
    assert!(fig.grid);
    assert!(fig.legend_shown);
    assert_eq!(fig.items[0].label(), "line");
    assert_eq!(fig.items[1].label(), "points");

    // grid off
    let figs = figures_of("grid on; grid off;");
    assert!(!figs[0].grid);
}

#[test]
fn bar_and_histogram() {
    let figs = figures_of("bar([1 2 3]);");
    match &figs[0].items[0] {
        PlotItem::Bar {
            positions, values, ..
        } => {
            assert_eq!(positions, &[1.0, 2.0, 3.0]);
            assert_eq!(values, &[1.0, 2.0, 3.0]);
        }
        other => panic!("expected Bar, got {:?}", other),
    }

    let figs = figures_of("bar([1 2], [5 6]); histogram([1 2 3 3 4], 3);");
    assert_eq!(figs[0].items.len(), 2);
    match &figs[0].items[1] {
        PlotItem::Histogram { values, bins, .. } => {
            assert_eq!(values.len(), 5);
            assert_eq!(*bins, 3);
        }
        other => panic!("expected Histogram, got {:?}", other),
    }
}

#[test]
fn clf_and_close() {
    let figs = figures_of("figure(2); plot([1], [1]); clf; plot([2], [2]);");
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].items.len(), 1, "clf resets the current figure");
    assert_eq!(
        figs[0].items[0].label(),
        "data 1",
        "series numbering restarts after clf"
    );

    let figs = figures_of("figure(1); figure(2); close; figure(1); close all;");
    assert!(figs.is_empty());

    let figs = figures_of("figure(1); figure(2); close(2);");
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].handle, 1);
}

#[test]
fn plotting_produces_no_console_output() {
    let result = run_source("plot([1 2], [3 4]); title('t');");
    let r = result.unwrap();
    assert!(r.stdout.is_empty(), "stdout: {:?}", r.stdout);
    // The empty 'ans' records exist but carry an empty matrix.
    for out in &r.outputs {
        assert!(out
            .value
            .as_matrix()
            .map(|m| m.data.is_empty())
            .unwrap_or(false));
    }
}

#[test]
fn bad_graphics_arguments() {
    assert!(run_source("title(42);").is_err());
    assert!(run_source("grid(1);").is_err());
    assert!(run_source("figure(0);").is_err());
    assert!(run_source("histogram([1 2], 0);").is_err());
    assert!(
        run_source("legend('a','b');").is_err(),
        "legend with no series must fail"
    );
    assert!(run_source("plot('abc');").is_err());
}
