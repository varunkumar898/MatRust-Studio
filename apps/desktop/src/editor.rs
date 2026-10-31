//! Multi-document script editor with syntax formatting, gutter breakpoints, code folding, and completion.

use crate::execution::VariableRow;
use crate::theme::Theme;
use egui::{Key, RichText, ScrollArea, TextEdit, TextStyle, Ui, Vec2};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// A foldable code block defined by start and end lines (1-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
    pub kind: &'static str,
}

/// Identifies all syntactically foldable language blocks in source code.
pub fn find_foldable_blocks(source: &str) -> Vec<FoldRange> {
    let mut blocks = Vec::new();
    let mut stack: Vec<(usize, &'static str)> = Vec::new();

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        // Strip comment
        let line = raw_line.split('%').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let first_word = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");

        match first_word {
            "function" => stack.push((line_no, "function")),
            "for" => stack.push((line_no, "for")),
            "while" => stack.push((line_no, "while")),
            "if" => stack.push((line_no, "if")),
            "switch" => stack.push((line_no, "switch")),
            "try" => stack.push((line_no, "try")),
            _ => {}
        }

        // Check for closing 'end'
        let words: Vec<&str> = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty())
            .collect();

        if words.contains(&"end")
            && (first_word == "end"
                || line.ends_with("end")
                || line.contains("end;")
                || line.contains("end,"))
        {
            if let Some((start_line, kind)) = stack.pop() {
                if line_no > start_line {
                    blocks.push(FoldRange {
                        start_line,
                        end_line: line_no,
                        kind,
                    });
                }
            }
        }
    }
    blocks
}

#[derive(Clone, Debug)]
pub struct DocumentTab {
    pub path: Option<PathBuf>,
    pub title: String,
    pub content: String,
    pub is_dirty: bool,
    pub breakpoints: HashSet<usize>,
    pub folded_blocks: HashSet<usize>, // Set of start lines folded
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub error_line: Option<usize>,
}

impl DocumentTab {
    pub fn new_untitled(idx: usize) -> Self {
        let initial_script = format!(
            "% Script Untitled{idx}.m\n% MatRust Studio Script\n\nx = linspace(0, 2*pi, 100);\ny = sin(x);\n\nfigure(1);\nplot(x, y);\ntitle('Sine Wave');\ngrid on;\n"
        );
        Self {
            path: None,
            title: format!("Untitled{idx}.m"),
            content: initial_script,
            is_dirty: true,
            breakpoints: HashSet::new(),
            folded_blocks: HashSet::new(),
            cursor_line: 1,
            cursor_col: 1,
            error_line: None,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read file '{}': {e}", path.display()))?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Script.m")
            .to_string();

        Ok(Self {
            path: Some(path),
            title,
            content,
            is_dirty: false,
            breakpoints: HashSet::new(),
            folded_blocks: HashSet::new(),
            cursor_line: 1,
            cursor_col: 1,
            error_line: None,
        })
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(ref path) = self.path {
            fs::write(path, &self.content)
                .map_err(|e| format!("Cannot save file '{}': {e}", path.display()))?;
            self.is_dirty = false;
            Ok(())
        } else {
            Err("File has no assigned path (use Save As)".to_string())
        }
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<(), String> {
        fs::write(&path, &self.content)
            .map_err(|e| format!("Cannot save file '{}': {e}", path.display()))?;
        self.title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Script.m")
            .to_string();
        self.path = Some(path);
        self.is_dirty = false;
        Ok(())
    }
}

pub struct EditorPanel {
    pub tabs: Vec<DocumentTab>,
    pub active_tab_idx: usize,
    pub next_untitled_id: usize,

    // Code completion state
    pub completion_open: bool,
    pub completion_filter: String,
    pub completion_selected_idx: usize,
    pub completion_candidates: Vec<String>,
}

impl Default for EditorPanel {
    fn default() -> Self {
        let first = DocumentTab::new_untitled(1);
        Self {
            tabs: vec![first],
            active_tab_idx: 0,
            next_untitled_id: 2,
            completion_open: false,
            completion_filter: String::new(),
            completion_selected_idx: 0,
            completion_candidates: Vec::new(),
        }
    }
}

impl EditorPanel {
    pub fn active_doc(&self) -> Option<&DocumentTab> {
        self.tabs.get(self.active_tab_idx)
    }

    pub fn active_doc_mut(&mut self) -> Option<&mut DocumentTab> {
        self.tabs.get_mut(self.active_tab_idx)
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<(), String> {
        if let Some(pos) = self
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            self.active_tab_idx = pos;
            return Ok(());
        }
        let doc = DocumentTab::from_file(path)?;
        self.tabs.push(doc);
        self.active_tab_idx = self.tabs.len() - 1;
        Ok(())
    }

    pub fn new_script(&mut self) {
        let doc = DocumentTab::new_untitled(self.next_untitled_id);
        self.next_untitled_id += 1;
        self.tabs.push(doc);
        self.active_tab_idx = self.tabs.len() - 1;
    }

    pub fn close_active_tab(&mut self) {
        if self.tabs.len() <= 1 {
            // Keep at least one tab
            self.tabs[0] = DocumentTab::new_untitled(self.next_untitled_id);
            self.next_untitled_id += 1;
            return;
        }
        self.tabs.remove(self.active_tab_idx);
        if self.active_tab_idx >= self.tabs.len() {
            self.active_tab_idx = self.tabs.len().saturating_sub(1);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        workspace_vars: &[VariableRow],
        on_run_script: &mut Option<String>,
        on_run_selection: &mut Option<String>,
    ) {
        ui.vertical(|ui| {
            // Document Tab Bar
            ui.horizontal(|ui| {
                let mut close_tab_idx = None;

                for (idx, tab) in self.tabs.iter().enumerate() {
                    let is_active = idx == self.active_tab_idx;
                    let tab_title = if tab.is_dirty {
                        format!("● {} *", tab.title)
                    } else {
                        tab.title.clone()
                    };

                    let btn = ui.selectable_label(is_active, tab_title);
                    if btn.clicked() {
                        self.active_tab_idx = idx;
                    }
                    if is_active && ui.small_button("✕").on_hover_text("Close tab").clicked() {
                        close_tab_idx = Some(idx);
                    }
                }

                if let Some(idx) = close_tab_idx {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(idx);
                        if self.active_tab_idx >= self.tabs.len() {
                            self.active_tab_idx = self.tabs.len() - 1;
                        }
                    }
                }

                if ui
                    .button("+")
                    .on_hover_text("New Script (Ctrl+N)")
                    .clicked()
                {
                    self.new_script();
                }

                // Run buttons on right of tab bar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("▶ Run Selection")
                        .on_hover_text("Execute selected text (F9)")
                        .clicked()
                    {
                        if let Some(doc) = self.active_doc() {
                            *on_run_selection = Some(doc.content.clone());
                        }
                    }
                    if ui
                        .button("▶ Run Script")
                        .on_hover_text("Execute active script (F5)")
                        .clicked()
                    {
                        if let Some(doc) = self.active_doc() {
                            *on_run_script = Some(doc.content.clone());
                        }
                    }
                });
            });

            ui.separator();

            let mut trigger_completion = false;
            let active_idx = self.active_tab_idx;
            if active_idx >= self.tabs.len() {
                return;
            }

            {
                let doc = &mut self.tabs[active_idx];
                let line_count = doc.content.lines().count().max(1);

                // Gutter + Text Edit Layout
                ui.horizontal(|ui| {
                    // Line Number Gutter with Breakpoints and Code Folding
                    let gutter_width = 62.0;
                    let blocks = find_foldable_blocks(&doc.content);

                    ui.allocate_ui_with_layout(
                        Vec2::new(gutter_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ScrollArea::vertical()
                                .id_salt("editor_gutter")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for line_no in 1..=line_count {
                                        let has_bp = doc.breakpoints.contains(&line_no);
                                        let is_err = doc.error_line == Some(line_no);
                                        let is_fold_start =
                                            blocks.iter().any(|b| b.start_line == line_no);
                                        let is_folded = doc.folded_blocks.contains(&line_no);

                                        let fold_icon = if is_fold_start {
                                            if is_folded {
                                                "▶"
                                            } else {
                                                "▼"
                                            }
                                        } else {
                                            " "
                                        };

                                        let bp_marker = if has_bp {
                                            "●"
                                        } else if is_err {
                                            "▶"
                                        } else {
                                            " "
                                        };

                                        let label_text =
                                            format!("{bp_marker}{fold_icon}{:>3}", line_no);
                                        let resp = ui.selectable_label(
                                            has_bp,
                                            RichText::new(label_text).monospace().color(
                                                if has_bp || is_err {
                                                    theme.error
                                                } else if is_fold_start {
                                                    theme.accent
                                                } else {
                                                    theme.gutter_fg
                                                },
                                            ),
                                        );

                                        if resp.clicked() {
                                            if is_fold_start {
                                                if is_folded {
                                                    doc.folded_blocks.remove(&line_no);
                                                } else {
                                                    doc.folded_blocks.insert(line_no);
                                                }
                                            } else if doc.breakpoints.contains(&line_no) {
                                                doc.breakpoints.remove(&line_no);
                                            } else {
                                                doc.breakpoints.insert(line_no);
                                            }
                                        }
                                    }
                                });
                        },
                    );

                    ui.separator();

                    // Main Monospaced Editor Text Area
                    let prev_content = doc.content.clone();
                    let _editor_resp = ScrollArea::both()
                        .id_salt("editor_content")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::multiline(&mut doc.content)
                                    .font(TextStyle::Monospace)
                                    .code_editor()
                                    .lock_focus(true),
                            )
                        });

                    if doc.content != prev_content {
                        doc.is_dirty = true;
                    }

                    // Handle Ctrl+Space for Code Completion
                    let ctrl_space = ui.input(|i| i.modifiers.command && i.key_pressed(Key::Space));
                    if ctrl_space {
                        trigger_completion = true;
                    }
                });
            }

            if trigger_completion {
                self.completion_open = true;
                self.populate_completion(workspace_vars);
            }

            // Completion Popup
            if self.completion_open {
                self.show_completion_popup(ui, theme);
            }
        });
    }

    fn populate_completion(&mut self, workspace_vars: &[VariableRow]) {
        let keywords = [
            "function",
            "end",
            "if",
            "else",
            "elseif",
            "for",
            "while",
            "return",
            "break",
            "continue",
            "switch",
            "case",
            "otherwise",
            "try",
            "catch",
            "global",
            "persistent",
        ];
        let builtins = [
            "zeros", "ones", "eye", "rand", "randn", "linspace", "plot", "scatter", "bar", "title",
            "xlabel", "ylabel", "grid", "figure", "clf", "close", "inv", "eig", "svd", "size",
            "length", "numel", "disp", "fprintf", "sprintf", "mean", "std", "var", "sum", "prod",
            "min", "max", "sort", "reshape", "cat",
        ];

        let mut items = Vec::new();
        for k in keywords {
            items.push(format!("keyword: {k}"));
        }
        for b in builtins {
            items.push(format!("fn: {b}()"));
        }
        for v in workspace_vars {
            items.push(format!("var: {} [{}]", v.name, v.type_name));
        }

        self.completion_candidates = items;
        self.completion_selected_idx = 0;
    }

    fn show_completion_popup(&mut self, ui: &mut Ui, _theme: &Theme) {
        egui::Window::new("Completion (Ctrl+Space)")
            .fixed_size([260.0, 200.0])
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.completion_filter);
                    if ui.small_button("✕").clicked() {
                        self.completion_open = false;
                    }
                });
                ui.separator();

                let filter = self.completion_filter.to_lowercase();
                let filtered: Vec<&String> = self
                    .completion_candidates
                    .iter()
                    .filter(|s| filter.is_empty() || s.to_lowercase().contains(&filter))
                    .collect();

                // Arrow navigation
                if ui.input(|i| i.key_pressed(Key::ArrowDown)) && !filtered.is_empty() {
                    self.completion_selected_idx =
                        (self.completion_selected_idx + 1) % filtered.len();
                }
                if ui.input(|i| i.key_pressed(Key::ArrowUp)) && !filtered.is_empty() {
                    self.completion_selected_idx = self
                        .completion_selected_idx
                        .checked_sub(1)
                        .unwrap_or(filtered.len().saturating_sub(1));
                }

                let enter_hit = ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Tab));
                let esc_hit = ui.input(|i| i.key_pressed(Key::Escape));

                if esc_hit {
                    self.completion_open = false;
                    return;
                }

                let mut chosen_name: Option<String> = None;

                ScrollArea::vertical().show(ui, |ui| {
                    for (idx, item) in filtered.iter().enumerate() {
                        let is_sel = idx == self.completion_selected_idx;
                        let resp = ui.selectable_label(is_sel, RichText::new(*item).monospace());

                        if (is_sel && enter_hit) || resp.clicked() {
                            let clean_name = item
                                .split(':')
                                .nth(1)
                                .unwrap_or(item)
                                .trim()
                                .split('[')
                                .next()
                                .unwrap_or("")
                                .trim();

                            chosen_name = Some(clean_name.to_string());
                            break;
                        }
                    }
                });

                if let Some(name) = chosen_name {
                    if let Some(doc) = self.active_doc_mut() {
                        doc.content.push_str(&name);
                        doc.is_dirty = true;
                    }
                    self.completion_open = false;
                }
            });
    }
}
