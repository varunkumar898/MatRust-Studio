//! Multi-document script editor with syntax formatting, gutter breakpoints, code folding, and completion.

use crate::execution::VariableRow;
use crate::syntax::{SyntaxDiagnostic, SyntaxService, SYNTAX_DEBOUNCE};
use crate::theme::Theme;
use crate::ui_registry;
use egui::text::CCursorRange;
use egui::{
    text::CCursor, text_edit::TextEditState, Align, Align2, Color32, FontId, Id, Key, Layout,
    Margin, Modifiers, Pos2, Rect, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Vec2,
};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Default editor font size in points.
pub const EDITOR_DEFAULT_FONT_SIZE: f32 = 14.0;
/// Smallest editor font size accepted by the shortcuts.
pub const EDITOR_MIN_FONT_SIZE: f32 = 8.0;
/// Largest editor font size accepted by the shortcuts.
pub const EDITOR_MAX_FONT_SIZE: f32 = 32.0;
/// Requested minimum height of the editor body, in points.
pub const EDITOR_MIN_PANEL_HEIGHT: f32 = 300.0;
/// Number of spaces inserted by Tab in the editor.
pub const EDITOR_TAB_SPACES: usize = 4;
/// Vertical padding above the first text row and below the last row.
pub const EDITOR_TEXT_PADDING: f32 = 6.0;
const GUTTER_LEFT_PAD: f32 = 5.0;
const GUTTER_RIGHT_PAD: f32 = 7.0;
const GUTTER_MARKER_WIDTH: f32 = 20.0;

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

/// Extracts the active section containing `cursor_line` (1-based), delimited by `%%`.
pub fn extract_active_section(content: &str, cursor_line: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let target_idx = cursor_line.saturating_sub(1).min(lines.len() - 1);

    // Scan upward to find preceding %%
    let mut start_idx = 0;
    for i in (0..=target_idx).rev() {
        if lines[i].trim_start().starts_with("%%") {
            start_idx = i;
            break;
        }
    }

    // Scan downward to find following %%
    let mut end_idx = lines.len();
    for (i, line) in lines.iter().enumerate().skip(target_idx + 1) {
        if line.trim_start().starts_with("%%") {
            end_idx = i;
            break;
        }
    }

    lines[start_idx..end_idx].join("\n")
}

/// Checks MATLAB script syntax using `rustlab_parse`.
pub fn check_syntax(content: &str) -> Result<(), String> {
    rustlab_parse::parse(content)
        .map(|_| ())
        .map_err(|e| format!("{e}"))
}

/// Number of logical lines in a document. An empty document has one empty
/// line, and a trailing newline opens a new final line.
pub fn logical_line_count(content: &str) -> usize {
    content.split('\n').count().max(1)
}

/// Display width of one line in monospace character cells.
pub fn display_columns(line: &str) -> usize {
    line.chars()
        .map(|ch| if ch == '\t' { EDITOR_TAB_SPACES } else { 1 })
        .sum()
}

/// Width of the longest logical line, in monospace character cells.
pub fn max_line_chars(content: &str) -> usize {
    content.split('\n').map(display_columns).max().unwrap_or(0)
}

/// Number of decimal digits used by a positive line number.
pub fn gutter_digits(line_count: usize) -> usize {
    let mut value = line_count.max(1);
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

/// Cursor position in character-index, 1-based line and 1-based column form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorCursor {
    /// Zero-based index in Unicode scalar values.
    pub char_index: usize,
    /// One-based line number.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
}

/// Convert a character index (as used by egui's `CCursor`) into line/column.
/// Convert a one-based line and character column to a character index.
pub fn line_col_to_char_index(content: &str, line: usize, column: usize) -> usize {
    let mut index = 0usize;
    for (line_idx, text) in content.split('\n').enumerate() {
        if line_idx + 1 == line {
            let line_chars = text.chars().count();
            return index + column.saturating_sub(1).min(line_chars);
        }
        index += text.chars().count() + 1;
    }
    content.chars().count()
}

pub fn char_index_to_line_col(content: &str, char_index: usize) -> EditorCursor {
    let target = char_index.min(content.chars().count());
    let mut line = 1;
    let mut column = 1;
    for ch in content.chars().take(target) {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    EditorCursor {
        char_index: target,
        line,
        column,
    }
}

/// Find the matching bracket at or immediately before `cursor_char_index`.
/// Returns `[cursor_index, matching_index]` when both characters are brackets.
pub fn matching_bracket(content: &str, cursor_char_index: usize) -> Option<[usize; 2]> {
    let chars: Vec<char> = content.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let cursor = cursor_char_index.min(chars.len());
    let candidate = if cursor < chars.len() {
        cursor
    } else {
        cursor.saturating_sub(1)
    };
    let opening = match chars.get(candidate) {
        Some('(') | Some('[') | Some('{') => Some((chars[candidate], true)),
        Some(')') | Some(']') | Some('}') => Some((chars[candidate], false)),
        _ => None,
    };
    let (bracket, forward) = opening?;
    let (open, close) = match bracket {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        '{' => ('{', '}'),
        ')' => ('(', ')'),
        ']' => ('[', ']'),
        '}' => ('{', '}'),
        _ => return None,
    };
    let mut depth = 0i32;
    if forward {
        let mut index = candidate;
        while index < chars.len() {
            if chars[index] == open {
                depth += 1;
            } else if chars[index] == close {
                depth -= 1;
                if depth == 0 {
                    return Some([candidate, index]);
                }
            }
            index += 1;
        }
    } else {
        let mut index = candidate + 1;
        while index > 0 {
            index -= 1;
            if chars[index] == close {
                depth += 1;
            } else if chars[index] == open {
                depth -= 1;
                if depth == 0 {
                    return Some([candidate, index]);
                }
            }
        }
    }
    None
}

/// One visible gutter row. `has_text` is false past end-of-file, where the
/// number is still drawn (dimmed) so the gutter fills the viewport.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorRow {
    /// One-based line number shown in the gutter.
    pub line_no: usize,
    /// True when this row belongs to the document text.
    pub has_text: bool,
    /// Screen-space top of the row.
    pub top: f32,
    /// Fixed row height shared by gutter and text.
    pub height: f32,
}

/// Pure geometry for one editor pass. The renderer paints from this model; the
/// tests compare it with egui's real galley rows.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorLayout {
    /// The full panel body given to the editor.
    pub panel_rect: Rect,
    /// The content occupies exactly the panel body.
    pub content_rect: Rect,
    /// Fixed-width line-number column.
    pub gutter_rect: Rect,
    /// Text viewport to the right of the gutter.
    pub text_rect: Rect,
    /// Shared monospace row height.
    pub line_height: f32,
    /// Editor font size used for this layout.
    pub font_size: f32,
    /// Monospace digit width.
    pub char_width: f32,
    /// Gutter width after digit-count sizing.
    pub gutter_width: f32,
    /// Number of rows that fit in the viewport (including a partial last row).
    pub visible_rows: usize,
    /// First visible logical line number.
    pub first_visible_line: usize,
    /// One row per visible row, including dimmed rows past EOF.
    pub rows: Vec<EditorRow>,
    /// Current scroll offset from the ScrollArea.
    pub scroll_offset: Vec2,
    /// Full scrollable content size.
    pub content_size: Vec2,
    /// Logical line count of the document.
    pub line_count: usize,
    /// Word-wrap mode used for the measurement.
    pub word_wrap: bool,
}

impl EditorLayout {
    /// Top of one visible row (screen pixels), independent of the row data.
    pub fn text_row_top(&self, visible_index: usize) -> Option<f32> {
        if visible_index >= self.visible_rows {
            return None;
        }
        Some(
            self.panel_rect.top() + EDITOR_TEXT_PADDING + visible_index as f32 * self.line_height
                - self.scroll_offset.y,
        )
    }

    /// Screen-space y of a visible row's gutter number centre.
    pub fn row_center_y(&self, visible_index: usize) -> Option<f32> {
        self.text_row_top(visible_index)
            .map(|top| top + self.line_height * 0.5)
    }
}

/// Measure the editor. This function has no egui side effects; it is the single
/// source of truth for row positions, gutter width and content size.
pub fn compute_editor_layout(
    panel_rect: Rect,
    content: &str,
    font_size: f32,
    line_height: f32,
    char_width: f32,
    scroll_offset: Vec2,
    word_wrap: bool,
) -> EditorLayout {
    let font_size = if font_size.is_finite() {
        font_size.clamp(EDITOR_MIN_FONT_SIZE, EDITOR_MAX_FONT_SIZE)
    } else {
        EDITOR_DEFAULT_FONT_SIZE
    };
    let line_height = if line_height.is_finite() && line_height > 0.0 {
        line_height
    } else {
        font_size * 1.25
    };
    let char_width = if char_width.is_finite() && char_width > 0.0 {
        char_width
    } else {
        font_size * 0.6
    };

    let line_count = logical_line_count(content);
    let usable_height = (panel_rect.height() - 2.0 * EDITOR_TEXT_PADDING).max(line_height);
    let visible_rows = ((usable_height / line_height).ceil() as usize).max(1);
    let digits = gutter_digits(line_count.max(visible_rows));
    let gutter_width =
        GUTTER_LEFT_PAD + GUTTER_MARKER_WIDTH + digits as f32 * char_width + GUTTER_RIGHT_PAD;
    let text_viewport_width = (panel_rect.width() - gutter_width).max(1.0);
    let longest_line = max_line_chars(content) as f32 * char_width;
    let text_content_width = if word_wrap {
        text_viewport_width
    } else {
        (longest_line + char_width).max(text_viewport_width)
    };
    let content_width = gutter_width + text_content_width;
    let display_rows = line_count.max(visible_rows);
    let content_height =
        (2.0 * EDITOR_TEXT_PADDING + display_rows as f32 * line_height).max(panel_rect.height());

    let scroll_offset = Vec2::new(scroll_offset.x.max(0.0), scroll_offset.y.max(0.0));
    let first_index =
        (((scroll_offset.y - EDITOR_TEXT_PADDING).max(0.0)) / line_height).floor() as usize;
    let first_visible_line = (first_index + 1).min(line_count.max(1));

    let rows = (0..visible_rows)
        .map(|index| {
            let line_no = first_visible_line + index;
            EditorRow {
                line_no,
                has_text: line_no <= line_count,
                top: panel_rect.top() + EDITOR_TEXT_PADDING + index as f32 * line_height
                    - scroll_offset.y,
                height: line_height,
            }
        })
        .collect();

    EditorLayout {
        panel_rect,
        content_rect: panel_rect,
        gutter_rect: Rect::from_min_size(
            panel_rect.min,
            Vec2::new(gutter_width, panel_rect.height()),
        ),
        text_rect: Rect::from_min_size(
            Pos2::new(panel_rect.left() + gutter_width, panel_rect.top()),
            Vec2::new(text_viewport_width, panel_rect.height()),
        ),
        line_height,
        font_size,
        char_width,
        gutter_width,
        visible_rows,
        first_visible_line,
        rows,
        scroll_offset,
        content_size: Vec2::new(content_width, content_height),
        line_count,
        word_wrap,
    }
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
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

impl DocumentTab {
    pub fn new_untitled(idx: usize) -> Self {
        Self {
            path: None,
            title: format!("Untitled{idx}.m"),
            // G2: a new script starts empty, with no pre-filled demo content.
            content: String::new(),
            is_dirty: true,
            breakpoints: HashSet::new(),
            folded_blocks: HashSet::new(),
            cursor_line: 1,
            cursor_col: 1,
            error_line: None,
            diagnostics: Vec::new(),
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
            diagnostics: Vec::new(),
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

    /// Stable identity for egui scopes and ScrollArea/TextEdit ids.
    ///
    /// Saved files use their full path; untitled documents use their unique
    /// title (`Untitled1.m`, `Untitled2.m`, ...). Renaming a saved file via
    /// Save As intentionally changes the key, which gives the new path fresh
    /// undo/scroll state.
    pub fn file_key(&self) -> String {
        match &self.path {
            Some(path) => format!("path:{}", path.display()),
            None => format!("untitled:{}", self.title),
        }
    }
}

pub struct EditorPanel {
    pub tabs: Vec<DocumentTab>,
    pub active_tab_idx: usize,
    pub next_untitled_id: usize,

    // Presentation state
    pub font_size: f32,
    pub word_wrap: bool,

    // Last rendered geometry (used by the view-model tests and the next frame's
    // highlight pass). It is intentionally public read-only data.
    pub last_layout: Option<EditorLayout>,
    pub last_text_id: Option<Id>,
    pub last_scroll_id: Option<Id>,
    pub last_cursor: Option<EditorCursor>,
    pub last_galley_row_tops: Option<Vec<f32>>,
    pub last_focused: bool,

    // Background syntax checking. Parsing is never done on the UI thread.
    pub syntax: SyntaxService,
    pending_jump: Option<usize>,
    /// Set by the section marker in the gutter, consumed by `show`.
    pub section_run_request: Option<String>,

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
            font_size: EDITOR_DEFAULT_FONT_SIZE,
            word_wrap: false,
            last_layout: None,
            last_text_id: None,
            last_scroll_id: None,
            last_cursor: None,
            last_galley_row_tops: None,
            last_focused: false,
            syntax: SyntaxService::default(),
            pending_jump: None,
            section_run_request: None,
            completion_open: false,
            completion_filter: String::new(),
            completion_selected_idx: 0,
            completion_candidates: Vec::new(),
        }
    }
}

/// Measure the real row pitch of a multi-line galley.
///
/// `Fonts::row_height` includes extra leading in some egui versions, while a
/// laid-out multi-line galley advances by the visible glyph row height. The
/// gutter must use the latter, otherwise row 100 drifts a few pixels away from
/// the text even though row 0 looks aligned.
fn monospace_line_height(ui: &Ui, font_id: &FontId) -> f32 {
    let job = egui::text::LayoutJob::simple(
        "M\nM".to_owned(),
        font_id.clone(),
        Color32::WHITE,
        f32::INFINITY,
    );
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    galley
        .rows
        .get(1)
        .map(|row| row.rect.top())
        .filter(|height| height.is_finite() && *height > 0.0)
        .unwrap_or_else(|| ui.fonts(|fonts| fonts.row_height(font_id)))
}

/// Put the egui TextEdit cursor at a character index and store its state.
fn set_text_edit_cursor(ctx: &egui::Context, text_id: Id, char_index: usize) {
    let mut state = TextEditState::load(ctx, text_id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(CCursorRange::one(CCursor::new(char_index))));
    state.store(ctx, text_id);
}

/// Mutable outputs produced by the editor toolbar. Grouping them keeps the
/// public `show` signature readable as the toolbar grows.
pub struct EditorCallbacks<'a> {
    pub run_script: &'a mut Option<String>,
    pub run_selection: &'a mut Option<String>,
    pub run_section: &'a mut Option<String>,
    pub stop: &'a mut bool,
}

impl EditorPanel {
    pub fn active_doc(&self) -> Option<&DocumentTab> {
        self.tabs.get(self.active_tab_idx)
    }

    pub fn active_doc_mut(&mut self) -> Option<&mut DocumentTab> {
        self.tabs.get_mut(self.active_tab_idx)
    }

    /// Stable file key for the active document, used to salt egui ids.
    pub fn file_key_for_active(&self) -> Option<String> {
        self.active_doc().map(DocumentTab::file_key)
    }

    /// Latest syntax diagnostics for the active document.
    pub fn diagnostics(&self) -> &[SyntaxDiagnostic] {
        self.active_doc()
            .map(|doc| doc.diagnostics.as_slice())
            .unwrap_or(&[])
    }

    /// Text for Run Selection: the current TextEdit selection, or the cursor's
    /// line when nothing is selected. This uses the same state egui stores for
    /// copy/paste, so there is only one selection model.
    pub fn code_for_selection(&self, ctx: &egui::Context) -> Option<String> {
        let doc = self.active_doc()?;
        let text_id = self.last_text_id?;
        if let Some(state) = TextEditState::load(ctx, text_id) {
            if let Some(range) = state.cursor.char_range() {
                let start = range.primary.index.min(range.secondary.index);
                let end = range.primary.index.max(range.secondary.index);
                if start != end {
                    return Some(doc.content.chars().skip(start).take(end - start).collect());
                }
            }
        }
        let line = doc
            .content
            .split('\n')
            .nth(doc.cursor_line.saturating_sub(1))?;
        if line.trim().is_empty() {
            return None;
        }
        Some(line.to_string())
    }

    /// Put the cursor on the first parser error and focus the editor.
    pub fn jump_to_first_error(&mut self, ctx: &egui::Context) {
        let Some(diagnostic) = self.diagnostics().first().cloned() else {
            return;
        };
        let Some(doc) = self.active_doc() else {
            return;
        };
        let index = line_col_to_char_index(&doc.content, diagnostic.line, diagnostic.column);
        self.pending_jump = Some(index);
        if let Some(text_id) = self.last_text_id {
            ctx.memory_mut(|memory| memory.request_focus(text_id));
        }
        ctx.request_repaint();
    }

    /// Clamp and apply an editor font size.
    pub fn set_font_size(&mut self, size: f32) {
        if size.is_finite() {
            self.font_size = size.clamp(EDITOR_MIN_FONT_SIZE, EDITOR_MAX_FONT_SIZE);
        }
    }

    /// Editor-only font shortcuts. Returns true when one was consumed.
    ///
    /// Ctrl+= (or Ctrl+Plus), Ctrl+-, and Ctrl+0. Global egui zoom is disabled
    /// by the app so these keys change the code font, not the whole UI.
    pub fn handle_font_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        let (plus, equals, minus, zero) = ctx.input_mut(|input| {
            (
                input.consume_key(Modifiers::COMMAND, Key::Plus),
                input.consume_key(Modifiers::COMMAND, Key::Equals),
                input.consume_key(Modifiers::COMMAND, Key::Minus),
                input.consume_key(Modifiers::COMMAND, Key::Num0),
            )
        });
        if plus || equals {
            self.set_font_size(self.font_size + 1.0);
            changed = true;
        }
        if minus {
            self.set_font_size(self.font_size - 1.0);
            changed = true;
        }
        if zero {
            self.set_font_size(EDITOR_DEFAULT_FONT_SIZE);
            changed = true;
        }
        changed
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

    /// Draw the editor toolbar and then give the remaining body exactly to the
    /// code viewport. The toolbar is the only part that is not editor content.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        workspace_vars: &[VariableRow],
        callbacks: EditorCallbacks<'_>,
    ) {
        let EditorCallbacks {
            run_script: on_run_script,
            run_selection: on_run_selection,
            run_section: on_run_section,
            stop: on_stop,
        } = callbacks;
        // The dock gives this panel an exact body rect. egui child UIs inherit
        // the central panel's wider clip by default, so pin the clip to our own
        // body; otherwise a dense toolbar can paint over the next panel.
        let panel_clip = ui.clip_rect().intersect(ui.max_rect());
        ui.set_clip_rect(panel_clip);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut close_tab_idx = None;
                let mut select_idx = None;

                for (idx, tab) in self.tabs.iter().enumerate() {
                    let is_active = idx == self.active_tab_idx;
                    let tab_title = if tab.is_dirty {
                        format!("{} *", tab.title)
                    } else {
                        tab.title.clone()
                    };
                    let btn = ui.selectable_label(is_active, tab_title);
                    let _ = ui_registry::register(ui.ctx(), "editor.tab", btn.id, btn.rect);
                    if btn.clicked() {
                        select_idx = Some(idx);
                    }
                    if is_active {
                        let close = ui.small_button("X").on_hover_text("Close tab");
                        let _ = ui_registry::register(
                            ui.ctx(),
                            "editor.tab_close",
                            close.id,
                            close.rect,
                        );
                        if close.clicked() {
                            close_tab_idx = Some(idx);
                        }
                    }
                }

                if let Some(idx) = select_idx {
                    self.active_tab_idx = idx;
                }
                if let Some(idx) = close_tab_idx {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(idx);
                        if self.active_tab_idx >= self.tabs.len() {
                            self.active_tab_idx = self.tabs.len() - 1;
                        }
                    }
                }

                let new_btn = ui.button("+").on_hover_text("New Script (Ctrl+N)");
                let _ =
                    ui_registry::register(ui.ctx(), "editor.new_script", new_btn.id, new_btn.rect);
                if new_btn.clicked() {
                    self.new_script();
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let run_script = ui.button("Run").on_hover_text("Execute active script (F5)");
                let _ = ui_registry::register(
                    ui.ctx(),
                    "editor.run_script",
                    run_script.id,
                    run_script.rect,
                );
                if run_script.clicked() {
                    if let Some(doc) = self.active_doc() {
                        *on_run_script = Some(doc.content.clone());
                    }
                }

                let run_section = ui
                    .button("Run Section")
                    .on_hover_text("Execute active section surrounded by %% (Ctrl+Enter)");
                let _ = ui_registry::register(
                    ui.ctx(),
                    "editor.run_section",
                    run_section.id,
                    run_section.rect,
                );
                if run_section.clicked() {
                    if let Some(doc) = self.active_doc() {
                        *on_run_section =
                            Some(extract_active_section(&doc.content, doc.cursor_line));
                    }
                }

                let run_selection = ui
                    .button("Run Selection")
                    .on_hover_text("Execute selected text (F9)");
                let _ = ui_registry::register(
                    ui.ctx(),
                    "editor.run_selection",
                    run_selection.id,
                    run_selection.rect,
                );
                if run_selection.clicked() {
                    if let Some(code) = self.code_for_selection(ui.ctx()) {
                        *on_run_selection = Some(code);
                    }
                }

                let stop = ui
                    .button("Stop")
                    .on_hover_text("Stop the running script (Ctrl+C)");
                let _ = ui_registry::register(ui.ctx(), "editor.stop", stop.id, stop.rect);
                if stop.clicked() {
                    *on_stop = true;
                }

                ui.separator();

                let smaller = ui
                    .small_button("A-")
                    .on_hover_text("Smaller editor font (Ctrl+-)");
                let _ = ui_registry::register(
                    ui.ctx(),
                    "editor.font_decrease",
                    smaller.id,
                    smaller.rect,
                );
                if smaller.clicked() {
                    self.set_font_size(self.font_size - 1.0);
                }

                let larger = ui
                    .small_button("A+")
                    .on_hover_text("Larger editor font (Ctrl+=)");
                let _ =
                    ui_registry::register(ui.ctx(), "editor.font_increase", larger.id, larger.rect);
                if larger.clicked() {
                    self.set_font_size(self.font_size + 1.0);
                }

                let reset = ui
                    .small_button(format!("{:.0} px", self.font_size))
                    .on_hover_text("Reset editor font size (Ctrl+0)");
                let _ = ui_registry::register(ui.ctx(), "editor.font_reset", reset.id, reset.rect);
                if reset.clicked() {
                    self.set_font_size(EDITOR_DEFAULT_FONT_SIZE);
                }

                let wrap = ui.checkbox(&mut self.word_wrap, "Wrap");
                let _ = ui_registry::register(ui.ctx(), "editor.wrap_toggle", wrap.id, wrap.rect);
            });

            if self.active_tab_idx >= self.tabs.len() {
                return;
            }

            // Everything left after the toolbar belongs to the editor. Use the
            // exact remaining rect: expanding past it would paint over the
            // neighbouring dock panel and would break the 1 px rect contract.
            let available = ui.available_rect_before_wrap();
            let body_rect = Rect::from_min_size(
                available.min,
                Vec2::new(available.width().max(1.0), available.height().max(1.0)),
            );
            let mut body = ui.new_child(
                UiBuilder::default()
                    .max_rect(body_rect)
                    .layout(Layout::top_down(Align::LEFT)),
            );
            body.set_min_size(body_rect.size());
            let _ = self.show_content(&mut body, body_rect, theme);
            if let Some(section) = self.section_run_request.take() {
                *on_run_section = Some(section);
            }

            let ctrl_space =
                ui.input(|input| input.modifiers.command && input.key_pressed(Key::Space));
            if ctrl_space {
                self.completion_open = true;
                self.populate_completion(workspace_vars);
            }
            if self.completion_open {
                self.show_completion_popup(ui, theme);
            }
        });
    }

    /// Paint the complete editor inside `panel_rect`.
    ///
    /// This is public so the headless tests can supply the exact body rect the
    /// real panel receives. It never allocates less than `panel_rect`; the
    /// returned layout is also stored in `last_layout`.
    pub fn show_content(&mut self, ui: &mut Ui, panel_rect: Rect, theme: &Theme) -> EditorLayout {
        let panel_clip = ui.clip_rect().intersect(panel_rect);
        ui.set_clip_rect(panel_clip);
        let active_idx = self.active_tab_idx;
        let font_size = self.font_size;
        let word_wrap = self.word_wrap;
        let font_id = FontId::monospace(font_size);
        let line_height = monospace_line_height(ui, &font_id);
        let char_width = ui.fonts(|fonts| fonts.glyph_width(&font_id, '0'));
        if active_idx < self.tabs.len() {
            let source = self.tabs[active_idx].content.clone();
            self.syntax.observe(&source);
            if let Some(diagnostics) = self.syntax.poll(&source, std::time::Instant::now()) {
                let first_line = diagnostics.first().map(|d| d.line);
                self.tabs[active_idx].diagnostics = diagnostics;
                self.tabs[active_idx].error_line = first_line;
            }
            if self.syntax.is_pending() {
                ui.ctx()
                    .request_repaint_after(Duration::from_millis(100).max(SYNTAX_DEBOUNCE / 3));
            }
        }
        let previous_scroll = self
            .last_layout
            .as_ref()
            .map(|layout| layout.scroll_offset)
            .unwrap_or(Vec2::ZERO);

        ui.painter().rect_filled(panel_rect, 0.0, theme.editor_bg);

        if active_idx >= self.tabs.len() {
            let layout = compute_editor_layout(
                panel_rect,
                "",
                font_size,
                line_height,
                char_width,
                previous_scroll,
                word_wrap,
            );
            self.last_layout = Some(layout.clone());
            self.last_text_id = None;
            self.last_scroll_id = None;
            self.last_cursor = None;
            self.last_galley_row_tops = Some(Vec::new());
            self.last_focused = false;
            return layout;
        }

        let file_key = self.tabs[active_idx].file_key();
        let text_id = Id::new(("matrust_editor_text", &file_key));
        if let Some(index) = self.pending_jump.take() {
            set_text_edit_cursor(ui.ctx(), text_id, index);
            ui.ctx().memory_mut(|memory| memory.request_focus(text_id));
        }
        let previous_content = self.tabs[active_idx].content.clone();
        let preview = compute_editor_layout(
            panel_rect,
            &previous_content,
            font_size,
            line_height,
            char_width,
            previous_scroll,
            word_wrap,
        );
        let text_width = (preview.content_size.x - preview.gutter_width).max(1.0);
        let text_height =
            (preview.content_size.y - 2.0 * EDITOR_TEXT_PADDING).max(preview.line_height);
        let line_count = preview.line_count;

        self.last_text_id = Some(text_id);
        self.last_scroll_id = None;

        let mut content_changed = false;
        let mut cursor_char_index = 0usize;
        let mut galley_row_tops: Vec<f32> = Vec::new();
        let mut clicked_empty = false;
        let mut actual_scroll = previous_scroll;
        let mut scroll_visual_id = Id::NULL;
        let mut scroll_visual_rect = panel_rect;

        let doc = &mut self.tabs[active_idx];
        let _ = ui.push_id(("matrust_editor_doc", file_key.as_str()), |ui| {
            ui.painter().rect_filled(panel_rect, 0.0, theme.editor_bg);

            let scroll_output = ScrollArea::both()
                .id_salt(("matrust_editor_scroll", file_key.as_str()))
                .auto_shrink([false, false])
                .show(ui, |scroll_ui| {
                    scroll_ui.set_min_size(preview.content_size);
                    let (_, content_rect) = scroll_ui.allocate_space(preview.content_size);

                    if doc.cursor_line <= line_count {
                        let highlight_top = content_rect.top()
                            + EDITOR_TEXT_PADDING
                            + (doc.cursor_line.saturating_sub(1)) as f32 * line_height;
                        scroll_ui.painter().rect_filled(
                            Rect::from_min_size(
                                Pos2::new(
                                    content_rect.left() + preview.gutter_width,
                                    highlight_top,
                                ),
                                Vec2::new(text_width, line_height),
                            ),
                            0.0,
                            Color32::from_rgba_unmultiplied(
                                theme.accent.r(),
                                theme.accent.g(),
                                theme.accent.b(),
                                34,
                            ),
                        );
                    }

                    let text_rect = Rect::from_min_size(
                        Pos2::new(
                            content_rect.left() + preview.gutter_width,
                            content_rect.top() + EDITOR_TEXT_PADDING,
                        ),
                        Vec2::new(text_width, text_height),
                    );
                    let mut text_ui = scroll_ui.new_child(
                        UiBuilder::default()
                            .max_rect(text_rect)
                            .layout(Layout::top_down(Align::LEFT)),
                    );

                    let has_focus = text_ui.ctx().memory(|memory| memory.has_focus(text_id));
                    if has_focus {
                        let tab_pressed = text_ui
                            .ctx()
                            .input_mut(|input| input.consume_key(Modifiers::NONE, Key::Tab));
                        if tab_pressed {
                            text_ui.ctx().input_mut(|input| {
                                input
                                    .events
                                    .push(egui::Event::Text(" ".repeat(EDITOR_TAB_SPACES)));
                            });
                        }
                    }

                    let desired_width = if word_wrap {
                        text_rect.width()
                    } else {
                        text_width
                    };
                    let output = TextEdit::multiline(&mut doc.content)
                        .id(text_id)
                        .font(font_id.clone())
                        .text_color(theme.text_primary)
                        .desired_width(desired_width)
                        .desired_rows(line_count)
                        .min_size(text_rect.size())
                        .margin(Margin::ZERO)
                        .frame(false)
                        .lock_focus(true)
                        .show(&mut text_ui);

                    let _ = ui_registry::register(
                        text_ui.ctx(),
                        "editor.text",
                        output.response.id,
                        output.response.rect,
                    );
                    content_changed = output.response.changed();
                    cursor_char_index = output
                        .cursor_range
                        .map(|range| range.primary.ccursor.index)
                        .unwrap_or(0);
                    galley_row_tops = output
                        .galley
                        .rows
                        .iter()
                        .map(|row| output.galley_pos.y + row.rect.top())
                        .collect();

                    if let Some([cursor, matched]) =
                        matching_bracket(&doc.content, cursor_char_index)
                    {
                        let bracket_stroke = egui::Stroke::new(1.0_f32, theme.accent);
                        for index in [cursor, matched] {
                            let local = output.galley.pos_from_ccursor(CCursor::new(index));
                            let screen = local.translate(output.galley_pos.to_vec2());
                            text_ui.painter().rect_stroke(
                                screen.expand(1.0),
                                1.0,
                                bracket_stroke,
                                egui::StrokeKind::Outside,
                            );
                        }
                    }

                    // Red zigzag under the first parser error. It uses the
                    // same galley positions as the text, so it follows scroll,
                    // font size and wrapping without a second layout model.
                    if let Some(diagnostic) = doc.diagnostics.first() {
                        let line_start = line_col_to_char_index(&doc.content, diagnostic.line, 1);
                        let line_chars = doc
                            .content
                            .split('\n')
                            .nth(diagnostic.line.saturating_sub(1))
                            .map(|line| line.chars().count())
                            .unwrap_or(0);
                        let start =
                            line_start + diagnostic.column.saturating_sub(1).min(line_chars);
                        let highlight_len = diagnostic
                            .length
                            .min(line_chars.saturating_sub(diagnostic.column - 1))
                            .max(1);
                        let end = (start + highlight_len).min(line_start + line_chars);
                        let start_rect = output.galley.pos_from_ccursor(CCursor::new(start));
                        let end_rect = output.galley.pos_from_ccursor(CCursor::new(end));
                        let y = output.galley_pos.y + start_rect.bottom() + 1.5;
                        let mut x = output.galley_pos.x + start_rect.left();
                        let end_x = (output.galley_pos.x + end_rect.left()).max(x + 2.0);
                        let stroke = egui::Stroke::new(1.2_f32, theme.error);
                        let painter = text_ui
                            .painter()
                            .with_clip_rect(text_ui.clip_rect().intersect(panel_rect));
                        let mut up = true;
                        while x < end_x {
                            let next = (x + 3.0).min(end_x);
                            let offset = if up { 1.5 } else { -1.5 };
                            painter.line_segment(
                                [Pos2::new(x, y), Pos2::new(next, y + offset)],
                                stroke,
                            );
                            x = next;
                            up = !up;
                        }
                        let _ = output
                            .response
                            .clone()
                            .on_hover_text(diagnostic.message.clone());
                    }

                    let click_pos = text_ui.input(|input| {
                        if input.pointer.primary_clicked() {
                            input.pointer.interact_pos()
                        } else {
                            None
                        }
                    });
                    if let Some(pos) = click_pos {
                        let last_line_bottom = text_rect.top() + line_count as f32 * line_height;
                        if panel_rect.contains(pos) && pos.y > last_line_bottom {
                            clicked_empty = true;
                        }
                    }
                });

            actual_scroll = scroll_output.state.offset;
            scroll_visual_id = scroll_output.id;
            scroll_visual_rect = scroll_output.inner_rect;
        });

        if clicked_empty {
            cursor_char_index = doc.content.chars().count();
            set_text_edit_cursor(ui.ctx(), text_id, cursor_char_index);
            ui.ctx().memory_mut(|memory| memory.request_focus(text_id));
        }

        if content_changed {
            doc.is_dirty = true;
            self.syntax.observe(&doc.content);
        }
        let cursor = char_index_to_line_col(&doc.content, cursor_char_index);
        doc.cursor_line = cursor.line;
        doc.cursor_col = cursor.column;

        let layout = compute_editor_layout(
            panel_rect,
            &doc.content,
            font_size,
            line_height,
            char_width,
            actual_scroll,
            word_wrap,
        );
        self.last_scroll_id = Some(scroll_visual_id);
        self.last_cursor = Some(cursor);
        self.last_galley_row_tops = Some(galley_row_tops);
        self.last_focused = ui.ctx().memory(|memory| memory.has_focus(text_id));

        let _ = ui_registry::register(
            ui.ctx(),
            "editor.scroll",
            scroll_visual_id,
            scroll_visual_rect,
        );

        // Gutter overlay: same vertical model as the text, fixed horizontally.
        // It is painted after the scroll area so text scrolled left is covered.
        let painter = ui
            .painter()
            .with_clip_rect(ui.clip_rect().intersect(panel_rect));
        painter.rect_filled(layout.gutter_rect, 0.0, theme.editor_bg);
        let blocks = find_foldable_blocks(&doc.content);
        let block_font = FontId::monospace((font_size * 0.7).max(8.0));
        for row in layout.rows.iter() {
            let row_rect = Rect::from_min_size(
                Pos2::new(layout.gutter_rect.left(), row.top),
                Vec2::new(layout.gutter_width, row.height),
            );
            let visible_row_rect = row_rect.intersect(panel_rect);
            if visible_row_rect.height() < 1.0 {
                continue;
            }

            let is_current = row.line_no == doc.cursor_line;
            let is_error = doc.error_line == Some(row.line_no);
            let has_breakpoint = doc.breakpoints.contains(&row.line_no);
            let line_text = doc
                .content
                .split('\n')
                .nth(row.line_no.saturating_sub(1))
                .unwrap_or("");
            let is_section_start = row.has_text && line_text.trim_start().starts_with("%%");
            let number_color = if is_current {
                theme.accent
            } else if is_error {
                theme.error
            } else if row.has_text {
                theme.gutter_fg
            } else {
                Color32::from_rgba_unmultiplied(
                    theme.gutter_fg.r(),
                    theme.gutter_fg.g(),
                    theme.gutter_fg.b(),
                    84,
                )
            };

            if is_current {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(row_rect.left(), row.top),
                        Vec2::new(2.0, row.height),
                    ),
                    0.0,
                    theme.accent,
                );
            }

            painter.text(
                Pos2::new(
                    layout.gutter_rect.right() - GUTTER_RIGHT_PAD,
                    row.top + row.height * 0.5,
                ),
                Align2::RIGHT_CENTER,
                row.line_no.to_string(),
                font_id.clone(),
                number_color,
            );

            if row.has_text {
                let fold_start = blocks.iter().any(|block| block.start_line == row.line_no);
                if fold_start {
                    let folded = doc.folded_blocks.contains(&row.line_no);
                    painter.text(
                        Pos2::new(
                            layout.gutter_rect.left() + GUTTER_LEFT_PAD,
                            row.top + row.height * 0.5,
                        ),
                        Align2::LEFT_CENTER,
                        if folded { ">" } else { "v" },
                        block_font.clone(),
                        theme.accent,
                    );
                }
                let marker_center = Pos2::new(
                    layout.gutter_rect.left() + GUTTER_LEFT_PAD + 13.0,
                    row.top + row.height * 0.5,
                );
                if has_breakpoint {
                    painter.circle_filled(marker_center, 3.0, theme.error);
                } else if is_error {
                    painter.text(
                        marker_center,
                        Align2::CENTER_CENTER,
                        ">",
                        block_font.clone(),
                        theme.error,
                    );
                }
                if is_section_start {
                    // A small ASCII marker next to a %% line. The whole row is
                    // clickable but its registered id is section-specific, so
                    // the affordance is auditable and never a dead control.
                    painter.text(
                        Pos2::new(
                            layout.gutter_rect.left() + GUTTER_LEFT_PAD + 3.0,
                            row.top + row.height * 0.5,
                        ),
                        Align2::CENTER_CENTER,
                        "R",
                        block_font.clone(),
                        theme.success,
                    );
                }
            }

            let row_id = Id::new(("matrust_editor_gutter_row", file_key.as_str(), row.line_no));
            let response = ui.interact(visible_row_rect, row_id, Sense::click());
            let _ =
                ui_registry::register(ui.ctx(), "editor.gutter_row", response.id, response.rect);
            if is_section_start {
                let control_id = format!("editor.run_section_marker.{}", row.line_no);
                let _ = ui_registry::register(ui.ctx(), &control_id, response.id, response.rect);
                let _ =
                    response.on_hover_text(format!("Run section starting at line {}", row.line_no));
            };
        }

        // The scroll area owns the whole panel rect, so a gutter `Response`
        // can lose the click race. Resolve gutter clicks from the raw pointer
        // once per frame instead; this is deterministic and keeps breakpoints
        // and the %% run marker alive in the real window.
        let gutter_click = ui.input(|input| {
            if input.pointer.primary_clicked() {
                input.pointer.interact_pos()
            } else {
                None
            }
        });
        if let Some(pos) = gutter_click {
            if layout.gutter_rect.contains(pos) {
                if let Some(row) = layout
                    .rows
                    .iter()
                    .find(|row| row.has_text && pos.y >= row.top && pos.y < row.top + row.height)
                {
                    let line_text = doc
                        .content
                        .split('\n')
                        .nth(row.line_no.saturating_sub(1))
                        .unwrap_or("");
                    if line_text.trim_start().starts_with("%%") {
                        self.section_run_request =
                            Some(extract_active_section(&doc.content, row.line_no));
                    } else {
                        let fold_start = blocks.iter().any(|block| block.start_line == row.line_no);
                        if fold_start {
                            if doc.folded_blocks.contains(&row.line_no) {
                                doc.folded_blocks.remove(&row.line_no);
                            } else {
                                doc.folded_blocks.insert(row.line_no);
                            }
                        } else if doc.breakpoints.contains(&row.line_no) {
                            doc.breakpoints.remove(&row.line_no);
                        } else {
                            doc.breakpoints.insert(row.line_no);
                        }
                    }
                }
            }
        }

        self.last_layout = Some(layout.clone());
        layout
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
        let file_key = self
            .file_key_for_active()
            .unwrap_or_else(|| "completion".to_string());
        egui::Window::new("Completion (Ctrl+Space)")
            .id(Id::new(("matrust_editor_completion", file_key.as_str())))
            .fixed_size([260.0, 200.0])
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    let filter = ui.text_edit_singleline(&mut self.completion_filter);
                    let _ = ui_registry::register(
                        ui.ctx(),
                        "editor.completion_filter",
                        filter.id,
                        filter.rect,
                    );
                    let close = ui.small_button("X");
                    let _ = ui_registry::register(
                        ui.ctx(),
                        "editor.completion_close",
                        close.id,
                        close.rect,
                    );
                    if close.clicked() {
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

                let completion_scroll = ScrollArea::vertical()
                    .id_salt(("matrust_editor_completion_scroll", file_key.as_str()))
                    .show(ui, |ui| {
                        for (idx, item) in filtered.iter().enumerate() {
                            let is_sel = idx == self.completion_selected_idx;
                            let resp =
                                ui.selectable_label(is_sel, RichText::new(*item).monospace());
                            let _ = ui_registry::register(
                                ui.ctx(),
                                "editor.completion_item",
                                resp.id,
                                resp.rect,
                            );

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
                let _ = ui_registry::register(
                    ui.ctx(),
                    "editor.completion_scroll",
                    completion_scroll.id,
                    completion_scroll.inner_rect,
                );

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
