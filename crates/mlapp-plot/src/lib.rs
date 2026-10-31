//! Renderer-agnostic 2D plotting model for UIAxes and Figure in App Designer.

#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineSeries {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub label: Option<String>,
    pub color: [f32; 4],
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScatterSeries {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub label: Option<String>,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarSeries {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlotSeries {
    Line(LineSeries),
    Scatter(ScatterSeries),
    Bar(BarSeries),
}

/// Renderer-agnostic representation of a UIAxes component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxesModel {
    pub name: String,
    pub title: String,
    pub xlabel: String,
    pub ylabel: String,
    pub xlim: Option<[f64; 2]>,
    pub ylim: Option<[f64; 2]>,
    pub grid_on: bool,
    pub hold_on: bool,
    pub series: Vec<PlotSeries>,
}

impl AxesModel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            title: String::new(),
            xlabel: String::new(),
            ylabel: String::new(),
            xlim: None,
            ylim: None,
            grid_on: true,
            hold_on: false,
            series: Vec::new(),
        }
    }

    pub fn plot(&mut self, x: Vec<f64>, y: Vec<f64>, label: Option<String>) {
        if !self.hold_on {
            self.series.clear();
        }
        self.series.push(PlotSeries::Line(LineSeries {
            x,
            y,
            label,
            color: [0.0, 0.447, 0.741, 1.0], // default MATLAB blue
            width: 1.5,
        }));
    }

    pub fn scatter(&mut self, x: Vec<f64>, y: Vec<f64>, label: Option<String>) {
        if !self.hold_on {
            self.series.clear();
        }
        self.series.push(PlotSeries::Scatter(ScatterSeries {
            x,
            y,
            label,
            size: 6.0,
        }));
    }

    pub fn bar(&mut self, x: Vec<f64>, y: Vec<f64>, label: Option<String>) {
        if !self.hold_on {
            self.series.clear();
        }
        self.series.push(PlotSeries::Bar(BarSeries { x, y, label }));
    }

    /// Exports simple deterministic SVG for headless verification and snapshot testing.
    pub fn to_svg(&self, width: f64, height: f64) -> String {
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"##,
        );
        svg.push_str(r##"<rect width="100%" height="100%" fill="#ffffff" stroke="#e0e0e0"/>"##);

        // Render Title
        if !self.title.is_empty() {
            svg.push_str(&format!(
                r##"<text x="{}" y="25" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" fill="#000000">{}</text>"##,
                width / 2.0, self.title
            ));
        }

        // Plot axes lines
        let pad = 40.0;
        let plot_w = width - 2.0 * pad;
        let plot_h = height - 2.0 * pad;
        svg.push_str(&format!(
            r##"<rect x="{pad}" y="{pad}" width="{plot_w}" height="{plot_h}" fill="none" stroke="#666666"/>"##,
        ));

        // Render lines
        for s in &self.series {
            if let PlotSeries::Line(line) = s {
                if line.x.len() >= 2 && line.x.len() == line.y.len() {
                    let min_x = line.x.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max_x = line.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let min_y = line.y.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max_y = line.y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                    let dx = if (max_x - min_x).abs() < 1e-12 {
                        1.0
                    } else {
                        max_x - min_x
                    };
                    let dy = if (max_y - min_y).abs() < 1e-12 {
                        1.0
                    } else {
                        max_y - min_y
                    };

                    let mut points = Vec::new();
                    for (px, py) in line.x.iter().zip(line.y.iter()) {
                        let sx = pad + ((px - min_x) / dx) * plot_w;
                        let sy = pad + plot_h - ((py - min_y) / dy) * plot_h;
                        points.push(format!("{sx:.1},{sy:.1}"));
                    }
                    svg.push_str(&format!(
                        r##"<polyline fill="none" stroke="#0072BD" stroke-width="{}" points="{}" />"##,
                        line.width,
                        points.join(" ")
                    ));
                }
            }
        }

        svg.push_str("</svg>");
        svg
    }
}
