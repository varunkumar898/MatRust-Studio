//! Workspace variable browser and detailed matrix inspector.

use crate::actions::AppAction;
use crate::execution::VariableRow;
use crate::theme::Theme;
use egui::{RichText, ScrollArea, Ui, Window};

#[derive(Default)]
pub struct WorkspacePanel {
    pub search_query: String,
    pub selected_variable: Option<VariableRow>,
    pub inspector_open: bool,
}

impl WorkspacePanel {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        variables: &[VariableRow],
        on_clear: &mut bool,
        on_refresh: &mut bool,
    ) {
        ui.vertical(|ui| {
            // Header toolbar. Clear Workspace wraps below the title in a
            // narrow panel instead of overlapping Refresh.
            ui.horizontal_wrapped(|ui| {
                ui.heading("Workspace");
                let refresh = ui.button("Refresh").on_hover_text("Refresh variable list");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "workspace.refresh",
                    refresh.id,
                    refresh.rect,
                );
                if refresh.clicked() {
                    *on_refresh = true;
                }
                let clear = ui
                    .button("Clear Workspace")
                    .on_hover_text("Clear workspace variables after confirmation");
                AppAction::ClearWorkspace.register_control(ui.ctx(), clear.id, clear.rect);
                if clear.clicked() {
                    *on_clear = true;
                }
            });

            ui.add_space(4.0);

            // Filter search box
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.search_query);
                if !self.search_query.is_empty() && ui.small_button("X").clicked() {
                    self.search_query.clear();
                }
            });

            ui.separator();

            // Variable table headers
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").strong().color(theme.text_secondary));
                ui.separator();
                ui.label(RichText::new("Value").strong().color(theme.text_secondary));
                ui.separator();
                ui.label(RichText::new("Size").strong().color(theme.text_secondary));
                ui.separator();
                ui.label(RichText::new("Class").strong().color(theme.text_secondary));
            });
            ui.separator();

            let query = self.search_query.to_lowercase();
            let filtered: Vec<&VariableRow> = variables
                .iter()
                .filter(|v| query.is_empty() || v.name.to_lowercase().contains(&query))
                .collect();

            if filtered.is_empty() {
                ui.centered_and_justified(|ui| {
                    if variables.is_empty() {
                        ui.label(
                            RichText::new("No variables in workspace")
                                .italics()
                                .color(theme.text_secondary),
                        );
                    } else {
                        ui.label(
                            RichText::new("No matching variables")
                                .italics()
                                .color(theme.text_secondary),
                        );
                    }
                });
                return;
            }

            ScrollArea::vertical()
                .id_salt("workspace_rows_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for var in filtered {
                        // G5: stable per-row id keyed by the variable name.
                        ui.push_id(("ws_row", var.name.as_str()), |ui| {
                            let is_selected =
                                self.selected_variable.as_ref().map(|s| &s.name) == Some(&var.name);

                            let row_resp = ui.horizontal(|ui| {
                                let name_label = RichText::new(&var.name)
                                    .color(if is_selected {
                                        theme.accent
                                    } else {
                                        theme.text_primary
                                    })
                                    .strong();
                                ui.label(name_label);

                                ui.label(RichText::new(&var.preview).color(theme.text_secondary));
                                ui.label(RichText::new(&var.size_str).color(theme.text_secondary));
                                ui.label(RichText::new(&var.type_name).color(theme.syn_builtin));
                            });

                            if row_resp.response.interact(egui::Sense::click()).clicked() {
                                self.selected_variable = Some(var.clone());
                            }

                            if row_resp.response.double_clicked() {
                                self.selected_variable = Some(var.clone());
                                self.inspector_open = true;
                            }
                        });
                    }
                });
        });
    }

    pub fn show_inspector(&mut self, ctx: &egui::Context, _theme: &Theme) {
        if !self.inspector_open {
            return;
        }

        let mut open = self.inspector_open;
        if let Some(ref var) = self.selected_variable {
            Window::new(format!("Variable: {}", var.name))
                .open(&mut open)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Class:").strong());
                        ui.label(&var.type_name);
                        ui.separator();
                        ui.label(RichText::new("Size:").strong());
                        ui.label(&var.size_str);
                        ui.separator();
                        ui.label(RichText::new("Bytes:").strong());
                        ui.label(format!("{} B", var.bytes));
                    });
                    ui.separator();

                    ui.label(RichText::new("Value Preview:").strong());
                    ScrollArea::both().show(ui, |ui| {
                        ui.monospace(&var.preview);
                    });
                });
        }
        self.inspector_open = open;
    }
}
