//! A self-contained multiline code editor widget built on egui painting
//! primitives. Deliberately not `egui::TextEdit`: we need a line-number
//! gutter, current-line highlighting, per-token syntax coloring, and
//! end-to-end control of cursor behavior.
//!
//! Canonical state is `lines: Vec<String>`. Cursor/selection are
//! `(line, col)` in **char** units. IME composition is not implemented yet
//! (see docs/desktop-ui.md).

use egui::{Align2, Color32, Event, FontId, Key, Sense, TextStyle};

use crate::theme::{SyntaxPalette, ThemeKind};

/// Words highlighted as keywords (forward-compatible: the interpreter does
/// not parse these yet).
const KEYWORDS: &[&str] = &[
    "end",
    "if",
    "else",
    "elseif",
    "while",
    "for",
    "break",
    "continue",
    "function",
    "return",
    "switch",
    "case",
    "otherwise",
    "try",
    "catch",
];

/// Builtins used for highlighting (mirrors `crates/runtime/src/builtins.rs`).
const BUILTINS: &[&str] = &[
    "disp",
    "zeros",
    "ones",
    "eye",
    "size",
    "length",
    "numel",
    "abs",
    "sqrt",
    "floor",
    "ceil",
    "round",
    "exp",
    "log",
    "sum",
    "max",
    "min",
    "pi",
    "figure",
    "clf",
    "close",
    "plot",
    "scatter",
    "bar",
    "histogram",
    "title",
    "xlabel",
    "ylabel",
    "legend",
    "grid",
    "fix",
    "log2",
    "log10",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "sinh",
    "cosh",
    "tanh",
    "deg2rad",
    "rad2deg",
    "sind",
    "cosd",
    "tand",
    "asind",
    "acosd",
    "atand",
    "sign",
    "hypot",
    "atan2",
    "nthroot",
    "mod",
    "rem",
    "gcd",
    "lcm",
    "factorial",
    "nchoosek",
    "primes",
    "isprime",
    "factor",
    "bitand",
    "bitor",
    "bitxor",
    "bitcmp",
    "bitshift",
    "dec2bin",
    "bin2dec",
    "dec2hex",
    "hex2dec",
    "mean",
    "median",
    "mode",
    "std",
    "var",
    "range",
    "iqr",
    "prod",
    "lu",
    "det",
    "inv",
    "trace",
    "rank",
    "norm",
    "svd",
    "pinv",
    "mldivide",
    "mrdivide",
    "ldivide",
    "rdivide",
    "complex",
    "real",
    "imag",
    "conj",
    "angle",
    "isreal",
    "i",
    "j",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

impl Cursor {
    fn key(self) -> (usize, usize) {
        (self.line, self.col)
    }
}

#[derive(Debug, Clone)]
struct Snapshot {
    lines: Vec<String>,
    cursor: Cursor,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    pub char_w: f32,
    pub line_h: f32,
    pub font: FontId,
}

/// What the editor did this frame.
#[derive(Debug, Clone)]
pub struct EditorOutput {
    pub changed: bool,
}

pub struct CodeEditor {
    lines: Vec<String>,
    cursor: Cursor,
    anchor: Option<Cursor>,
    desired_col: Option<usize>,

    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    last_snapshot_kind: u8,
    last_snapshot_time: f64,

    /// 0-based index of the line holding the last parse error, if any.
    pub error_line: Option<usize>,

    /// Widget id salt (unique per tab).
    salt: String,

    /// Pending search highlight query (set by `find_next` / clear_search).
    search_query: Option<String>,
}

impl CodeEditor {
    pub fn new(salt: impl Into<String>) -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor { line: 0, col: 0 },
            anchor: None,
            desired_col: None,
            undo: Vec::new(),
            redo: Vec::new(),
            last_snapshot_kind: 0,
            last_snapshot_time: 0.0,
            error_line: None,
            salt: salt.into(),
            search_query: None,
        }
    }

    pub fn with_text(text: impl Into<String>, salt: impl Into<String>) -> Self {
        let mut editor = Self::new(salt);
        editor.set_text(text);
        editor
    }

    /// Construct an editor pre-loaded with `text` (used for opened files and
    /// the seeded example).

    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(str::to_string).collect()
        };
        self.cursor = Cursor { line: 0, col: 0 };
        self.anchor = None;
        self.desired_col = None;
        self.undo.clear();
        self.redo.clear();
        self.error_line = None;
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Used by unit tests; cheap and useful to keep.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn selection_text(&self) -> Option<String> {
        let (a, b) = self.sel_range()?;
        Some(self.range_text(a, b))
    }

    // ------------------------------------------------------------ helpers

    fn line_len(&self, line: usize) -> usize {
        self.lines.get(line).map(|l| l.chars().count()).unwrap_or(0)
    }

    fn clamp_cursor(&self, c: Cursor) -> Cursor {
        let last = self.lines.len().saturating_sub(1);
        let line = c.line.min(last);
        Cursor {
            line,
            col: c.col.min(self.line_len(line)),
        }
    }

    /// Ordered selection range (None when collapsed).
    fn sel_range(&self) -> Option<(Cursor, Cursor)> {
        let anchor = self.anchor?;
        let a = self.clamp_cursor(anchor);
        let b = self.clamp_cursor(self.cursor);
        if a.key() == b.key() {
            None
        } else if a.key() <= b.key() {
            Some((a, b))
        } else {
            Some((b, a))
        }
    }

    fn range_text(&self, a: Cursor, b: Cursor) -> String {
        let mut out = String::new();
        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .skip(a.line)
            .take(b.line - a.line + 1)
        {
            let chars: Vec<char> = line.chars().collect();
            let start = if i == a.line {
                a.col.min(chars.len())
            } else {
                0
            };
            let end = if i == b.line {
                b.col.min(chars.len())
            } else {
                chars.len()
            };
            out.extend(&chars[start..end]);
            if i < b.line {
                out.push('\n');
            }
        }
        out
    }

    /// Replace range `[a, b)` with `replacement` (may contain newlines).
    /// Returns the cursor at the end of the *inserted* text.
    fn splice_replace(&mut self, a: Cursor, b: Cursor, replacement: &str) -> Cursor {
        let a = self.clamp_cursor(a);
        let b = self.clamp_cursor(b);
        debug_assert!(a.key() <= b.key(), "range must be ordered");

        let head: String = self.lines[a.line].chars().take(a.col).collect();
        let tail: String = self.lines[b.line].chars().skip(b.col).collect();
        let pieces: Vec<String> = replacement.split('\n').map(str::to_string).collect();

        let end;
        if pieces.len() == 1 {
            let col = a.col + pieces[0].chars().count();
            end = Cursor { line: a.line, col };
            let joined = format!("{}{}{}", head, pieces[0], tail);
            self.lines.splice(a.line..=b.line, std::iter::once(joined));
        } else {
            let last_piece_len = pieces[pieces.len() - 1].chars().count();
            let mut new_lines: Vec<String> = Vec::with_capacity(pieces.len());
            new_lines.push(format!("{}{}", head, pieces[0]));
            new_lines.extend(pieces[1..pieces.len() - 1].iter().cloned());
            new_lines.push(format!("{}{}", pieces[pieces.len() - 1], tail));
            end = Cursor {
                line: a.line + new_lines.len() - 1,
                col: last_piece_len,
            };
            self.lines.splice(a.line..=b.line, new_lines);
        }
        end
    }

    // ------------------------------------------------------------ mutations

    fn snapshot(&mut self, kind: u8, now: f64) {
        // Coalesce rapid typing/backspacing/deleting into one undo step.
        let coalesce =
            kind != 0 && kind == self.last_snapshot_kind && now - self.last_snapshot_time < 0.4;
        if !coalesce {
            self.undo.push(Snapshot {
                lines: self.lines.clone(),
                cursor: self.cursor,
            });
            if self.undo.len() > 200 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        self.last_snapshot_kind = kind;
        self.last_snapshot_time = now;
    }

    /// Delete the active selection if any; returns the collapsed cursor.
    fn collapse_selection(&mut self) -> Cursor {
        if let Some((a, _b)) = self.sel_range() {
            self.splice_replace(a, _b, "");
            self.anchor = None;
            Cursor {
                line: a.line,
                col: a.col,
            }
        } else {
            self.clamp_cursor(self.cursor)
        }
    }

    pub fn undo(&mut self) {
        let Some(snap) = self.undo.pop() else {
            return;
        };
        self.redo.push(Snapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        });
        self.lines = snap.lines;
        self.cursor = self.clamp_cursor(snap.cursor);
        self.anchor = None;
        self.desired_col = None;
        self.last_snapshot_kind = 0;
    }

    pub fn redo(&mut self) {
        let Some(snap) = self.redo.pop() else {
            return;
        };
        self.undo.push(Snapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        });
        self.lines = snap.lines;
        self.cursor = self.clamp_cursor(snap.cursor);
        self.anchor = None;
        self.desired_col = None;
        self.last_snapshot_kind = 0;
    }

    // ------------------------------------------------------------ navigation

    fn move_horiz(&mut self, delta: isize, extend: bool) {
        let mut c = self.clamp_cursor(self.cursor);
        let step = delta.signum();
        for _ in 0..delta.abs() {
            if step < 0 {
                if c.col > 0 {
                    c.col -= 1;
                } else if c.line > 0 {
                    c.line -= 1;
                    c.col = self.line_len(c.line);
                }
            } else if c.col < self.line_len(c.line) {
                c.col += 1;
            } else if c.line < self.lines.len() - 1 {
                c.line += 1;
                c.col = 0;
            }
        }
        self.desired_col = None;
        if !extend {
            self.anchor = None;
        }
        self.cursor = c;
    }

    fn move_line(&mut self, delta: isize, extend: bool) {
        let mut c = self.clamp_cursor(self.cursor);
        let want = self.desired_col.unwrap_or(c.col);
        let step = delta.signum();
        for _ in 0..delta.abs() {
            let next = c.line as isize + step;
            if next < 0 || next > self.lines.len() as isize - 1 {
                break;
            }
            c.line = next as usize;
        }
        c.col = want.min(self.line_len(c.line));
        self.desired_col = Some(want);
        if !extend {
            self.anchor = None;
        }
        self.cursor = c;
    }

    fn home_or_end(&mut self, home: bool, extend: bool) {
        let c = self.clamp_cursor(self.cursor);
        if extend {
            self.anchor = self.anchor.or(Some(c));
        } else {
            self.anchor = None;
        }
        let col = if home {
            let line = &self.lines[c.line];
            let first_non = line.chars().count() - line.trim_start().chars().count();
            if c.col == first_non {
                0
            } else {
                first_non
            }
        } else {
            self.line_len(c.line)
        };
        self.cursor = Cursor { line: c.line, col };
        self.desired_col = None;
    }

    fn select_all(&mut self) {
        self.anchor = Some(Cursor { line: 0, col: 0 });
        let last = self.lines.len() - 1;
        self.cursor = Cursor {
            line: last,
            col: self.line_len(last),
        };
    }

    fn select_word_at(&mut self, c: Cursor) {
        let line = self.lines.get(c.line).cloned().unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        if c.col >= chars.len() {
            return;
        }
        let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';
        if !is_word(chars[c.col]) {
            return;
        }
        let mut start = c.col;
        let mut end = c.col;
        while start > 0 && is_word(chars[start - 1]) {
            start -= 1;
        }
        while end + 1 < chars.len() && is_word(chars[end + 1]) {
            end += 1;
        }
        self.anchor = Some(Cursor {
            line: c.line,
            col: start,
        });
        self.cursor = Cursor {
            line: c.line,
            col: end + 1,
        };
    }

    /// Insert `text` (may contain newlines) at `c`; returns end cursor.
    fn insert_at(&mut self, c: Cursor, text: &str) -> Cursor {
        self.splice_replace(c, c, text)
    }

    /// Prefix each line in the (partial) selection with 4 spaces.
    fn indent(&mut self) {
        let Some((a, b)) = self.sel_range() else {
            return;
        };
        for li in a.line..=b.line {
            let start = Cursor { line: li, col: 0 };
            self.splice_replace(start, start, "    ");
        }
        let last = self.lines.len() - 1;
        let _ = last;
        self.cursor = Cursor {
            line: b.line,
            col: self.line_len(b.line),
        };
        self.anchor = Some(Cursor {
            line: a.line,
            col: 0,
        });
    }

    /// Remove up to 4 leading spaces on each line in the selection.
    fn outdent(&mut self) {
        let Some((a, b)) = self.sel_range() else {
            return;
        };
        for li in a.line..=b.line {
            let spaces = self
                .lines
                .get(li)
                .map(|l| l.chars().take_while(|c| *c == ' ').count().min(4))
                .unwrap_or(0);
            if spaces > 0 {
                self.splice_replace(
                    Cursor { line: li, col: 0 },
                    Cursor {
                        line: li,
                        col: spaces,
                    },
                    "",
                );
            }
        }
        self.cursor = Cursor {
            line: b.line,
            col: self.line_len(b.line),
        };
        self.anchor = Some(Cursor {
            line: a.line,
            col: 0,
        });
    }

    /// Insert a code snippet at the cursor (used by the Plots toolstrip and
    /// welcome screen). Returns nothing; `show` reports changes next frame.
    pub fn insert_snippet_at_cursor(&mut self, snippet: &str) {
        self.snapshot(0, 0.0);
        let start = self.collapse_selection();
        self.cursor = self.insert_at(start, snippet);
        self.anchor = None;
    }

    // ------------------------------------------------------------ search

    /// Jump to the next occurrence of `query` from the cursor, selecting it.
    /// `backwards` selects the previous match (wrapping). Returns true if found.
    pub fn find_next(&mut self, query: &str, backwards: bool) -> bool {
        if query.is_empty() {
            return false;
        }
        self.search_query = Some(query.to_string());
        let start = self.clamp_cursor(self.cursor);
        let qlen = query.chars().count();
        let needle: Vec<char> = query.chars().collect();
        let mut hits: Vec<Cursor> = Vec::new();
        for (li, line) in self.lines.iter().enumerate() {
            let hay: Vec<char> = line.chars().collect();
            if hay.len() < qlen {
                continue;
            }
            for ci in 0..=hay.len() - qlen {
                if hay[ci..ci + qlen] == needle[..] {
                    hits.push(Cursor { line: li, col: ci });
                }
            }
        }
        if hits.is_empty() {
            return false;
        }
        let target = if backwards {
            match hits.iter().rev().find(|h| h.key() < start.key()) {
                Some(h) => *h,
                None => hits[hits.len() - 1],
            }
        } else {
            match hits.iter().find(|h| h.key() > start.key()) {
                Some(h) => *h,
                None => hits[0],
            }
        };
        self.anchor = Some(target);
        self.cursor = Cursor {
            line: target.line,
            col: target.col + qlen,
        };
        true
    }

    pub fn clear_search(&mut self) {
        self.search_query = None;
    }
}

// ---------------------------------------------------------------- show (painting + input)

impl CodeEditor {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        kind: ThemeKind,
        palette: &SyntaxPalette,
    ) -> EditorOutput {
        let metrics = metrics(ui);
        let mut changed = false;

        let id = egui::Id::unique(("matrust_editor", self.salt.clone()));
        let gutter_w = gutter_width(self.lines.len(), &metrics);
        let max_chars = self
            .lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);
        let content_w = gutter_w + max_chars as f32 * metrics.char_w + 24.0;
        let content_h = self.lines.len() as f32 * metrics.line_h + 12.0;

        // Snapshot for painting (cursor updates land next frame; visually fine).
        let paint_lines = self.lines.clone();
        let sel = self.sel_range();
        let caret_screen = self.clamp_cursor(self.cursor);

        let egui_out = egui::ScrollArea::both()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(content_w, content_h + 8.0),
                    Sense::click_and_drag(),
                );
                let painter = ui.painter();
                let text_x = rect.min.x + gutter_w;
                let row_center_y =
                    |li: usize| rect.min.y + li as f32 * metrics.line_h + metrics.line_h * 0.5;

                // Row backgrounds (current line + error line)
                for li in 0..paint_lines.len() {
                    let row_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(0.0, li as f32 * metrics.line_h),
                        egui::vec2(content_w.max(rect.width()), metrics.line_h),
                    );
                    if Some(li) == self.error_line {
                        painter.rect_filled(row_rect, 0.0, palette.error_bg);
                    }
                    if caret_screen.line == li {
                        painter.rect_filled(row_rect, 0.0, palette.line_highlight);
                    }
                }
                // Selection fills
                if let Some((a, b)) = sel {
                    for li in a.line..=b.line {
                        let col_start = if li == a.line { a.col as f32 } else { 0.0 };
                        let col_end = if li == b.line {
                            b.col as f32
                        } else {
                            self.line_len(li) as f32
                        };
                        let sel_rect = egui::Rect::from_min_size(
                            egui::pos2(
                                text_x + col_start * metrics.char_w,
                                rect.min.y + li as f32 * metrics.line_h,
                            ),
                            egui::vec2((col_end - col_start) * metrics.char_w, metrics.line_h),
                        );
                        painter.rect_filled(sel_rect, 0.0, palette.selection_bg);
                    }
                }
                // Search highlights
                if let Some(query) = &self.search_query {
                    let q: Vec<char> = query.chars().collect();
                    if !q.is_empty() {
                        for (li, line) in paint_lines.iter().enumerate() {
                            let hay: Vec<char> = line.chars().collect();
                            if hay.len() < q.len() {
                                continue;
                            }
                            for ci in 0..=hay.len() - q.len() {
                                if hay[ci..ci + q.len()] == q[..] {
                                    let mrect = egui::Rect::from_min_size(
                                        egui::pos2(
                                            text_x + ci as f32 * metrics.char_w,
                                            rect.min.y + li as f32 * metrics.line_h,
                                        ),
                                        egui::vec2(q.len() as f32 * metrics.char_w, metrics.line_h),
                                    );
                                    painter.rect_filled(
                                        mrect,
                                        0.0,
                                        crate::theme::tint(crate::theme::accent(kind), 0.6),
                                    );
                                }
                            }
                        }
                    }
                }

                // Rows: gutter + syntax-colored text
                for (li, line) in paint_lines.iter().enumerate() {
                    let gcolor = if caret_screen.line == li {
                        palette.gutter_active
                    } else {
                        palette.gutter
                    };
                    painter.text(
                        egui::pos2(rect.min.x + gutter_w - 10.0, row_center_y(li)),
                        Align2::RIGHT_CENTER,
                        format!("{}", li + 1),
                        metrics.font.clone(),
                        gcolor,
                    );
                    painter.text(
                        egui::pos2(text_x, row_center_y(li)),
                        Align2::LEFT_CENTER,
                        line.clone(),
                        metrics.font.clone(),
                        palette.ident,
                    );
                    for token in colorize_row(line, palette) {
                        if token.color == palette.ident {
                            continue; // base pass already painted identifiers
                        }
                        let sub: String = line.chars().skip(token.start).take(token.len).collect();
                        if sub.is_empty() {
                            continue;
                        }
                        painter.text(
                            egui::pos2(
                                text_x + token.start as f32 * metrics.char_w,
                                row_center_y(li),
                            ),
                            Align2::LEFT_CENTER,
                            sub,
                            metrics.font.clone(),
                            token.color,
                        );
                    }
                }
                // Caret (blinks when focused)
                if response.has_focus() && ui.input(|i| (i.time % 1.2) < 0.8) {
                    let caret = egui::Rect::from_center_size(
                        egui::pos2(
                            text_x + caret_screen.col as f32 * metrics.char_w,
                            row_center_y(caret_screen.line),
                        ),
                        egui::vec2(2.0, metrics.line_h - 4.0),
                    );
                    painter.rect_filled(caret, 0.0, palette.caret);
                }

                (rect, response)
            });
        let (rect, response) = egui_out.inner;

        // ------------------------------------------------------ mouse input
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let c = self.cursor_at(pos, &rect, &metrics);
                self.cursor = c;
                self.anchor = Some(c);
                self.desired_col = None;
                response.request_focus();
            }
        }
        if response.double_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let c = self.cursor_at(pos, &rect, &metrics);
                self.select_word_at(c);
            }
        }
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let c = self.cursor_at(pos, &rect, &metrics);
                self.cursor = c; // extends from existing anchor
                self.desired_col = None;
            }
        }

        // --------------------------------------------------- keyboard input
        if response.has_focus() {
            let events: Vec<Event> = ui.input(|i| i.events.clone());
            let now = ui.input(|i| i.time);
            for ev in &events {
                match ev {
                    Event::Text(t) => {
                        if t.contains('\n') || t.contains('\r') {
                            continue;
                        }
                        self.snapshot(1, now);
                        let start = self.collapse_selection();
                        self.cursor = self.insert_at(start, t);
                        self.anchor = None;
                        changed = true;
                    }
                    Event::Paste(t) => {
                        self.snapshot(0, now);
                        let start = self.collapse_selection();
                        self.cursor = self.insert_at(start, t);
                        self.anchor = None;
                        changed = true;
                    }
                    Event::Copy => {
                        if let Some(sel_text) = self.selection_text() {
                            ui.ctx().copy_text(sel_text);
                        }
                    }
                    Event::Cut => {
                        if let Some(sel_text) = self.selection_text() {
                            ui.ctx().copy_text(sel_text);
                            self.snapshot(0, now);
                            self.collapse_selection();
                            changed = true;
                        }
                    }
                    Event::Key {
                        key,
                        modifiers,
                        pressed: true,
                        repeat: _,
                        physical_key: _,
                    } => {
                        changed |= self.handle_key(
                            *key,
                            modifiers.shift,
                            modifiers.ctrl || modifiers.command,
                            now,
                        );
                    }
                    _ => {}
                }
            }
            // Scroll caret into view whenever it may have moved.
            let caret_pos = self.clamp_cursor(self.cursor);
            let caret_x = rect.min.x + gutter_w + caret_pos.col as f32 * metrics.char_w;
            let caret_y = rect.min.y + caret_pos.line as f32 * metrics.line_h;
            ui.scroll_to_rect(
                egui::Rect::from_min_size(
                    egui::pos2(caret_x, caret_y),
                    egui::vec2(metrics.char_w, metrics.line_h),
                ),
                None,
            );
        }

        EditorOutput { changed }
    }

    fn cursor_at(&self, pointer: egui::Pos2, rect: &egui::Rect, m: &Metrics) -> Cursor {
        let line = ((pointer.y - rect.min.y) / m.line_h).floor() as isize;
        let line = line.clamp(0, self.lines.len() as isize - 1) as usize;
        let text_x = rect.min.x + gutter_width(self.lines.len(), m);
        let col = ((pointer.x - text_x) / m.char_w).round() as isize;
        let col = col.clamp(0, self.line_len(line) as isize) as usize;
        Cursor { line, col }
    }

    fn handle_key(&mut self, key: Key, shift: bool, ctrl: bool, now: f64) -> bool {
        match key {
            Key::Enter => {
                self.snapshot(1, now);
                let start = self.collapse_selection();
                self.cursor = self.insert_at(start, "\n");
                self.anchor = None;
                true
            }
            Key::Backspace => {
                self.snapshot(2, now);
                if self.sel_range().is_some() {
                    self.collapse_selection();
                    self.anchor = None;
                } else {
                    let c = self.clamp_cursor(self.cursor);
                    let target = if c.col > 0 {
                        Some(Cursor {
                            line: c.line,
                            col: c.col - 1,
                        })
                    } else if c.line > 0 {
                        Some(Cursor {
                            line: c.line - 1,
                            col: self.line_len(c.line - 1),
                        })
                    } else {
                        None
                    };
                    if let Some(prev) = target {
                        self.cursor = self.splice_replace(prev, c, "");
                    }
                    self.anchor = None;
                }
                true
            }
            Key::Delete => {
                self.snapshot(3, now);
                if self.sel_range().is_some() {
                    self.collapse_selection();
                    self.anchor = None;
                } else {
                    let c = self.clamp_cursor(self.cursor);
                    let target = if c.col < self.line_len(c.line) {
                        Some(Cursor {
                            line: c.line,
                            col: c.col + 1,
                        })
                    } else if c.line < self.lines.len() - 1 {
                        Some(Cursor {
                            line: c.line + 1,
                            col: 0,
                        })
                    } else {
                        None
                    };
                    if let Some(next) = target {
                        self.cursor = self.splice_replace(c, next, "");
                    }
                    self.anchor = None;
                }
                true
            }
            Key::Tab => {
                if shift {
                    self.snapshot(0, now);
                    self.outdent();
                } else if self.sel_range().is_some() {
                    self.snapshot(0, now);
                    self.indent();
                } else {
                    self.snapshot(1, now);
                    let start = self.collapse_selection();
                    self.cursor = self.insert_at(start, "    ");
                    self.anchor = None;
                }
                true
            }
            Key::ArrowLeft => {
                self.move_horiz(-1, shift);
                true
            }
            Key::ArrowRight => {
                self.move_horiz(1, shift);
                true
            }
            Key::ArrowUp => {
                self.move_line(-1, shift);
                true
            }
            Key::ArrowDown => {
                self.move_line(1, shift);
                true
            }
            Key::Home => {
                self.home_or_end(true, shift);
                true
            }
            Key::End => {
                self.home_or_end(false, shift);
                true
            }
            Key::PageUp => {
                self.move_line(-12, shift);
                true
            }
            Key::PageDown => {
                self.move_line(12, shift);
                true
            }
            Key::Escape => {
                self.anchor = None;
                true
            }
            Key::A if ctrl => {
                self.select_all();
                true
            }
            Key::Z if ctrl && !shift => {
                self.undo();
                true
            }
            Key::Z if ctrl && shift => {
                self.redo();
                true
            }
            Key::Y if ctrl => {
                self.redo();
                true
            }
            _ => false,
        }
    }
}

fn gutter_width(line_count: usize, m: &Metrics) -> f32 {
    let digits = line_count.to_string().len().max(3);
    digits as f32 * m.char_w + 20.0
}

fn metrics(ui: &egui::Ui) -> Metrics {
    let font = match ui.style().text_styles.get(&TextStyle::Monospace) {
        Some(f) => f.clone(),
        None => FontId::monospace(13.5),
    };
    ui.ctx().fonts_mut(|f| {
        let probe = f.layout_no_wrap("00000".to_string(), font.clone(), Color32::WHITE);
        let size = probe.size();
        Metrics {
            char_w: (size.x / 5.0).max(1.0),
            line_h: size.y.max(font.size * 1.2),
            font,
        }
    })
}

// ---------------------------------------------------------------- coloring

#[derive(Debug, Clone, Copy)]
struct Token {
    start: usize, // char index of start
    len: usize,
    color: Color32,
}

fn colorize_row(line: &str, p: &SyntaxPalette) -> Vec<Token> {
    let chars: Vec<char> = line.chars().collect();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0;
    let prev_is_word = |i: usize| -> bool {
        if i == 0 {
            return false;
        }
        let c = chars[i - 1];
        c.is_alphanumeric() || c == '_' || c == ')' || c == ']' || c == '}' || c == '\''
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '%' {
            out.push(Token {
                start: i,
                len: chars.len() - i,
                color: p.comment,
            });
            break;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '\'' {
            if prev_is_word(i) {
                out.push(Token {
                    start: i,
                    len: 1,
                    color: p.operator,
                }); // transpose
                i += 1;
            } else {
                // string literal; '' escapes a quote inside
                let mut j = i + 1;
                loop {
                    if j >= chars.len() {
                        break;
                    }
                    if chars[j] == '\'' {
                        if j + 1 < chars.len() && chars[j + 1] == '\'' {
                            j += 2;
                            continue;
                        }
                        j += 1;
                        break;
                    }
                    j += 1;
                }
                out.push(Token {
                    start: i,
                    len: j - i,
                    color: p.string,
                });
                i = j;
            }
        } else if c.is_ascii_digit()
            || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                j += 1;
            }
            if j < chars.len() && (chars[j] == 'e' || chars[j] == 'E') {
                let mut k = j + 1;
                if k < chars.len() && (chars[k] == '+' || chars[k] == '-') {
                    k += 1;
                }
                if k < chars.len() && chars[k].is_ascii_digit() {
                    while k < chars.len() && chars[k].is_ascii_digit() {
                        k += 1;
                    }
                    j = k;
                }
            }
            out.push(Token {
                start: i,
                len: j - i,
                color: p.number,
            });
            i = j;
        } else if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let color = if KEYWORDS.contains(&word.as_str()) {
                p.keyword
            } else if BUILTINS.contains(&word.as_str()) {
                p.builtin
            } else {
                p.ident
            };
            out.push(Token {
                start: i,
                len: j - i,
                color,
            });
            i = j;
        } else {
            out.push(Token {
                start: i,
                len: 1,
                color: p.operator,
            });
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_read_text_roundtrip() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("A = [1 2; 3 4];\ndisp(A);\n");
        assert_eq!(ed.line_count(), 3);
        assert_eq!(ed.text(), "A = [1 2; 3 4];\ndisp(A);\n");
    }

    #[test]
    fn insert_multiline_at_cursor() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("ab\ncd");
        ed.cursor = Cursor { line: 0, col: 1 };
        ed.cursor = ed.insert_at(ed.cursor, "\nx");
        assert_eq!(ed.text(), "a\nxb\ncd");
        assert_eq!(ed.cursor(), Cursor { line: 1, col: 1 });
    }

    #[test]
    fn splice_replaces_range_in_middle() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("hello world");
        ed.cursor = ed.splice_replace(
            Cursor { line: 0, col: 6 },
            Cursor { line: 0, col: 11 },
            "there",
        );
        assert_eq!(ed.text(), "hello there");
        assert_eq!(ed.cursor(), Cursor { line: 0, col: 11 });
    }

    #[test]
    fn undo_redo_restore_text() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("a = 1;");
        ed.snapshot(0, 1.0);
        ed.cursor = ed.insert_at(Cursor { line: 0, col: 5 }, " 2");
        assert_eq!(ed.text(), "a = 1 2;");
        ed.undo();
        assert_eq!(ed.text(), "a = 1;");
        ed.redo();
        assert_eq!(ed.text(), "a = 1 2;");
    }

    #[test]
    fn selection_fetches_range_text() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("zero\none\ntwo\n");
        ed.anchor = Some(Cursor { line: 0, col: 2 });
        ed.cursor = Cursor { line: 2, col: 1 };
        assert_eq!(ed.selection_text(), Some("ro\none\nt".to_string()));
    }

    #[test]
    fn find_next_wraps_and_selects() {
        let mut ed = CodeEditor::new("t");
        ed.set_text("alpha beta alpha");
        ed.cursor = Cursor { line: 0, col: 0 };
        assert!(ed.find_next("alpha", false));
        assert_eq!(ed.selection_text(), Some("alpha".to_string()));
        // Strictly-after search: the cursor sits at the start of a match that
        // begins after the caret, on the end of the found match.
        assert_eq!(ed.cursor(), Cursor { line: 0, col: 16 });
        // wraps back to the first occurrence
        assert!(ed.find_next("alpha", false));
        assert_eq!(ed.cursor(), Cursor { line: 0, col: 5 });
    }

    #[test]
    fn colorize_classifies_tokens() {
        let p = SyntaxPalette::dark();
        let line = "x = 3 + sqrt(2) 's' + x' % note";
        let tokens = colorize_row(line, &p);
        let text_at =
            |start: usize, len: usize| line.chars().skip(start).take(len).collect::<String>();
        let mut by_text: Vec<(String, Color32)> = Vec::new();
        for t in tokens {
            by_text.push((text_at(t.start, t.len), t.color));
        }
        assert!(by_text.contains(&("sqrt".to_string(), p.builtin)));
        assert!(by_text.contains(&("3".to_string(), p.number)));
        assert!(by_text.contains(&("+".to_string(), p.operator)));
        assert!(by_text.contains(&("'s'".to_string(), p.string)));
        assert!(by_text.contains(&("x".to_string(), p.ident)));
        // trailing transpose is one char operator
        assert!(by_text.contains(&("'".to_string(), p.operator)));
        assert!(by_text.contains(&("% note".to_string(), p.comment)));
    }

    #[test]
    fn keyword_and_builtin_lists_cover_runtime() {
        // Keep highlighting in sync with the real runtime dispatcher.
        for name in BUILTINS {
            assert!(
                !KEYWORDS.contains(name),
                "{} is both keyword and builtin",
                name
            );
        }
        assert!(BUILTINS.contains(&"plot"));
    }
}
