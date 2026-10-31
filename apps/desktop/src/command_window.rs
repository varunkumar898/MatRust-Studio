//! Command Window interactive console and execution command history.

use crate::theme::Theme;
use egui::{Key, RichText, ScrollArea, TextEdit, Ui};

#[derive(Clone, Debug)]
pub enum ConsoleLogKind {
    Input,
    Output,
    Error,
}

#[derive(Clone, Debug)]
pub struct ConsoleLogEntry {
    pub kind: ConsoleLogKind,
    pub text: String,
}

pub struct CommandWindow {
    pub logs: Vec<ConsoleLogEntry>,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_cursor: Option<usize>,
    pub auto_scroll: bool,
}

impl Default for CommandWindow {
    fn default() -> Self {
        let welcome = vec![
            ConsoleLogEntry {
                kind: ConsoleLogKind::Output,
                text: "MatRust Studio Numerical Computing Environment\n".to_string(),
            },
            ConsoleLogEntry {
                kind: ConsoleLogKind::Output,
                text: "Type commands at the >> prompt. Type 'help' for builtins.\n\n".to_string(),
            },
        ];
        Self {
            logs: welcome,
            input_buffer: String::new(),
            history: Vec::new(),
            history_cursor: None,
            auto_scroll: true,
        }
    }
}

impl CommandWindow {
    pub fn push_output(&mut self, text: String) {
        if !text.is_empty() {
            self.logs.push(ConsoleLogEntry {
                kind: ConsoleLogKind::Output,
                text,
            });
            self.auto_scroll = true;
        }
    }

    pub fn push_error(&mut self, text: String) {
        if !text.is_empty() {
            self.logs.push(ConsoleLogEntry {
                kind: ConsoleLogKind::Error,
                text,
            });
            self.auto_scroll = true;
        }
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, on_execute: &mut Option<String>) {
        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.heading("Command Window");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("Clear Console")
                        .on_hover_text("Clear console buffer (clc)")
                        .clicked()
                    {
                        self.clear();
                    }
                });
            });

            ui.separator();

            // Scrollable console log area
            let available_height = ui.available_height() - 36.0;
            let mut scroll = ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(available_height);

            if self.auto_scroll {
                scroll = scroll.stick_to_bottom(true);
            }

            scroll.show(ui, |ui| {
                for entry in &self.logs {
                    match entry.kind {
                        ConsoleLogKind::Input => {
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.accent, ">>");
                                ui.monospace(RichText::new(&entry.text).color(theme.text_primary));
                            });
                        }
                        ConsoleLogKind::Output => {
                            ui.monospace(RichText::new(&entry.text).color(theme.text_primary));
                        }
                        ConsoleLogKind::Error => {
                            ui.monospace(RichText::new(&entry.text).color(theme.error));
                        }
                    }
                }
            });

            ui.separator();

            // Interactive input line: `>> `
            ui.horizontal(|ui| {
                ui.colored_label(theme.accent, ">>");

                // History navigation
                let up_pressed = ui.input(|i| i.key_pressed(Key::ArrowUp));
                let down_pressed = ui.input(|i| i.key_pressed(Key::ArrowDown));

                if up_pressed && !self.history.is_empty() {
                    let next_cursor = match self.history_cursor {
                        None => self.history.len().saturating_sub(1),
                        Some(0) => 0,
                        Some(c) => c - 1,
                    };
                    self.history_cursor = Some(next_cursor);
                    self.input_buffer = self.history[next_cursor].clone();
                } else if down_pressed {
                    if let Some(c) = self.history_cursor {
                        if c + 1 < self.history.len() {
                            let next = c + 1;
                            self.history_cursor = Some(next);
                            self.input_buffer = self.history[next].clone();
                        } else {
                            self.history_cursor = None;
                            self.input_buffer.clear();
                        }
                    }
                }

                let response = ui.add_sized(
                    [ui.available_width() - 80.0, 24.0],
                    TextEdit::singleline(&mut self.input_buffer)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Enter expression or statement..."),
                );

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                let run_clicked = ui.button("▶ Run").clicked();

                if enter_pressed || run_clicked {
                    let trimmed = self.input_buffer.trim().to_string();
                    if !trimmed.is_empty() {
                        // Handle `clc` builtin directly
                        if trimmed == "clc" {
                            self.clear();
                            self.input_buffer.clear();
                            self.history_cursor = None;
                            return;
                        }

                        // Echo into console
                        self.logs.push(ConsoleLogEntry {
                            kind: ConsoleLogKind::Input,
                            text: trimmed.clone(),
                        });

                        // Record in history
                        self.history.push(trimmed.clone());
                        self.history_cursor = None;

                        *on_execute = Some(trimmed);
                        self.input_buffer.clear();
                        self.auto_scroll = true;
                    }
                }

                if !response.has_focus() && self.input_buffer.is_empty() {
                    response.request_focus();
                }
            });
        });
    }

    /// Renders the standalone Command History panel.
    pub fn show_history_panel(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        on_execute: &mut Option<String>,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Command History");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() {
                        self.history.clear();
                    }
                });
            });
            ui.separator();

            if self.history.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("No command history")
                            .italics()
                            .color(theme.text_secondary),
                    );
                });
                return;
            }

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for cmd in self.history.iter().rev() {
                        let resp = ui.selectable_label(false, RichText::new(cmd).monospace());
                        if resp.double_clicked() {
                            *on_execute = Some(cmd.clone());
                        }
                    }
                });
        });
    }
}
