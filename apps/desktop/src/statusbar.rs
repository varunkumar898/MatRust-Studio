//! Bottom status bar displaying engine status, cursor location, and working directory.

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

pub struct StatusBar {
    pub status: EngineStatus,
    pub active_file: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub working_dir: PathBuf,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            status: EngineStatus::Ready,
            active_file: None,
            cursor_line: 1,
            cursor_col: 1,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

impl StatusBar {
    pub fn show(&self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;

            // Engine status indicator
            match &self.status {
                EngineStatus::Ready => {
                    ui.colored_label(theme.success, "● Ready");
                }
                EngineStatus::Running(start) => {
                    let elapsed = start.elapsed().as_secs_f32();
                    ui.colored_label(theme.warning, format!("⏳ Busy ({elapsed:.1}s)"));
                }
                EngineStatus::Paused(line) => {
                    ui.colored_label(theme.accent, format!("⏸ Paused (Line {line})"));
                }
                EngineStatus::Error(err) => {
                    let short_err = if err.len() > 40 { &err[..37] } else { err };
                    ui.colored_label(theme.error, format!("✕ Error: {short_err}"));
                }
            }

            ui.separator();

            // Working directory
            let dir_display = self
                .working_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(self.working_dir.to_str().unwrap_or("."));
            ui.label(format!("📁 {}", dir_display));

            ui.separator();

            // Active file
            if let Some(ref path) = self.active_file {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled.m");
                ui.label(format!("📄 {name}"));
            } else {
                ui.label("📄 No active file");
            }

            // Right-aligned cursor position and theme
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(match theme.mode {
                    crate::theme::ThemeMode::Dark => "🌙 Dark",
                    crate::theme::ThemeMode::Light => "☀️ Light",
                });
                ui.separator();
                ui.label(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col));
            });
        });
    }
}
