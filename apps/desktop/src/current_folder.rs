//! Filesystem current folder browser and project explorer.

use crate::theme::Theme;
use egui::{RichText, ScrollArea, Ui};
use std::fs;
use std::path::PathBuf;

pub struct CurrentFolderPanel {
    pub current_dir: PathBuf,
    pub selected_file: Option<PathBuf>,
}

impl Default for CurrentFolderPanel {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            selected_file: None,
        }
    }
}

impl CurrentFolderPanel {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, open_file_request: &mut Option<PathBuf>) {
        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.heading("Current Folder");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("⮤ Up")
                        .on_hover_text("Navigate to parent directory")
                        .clicked()
                    {
                        if let Some(parent) = self.current_dir.parent() {
                            self.current_dir = parent.to_path_buf();
                        }
                    }
                });
            });

            // Path breadcrumb
            let path_str = self.current_dir.to_str().unwrap_or(".");
            ui.label(RichText::new(path_str).small().color(theme.text_secondary));
            ui.separator();

            // Read directory entries
            let entries = match fs::read_dir(&self.current_dir) {
                Ok(read) => {
                    let mut items: Vec<_> = read.filter_map(|e| e.ok().map(|e| e.path())).collect();
                    items.sort_by(|a, b| {
                        let a_dir = a.is_dir();
                        let b_dir = b.is_dir();
                        if a_dir && !b_dir {
                            std::cmp::Ordering::Less
                        } else if !a_dir && b_dir {
                            std::cmp::Ordering::Greater
                        } else {
                            a.file_name().cmp(&b.file_name())
                        }
                    });
                    items
                }
                Err(err) => {
                    ui.colored_label(theme.error, format!("Cannot read folder: {err}"));
                    return;
                }
            };

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for path in entries {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                        // Determine icon & color
                        let (icon, color) = if path.is_dir() {
                            ("📁", theme.text_primary)
                        } else if file_name.ends_with(".m") {
                            ("📄", theme.accent)
                        } else if file_name.ends_with(".mat") {
                            ("📊", theme.syn_builtin)
                        } else if file_name.ends_with(".csv") {
                            ("📑", theme.warning)
                        } else {
                            ("  ", theme.text_secondary)
                        };

                        let is_selected = self.selected_file.as_ref() == Some(&path);
                        let label_text = format!("{icon} {file_name}");

                        let resp = ui
                            .selectable_label(is_selected, RichText::new(label_text).color(color));

                        if resp.clicked() {
                            self.selected_file = Some(path.clone());
                        }

                        if resp.double_clicked() {
                            if path.is_dir() {
                                self.current_dir = path;
                                break;
                            } else {
                                *open_file_request = Some(path);
                            }
                        }
                    }
                });
        });
    }
}
