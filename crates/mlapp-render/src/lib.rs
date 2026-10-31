//! Rendering abstraction and backend implementations (headless mock & egui).

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

impl HeadlessRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_click(&mut self, component_name: &str) {
        self.pending_events.push(UiEvent::ButtonClick {
            component_name: component_name.to_string(),
        });
    }

    fn render_node(&mut self, comp: &Component, ctx: &RenderContext) {
        let top_left = if comp.kind == ComponentKind::UIFigure || comp.parent.is_none() {
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
            self.render_node(child, &child_ctx);
        }
    }
}

impl RenderBackend for HeadlessRenderer {
    fn render_tree(&mut self, root: &Component) -> Vec<UiEvent> {
        let ctx = RenderContext {
            parent_height: root.position.height,
            parent_width: root.position.width,
        };
        self.render_node(root, &ctx);
        std::mem::take(&mut self.pending_events)
    }

    fn name(&self) -> &'static str {
        "headless"
    }
}

#[cfg(feature = "egui-backend")]
pub mod egui_backend {
    use super::*;

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
            let top_left = matlab_to_top_left_geometry(comp.position, parent_height);

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
                _ => {
                    for child in &comp.children {
                        self.render(ui, child, comp.position.height);
                    }
                }
            }
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
