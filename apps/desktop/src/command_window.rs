//! Command Window interactive console and execution command history.

use crate::actions::AppAction;
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
        // G2: no watermark banner; the empty state is a single hint line.
        let welcome = vec![ConsoleLogEntry {
            kind: ConsoleLogKind::Output,
            text: "Type commands at the >> prompt. Type 'help' for builtins.\n".to_string(),
        }];
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

    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        on_execute: &mut Option<String>,
        on_action: &mut Option<AppAction>,
    ) {
        ui.vertical(|ui| {
            // Header bar. The action wraps below the heading when the dock
            // column is narrow; it is never clipped or duplicated.
            ui.horizontal_wrapped(|ui| {
                ui.heading("Command Window");
                let clear = ui
                    .button("Clear Command Window")
                    .on_hover_text("Clear only the Command Window log");
                AppAction::ClearCommandWindow.register_control(ui.ctx(), clear.id, clear.rect);
                if clear.clicked() {
                    *on_action = Some(AppAction::ClearCommandWindow);
                }
            });

            ui.separator();

            // The input strip is allocated from the bottom first, so a very
            // short dock panel can never push the only way to type below the
            // visible edge. The log then fills exactly what is left.
            egui::TopBottomPanel::bottom("command_input_strip")
                .frame(egui::Frame::NONE)
                .exact_height(32.0)
                .show_inside(ui, |ui| {
                    ui.separator();
                    self.show_input_row(ui, theme, on_execute);
                });

            let mut scroll = ScrollArea::vertical()
                .id_salt("command_window_scroll")
                .auto_shrink([false, false]);

            if self.auto_scroll {
                scroll = scroll.stick_to_bottom(true);
            }

            scroll.show(ui, |ui| {
                // G5: stable per-row ids keyed by the log index, so UI ids can
                // never collide across rows or panels.
                for (row_idx, entry) in self.logs.iter().enumerate() {
                    ui.push_id(("cw_row", row_idx), |ui| match entry.kind {
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
                    });
                }
            });
        });
    }

    fn show_input_row(&mut self, ui: &mut Ui, theme: &Theme, on_execute: &mut Option<String>) {
        // Interactive input line: `>> `
        ui.horizontal(|ui| {
            ui.colored_label(theme.accent, ">>");

            let response = ui.add_sized(
                [ui.available_width() - 80.0, 24.0],
                TextEdit::singleline(&mut self.input_buffer)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("Enter expression or statement..."),
            );
            let _ =
                crate::ui_registry::register(ui.ctx(), "command.input", response.id, response.rect);

            // History navigation only when the command input has focus, so
            // ArrowUp/ArrowDown in the editor or elsewhere don't rewrite this
            // buffer.
            let input_focused = response.has_focus();
            let up_pressed = input_focused && ui.input(|i| i.key_pressed(Key::ArrowUp));
            let down_pressed = input_focused && ui.input(|i| i.key_pressed(Key::ArrowDown));

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

            let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            let run = ui.button("Run");
            let _ = crate::ui_registry::register(ui.ctx(), "command.run", run.id, run.rect);
            let run_clicked = run.clicked();

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

            // Only request focus when the panel is empty and nothing else has
            // focus; never steal focus from the editor or another panel.
            // Removed auto-focus: never steal focus from the editor or any
            // other panel. The user clicks into the command window (or tabs)
            // to focus it; the hint text guides them.
        });
    }

    /// Renders the standalone Command History panel.
    pub fn show_history_panel(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        on_execute: &mut Option<String>,
        on_action: &mut Option<AppAction>,
    ) {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Command History");
                let clear = ui.button("Clear History");
                AppAction::ClearHistory.register_control(ui.ctx(), clear.id, clear.rect);
                if clear.clicked() {
                    *on_action = Some(AppAction::ClearHistory);
                }
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
