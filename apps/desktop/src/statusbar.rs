//! Bottom status bar: engine state, cwd, active file, cursor, diagnostics, theme.
//!
//! GUI-CLEAN keeps every value plain ASCII so the bar is readable with the
//! bundled monospace/proportional fonts even on a clean Windows install.

use crate::theme::Theme;
use egui::Ui;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum EngineStatus {
    Ready,
    Running(Instant),
    Paused(usize),
    Error(String),
}

/// A user action requested by clicking the status bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusAction {
    /// Move the editor cursor to the first syntax problem.
    JumpToFirstError,
}

pub struct StatusBar {
    pub status: EngineStatus,
    pub active_file: Option<PathBuf>,
    /// Display title from the active editor tab (`Untitled1.m`, `demo.m`, ...).
    pub active_title: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub working_dir: PathBuf,
    pub error_count: usize,
    pub warning_count: usize,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            status: EngineStatus::Ready,
            active_file: None,
            active_title: String::new(),
            cursor_line: 1,
            cursor_col: 1,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            error_count: 0,
            warning_count: 0,
        }
    }
}

impl StatusBar {
    /// Active document name; an open untitled tab still has a name.
    pub fn active_file_label(&self) -> String {
        if !self.active_title.is_empty() {
            return self.active_title.clone();
        }
        if let Some(path) = &self.active_file {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                return name.to_string();
            }
        }
        "No file".to_string()
    }

    /// Render the bar. Returns the single click action, if any.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) -> Option<StatusAction> {
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            let (state_color, state_text) = match &self.status {
                EngineStatus::Ready => (theme.success, "Ready".to_string()),
                EngineStatus::Running(start) => (
                    theme.warning,
                    format!("Busy {:.1}s", start.elapsed().as_secs_f32()),
                ),
                EngineStatus::Paused(line) => (theme.accent, format!("Paused line {line}")),
                EngineStatus::Error(err) => {
                    let short_err = if err.chars().count() > 60 {
                        let mut short: String = err.chars().take(57).collect();
                        short.push_str("...");
                        short
                    } else {
                        err.clone()
                    };
                    (theme.error, format!("Error: {short_err}"))
                }
            };
            ui.colored_label(state_color, state_text);

            ui.separator();
            let dir = self.working_dir.display().to_string();
            ui.label(format!("Dir: {dir}")).on_hover_text(dir);

            ui.separator();
            ui.label(format!("File: {}", self.active_file_label()));

            ui.separator();
            ui.label(format!(
                "Line {}, Col {}",
                self.cursor_line, self.cursor_col
            ));

            ui.separator();
            if self.error_count > 0 {
                let errors = ui
                    .button(format!("Errors: {}", self.error_count))
                    .on_hover_text("Jump to the first syntax problem");
                let _ =
                    crate::ui_registry::register(ui.ctx(), "status.errors", errors.id, errors.rect);
                if errors.clicked() {
                    action = Some(StatusAction::JumpToFirstError);
                }
            } else {
                let errors = ui.label("Errors: 0");
                let _ =
                    crate::ui_registry::register(ui.ctx(), "status.errors", errors.id, errors.rect);
            }
            ui.label(format!("Warnings: {}", self.warning_count));

            ui.separator();
            let theme_name = match theme.mode {
                crate::theme::ThemeMode::Dark => "Dark",
                crate::theme::ThemeMode::Light => "Light",
            };
            ui.label(format!("Theme: {theme_name}"));
        });
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_untitled_tab_never_says_no_active_file() {
        let mut bar = StatusBar {
            active_title: "Untitled1.m".to_string(),
            ..Default::default()
        };
        assert_eq!(bar.active_file_label(), "Untitled1.m");
        bar.active_title.clear();
        bar.active_file = Some(PathBuf::from("demo.m"));
        assert_eq!(bar.active_file_label(), "demo.m");
    }

    #[test]
    fn status_bar_labels_are_ascii() {
        let bar = StatusBar::default();
        assert!(bar.active_file_label().is_ascii());
    }
}
