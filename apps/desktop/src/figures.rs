//! Figures panel: renders the runtime `GraphicsState` snapshot with
//! egui_plot. Purely presentational — all data comes from `RunResult::figures`
//! (the interpreter never touches GUI types).
//!
//! The panel is a right-hand column that is only materialized when the user
//! opens it or when the session creates a figure (see `MatRustApp`). With no
//! figures it takes no space at all — the editor keeps the room.

use egui::Color32;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Points};

use matlab_runtime::{FigureState, PlotItem};

use crate::theme::{self, SyntaxPalette, ThemeKind};

#[derive(Debug, Default)]
pub struct FiguresState {
    /// Mirror of the session's figures from the last execution.
    pub figures: Vec<FigureState>,
    /// Index of the figure currently displayed.
    pub view_index: usize,
}

/// What the header row asked the app to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    None,
    /// `close all` in the persistent session.
    ClearAll,
    /// Hide the Figures panel (space returns to the editor).
    HidePanel,
}

impl FiguresState {
    pub fn replace_from(&mut self, figures: Vec<FigureState>) {
        self.figures = figures;
        self.view_index = self.view_index.min(self.figures.len().saturating_sub(1));
    }
}

/// Compact header: figure selector tabs on the left, `Close All` and the
/// panel-close button on the right.
pub fn show_header(ui: &mut egui::Ui, state: &mut FiguresState, kind: ThemeKind) -> HeaderAction {
    let mut action = HeaderAction::None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Figures").strong());
        for (i, fig) in state.figures.clone().into_iter().enumerate() {
            let selected = i == state.view_index;
            let label = format!("Figure {}", fig.handle);
            let fill = if selected {
                theme::tint(theme::accent(kind), 0.4)
            } else {
                Color32::TRANSPARENT
            };
            if ui
                .add(egui::Button::new(label).small().fill(fill))
                .clicked()
            {
                state.view_index = i;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new("×").small())
                .on_hover_text(
                    "Hide the Figures panel (Window ▸ Figures re-opens it; so does a new plot)",
                )
                .clicked()
            {
                action = HeaderAction::HidePanel;
            }
            if ui
                .add(egui::Button::new("Close All").small())
                .on_hover_text("close all — clears every figure in the session")
                .clicked()
            {
                action = HeaderAction::ClearAll;
            }
        });
    });
    ui.spacing_mut().item_spacing.y = 4.0;
    action
}

/// Figure area: actual plot rendering, pan/zoom with the mouse. When the
/// session has no figures this stays a compact two-line state — no big
/// placeholder canvas.
pub fn show_body(
    ui: &mut egui::Ui,
    state: &mut FiguresState,
    kind: ThemeKind,
    _palette: &SyntaxPalette,
) {
    let Some(fig) = state.figures.get(state.view_index).cloned() else {
        ui.weak("No figures open.");
        ui.monospace(
            egui::RichText::new("Try: plot([1 2 3], [1 4 9]);")
                .color(theme::text_color_faint(kind)),
        );
        return;
    };

    if let Some(title) = &fig.title {
        ui.vertical_centered_justified(|ui| {
            ui.strong(title.clone());
        });
    }
    if let Some(ylabel) = &fig.ylabel {
        ui.vertical_centered_justified(|ui| {
            ui.weak(format!("y: {}", ylabel));
        });
    }

    let figure_plot = Plot::new(("figure_plot", fig.handle))
        .show_grid(fig.grid)
        .grid_color(grid_color(kind))
        .show_background(false)
        .show_axes(true);
    let figure_plot = if fig.legend_shown {
        figure_plot.legend(Legend::default())
    } else {
        figure_plot
    };

    figure_plot.show(ui, |plot_ui| {
        for (i, item) in fig.items.iter().enumerate() {
            let color = theme::plot_series_color(i, kind);
            let name = item.label().to_string();
            match item {
                PlotItem::Line { x, y, .. } => {
                    let pts: Vec<[f64; 2]> =
                        x.iter().zip(y.iter()).map(|(a, b)| [*a, *b]).collect();
                    if !pts.is_empty() {
                        plot_ui.line(
                            Line::new(name.clone(), PlotPoints::from(pts))
                                .color(color)
                                .width(1.6),
                        );
                    }
                }
                PlotItem::Scatter { x, y, .. } => {
                    let pts: Vec<[f64; 2]> =
                        x.iter().zip(y.iter()).map(|(a, b)| [*a, *b]).collect();
                    if !pts.is_empty() {
                        plot_ui.points(
                            Points::new(name.clone(), PlotPoints::from(pts))
                                .color(color)
                                .radius(2.5)
                                .filled(true),
                        );
                    }
                }
                PlotItem::Bar {
                    positions, values, ..
                } => {
                    let bars: Vec<Bar> = positions
                        .iter()
                        .zip(values.iter())
                        .map(|(p, v)| Bar::new(*p, *v).width(0.75).fill(color))
                        .collect();
                    if !bars.is_empty() {
                        plot_ui.bar_chart(BarChart::new(name.clone(), bars).color(color));
                    }
                }
                PlotItem::Histogram { values, bins, .. } => {
                    let bar_color = theme::tint(color, 0.85);
                    let (centers, heights) = histogram_bins(values, *bins);
                    let bars: Vec<Bar> = centers
                        .iter()
                        .zip(heights.iter())
                        .map(|(c, h)| Bar::new(*c, *h).width(bar_width(*bins)).fill(bar_color))
                        .collect();
                    if !bars.is_empty() {
                        plot_ui.bar_chart(
                            BarChart::new(name.clone(), bars)
                                .color(bar_color)
                                .vertical(),
                        );
                    }
                }
            }
        }
    });

    if let Some(xlabel) = &fig.xlabel {
        ui.vertical_centered_justified(|ui| {
            ui.weak(xlabel.clone());
        });
    }
    if !fig.items.is_empty() && fig.items.len() > 1 && !fig.legend_shown {
        ui.weak(format!(
            "{} series (legend() to label them)",
            fig.items.len()
        ));
    }
}

fn grid_color(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0x30, 0x33, 0x3a),
        ThemeKind::Light => Color32::from_rgb(0xd3, 0xda, 0xe2),
    }
}

fn bar_width(bins: usize) -> f64 {
    (0.8 / bins.max(1) as f64).min(0.8).max(0.05)
}

/// Uniform binning on the range min..max (histogram default behavior).
fn histogram_bins(values: &[f64], bins: usize) -> (Vec<f64>, Vec<f64>) {
    let bins = bins.max(1);
    if values.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        return (vec![min, max], vec![values.len() as f64]);
    }
    let width = (max - min) / bins as f64;
    let mut counts = vec![0usize; bins];
    for v in values {
        let mut idx = (((v - min) / width) as usize).min(bins - 1);
        if *v == max {
            idx = bins - 1;
        }
        counts[idx] += 1;
    }
    let centers: Vec<f64> = (0..bins).map(|i| min + (i as f64 + 0.5) * width).collect();
    let heights: Vec<f64> = counts.into_iter().map(|c| c as f64).collect();
    (centers, heights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_bins_count_and_spacing() {
        let (centers, heights) = histogram_bins(&[1.0, 2.0, 3.0, 3.0, 5.0], 4);
        assert_eq!(centers.len(), 4);
        assert_eq!(heights.iter().map(|h| *h as usize).sum::<usize>(), 5);
        assert!(centers.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn histogram_all_equal_values() {
        let (centers, heights) = histogram_bins(&[7.0, 7.0], 3);
        assert_eq!(heights.iter().map(|h| *h as usize).sum::<usize>(), 2);
        assert_eq!(centers.len(), 2);
    }
}
