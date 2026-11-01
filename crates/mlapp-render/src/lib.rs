//! Rendering abstraction and backend implementations (headless mock & egui).

#![deny(clippy::unwrap_used)]

pub mod autotest;
pub use autotest::{run_autotest_headless, AutotestAction, AutotestScript};

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

/// Encodes and saves the rendered component tree as a PNG screenshot.
pub fn save_screenshot_png(root: &Component, path: &std::path::Path) -> Result<(), std::io::Error> {
    let width = root.position.width as u32;
    let height = root.position.height as u32;
    let renderer = HeadlessRenderer::new();
    let rgba = renderer.capture_rgba_buffer(root, width, height);

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer
        .write_image_data(&rgba)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

#[cfg(feature = "egui-backend")]
pub mod egui_backend {
    use super::*;
    use mlapp_host::{AppSession, HostNotification};
    use mlapp_runtime::UiMutation;
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

    use crate::autotest::{
        find_component, find_component_mut, matches_property_value, parse_property_value,
        AutotestAction, AutotestScript,
    };
    use std::sync::{Arc, Mutex};

    pub struct AutotestRunnerState {
        pub script: AutotestScript,
        pub current_action: usize,
        pub step_deadline: Option<std::time::Instant>,
        pub expect_started: Option<std::time::Instant>,
        pub finished: bool,
    }

    /// Interactive egui application window running an AppSession.
    pub struct EguiAppWindow {
        pub session: AppSession,
        pub renderer: EguiRenderer,
        started: std::time::Instant,
        auto_close_after: Option<std::time::Duration>,
        pub autotest: Option<AutotestRunnerState>,
        pub shared_error: Option<Arc<Mutex<Option<String>>>>,
    }

    impl EguiAppWindow {
        pub fn new(session: AppSession) -> Self {
            Self::with_autotest(session, None, None)
        }

        pub fn with_autotest(
            session: AppSession,
            autotest: Option<AutotestScript>,
            shared_error: Option<Arc<Mutex<Option<String>>>>,
        ) -> Self {
            let auto_close_after = std::env::var("MLAPP_GUI_AUTOCLOSE_MS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .filter(|ms| *ms > 0)
                .map(std::time::Duration::from_millis);

            let autotest_state = autotest.map(|script| AutotestRunnerState {
                script,
                current_action: 0,
                step_deadline: None,
                expect_started: None,
                finished: false,
            });

            Self {
                session,
                renderer: EguiRenderer::new(),
                started: std::time::Instant::now(),
                auto_close_after,
                autotest: autotest_state,
                shared_error,
            }
        }
    }

    impl eframe::App for EguiAppWindow {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            while let Some(notif) = self.session.poll_notification() {
                match notif {
                    HostNotification::Mutations(mutations) => {
                        for m in mutations {
                            match m {
                                UiMutation::SetProperty {
                                    component,
                                    property,
                                    value,
                                } => {
                                    if let Some(comp) =
                                        find_component_mut(&mut self.session.ui_tree, &component)
                                    {
                                        let _ = comp.set_property(&property, value);
                                    }
                                }
                                UiMutation::Alert { title, message } => {
                                    eprintln!("[Alert Dialog] {title}: {message}");
                                }
                                UiMutation::PlotCall {
                                    axes,
                                    command,
                                    args,
                                } => {
                                    let num_points = args
                                        .iter()
                                        .filter_map(|v| v.to_double_array().ok())
                                        .map(|a| a.numel())
                                        .max()
                                        .unwrap_or(0);
                                    let axes_name = axes.strip_prefix("app.").unwrap_or(&axes);
                                    if let Some(comp) =
                                        find_component_mut(&mut self.session.ui_tree, axes_name)
                                    {
                                        let _ = comp.set_property(
                                            "HasPlottedData",
                                            mlapp_ui::PropertyValue::Boolean(true),
                                        );
                                        let _ = comp.set_property(
                                            "NumPoints",
                                            mlapp_ui::PropertyValue::Number(num_points as f64),
                                        );
                                        let _ = comp.set_property(
                                            "PlotCommand",
                                            mlapp_ui::PropertyValue::String(command),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    HostNotification::Alert { title, message } => {
                        eprintln!("[Alert Dialog] {title}: {message}");
                    }
                    HostNotification::Error(e) => {
                        eprintln!("[Worker Error]: {e}");
                    }
                    HostNotification::Terminated => {}
                }
            }

            if let Some(runner) = &mut self.autotest {
                if !runner.finished {
                    if runner.current_action >= runner.script.actions.len() {
                        runner.finished = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else {
                        let action = &runner.script.actions[runner.current_action];
                        match action {
                            AutotestAction::WaitMs { duration_ms } => {
                                let now = std::time::Instant::now();
                                if runner.step_deadline.is_none() {
                                    runner.step_deadline =
                                        Some(now + std::time::Duration::from_millis(*duration_ms));
                                } else if let Some(deadline) = runner.step_deadline {
                                    if now >= deadline {
                                        runner.step_deadline = None;
                                        runner.current_action += 1;
                                    }
                                }
                            }
                            AutotestAction::Click { component } => {
                                if find_component(&self.session.ui_tree, component).is_none() {
                                    if let Some(shared) = &self.shared_error {
                                        if let Ok(mut g) = shared.lock() {
                                            *g = Some(format!(
                                                "Component '{component}' not found in UI tree"
                                            ));
                                        }
                                    }
                                    runner.finished = true;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                } else {
                                    println!("[Autotest GUI] Click '{component}'");
                                    let _ = self.session.send_event(UiEvent::ButtonClick {
                                        component_name: component.clone(),
                                    });
                                    runner.current_action += 1;
                                }
                            }
                            AutotestAction::SetValue { component, value } => {
                                let parsed = parse_property_value(value);
                                if let Some(comp) =
                                    find_component_mut(&mut self.session.ui_tree, component)
                                {
                                    let _ = comp.set_property("Value", parsed.clone());
                                }
                                println!("[Autotest GUI] SetValue '{component}' = '{value}'");
                                let _ = self.session.send_event(UiEvent::ValueChanged {
                                    component_name: component.clone(),
                                    new_val: parsed,
                                });
                                runner.current_action += 1;
                            }
                            AutotestAction::ExpectProperty {
                                component,
                                property,
                                expected,
                            } => {
                                let now = std::time::Instant::now();
                                if runner.expect_started.is_none() {
                                    runner.expect_started = Some(now);
                                }
                                let mut matched = false;
                                if let Some(comp) = find_component(&self.session.ui_tree, component)
                                {
                                    if let Some(val) = comp.get_property(property) {
                                        if matches_property_value(val, expected) {
                                            matched = true;
                                        }
                                    }
                                }

                                if matched {
                                    println!(
                                        "[Autotest GUI] ExpectProperty matched '{component}.{property}' == '{expected}'"
                                    );
                                    runner.expect_started = None;
                                    runner.current_action += 1;
                                } else if let Some(started) = runner.expect_started {
                                    if now.duration_since(started)
                                        >= std::time::Duration::from_millis(3000)
                                    {
                                        let actual =
                                            find_component(&self.session.ui_tree, component)
                                                .and_then(|c| c.get_property(property))
                                                .map(|v| format!("{v:?}"))
                                                .unwrap_or_else(|| "<none>".to_string());
                                        if let Some(shared) = &self.shared_error {
                                            if let Ok(mut g) = shared.lock() {
                                                *g = Some(format!(
                                                    "ExpectProperty timeout for '{component}.{property}': expected '{expected}', got '{actual}'"
                                                ));
                                            }
                                        }
                                        runner.finished = true;
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                }
                            }
                            AutotestAction::Screenshot { path } => {
                                println!("[Autotest GUI] Screenshot saving to {:?}", path);
                                if let Err(e) =
                                    crate::save_screenshot_png(&self.session.ui_tree, path)
                                {
                                    if let Some(shared) = &self.shared_error {
                                        if let Ok(mut g) = shared.lock() {
                                            *g = Some(format!("Failed to save screenshot: {e}"));
                                        }
                                    }
                                    runner.finished = true;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                } else {
                                    runner.current_action += 1;
                                }
                            }
                            AutotestAction::Close => {
                                println!("[Autotest GUI] Close action");
                                runner.finished = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(&self.session.manifest.app_name);
                ui.separator();
                let parent_h = self.session.ui_tree.position.height;
                self.renderer.render(ui, &self.session.ui_tree, parent_h);
            });

            for ev in self.renderer.pending_events.drain(..) {
                let _ = self.session.send_event(ev);
            }

            if let Some(limit) = self.auto_close_after {
                if self.started.elapsed() >= limit {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }

    /// Launches an interactive eframe GUI window for the app session with optional autotest.
    pub fn run_gui_window_with_autotest(
        session: AppSession,
        autotest: Option<AutotestScript>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let title = session.manifest.app_name.clone();
        let w = session.ui_tree.position.width.max(640.0) as f32;
        let h = session.ui_tree.position.height.max(480.0) as f32;

        let shared_err = Arc::new(Mutex::new(None));
        let shared_err_clone = Arc::clone(&shared_err);

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([w, h])
                .with_title(&title),
            ..Default::default()
        };

        eframe::run_native(
            &title,
            native_options,
            Box::new(move |_cc| {
                Ok(Box::new(EguiAppWindow::with_autotest(
                    session,
                    autotest,
                    Some(shared_err_clone),
                )))
            }),
        )
        .map_err(|e| {
            Box::new(std::io::Error::other(e.to_string()))
                as Box<dyn std::error::Error + Send + Sync>
        })?;

        if let Ok(guard) = shared_err.lock() {
            if let Some(err_msg) = &*guard {
                return Err(format!("Autotest GUI failure: {err_msg}").into());
            }
        }

        Ok(())
    }

    /// Launches an interactive eframe GUI window for the app session.
    pub fn run_gui_window(
        session: AppSession,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        run_gui_window_with_autotest(session, None)
    }
}
