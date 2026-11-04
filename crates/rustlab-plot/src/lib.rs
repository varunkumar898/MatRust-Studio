//! # `rustlab-plot`
//!
//! Renderer-independent MATLAB-compatible plot description model with a
//! deterministic SVG renderer. No GUI, no egui, no resvg — pure data + SVG
//! string output. Safe to use under `--no-default-features`.
//!
//! The model mirrors MATLAB's figure/axes semantics: a `Figure` owns one or
//! more `Axes`, each `Axes` holds `Series` (line, scatter, bar, histogram),
//! optional text annotations and a legend, plus axis limits and grid state.
//! A thread-local `PlotRegistry` tracks figures across builtin calls so that
//! `plot`, `hold`, `xlabel`, `saveas`, etc. operate on a "current figure"
//! just like MATLAB's `gcf`/`gca`.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(
    clippy::needless_range_loop,
    clippy::type_complexity,
    clippy::needless_borrow
)]

mod state;
mod svg;
mod ticks;

pub use state::{plot_state_reset, with_plot_state, with_plot_state_ref};
pub use svg::to_svg;
pub use ticks::nice_ticks;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single data series attached to an `Axes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Series {
    Line(Line),
    Scatter(Scatter),
    Bar(Bar),
    Histogram(Histogram),
}

/// A connected line plot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: [f32; 4],
    pub width: f32,
}

impl Line {
    pub fn new(name: impl Into<String>, x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            x,
            y,
            color: [0.0, 0.4470, 0.7410, 1.0], // MATLAB default blue
            width: 1.5,
        }
    }
}

/// A scatter plot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scatter {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: [f32; 4],
    pub size: f32,
}

impl Scatter {
    pub fn new(name: impl Into<String>, x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            x,
            y,
            color: [0.0, 0.4470, 0.7410, 1.0],
            size: 36.0,
        }
    }
}

/// A bar plot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: [f32; 4],
}

impl Bar {
    pub fn new(name: impl Into<String>, x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            x,
            y,
            color: [0.0, 0.4470, 0.7410, 1.0],
        }
    }
}

/// A histogram of values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Histogram {
    pub name: String,
    pub values: Vec<f64>,
    pub bins: usize,
    pub color: [f32; 4],
}

impl Histogram {
    pub fn new(name: impl Into<String>, values: Vec<f64>, bins: usize) -> Self {
        Self {
            name: name.into(),
            values,
            bins,
            color: [0.0, 0.4470, 0.7410, 1.0],
        }
    }
}

/// A text annotation placed at data coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub content: String,
}

/// Axis scale. Only `Linear` is implemented; `Log` is reserved for the future.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    Linear,
    /// Reserved — not yet implemented.
    #[allow(dead_code)]
    Log,
}

/// Legend placement within an axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendLocation {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// A legend describing the series in an axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Legend {
    pub location: LegendLocation,
}

/// One coordinate system holding series, labels, limits and grid state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Axes {
    pub id: usize,
    pub title: String,
    pub xlabel: String,
    pub ylabel: String,
    pub zlabel: String,
    pub xlim: Option<[f64; 2]>,
    pub ylim: Option<[f64; 2]>,
    pub xscale: Scale,
    pub yscale: Scale,
    pub grid: bool,
    pub hold: bool,
    pub series: Vec<Series>,
    pub texts: Vec<Text>,
    pub legend: Option<Legend>,
}

impl Axes {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: String::new(),
            xlabel: "x".to_string(),
            ylabel: "y".to_string(),
            zlabel: String::new(),
            xlim: None,
            ylim: None,
            xscale: Scale::Linear,
            yscale: Scale::Linear,
            grid: true,
            hold: false,
            series: Vec::new(),
            texts: Vec::new(),
            legend: None,
        }
    }

    /// Push a series, clearing existing series unless hold is on.
    pub fn push_series(&mut self, series: Series) {
        if !self.hold {
            self.series.clear();
        }
        self.series.push(series);
    }

    /// Data bounds across all series, or None when there is no data.
    pub fn data_bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        let mut xs: Vec<f64> = Vec::new();
        let mut ys: Vec<f64> = Vec::new();
        for s in &self.series {
            match s {
                Series::Line(l) => {
                    xs.extend(l.x.iter().copied());
                    ys.extend(l.y.iter().copied());
                }
                Series::Scatter(sct) => {
                    xs.extend(sct.x.iter().copied());
                    ys.extend(sct.y.iter().copied());
                }
                Series::Bar(b) => {
                    xs.extend(b.x.iter().copied());
                    ys.extend(b.y.iter().copied());
                }
                Series::Histogram(h) => {
                    ys.extend(h.values.iter().copied());
                }
            }
        }
        if xs.is_empty() && ys.is_empty() {
            return None;
        }
        let x_range = if xs.is_empty() {
            [0.0, 1.0]
        } else {
            [
                xs.iter().cloned().fold(f64::INFINITY, f64::min),
                xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ]
        };
        let y_range = if ys.is_empty() {
            [0.0, 1.0]
        } else {
            [
                ys.iter().cloned().fold(f64::INFINITY, f64::min),
                ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ]
        };
        Some((x_range, y_range))
    }
}

/// A figure window owning one or more axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Figure {
    pub id: usize,
    pub title: String,
    pub axes: Vec<Axes>,
    pub current_axes: Option<usize>,
    pub visible: bool,
}

impl Figure {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: format!("Figure {}", id),
            axes: Vec::new(),
            current_axes: None,
            visible: true,
        }
    }

    pub fn current_axes_mut(&mut self) -> Option<&mut Axes> {
        self.current_axes
            .and_then(|id| self.axes.iter_mut().find(|a| a.id == id))
    }

    pub fn current_axes(&self) -> Option<&Axes> {
        self.current_axes
            .and_then(|id| self.axes.iter().find(|a| a.id == id))
    }

    pub fn add_axes(&mut self, axes: Axes) {
        if self.axes.is_empty() {
            self.current_axes = Some(axes.id);
        }
        self.axes.push(axes);
    }
}

/// Registry of all figures plus the current figure. Thread-local state
/// mirrors MATLAB's `gcf`/`gca` within one interpreter session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotRegistry {
    pub figures: HashMap<usize, Figure>,
    pub current_id: Option<usize>,
    pub next_id: usize,
}

impl PlotRegistry {
    pub fn new() -> Self {
        Self {
            figures: HashMap::new(),
            current_id: None,
            next_id: 1,
        }
    }

    pub fn create_figure(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let fig = Figure::new(id);
        self.current_id = Some(id);
        self.figures.insert(id, fig);
        id
    }

    pub fn get_figure(&self, id: usize) -> Option<&Figure> {
        self.figures.get(&id)
    }

    pub fn get_figure_mut(&mut self, id: usize) -> Option<&mut Figure> {
        self.figures.get_mut(&id)
    }

    pub fn current_figure(&self) -> Option<&Figure> {
        self.current_id.and_then(|id| self.figures.get(&id))
    }

    pub fn current_figure_mut(&mut self) -> Option<&mut Figure> {
        self.current_id.and_then(|id| self.figures.get_mut(&id))
    }

    pub fn set_current(&mut self, id: usize) -> bool {
        if self.figures.contains_key(&id) {
            self.current_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn figure_ids(&self) -> Vec<usize> {
        let mut ids: Vec<usize> = self.figures.keys().copied().collect();
        ids.sort();
        ids
    }
}

impl Default for PlotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figure_creation_and_switching() {
        let mut reg = PlotRegistry::new();
        let id1 = reg.create_figure();
        let id2 = reg.create_figure();
        assert_ne!(id1, id2);
        assert_eq!(reg.current_id, Some(id2));
        assert!(reg.set_current(id1));
        assert_eq!(reg.current_id, Some(id1));
        assert!(!reg.set_current(99999));
    }

    #[test]
    fn axes_data_bounds_empty() {
        let mut reg = PlotRegistry::new();
        reg.create_figure();
        let fig = reg.current_figure_mut().unwrap();
        let ax = Axes::new(1);
        fig.add_axes(ax);
        assert!(fig.current_axes().unwrap().data_bounds().is_none());
    }

    #[test]
    fn axes_data_bounds_line() {
        let mut ax = Axes::new(1);
        ax.push_series(Series::Line(Line::new(
            "l",
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        )));
        let (xr, yr) = ax.data_bounds().unwrap();
        assert!((xr[0] - 1.0).abs() < 1e-12 && (xr[1] - 3.0).abs() < 1e-12);
        assert!((yr[0] - 4.0).abs() < 1e-12 && (yr[1] - 6.0).abs() < 1e-12);
    }

    #[test]
    fn hold_clears_series_when_off() {
        let mut ax = Axes::new(1);
        ax.push_series(Series::Line(Line::new("a", vec![1.0], vec![2.0])));
        ax.hold = false;
        ax.push_series(Series::Line(Line::new("b", vec![3.0], vec![4.0])));
        assert_eq!(ax.series.len(), 1);
        assert_eq!(series_name(&ax.series[0]), "b");
    }

    #[test]
    fn hold_keeps_series_when_on() {
        let mut ax = Axes::new(1);
        ax.hold = true;
        ax.push_series(Series::Line(Line::new("a", vec![1.0], vec![2.0])));
        ax.push_series(Series::Line(Line::new("b", vec![3.0], vec![4.0])));
        assert_eq!(ax.series.len(), 2);
    }

    #[test]
    fn hold_off_returns_to_replace() {
        // With hold on, two plots append; switching hold off makes the next
        // plot replace the series again (MATLAB hold-off semantics).
        let mut ax = Axes::new(1);
        ax.hold = true;
        ax.push_series(Series::Line(Line::new("a", vec![1.0], vec![2.0])));
        ax.push_series(Series::Line(Line::new("b", vec![3.0], vec![4.0])));
        assert_eq!(ax.series.len(), 2);
        ax.hold = false;
        ax.push_series(Series::Line(Line::new("c", vec![5.0], vec![6.0])));
        assert_eq!(ax.series.len(), 1);
        assert_eq!(series_name(&ax.series[0]), "c");
    }

    #[test]
    fn hold_is_independent_across_axes() {
        // Hold on one axes must not affect a sibling axes (per-axes state,
        // as required for subplot grouping).
        let mut fig = Figure::new(1);
        let mut ax1 = Axes::new(1);
        ax1.hold = true;
        ax1.push_series(Series::Line(Line::new("a", vec![1.0], vec![2.0])));
        ax1.push_series(Series::Line(Line::new("b", vec![3.0], vec![4.0])));
        fig.add_axes(ax1);
        let mut ax2 = Axes::new(2);
        ax2.hold = false;
        ax2.push_series(Series::Line(Line::new("x", vec![1.0], vec![2.0])));
        ax2.push_series(Series::Line(Line::new("y", vec![3.0], vec![4.0])));
        fig.add_axes(ax2);
        let ax1 = fig.axes.iter().find(|a| a.id == 1).unwrap();
        let ax2 = fig.axes.iter().find(|a| a.id == 2).unwrap();
        assert_eq!(ax1.series.len(), 2, "held axes keeps both series");
        assert_eq!(ax2.series.len(), 1, "non-held axes replaced with last");
    }

    #[test]
    fn current_figure_and_axes_tracking() {
        let mut reg = PlotRegistry::new();
        let f1 = reg.create_figure();
        let f2 = reg.create_figure();
        assert_eq!(reg.current_id, Some(f2)); // last created is current
        assert!(reg.set_current(f1));
        assert_eq!(reg.current_id, Some(f1));
        let fig = reg.current_figure_mut().unwrap();
        assert_eq!(fig.id, f1);
        // Adding the first axes makes it current.
        fig.add_axes(Axes::new(1));
        assert_eq!(fig.current_axes().unwrap().id, 1);
    }

    fn series_name(s: &Series) -> String {
        match s {
            Series::Line(l) => l.name.clone(),
            Series::Scatter(s) => s.name.clone(),
            Series::Bar(b) => b.name.clone(),
            Series::Histogram(h) => h.name.clone(),
        }
    }
}
