//! Command window: the interactive numerical prompt (`>>`), transcript, and
//! command history. Everything displayed comes from executions of the
//! persistent session; no fake state.
//!
//! Layout contract for the bottom panel: the ENTIRE content is one scroll
//! block (transcript + the inline `>>` prompt as its last row) — the prompt
//! scrolls with the history and sits at the bottom exactly like in the
//! numerical desktop being reproduced. Nothing may be painted after the
//! scroll area: if a widget follows the scroll inside a resizable bottom
//! panel, the panel's outer rect is expanded by the overflow, persisted, and
//! grows unboundedly across frames (egui 0.37 `Panel::show` uses the frame's
//! content rect as the panel's outer rect). Keeping the whole body inside a
//! single scroll block is load-bearing, not cosmetic. Widgets BEFORE the
//! scroll (a header row) are safe: the scroll then fills the exact remaining
//! height.

use egui::{Color32, Event, Key, TextEdit, TextStyle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// Echo of the submitted command.
    Echo(String),
    /// Regular interpreter output (`disp`, `ans`, assignments).
    Out(String),
    /// Execution/parse diagnostics.
    Err(String),
    /// Informational (hints, `who` output, run banners).
    Info(String),
}

#[derive(Debug, Default)]
pub struct CommandWindowState {
    pub lines: Vec<Line>,
    /// The current, not-yet-submitted command line.
    pub input: String,
    pub history: Vec<String>,
    /// Index into `history` while browsing with the arrow keys.
    history_pos: Option<usize>,
    /// The in-progress draft saved when history browsing starts.
    input_saved: Option<String>,
    /// Focus the input on this frame (set after a reply arrives).
    pub wants_focus: bool,
    /// Whether the prompt widget had keyboard focus during the previous pass
    /// (used to gate pre-read Enter/arrow keys; see `show_body`).
    prompt_focus: bool,
}

impl CommandWindowState {
    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
        if self.lines.len() > 2000 {
            self.lines.drain(..self.lines.len() - 2000);
        }
    }

    pub fn remember_history(&mut self, command: &str) {
        if command.is_empty() {
            return;
        }
        if self.history.last().map(|h| h == command).unwrap_or(false) {
            return;
        }
        self.history.push(command.to_string());
        if self.history.len() > 200 {
            self.history.remove(0);
        }
        self.history_pos = None;
    }

    fn browse_history(&mut self, up: bool) {
        if self.history.is_empty() {
            return;
        }
        if up {
            if self.history_pos.is_none() {
                self.input_saved = Some(self.input.clone());
            }
            let idx = match self.history_pos {
                None => self.history.len() - 1,
                Some(i) => i.saturating_sub(1),
            };
            self.history_pos = Some(idx);
            self.input = self.history[idx].clone();
        } else if let Some(i) = self.history_pos {
            if i + 1 < self.history.len() {
                self.history_pos = Some(i + 1);
                self.input = self.history[i + 1].clone();
            } else {
                self.history_pos = None;
                self.input = self.input_saved.take().unwrap_or_default();
            }
        }
    }
}

/// What the app should do after Enter was pressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Submitted {
    /// Execute the command in the persistent session.
    Command(String),
    /// `who` — handled by the app from the workspace snapshot.
    Who,
    /// `clear` — clear all variables.
    ClearWorkspace,
    /// `exit` / `quit`.
    Exit,
}

/// Render the transcript + inline prompt as ONE scroll block. Callers may put
/// a header row before this, but must not add widgets after it.
/// Returns an action when a command was submitted.
pub fn show_body(
    ui: &mut egui::Ui,
    state: &mut CommandWindowState,
    kind: crate::theme::ThemeKind,
) -> Option<Submitted> {
    let mut submitted: Option<Submitted> = None;

    // The singleline TextEdit consumes the Enter event while it is shown
    // (egui 0.37 gives it a default `return_key`), so the press must be read
    // BEFORE the widget is added. Gated on the prompt widget's focus from the
    // previous pass, tracked in `prompt_focus`.
    let was_focused = state.prompt_focus;
    let (enter_pressed, up_pressed, down_pressed) = if was_focused {
        ui.input(|i| {
            let enter = i.events.iter().any(|e| {
                matches!(
                    e,
                    Event::Key {
                        key: Key::Enter,
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.shift
                )
            });
            let up = i.events.iter().any(|e| {
                matches!(
                    e,
                    Event::Key {
                        key: Key::ArrowUp,
                        pressed: true,
                        ..
                    }
                )
            });
            let down = i.events.iter().any(|e| {
                matches!(
                    e,
                    Event::Key {
                        key: Key::ArrowDown,
                        pressed: true,
                        ..
                    }
                )
            });
            (enter, up, down)
        })
    } else {
        (false, false, false)
    };
    state.prompt_focus = false; // refreshed by the widget pass below

    egui::ScrollArea::vertical()
        .id_salt("matrust_cmd_transcript")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            if state.lines.is_empty() {
                ui.monospace(
                    egui::RichText::new("Enter commands below, e.g. C = A * B.")
                        .color(crate::theme::text_color_faint(kind)),
                );
            }
            for line in &state.lines {
                let (text, color): (String, Color32) = match line {
                    Line::Echo(t) => (format!(">> {}", t), crate::theme::accent(kind)),
                    Line::Out(t) => (t.clone(), crate::theme::text_color(kind)),
                    Line::Err(t) => (t.clone(), crate::theme::error_color(kind)),
                    Line::Info(t) => (t.clone(), crate::theme::text_color_faint(kind)),
                };
                ui.monospace(egui::RichText::new(text).color(color));
            }

            // ---- inline prompt: the last row of the same scroll block ----
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.monospace(
                    egui::RichText::new(">> ")
                        .monospace()
                        .color(crate::theme::accent(kind)),
                );
                let response = ui.add(
                    TextEdit::singleline(&mut state.input)
                        .id_salt("matrust_cmd_prompt")
                        .font(TextStyle::Monospace)
                        .frame(egui::Frame::NONE)
                        .desired_width(ui.available_width().max(80.0)),
                );
                state.prompt_focus = response.has_focus();
                if state.wants_focus {
                    state.wants_focus = false;
                    response.request_focus();
                }
                // Apply the pre-read navigation keys.
                if up_pressed {
                    state.browse_history(true);
                }
                if down_pressed {
                    state.browse_history(false);
                }
                if enter_pressed {
                    submitted = Some(Submitted::Command(String::new()));
                }
            });
        });

    let Some(Submitted::Command(_)) = submitted else {
        return None;
    };

    // Enter was pressed: gather the line, echo it, and classify it.
    let command = state.input.trim().to_string();
    state.input.clear();
    state.history_pos = None;
    if command.is_empty() {
        return None;
    }
    state.push(Line::Echo(command.clone()));
    state.remember_history(&command);
    match command.as_str() {
        "who" => Some(Submitted::Who),
        "clear" => Some(Submitted::ClearWorkspace),
        "clc" => {
            state.lines.clear();
            None
        }
        "exit" | "quit" => Some(Submitted::Exit),
        _ => Some(Submitted::Command(command)),
    }
}
