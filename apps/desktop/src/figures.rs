//! Scientific graphics and interactive figure canvas.

use crate::execution::{GraphicsState, PlotItem};
use crate::theme::Theme;
use egui::{Color32, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

pub struct FiguresPanel {
    pub selected_figure_id: usize,
    pub view_offset: Vec2,
    pub view_zoom: f32,
}

impl Default for FiguresPanel {
    fn default() -> Self {
        Self {
            selected_figure_id: 1,
            view_offset: Vec2::ZERO,
            view_zoom: 1.0,
        }
    }
}

impl FiguresPanel {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, graphics: &mut GraphicsState) {
        ui.vertical(|ui| {
            // Figure Tabs header
            ui.horizontal(|ui| {
                ui.heading("Figures");
                ui.separator();

                if graphics.figures.is_empty() {
                    // Provide a default empty figure
                    graphics.get_or_create_figure(1);
                }

                let mut to_close = None;

                for fig in &graphics.figures {
                    let is_selected = self.selected_figure_id == fig.id;
                    let tab_title = if is_selected {
                        format!("● {}", fig.title)
                    } else {
                        fig.title.clone()
                    };

                    let btn = ui.selectable_label(is_selected, tab_title);
                    if btn.clicked() {
                        self.selected_figure_id = fig.id;
                        graphics.current_figure = Some(fig.id);
                    }
                    if ui.small_button("✕").on_hover_text("Close figure").clicked() {
                        to_close = Some(fig.id);
                    }
                }

                if let Some(id) = to_close {
                    graphics.close_figure(id);
                    if let Some(first) = graphics.figures.first() {
                        self.selected_figure_id = first.id;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("+ New")
                        .on_hover_text("Create new figure (figure)")
                        .clicked()
                    {
                        let next_id = graphics.figures.iter().map(|f| f.id).max().unwrap_or(0) + 1;
                        graphics.get_or_create_figure(next_id);
                        self.selected_figure_id = next_id;
                    }
                    if ui.button("Reset View").clicked() {
                        self.view_offset = Vec2::ZERO;
                        self.view_zoom = 1.0;
                    }
                });
            });

            ui.separator();

            let fig = match graphics
                .figures
                .iter_mut()
                .find(|f| f.id == self.selected_figure_id)
            {
                Some(f) => f,
                None => {
                    if let Some(first) = graphics.figures.first_mut() {
                        self.selected_figure_id = first.id;
                        first
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No active figures");
                        });
                        return;
                    }
                }
            };

            // Plot controls bar
            ui.horizontal(|ui| {
                ui.label(RichText::new(&fig.title).strong().color(theme.accent));
                ui.separator();
                ui.checkbox(&mut fig.grid, "Grid");
                ui.separator();
                ui.label(format!("Series: {}", fig.items.len()));
            });

            ui.add_space(4.0);

            // Render interactive plot canvas
            let (canvas_rect, response) =
                ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

            // Pan and zoom interaction
            if response.dragged() {
                self.view_offset += response.drag_delta();
            }

            let painter = ui.painter_at(canvas_rect);

            // Background fill
            painter.rect_filled(canvas_rect, 0.0, theme.editor_bg);
            painter.rect_stroke(
                canvas_rect,
                0.0,
                Stroke::new(1.0_f32, theme.border),
                StrokeKind::Inside,
            );

            if fig.items.is_empty() {
                painter.text(
                    canvas_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Empty Figure (Run plot(x, y), scatter(x, y), or bar(x, y) to display data)",
                    egui::FontId::proportional(14.0),
                    theme.text_secondary,
                );
                return;
            }

            // Determine data bounds
            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for item in &fig.items {
                match item {
                    PlotItem::Line { x, y, .. }
                    | PlotItem::Scatter { x, y, .. }
                    | PlotItem::Bar { x, y, .. } => {
                        for (&vx, &vy) in x.iter().zip(y.iter()) {
                            if vx.is_finite() && vy.is_finite() {
                                min_x = min_x.min(vx);
                                max_x = max_x.max(vx);
                                min_y = min_y.min(vy);
                                max_y = max_y.max(vy);
                            }
                        }
                    }
                    PlotItem::Histogram { values, .. } => {
                        for &v in values {
                            if v.is_finite() {
                                min_x = min_x.min(v);
                                max_x = max_x.max(v);
                            }
                        }
                        min_y = 0.0;
                        max_y = max_y.max(10.0);
                    }
                }
            }

            if !min_x.is_finite() || !max_x.is_finite() || min_x == max_x {
                min_x = 0.0;
                max_x = 10.0;
            }
            if !min_y.is_finite() || !max_y.is_finite() || min_y == max_y {
                min_y = 0.0;
                max_y = 10.0;
            }

            // Margin for plot axes
            let plot_box = Rect::from_min_max(
                Pos2::new(canvas_rect.min.x + 50.0, canvas_rect.min.y + 20.0),
                Pos2::new(canvas_rect.max.x - 20.0, canvas_rect.max.y - 40.0),
            );

            // Coordinate mapping: data (x, y) -> screen (px, py)
            let to_screen = |dx: f64, dy: f64| -> Pos2 {
                let norm_x = (dx - min_x) / (max_x - min_x);
                let norm_y = (dy - min_y) / (max_y - min_y);
                Pos2::new(
                    plot_box.min.x + (norm_x as f32) * plot_box.width() + self.view_offset.x,
                    plot_box.max.y - (norm_y as f32) * plot_box.height() + self.view_offset.y,
                )
            };

            // Draw grid lines
            if fig.grid {
                let grid_stroke =
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(80, 80, 80, 70));
                for i in 0..=5 {
                    let frac = i as f32 / 5.0;
                    let gx = plot_box.min.x + frac * plot_box.width();
                    let gy = plot_box.min.y + frac * plot_box.height();
                    painter.line_segment(
                        [Pos2::new(gx, plot_box.min.y), Pos2::new(gx, plot_box.max.y)],
                        grid_stroke,
                    );
                    painter.line_segment(
                        [Pos2::new(plot_box.min.x, gy), Pos2::new(plot_box.max.x, gy)],
                        grid_stroke,
                    );
                }
            }

            // Draw Axes border
            painter.rect_stroke(
                plot_box,
                0.0,
                Stroke::new(1.5_f32, theme.border),
                StrokeKind::Inside,
            );

            // Axis labels & tick numbers
            for i in 0..=5 {
                let frac = i as f64 / 5.0;
                let val_x = min_x + frac * (max_x - min_x);
                let val_y = min_y + frac * (max_y - min_y);
                let pt_x = to_screen(val_x, min_y);
                let pt_y = to_screen(min_x, val_y);

                painter.text(
                    Pos2::new(pt_x.x, plot_box.max.y + 8.0),
                    egui::Align2::CENTER_TOP,
                    format!("{val_x:.1}"),
                    egui::FontId::monospace(10.0),
                    theme.text_secondary,
                );
                painter.text(
                    Pos2::new(plot_box.min.x - 8.0, pt_y.y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{val_y:.1}"),
                    egui::FontId::monospace(10.0),
                    theme.text_secondary,
                );
            }

            // Axis titles
            painter.text(
                Pos2::new(plot_box.center().x, plot_box.max.y + 24.0),
                egui::Align2::CENTER_TOP,
                &fig.x_label,
                egui::FontId::proportional(12.0),
                theme.text_primary,
            );
            painter.text(
                Pos2::new(plot_box.min.x - 36.0, plot_box.center().y),
                egui::Align2::CENTER_CENTER,
                &fig.y_label,
                egui::FontId::proportional(12.0),
                theme.text_primary,
            );

            // Draw plot items
            for item in &fig.items {
                match item {
                    PlotItem::Line {
                        x, y, color, width, ..
                    } => {
                        let stroke = Stroke::new(
                            *width,
                            Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
                        );
                        let points: Vec<Pos2> = x
                            .iter()
                            .zip(y.iter())
                            .filter(|(&vx, &vy)| vx.is_finite() && vy.is_finite())
                            .map(|(&vx, &vy)| to_screen(vx, vy))
                            .collect();

                        for pair in points.windows(2) {
                            if plot_box.contains(pair[0]) || plot_box.contains(pair[1]) {
                                painter.line_segment([pair[0], pair[1]], stroke);
                            }
                        }
                    }
                    PlotItem::Scatter {
                        x,
                        y,
                        color,
                        radius,
                        ..
                    } => {
                        let fill =
                            Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
                        for (&vx, &vy) in x.iter().zip(y.iter()) {
                            let pt = to_screen(vx, vy);
                            if plot_box.contains(pt) {
                                painter.circle_filled(pt, *radius, fill);
                            }
                        }
                    }
                    PlotItem::Bar { x, y, color, .. } => {
                        let fill =
                            Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
                        let bar_width = (plot_box.width() / (x.len() as f32).max(1.0)) * 0.6;
                        for (&vx, &vy) in x.iter().zip(y.iter()) {
                            let top = to_screen(vx, vy);
                            let bottom = to_screen(vx, 0.0);
                            let bar_rect = Rect::from_min_max(
                                Pos2::new(top.x - bar_width / 2.0, top.y),
                                Pos2::new(top.x + bar_width / 2.0, bottom.y),
                            );
                            painter.rect_filled(bar_rect, 0.0, fill);
                        }
                    }
                    PlotItem::Histogram {
                        values,
                        bins,
                        color,
                        ..
                    } => {
                        let fill =
                            Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
                        let num_bins = (*bins).max(5);
                        let _bin_width_data = (max_x - min_x) / num_bins as f64;
                        let mut counts = vec![0usize; num_bins];

                        for &v in values {
                            if v >= min_x && v <= max_x {
                                let b = (((v - min_x) / (max_x - min_x)) * (num_bins as f64 - 1.0))
                                    as usize;
                                if b < num_bins {
                                    counts[b] += 1;
                                }
                            }
                        }
                        let max_count = counts.iter().copied().max().unwrap_or(1) as f64;
                        let screen_bin_w = plot_box.width() / num_bins as f32;

                        for (i, &cnt) in counts.iter().enumerate() {
                            let h = (cnt as f64 / max_count) * (plot_box.height() as f64);
                            let rx = plot_box.min.x + i as f32 * screen_bin_w;
                            let ry = plot_box.max.y - h as f32;
                            let r = Rect::from_min_max(
                                Pos2::new(rx, ry),
                                Pos2::new(rx + screen_bin_w * 0.9, plot_box.max.y),
                            );
                            painter.rect_filled(r, 0.0, fill);
                        }
                    }
                }
            }
        });
    }
}
