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
pub use svg::{axis_limits, resolve_limits, to_svg};
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
    Surface(Surface),
    Line3D(Line3D),
}

/// A 3D line plot (plot3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line3D {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub color: [f32; 4],
    pub width: f32,
}

impl Line3D {
    pub fn new(name: impl Into<String>, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            x,
            y,
            z,
            color: [0.0, 0.4470, 0.7410, 1.0], // MATLAB default blue
            width: 1.5,
        }
    }
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

/// A surface (mesh) plot over an N×M grid of (x, y) sample points with Z
/// heights. `x`, `y`, `z` each contain `rows * cols` values in column-major
/// order: element `(i, j)` is stored at index `i + j * rows`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub color: [f32; 4],
}

impl Surface {
    pub fn new(
        name: impl Into<String>,
        rows: usize,
        cols: usize,
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<f64>,
    ) -> Self {
        Self {
            name: name.into(),
            rows,
            cols,
            x,
            y,
            z,
            color: [0.0, 0.4470, 0.7410, 1.0],
        }
    }
}

/// Camera orientation for 3D views.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub azimuth: f64,   // degrees, rotation around Z axis
    pub elevation: f64, // degrees, tilt up from the XY plane
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            azimuth: -37.5,
            elevation: 30.0,
        } // MATLAB default view
    }
}

/// Colorbar attached to an axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Colorbar {
    pub location: ColorbarLocation,
    pub label: String,
}

/// Colorbar placement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorbarLocation {
    East,
    West,
    North,
    South,
}

/// Surface mesh shading mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shading {
    /// One color per quad (MATLAB default).
    Flat,
    /// Smooth interpolation between height samples.
    Interp,
}

/// Our own perceptual colormap: "rustlab". Built from hand-picked anchor
/// colors (dark navy -> steel blue -> emerald green -> bright yellow -> deep
/// red) interpolated in CIE-like perceptual space. Not copied from any
/// MathWorks colormap.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Colormap {
    Rustlab,
}

/// Look up an RGB color in `[0,1]` at normalized position `t` in `[0,1]` for the
/// given colormap.
pub fn colormap_lookup(cmap: Colormap, t: f64) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    match cmap {
        Colormap::Rustlab => rustlab_colormap(t),
    }
}

/// "rustlab" perceptual colormap. Anchors chosen for roughly monotonic
/// perceptual lightness: dark navy (#001f3f) -> steel blue (#0074a2) ->
/// emerald green (#2ab07f) -> bright yellow (#fde725) -> deep red (#7a0403).
fn rustlab_colormap(t: f64) -> [f32; 3] {
    let anchors: [(f64, f64, f64, f64); 5] = [
        (0.0, 0.0, 0.122, 0.247),    // #001f3f
        (0.25, 0.0, 0.455, 0.635),   // #0074a2
        (0.5, 0.165, 0.690, 0.498),  // #2ab07f
        (0.75, 0.992, 0.902, 0.145), // #fde725
        (1.0, 0.478, 0.016, 0.012),  // #7a0403
    ];
    // Find segment
    let mut i0 = 0usize;
    let mut i1 = anchors.len() - 1;
    for k in 0..anchors.len() - 1 {
        if anchors[k].0 <= t && t <= anchors[k + 1].0 {
            i0 = k;
            i1 = k + 1;
            break;
        }
    }
    let (t0, r0, g0, b0) = anchors[i0];
    let (t1, r1, g1, b1) = anchors[i1];
    let s = if (t1 - t0).abs() < 1e-12 {
        0.0
    } else {
        (t - t0) / (t1 - t0)
    };
    let s = s * s * (3.0 - 2.0 * s); // smoothstep
    [
        (r0 + (r1 - r0) * s) as f32,
        (g0 + (g1 - g0) * s) as f32,
        (b0 + (b1 - b0) * s) as f32,
    ]
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

/// Per-subplot interactive state: selection, hover, cursor readout coordinates, and view offsets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxesInteraction {
    pub is_selected: bool,
    pub is_hovered: bool,
    pub cursor_data: Option<[f64; 2]>,
    pub nearest_point: Option<NearestPoint>,
    pub view_offset: [f64; 2],
    pub zoom_factor: f64,
}

impl Default for AxesInteraction {
    fn default() -> Self {
        Self {
            is_selected: false,
            is_hovered: false,
            cursor_data: None,
            nearest_point: None,
            view_offset: [0.0, 0.0],
            zoom_factor: 1.0,
        }
    }
}

/// Nearest series data point to the cursor during hover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearestPoint {
    pub series_name: String,
    pub series_index: usize,
    pub point_index: usize,
    pub x: f64,
    pub y: f64,
    pub distance_px: f64,
}

/// Subplot grid placement within a figure.
///
/// Supports m-by-n grid positioning with 1-based indices, as well as spanning
/// multiple cell positions (e.g. `[1, 2]` across the top row of a 2x2 grid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubplotPosition {
    pub rows: usize,
    pub cols: usize,
    pub indices: Vec<usize>,
}

impl SubplotPosition {
    pub fn single(rows: usize, cols: usize, index: usize) -> Self {
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
            indices: vec![index.max(1)],
        }
    }

    pub fn spanning(rows: usize, cols: usize, indices: Vec<usize>) -> Self {
        let indices = if indices.is_empty() { vec![1] } else { indices };
        Self {
            rows: rows.max(1),
            cols: cols.max(1),
            indices,
        }
    }

    /// Primary 1-based index (for axes ID or title).
    pub fn primary_index(&self) -> usize {
        self.indices.first().copied().unwrap_or(1)
    }

    /// Returns 0-based grid bounding box `(min_col, max_col, min_row, max_row)`.
    pub fn grid_bounds(&self) -> (usize, usize, usize, usize) {
        let cols = self.cols.max(1);
        let mut min_r = usize::MAX;
        let mut max_r = 0;
        let mut min_c = usize::MAX;
        let mut max_c = 0;

        for &idx in &self.indices {
            let zero_idx = idx.saturating_sub(1);
            let r = zero_idx / cols;
            let c = zero_idx % cols;
            min_r = min_r.min(r);
            max_r = max_r.max(r);
            min_c = min_c.min(c);
            max_c = max_c.max(c);
        }

        if min_r == usize::MAX {
            (0, 0, 0, 0)
        } else {
            (min_c, max_c, min_r, max_r)
        }
    }

    /// Computes pixel rect `(x0, y0, width, height)` within the figure canvas.
    /// Supports proportional margins between cells.
    pub fn compute_rect(
        &self,
        total_x0: f64,
        total_y0: f64,
        total_w: f64,
        total_h: f64,
        gap_x: f64,
        gap_y: f64,
    ) -> (f64, f64, f64, f64) {
        let rows = self.rows.max(1);
        let cols = self.cols.max(1);

        let cell_w = (total_w - (cols as f64 - 1.0) * gap_x).max(10.0) / (cols as f64);
        let cell_h = (total_h - (rows as f64 - 1.0) * gap_y).max(10.0) / (rows as f64);

        let (min_c, max_c, min_r, max_r) = self.grid_bounds();
        let span_c = (max_c - min_c + 1) as f64;
        let span_r = (max_r - min_r + 1) as f64;

        let x0 = total_x0 + (min_c as f64) * (cell_w + gap_x);
        let y0 = total_y0 + (min_r as f64) * (cell_h + gap_y);
        let w = span_c * cell_w + (span_c - 1.0) * gap_x;
        let h = span_r * cell_h + (span_r - 1.0) * gap_y;

        (x0, y0, w, h)
    }
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
    pub zlim: Option<[f64; 2]>,
    pub xscale: Scale,
    pub yscale: Scale,
    pub zscale: Scale,
    pub xticks: Option<Vec<f64>>,
    pub xticklabels: Option<Vec<String>>,
    pub yticks: Option<Vec<f64>>,
    pub yticklabels: Option<Vec<String>>,
    pub grid: bool,
    pub hold: bool,
    pub colormap: Colormap,
    pub shading: Shading,
    pub series: Vec<Series>,
    pub texts: Vec<Text>,
    pub legend: Option<Legend>,
    pub colorbar: Option<Colorbar>,
    pub camera: Camera,
    pub subplot_pos: Option<SubplotPosition>,
    pub shared_x_with: Option<usize>,
    pub shared_y_with: Option<usize>,
    #[serde(default)]
    pub interaction: AxesInteraction,
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
            zlim: None,
            xscale: Scale::Linear,
            yscale: Scale::Linear,
            zscale: Scale::Linear,
            xticks: None,
            xticklabels: None,
            yticks: None,
            yticklabels: None,
            grid: true,
            hold: false,
            colormap: Colormap::Rustlab,
            shading: Shading::Flat,
            series: Vec::new(),
            texts: Vec::new(),
            legend: None,
            colorbar: None,
            camera: Camera::default(),
            subplot_pos: None,
            shared_x_with: None,
            shared_y_with: None,
            interaction: AxesInteraction::default(),
        }
    }

    /// Resolve limits for this axes against sibling axes in a figure.
    pub fn resolve_limits(&self, all_axes: &[Axes]) -> (f64, f64, f64, f64, f64, f64) {
        crate::svg::resolve_limits(self, all_axes)
    }

    /// Map pixel coordinates within `rect` `(px0, py0, pw, ph)` to data coordinates `(x, y)`
    /// using `limits` `(x_min, x_max, y_min, y_max)`.
    pub fn pixel_to_data(
        &self,
        px: f64,
        py: f64,
        rect: (f64, f64, f64, f64),
        limits: (f64, f64, f64, f64),
    ) -> (f64, f64) {
        let (rx, ry, rw, rh) = rect;
        let (x_min, x_max, y_min, y_max) = limits;
        let norm_x = if rw.abs() < 1e-12 {
            0.0
        } else {
            ((px - rx) / rw).clamp(0.0, 1.0)
        };
        let norm_y = if rh.abs() < 1e-12 {
            0.0
        } else {
            ((ry + rh - py) / rh).clamp(0.0, 1.0)
        };

        let data_x = match self.xscale {
            Scale::Linear => x_min + norm_x * (x_max - x_min),
            Scale::Log => {
                let x_min = if x_min <= 0.0 { 1e-10 } else { x_min };
                let x_max = if x_max <= x_min { x_min * 10.0 } else { x_max };
                let lx = x_min.log10() + norm_x * (x_max.log10() - x_min.log10());
                10f64.powf(lx)
            }
        };

        let data_y = match self.yscale {
            Scale::Linear => y_min + norm_y * (y_max - y_min),
            Scale::Log => {
                let y_min = if y_min <= 0.0 { 1e-10 } else { y_min };
                let y_max = if y_max <= y_min { y_min * 10.0 } else { y_max };
                let ly = y_min.log10() + norm_y * (y_max.log10() - y_min.log10());
                10f64.powf(ly)
            }
        };

        (data_x, data_y)
    }

    /// Map data coordinates `(dx, dy)` to pixel coordinates within `rect` `(px0, py0, pw, ph)`
    /// using `limits` `(x_min, x_max, y_min, y_max)`.
    pub fn data_to_pixel(
        &self,
        dx: f64,
        dy: f64,
        rect: (f64, f64, f64, f64),
        limits: (f64, f64, f64, f64),
    ) -> (f64, f64) {
        let (rx, ry, rw, rh) = rect;
        let (x_min, x_max, y_min, y_max) = limits;
        let px = match self.xscale {
            Scale::Linear => {
                let span = x_max - x_min;
                if span.abs() < 1e-12 {
                    rx
                } else {
                    rx + ((dx - x_min) / span) * rw
                }
            }
            Scale::Log => {
                let dx = if dx <= 0.0 { 1e-10 } else { dx };
                let x_min = if x_min <= 0.0 { 1e-10 } else { x_min };
                let x_max = if x_max <= x_min { x_min * 10.0 } else { x_max };
                let span = x_max.log10() - x_min.log10();
                if span.abs() < 1e-12 {
                    rx
                } else {
                    rx + ((dx.log10() - x_min.log10()) / span) * rw
                }
            }
        };

        let py = match self.yscale {
            Scale::Linear => {
                let span = y_max - y_min;
                if span.abs() < 1e-12 {
                    ry + rh
                } else {
                    ry + rh - ((dy - y_min) / span) * rh
                }
            }
            Scale::Log => {
                let dy = if dy <= 0.0 { 1e-10 } else { dy };
                let y_min = if y_min <= 0.0 { 1e-10 } else { y_min };
                let y_max = if y_max <= y_min { y_min * 10.0 } else { y_max };
                let span = y_max.log10() - y_min.log10();
                if span.abs() < 1e-12 {
                    ry + rh
                } else {
                    ry + rh - ((dy.log10() - y_min.log10()) / span) * rh
                }
            }
        };

        (px, py)
    }

    /// Find the nearest series point to screen coordinate `(px, py)`
    /// using `limits` `(x_min, x_max, y_min, y_max)`.
    pub fn find_nearest_point(
        &self,
        px: f64,
        py: f64,
        rect: (f64, f64, f64, f64),
        limits: (f64, f64, f64, f64),
    ) -> Option<NearestPoint> {
        let mut best: Option<NearestPoint> = None;
        let mut min_dist_sq = f64::INFINITY;

        for (s_idx, series) in self.series.iter().enumerate() {
            let (name, xs, ys): (&str, &[f64], &[f64]) = match series {
                Series::Line(l) => (&l.name, &l.x, &l.y),
                Series::Scatter(s) => (&s.name, &s.x, &s.y),
                Series::Bar(b) => (&b.name, &b.x, &b.y),
                _ => continue,
            };

            for (p_idx, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
                let (pt_px, pt_py) = self.data_to_pixel(x, y, rect, limits);
                let d2 = (pt_px - px).powi(2) + (pt_py - py).powi(2);
                if d2 < min_dist_sq {
                    min_dist_sq = d2;
                    best = Some(NearestPoint {
                        series_name: name.to_string(),
                        series_index: s_idx,
                        point_index: p_idx,
                        x,
                        y,
                        distance_px: d2.sqrt(),
                    });
                }
            }
        }
        best
    }

    pub fn set_xlim(&mut self, lo: f64, hi: f64) {
        self.xlim = Some([lo, hi]);
    }

    pub fn set_ylim(&mut self, lo: f64, hi: f64) {
        self.ylim = Some([lo, hi]);
    }

    pub fn set_xscale(&mut self, scale: Scale) {
        self.xscale = scale;
    }

    pub fn set_yscale(&mut self, scale: Scale) {
        self.yscale = scale;
    }

    pub fn set_xticks(&mut self, ticks: Vec<f64>) {
        self.xticks = Some(ticks);
    }

    pub fn set_xticklabels(&mut self, labels: Vec<String>) {
        self.xticklabels = Some(labels);
    }

    pub fn set_yticks(&mut self, ticks: Vec<f64>) {
        self.yticks = Some(ticks);
    }

    pub fn set_yticklabels(&mut self, labels: Vec<String>) {
        self.yticklabels = Some(labels);
    }

    pub fn set_grid(&mut self, grid: bool) {
        self.grid = grid;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_xlabel(&mut self, label: impl Into<String>) {
        self.xlabel = label.into();
    }

    pub fn set_ylabel(&mut self, label: impl Into<String>) {
        self.ylabel = label.into();
    }

    pub fn set_legend(&mut self, location: LegendLocation) {
        self.legend = Some(Legend { location });
    }

    pub fn link_x_with(&mut self, other_id: usize) {
        self.shared_x_with = Some(other_id);
    }

    pub fn link_y_with(&mut self, other_id: usize) {
        self.shared_y_with = Some(other_id);
    }

    pub fn unlink(&mut self) {
        self.shared_x_with = None;
        self.shared_y_with = None;
    }

    /// Push a series, clearing existing series unless hold is on.
    pub fn push_series(&mut self, series: Series) {
        if !self.hold {
            self.series.clear();
        }
        self.series.push(series);
    }

    /// Data bounds across all series, or None when there is no data.
    pub fn data_bounds(&self) -> Option<([f64; 2], [f64; 2], [f64; 2])> {
        let mut xs: Vec<f64> = Vec::new();
        let mut ys: Vec<f64> = Vec::new();
        let mut zs: Vec<f64> = Vec::new();
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
                Series::Surface(s) => {
                    xs.extend(s.x.iter().copied());
                    ys.extend(s.y.iter().copied());
                    zs.extend(s.z.iter().copied());
                }
                Series::Line3D(l) => {
                    xs.extend(l.x.iter().copied());
                    ys.extend(l.y.iter().copied());
                    zs.extend(l.z.iter().copied());
                }
            }
        }
        if xs.is_empty() && ys.is_empty() && zs.is_empty() {
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
        let z_range = if zs.is_empty() {
            [0.0, 1.0]
        } else {
            [
                zs.iter().cloned().fold(f64::INFINITY, f64::min),
                zs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ]
        };
        Some((x_range, y_range, z_range))
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

    /// Returns axes sorted in deterministic row-major layout order:
    /// by (min_row, min_col, id).
    pub fn deterministic_axes(&self) -> Vec<&Axes> {
        let mut sorted: Vec<&Axes> = self.axes.iter().collect();
        sorted.sort_by_key(|a| {
            if let Some(pos) = &a.subplot_pos {
                let (min_c, _, min_r, _) = pos.grid_bounds();
                (min_r, min_c, a.id)
            } else {
                (0, 0, a.id)
            }
        });
        sorted
    }

    /// Select or create axes at subplot position `(m, n, p)`.
    pub fn subplot(&mut self, m: usize, n: usize, p: usize) -> &mut Axes {
        let pos = SubplotPosition::single(m, n, p);
        if let Some(existing_idx) = self
            .axes
            .iter()
            .position(|a| a.subplot_pos.as_ref() == Some(&pos))
        {
            let id = self.axes[existing_idx].id;
            self.current_axes = Some(id);
            return &mut self.axes[existing_idx];
        }
        let id = p;
        let mut ax = Axes::new(id);
        ax.title = format!("subplot({m},{n},{p})");
        ax.subplot_pos = Some(pos);
        self.add_axes(ax);
        self.current_axes = Some(id);
        let len = self.axes.len();
        &mut self.axes[len - 1]
    }

    /// Select or create spanning axes at subplot position `(m, n, indices)`.
    pub fn subplot_spanning(&mut self, m: usize, n: usize, indices: Vec<usize>) -> &mut Axes {
        let pos = SubplotPosition::spanning(m, n, indices);
        if let Some(existing_idx) = self
            .axes
            .iter()
            .position(|a| a.subplot_pos.as_ref() == Some(&pos))
        {
            let id = self.axes[existing_idx].id;
            self.current_axes = Some(id);
            return &mut self.axes[existing_idx];
        }
        let id = pos.primary_index();
        let mut ax = Axes::new(id);
        let idx_str = pos
            .indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        ax.title = format!("subplot({m},{n},[{idx_str}])");
        ax.subplot_pos = Some(pos);
        self.add_axes(ax);
        self.current_axes = Some(id);
        let len = self.axes.len();
        &mut self.axes[len - 1]
    }

    /// Link multiple axes along 'x', 'y', or 'xy' / 'both'.
    pub fn link_axes(&mut self, ax_ids: &[usize], axis: &str) {
        if ax_ids.len() < 2 {
            return;
        }
        let lead_id = ax_ids[0];
        let link_x = axis.eq_ignore_ascii_case("x")
            || axis.eq_ignore_ascii_case("xy")
            || axis.eq_ignore_ascii_case("both");
        let link_y = axis.eq_ignore_ascii_case("y")
            || axis.eq_ignore_ascii_case("xy")
            || axis.eq_ignore_ascii_case("both");

        for ax in &mut self.axes {
            if ax_ids.contains(&ax.id) {
                if link_x {
                    ax.shared_x_with = Some(lead_id);
                }
                if link_y {
                    ax.shared_y_with = Some(lead_id);
                }
            }
        }
    }

    /// Unlink specified axes so each retains independent limits.
    pub fn unlink_axes(&mut self, ax_ids: &[usize]) {
        for ax in &mut self.axes {
            if ax_ids.contains(&ax.id) {
                ax.shared_x_with = None;
                ax.shared_y_with = None;
            }
        }
    }

    /// Compute screen rects `(px0, py0, pw, ph)` for all axes at the given canvas dimensions.
    pub fn compute_axes_rects(
        &self,
        width: f64,
        height: f64,
    ) -> Vec<(usize, (f64, f64, f64, f64))> {
        let is_single_default = self.axes.len() == 1 && self.axes[0].subplot_pos.is_none();
        if is_single_default {
            let pad = 50.0;
            let plot_w = (width - 2.0 * pad).max(10.0);
            let plot_h = (height - 2.0 * pad).max(10.0);
            return vec![(self.axes[0].id, (pad, 60.0, plot_w, plot_h))];
        }

        let sorted = self.deterministic_axes();
        let total_x0 = 60.0;
        let total_y0 = 60.0;
        let total_w = (width - 110.0).max(10.0);
        let total_h = (height - 110.0).max(10.0);
        let gap_x = 60.0;
        let gap_y = 60.0;
        let n = sorted.len();

        let mut rects = Vec::with_capacity(sorted.len());
        for (idx, ax) in sorted.iter().enumerate() {
            let rect = if let Some(pos) = &ax.subplot_pos {
                pos.compute_rect(total_x0, total_y0, total_w, total_h, gap_x, gap_y)
            } else {
                SubplotPosition::single(n, 1, idx + 1)
                    .compute_rect(total_x0, total_y0, total_w, total_h, gap_x, gap_y)
            };
            rects.push((ax.id, rect));
        }
        rects
    }

    /// Hit-test pointer screen coordinates `(px, py)` against all axes.
    /// Returns the ID of the axes containing the pointer, if any.
    pub fn hit_test_axes(&self, px: f64, py: f64, width: f64, height: f64) -> Option<usize> {
        let rects = self.compute_axes_rects(width, height);
        for (id, (rx, ry, rw, rh)) in rects {
            if px >= rx && px <= rx + rw && py >= ry && py <= ry + rh {
                return Some(id);
            }
        }
        None
    }

    /// Update hover and cursor readout for pointer coordinates `(px, py)` on a canvas of `(width, height)`.
    /// Automatically updates each axes' `is_hovered`, `cursor_data`, and `nearest_point`.
    /// Returns the ID of the hovered axes, if any.
    pub fn handle_pointer_move(
        &mut self,
        px: f64,
        py: f64,
        width: f64,
        height: f64,
    ) -> Option<usize> {
        let rects = self.compute_axes_rects(width, height);
        let mut hovered_id = None;

        for (id, rect) in rects {
            let (rx, ry, rw, rh) = rect;
            let inside = px >= rx && px <= rx + rw && py >= ry && py <= ry + rh;

            let (x_min, x_max, y_min, y_max, _, _) = {
                let ax = match self.axes.iter().find(|a| a.id == id) {
                    Some(a) => a,
                    None => continue,
                };
                crate::svg::resolve_limits(ax, &self.axes)
            };

            let ax = match self.axes.iter_mut().find(|a| a.id == id) {
                Some(a) => a,
                None => continue,
            };

            ax.interaction.is_hovered = inside;
            if inside {
                hovered_id = Some(id);
                let limits = (x_min, x_max, y_min, y_max);
                let (dx, dy) = ax.pixel_to_data(px, py, rect, limits);
                ax.interaction.cursor_data = Some([dx, dy]);
                ax.interaction.nearest_point = ax.find_nearest_point(px, py, rect, limits);
            } else {
                ax.interaction.cursor_data = None;
                ax.interaction.nearest_point = None;
            }
        }

        hovered_id
    }

    /// Handle pointer click: selects the clicked axes, unselects other axes,
    /// and sets `current_axes` to the clicked axes ID.
    pub fn handle_pointer_click(
        &mut self,
        px: f64,
        py: f64,
        width: f64,
        height: f64,
    ) -> Option<usize> {
        let hit_id = self.hit_test_axes(px, py, width, height);
        if let Some(id) = hit_id {
            self.select_axes(id);
        }
        hit_id
    }

    /// Select an axes by ID, setting it as `current_axes` and updating `is_selected`.
    pub fn select_axes(&mut self, ax_id: usize) {
        self.current_axes = Some(ax_id);
        for ax in &mut self.axes {
            ax.interaction.is_selected = ax.id == ax_id;
        }
    }

    /// Zoom the specified subplot by `zoom_ratio` (e.g. 0.5 = zoom in 2x, 2.0 = zoom out 2x).
    /// `center` is an optional data coordinate `(cx, cy)` around which to zoom; if None, centers on mid-range.
    /// If axes are linked (via `shared_x_with` or `shared_y_with`), linked subplots move together.
    /// Unlinked subplots remain completely unaffected.
    pub fn zoom_subplot(
        &mut self,
        ax_id: usize,
        zoom_ratio: f64,
        center: Option<(f64, f64)>,
    ) -> bool {
        if zoom_ratio <= 0.0 || zoom_ratio.is_nan() {
            return false;
        }

        let (orig_x_min, orig_x_max, orig_y_min, orig_y_max) = {
            let ax = match self.axes.iter().find(|a| a.id == ax_id) {
                Some(a) => a,
                None => return false,
            };
            let (x0, x1, y0, y1, _, _) = crate::svg::resolve_limits(ax, &self.axes);
            (x0, x1, y0, y1)
        };

        let (cx, cy) = center.unwrap_or((
            (orig_x_min + orig_x_max) / 2.0,
            (orig_y_min + orig_y_max) / 2.0,
        ));

        let new_x_min = cx - (cx - orig_x_min) * zoom_ratio;
        let new_x_max = cx + (orig_x_max - cx) * zoom_ratio;
        let new_y_min = cy - (cy - orig_y_min) * zoom_ratio;
        let new_y_max = cy + (orig_y_max - cy) * zoom_ratio;

        let target_x_lead = self
            .axes
            .iter()
            .find(|a| a.id == ax_id)
            .and_then(|a| a.shared_x_with)
            .unwrap_or(ax_id);
        let target_y_lead = self
            .axes
            .iter()
            .find(|a| a.id == ax_id)
            .and_then(|a| a.shared_y_with)
            .unwrap_or(ax_id);

        for ax in &mut self.axes {
            let shares_x = ax.id == ax_id
                || ax.shared_x_with == Some(target_x_lead)
                || (ax.id == target_x_lead && target_x_lead != ax_id);
            let shares_y = ax.id == ax_id
                || ax.shared_y_with == Some(target_y_lead)
                || (ax.id == target_y_lead && target_y_lead != ax_id);

            if shares_x {
                ax.xlim = Some([new_x_min, new_x_max]);
            }
            if shares_y {
                ax.ylim = Some([new_y_min, new_y_max]);
            }
            if ax.id == ax_id {
                ax.interaction.zoom_factor *= 1.0 / zoom_ratio;
            }
        }

        true
    }

    /// Pan the specified subplot by data offsets `(dx, dy)`.
    /// If axes are linked, linked subplots move together.
    /// Unlinked subplots remain completely unaffected.
    pub fn pan_subplot(&mut self, ax_id: usize, dx: f64, dy: f64) -> bool {
        let (orig_x_min, orig_x_max, orig_y_min, orig_y_max) = {
            let ax = match self.axes.iter().find(|a| a.id == ax_id) {
                Some(a) => a,
                None => return false,
            };
            let (x0, x1, y0, y1, _, _) = crate::svg::resolve_limits(ax, &self.axes);
            (x0, x1, y0, y1)
        };

        let new_x_min = orig_x_min + dx;
        let new_x_max = orig_x_max + dx;
        let new_y_min = orig_y_min + dy;
        let new_y_max = orig_y_max + dy;

        let target_x_lead = self
            .axes
            .iter()
            .find(|a| a.id == ax_id)
            .and_then(|a| a.shared_x_with)
            .unwrap_or(ax_id);
        let target_y_lead = self
            .axes
            .iter()
            .find(|a| a.id == ax_id)
            .and_then(|a| a.shared_y_with)
            .unwrap_or(ax_id);

        for ax in &mut self.axes {
            let shares_x = ax.id == ax_id
                || ax.shared_x_with == Some(target_x_lead)
                || (ax.id == target_x_lead && target_x_lead != ax_id);
            let shares_y = ax.id == ax_id
                || ax.shared_y_with == Some(target_y_lead)
                || (ax.id == target_y_lead && target_y_lead != ax_id);

            if shares_x {
                ax.xlim = Some([new_x_min, new_x_max]);
                ax.interaction.view_offset[0] += dx;
            }
            if shares_y {
                ax.ylim = Some([new_y_min, new_y_max]);
                ax.interaction.view_offset[1] += dy;
            }
        }

        true
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
        let (xr, yr, _zr) = ax.data_bounds().unwrap();
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
            Series::Surface(s) => s.name.clone(),
            Series::Line3D(l) => l.name.clone(),
        }
    }
}
