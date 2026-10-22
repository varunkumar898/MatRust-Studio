//! Workspace panel + variable inspector: live view of the interpreter
//! environment. Data comes from `variable_rows()` on the environment
//! snapshot; the inspector reads actual variable values.

use egui::Color32;

use matlab_runtime::VarRow;
use matlab_values::{DenseMatrix, Value};

use crate::theme::{accent, error_color, text_color, text_color_faint, ThemeKind};

#[derive(Debug, Default)]
pub struct WorkspaceState {
    pub rows: Vec<VarRow>,
    pub selected: Option<String>,
    /// Name of the variable shown in the inspector window (if any).
    pub inspector: Option<String>,
}

impl WorkspaceState {
    pub fn update_from(&mut self, rows: Vec<VarRow>) {
        self.rows = rows;
        if let Some(sel) = &self.selected {
            if !self.rows.iter().any(|r| &r.name == sel) {
                self.selected = None;
            }
        }
        if let Some(ins) = &self.inspector {
            if !self.rows.iter().any(|r| &r.name == ins) {
                self.inspector = None;
            }
        }
    }
}

pub enum WorkspaceAction {
    None,
    ClearWorkspace,
}

/// Header row: title, count, and actions. Shown above the listing.
pub fn show_header(
    ui: &mut egui::Ui,
    state: &mut WorkspaceState,
    _kind: ThemeKind,
) -> WorkspaceAction {
    let mut action = WorkspaceAction::None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Workspace").strong());
        ui.weak(if state.rows.is_empty() {
            String::new()
        } else {
            format!("{}", state.rows.len())
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new("Clear").small())
                .on_hover_text("clear — remove all variables")
                .clicked()
            {
                action = WorkspaceAction::ClearWorkspace;
            }
            let enabled = state.selected.is_some();
            let inspect = ui
                .add_enabled(enabled, egui::Button::new("Inspect…").small())
                .on_hover_text(if enabled {
                    "Open the variable viewer for the selected variable"
                } else {
                    "Select a variable first"
                });
            if enabled && inspect.clicked() {
                state.inspector = state.selected.clone();
            }
        });
    });
    action
}

/// Rows only (no header). The caller supplies the scroll container; nothing
/// after the listing, so the panel content always fits.
pub fn show_body(ui: &mut egui::Ui, state: &mut WorkspaceState, kind: ThemeKind) -> Option<String> {
    if state.rows.is_empty() {
        ui.weak("No variables. Run a script or type assignments.");
        return None;
    }

    let mut opened: Option<String> = None;
    egui::Grid::new("workspace_table")
        .num_columns(4)
        .spacing([6.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Name");
            ui.strong("Value");
            ui.strong("Size");
            ui.strong("Type");
            ui.end_row();
            for row in &state.rows {
                let is_selected = state.selected.as_deref() == Some(row.name.as_str());
                let b = egui::Button::new(egui::RichText::new(row.name.clone()).monospace().color(
                    if is_selected {
                        text_color(kind)
                    } else {
                        text_color_faint(kind)
                    },
                ))
                .fill(if is_selected {
                    crate::theme::tint(accent(kind), 0.45)
                } else {
                    Color32::TRANSPARENT
                });
                let resp = ui.add(b);
                if resp.clicked() {
                    state.selected = Some(row.name.clone());
                }
                if resp.double_clicked() {
                    state.selected = Some(row.name.clone());
                    opened = Some(row.name.clone());
                }
                let value_color = if state.selected.as_deref() == Some(row.name.as_str()) {
                    text_color(kind)
                } else {
                    text_color_faint(kind)
                };
                ui.monospace(egui::RichText::new(shorten(&row.preview, 22)).color(value_color));
                ui.weak(if row.rows == 1 && row.cols == 1 {
                    "1x1".to_string()
                } else {
                    format!("{}x{}", row.rows, row.cols)
                });
                ui.weak(row.kind.clone());
                ui.end_row();
            }
        });
    opened
}

fn shorten(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}…", truncated)
    }
}

/// Inspector: grid of a single variable's contents.
pub fn show_inspector(ui: &mut egui::Ui, name: &str, value: Option<&Value>, kind: ThemeKind) {
    egui::Window::new(format!("Variable viewer — {}", name))
        .id(egui::Id::unique(("var_inspector", name.to_string())))
        .resizable(true)
        .default_width(560.0)
        .show(ui.ctx(), |ui| match value {
            None => {
                ui.label(format!("Variable '{}' no longer exists.", name));
            }
            Some(Value::Matrix(m)) => matrix_grid(ui, m, kind),
            Some(Value::String(s)) => {
                ui.monospace(format!("'{}'", s));
                ui.weak(format!("char ({})", s.chars().count()));
            }
            Some(Value::Boolean(b)) => {
                ui.monospace(if *b { "1" } else { "0" });
                ui.weak("logical");
            }
        });
}

const MAX_ROWS: usize = 200;
const MAX_COLS: usize = 60;

fn matrix_grid(ui: &mut egui::Ui, m: &DenseMatrix, kind: ThemeKind) {
    ui.label(format!(
        "double matrix, {}x{} ({} values)",
        m.rows,
        m.cols,
        m.data.len()
    ));
    if m.data.is_empty() {
        ui.weak("[] (empty)");
        return;
    }
    if m.is_scalar() {
        ui.monospace(m.format_matlab());
        return;
    }
    let disp_rows = m.rows.min(MAX_ROWS);
    let disp_cols = m.cols.min(MAX_COLS);
    egui::ScrollArea::both().show(ui, |ui| {
        let grid_id = egui::Id::unique(("var_grid", m.rows, m.cols));
        egui::Grid::new(grid_id)
            .striped(true)
            .spacing([10.0, 2.0])
            .show(ui, |grid| {
                grid.weak("   ");
                for c in 0..disp_cols {
                    grid.weak(format!("{}", c + 1));
                }
                grid.end_row();
                for r in 0..disp_rows {
                    grid.weak(format!("{}", r + 1));
                    for c in 0..disp_cols {
                        let v = m.data[r * m.cols + c];
                        let cell = cell_text(v);
                        grid.monospace(egui::RichText::new(cell).color(cell_color(v, kind)));
                    }
                    grid.end_row();
                }
            });
        if m.rows > disp_rows || m.cols > disp_cols {
            ui.weak(format!(
                "… showing {}x{} of {}x{} (scroll or increase view limits later)",
                disp_rows, disp_cols, m.rows, m.cols
            ));
        }
    });
}

fn cell_text(v: f64) -> String {
    if v.is_nan() {
        "NaN".to_string()
    } else if v.is_infinite() {
        if v > 0.0 {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn cell_color(v: f64, kind: ThemeKind) -> Color32 {
    if v.is_nan() || v.is_infinite() {
        error_color(kind)
    } else if v == 0.0 {
        text_color_faint(kind)
    } else {
        text_color(kind)
    }
}
