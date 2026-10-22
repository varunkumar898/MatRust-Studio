//! Bottom status bar: execution state, active file, working directory,
//! cursor position, last run duration, theme indicator.

use std::time::Duration;

use crate::theme::{self, ThemeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Ready,
    Running,
    Error,
}

impl Default for RunState {
    fn default() -> Self {
        RunState::Ready
    }
}

impl RunState {
    fn dot_color(&self, kind: ThemeKind) -> egui::Color32 {
        match self {
            RunState::Ready => theme::success_color(kind),
            RunState::Running => theme::accent(kind),
            RunState::Error => theme::error_color(kind),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            RunState::Ready => "Ready",
            RunState::Running => "Running…",
            RunState::Error => "Error",
        }
    }
}

#[derive(Debug, Default)]
pub struct StatusLine {
    pub run_state: RunState,
    pub active_file: Option<String>,
    pub modified: bool,
    pub working_dir: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub last_duration: Option<Duration>,
    pub theme_name: &'static str,
}

pub fn show(ui: &mut egui::Ui, s: &StatusLine, kind: ThemeKind) {
    ui.spacing_mut().item_spacing.x = 10.0;
    ui.horizontal(|ui| {
        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(dot_rect, 4.5, s.run_state.dot_color(kind));
        ui.weak(s.run_state.label());
        ui.weak(s.working_dir.clone());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.weak(s.theme_name);
            if let Some(d) = s.last_duration {
                ui.weak(format!("last run {:.1} ms", d.as_secs_f64() * 1000.0));
            }
            ui.weak(format!("Ln {}, Col {}", s.cursor_line, s.cursor_col));
            let file_prefix = if s.modified { "* " } else { "" };
            let file = s.active_file.as_deref().unwrap_or("no file");
            ui.weak(format!("{}{}", file_prefix, file));
        });
    });
}
