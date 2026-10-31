//! Rendering abstraction and backend implementations (headless mock & egui).

#![deny(clippy::unwrap_used)]

use mlapp_host::UiEvent;
use mlapp_ui::{
    figure_to_top_left_geometry, matlab_to_top_left_geometry, Component, ComponentKind,
};

/// Render context providing parent geometry and frame details.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub parent_height: f64,
    pub parent_width: f64,
}

/// Pluggable rendering backend trait to decouple UI frameworks (egui, Web, Tauri).
pub trait RenderBackend {
    fn render_tree(&mut self, root: &Component) -> Vec<UiEvent>;
    fn name(&self) -> &'static str;
}

/// Headless / Mock renderer for headless CI, tests, and CLI rendering.
#[derive(Debug, Default)]
pub struct HeadlessRenderer {
    pub rendered_log: Vec<String>,
    pub pending_events: Vec<UiEvent>,
}

struct PixelCanvas<'a> {
    buf: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> PixelCanvas<'a> {
    fn put_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
            if idx + 3 < self.buf.len() {
                self.buf[idx] = color[0];
                self.buf[idx + 1] = color[1];
                self.buf[idx + 2] = color[2];
                self.buf[idx + 3] = color[3];
            }
        }
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for py in y..(y + h) {
            for px in x..(x + w) {
                self.put_pixel(px, py, color);
            }
        }
    }

    fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for px in x..(x + w) {
            self.put_pixel(px, y, color);
            self.put_pixel(px, y + h - 1, color);
        }
        for py in y..(y + h) {
            self.put_pixel(x, py, color);
            self.put_pixel(x + w - 1, py, color);
        }
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_click(&mut self, component_name: &str) {
        self.pending_events.push(UiEvent::ButtonClick {
            component_name: component_name.to_string(),
        });
    }

    fn render_node(&mut self, comp: &Component, ctx: &RenderContext, is_root: bool) {
        let top_left = if is_root || comp.kind == ComponentKind::UIFigure {
            figure_to_top_left_geometry(comp.position, None)
        } else {
            matlab_to_top_left_geometry(comp.position, ctx.parent_height)
        };
        self.rendered_log.push(format!(
            "[{:?}] '{}' at ({:.0},{:.0} w={:.0} h={:.0})",
            comp.kind, comp.name, top_left.x, top_left.y, top_left.width, top_left.height
        ));

        let child_ctx = RenderContext {
            parent_height: comp.position.height,
            parent_width: comp.position.width,
        };

        for child in &comp.children {
            self.render_node(child, &child_ctx, false);
        }
    }

    /// Renders the component tree to an RGBA pixel buffer for headless screenshot verification.
    pub fn capture_rgba_buffer(&self, root: &Component, width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![255u8; (width * height * 4) as usize];
        let ctx = RenderContext {
            parent_height: root.position.height,
            parent_width: root.position.width,
        };
        let mut canvas = PixelCanvas {
            buf: &mut buf,
            width,
            height,
        };
        self.rasterize_component(&mut canvas, root, &ctx, true);
        buf
    }

    fn rasterize_component(
        &self,
        canvas: &mut PixelCanvas<'_>,
        comp: &Component,
        ctx: &RenderContext,
        is_root: bool,
    ) {
        let top_left = if is_root || comp.kind == ComponentKind::UIFigure {
            figure_to_top_left_geometry(comp.position, None)
        } else {
            matlab_to_top_left_geometry(comp.position, ctx.parent_height)
        };

        let x = top_left.x as i32;
        let y = top_left.y as i32;
        let w = top_left.width as i32;
        let h = top_left.height as i32;

        match comp.kind {
            ComponentKind::UIFigure => {
                canvas.fill_rect(x, y, w, h, [240, 240, 240, 255]);
            }
            ComponentKind::Panel => {
                canvas.fill_rect(x, y, w, h, [230, 230, 230, 255]);
                canvas.stroke_rect(x, y, w, h, [180, 180, 180, 255]);
            }
            ComponentKind::Button => {
                canvas.fill_rect(x, y, w, h, [210, 210, 210, 255]);
                canvas.stroke_rect(x, y, w, h, [120, 120, 120, 255]);
            }
            ComponentKind::Label => {
                canvas.fill_rect(x, y, w, h, [240, 240, 240, 255]);
            }
            ComponentKind::UIAxes => {
                canvas.fill_rect(x, y, w, h, [255, 255, 255, 255]);
                canvas.stroke_rect(x, y, w, h, [100, 100, 100, 255]);
                let pad = 20;
                if w > 2 * pad && h > 2 * pad {
                    canvas.stroke_rect(
                        x + pad,
                        y + pad,
                        w - 2 * pad,
                        h - 2 * pad,
                        [180, 180, 180, 255],
                    );
                    canvas.draw_line(
                        x + pad,
                        y + h - pad,
                        x + w - pad,
                        y + pad,
                        [0, 114, 189, 255],
                    );
                }
            }
            _ => {}
        }

        let child_ctx = RenderContext {
            parent_height: comp.position.height,
            parent_width: comp.position.width,
        };
        for child in &comp.children {
            self.rasterize_component(canvas, child, &child_ctx, false);
        }
    }
}

impl RenderBackend for HeadlessRenderer {
    fn render_tree(&mut self, root: &Component) -> Vec<UiEvent> {
        let ctx = RenderContext {
            parent_height: root.position.height,
            parent_width: root.position.width,
        };
        self.render_node(root, &ctx, true);
        std::mem::take(&mut self.pending_events)
    }

    fn name(&self) -> &'static str {
        "headless"
    }
}

#[cfg(feature = "egui-backend")]
pub mod egui_backend {
    use super::*;
    use mlapp_ui::PropertyValue;

    /// Egui UI renderer backend
    pub struct EguiRenderer {
        pub pending_events: Vec<UiEvent>,
    }

    impl EguiRenderer {
        pub fn new() -> Self {
            Self {
                pending_events: Vec::new(),
            }
        }

        pub fn render(&mut self, ui: &mut egui::Ui, comp: &Component, parent_height: f64) {
            let _top_left = matlab_to_top_left_geometry(comp.position, parent_height);

            match comp.kind {
                ComponentKind::Button => {
                    let text = comp
                        .get_property("text")
                        .and_then(|p| match p {
                            PropertyValue::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .unwrap_or(&comp.name);

                    if ui.button(text).clicked() {
                        self.pending_events.push(UiEvent::ButtonClick {
                            component_name: comp.name.clone(),
                        });
                    }
                }
                ComponentKind::Label => {
                    let text = comp
                        .get_property("text")
                        .and_then(|p| match p {
                            PropertyValue::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .unwrap_or("");
                    ui.label(text);
                }
                ComponentKind::Panel => {
                    ui.group(|panel_ui| {
                        for child in &comp.children {
                            self.render(panel_ui, child, comp.position.height);
                        }
                    });
                }
                ComponentKind::UIAxes => {
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(comp.position.width as f32, comp.position.height as f32),
                        egui::Sense::hover(),
                    );
                    let painter = ui.painter();
                    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(255, 255, 255));
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 100, 100)),
                        egui::StrokeKind::Outside,
                    );
                    let pad = 20.0;
                    let plot_rect = rect.shrink(pad);
                    painter.rect_stroke(
                        plot_rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(180, 180, 180)),
                        egui::StrokeKind::Outside,
                    );
                    painter.line_segment(
                        [plot_rect.left_bottom(), plot_rect.right_top()],
                        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(0, 114, 189)),
                    );
                }
                _ => {
                    for child in &comp.children {
                        self.render(ui, child, comp.position.height);
                    }
                }
            }
        }
    }

    impl Default for EguiRenderer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RenderBackend for EguiRenderer {
        fn render_tree(&mut self, _root: &Component) -> Vec<UiEvent> {
            std::mem::take(&mut self.pending_events)
        }

        fn name(&self) -> &'static str {
            "egui"
        }
    }
}
