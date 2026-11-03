//! Add-On Explorer and Package Manager Panel.
//!
//! Provides a catalog of App Builder applications (.mlapp), toolbox packages
//! (.mlappinstall), and clean-room built-in utilities.

use crate::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Ui};

/// Information about an installable or built-in application / add-on.
#[derive(Clone, Debug)]
pub struct AddOnItem {
    pub id: &'static str,
    pub title: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub is_installed: bool,
    pub is_builtin: bool,
}

/// Catalog of the built-in launcher apps that ship with the IDE. Additional
/// packages (a real install/download system) are future work.
pub static DEFAULT_ADDONS: &[AddOnItem] = &[
    AddOnItem {
        id: "app-data-importer",
        title: "Data Importer & Table Cleaner",
        version: "1.2.0",
        author: "MATSTUDIO",
        category: "Data Analysis",
        description: "Interactive GUI application for previewing, formatting, and importing delimited CSV, TSV, and tabular data directly into workspace matrices and tables.",
        is_installed: true,
        is_builtin: true,
    },
    AddOnItem {
        id: "app-plot-explorer",
        title: "Plot & Curve Explorer",
        version: "1.1.0",
        author: "MATSTUDIO",
        category: "Visualization",
        description: "Interactive visualization tool for exploring 2D curves, multi-series plots, histograms, and scatter plots with interactive pan, zoom, and axis styling.",
        is_installed: true,
        is_builtin: true,
    },
    AddOnItem {
        id: "app-signal-viewer",
        title: "Signal & Waveform Analyzer",
        version: "1.0.0",
        author: "MATSTUDIO",
        category: "Signal Processing",
        description: "Time-domain and frequency-domain waveform viewer supporting FFT spectrum estimation, filtering, and window analysis.",
        is_installed: true,
        is_builtin: true,
    },
    AddOnItem {
        id: "app-block-diagram-inspector",
        title: "SLX Block Model Inspector",
        version: "1.0.0",
        author: "MATSTUDIO",
        category: "Simulation & Modeling",
        description: "Inspects block hierarchies, parameter schemas, input/output connectivity, and annotations within .slx block diagram model packages without requiring proprietary runtimes.",
        is_installed: true,
        is_builtin: true,
    },
];

/// State of the Add-On Explorer panel.
pub struct AddOnsPanel {
    pub search_query: String,
    pub selected_addon_id: Option<String>,
    pub installed_ids: Vec<String>,
}

impl Default for AddOnsPanel {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_addon_id: Some("app-data-importer".to_string()),
            installed_ids: DEFAULT_ADDONS
                .iter()
                .filter(|a| a.is_installed)
                .map(|a| a.id.to_string())
                .collect(),
        }
    }
}

impl AddOnsPanel {
    /// Renders the Add-On Explorer panel.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, on_launch_app: &mut Option<&'static str>) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🧩 Add-On Explorer").color(theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} Installed Packages", self.installed_ids.len()))
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                });
            });
            ui.separator();

            // Search Bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(14.0));
                let search_box = egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search add-ons, apps, and toolboxes...")
                    .desired_width(ui.available_width() - 80.0);
                ui.add(search_box);
                if !self.search_query.is_empty() && ui.button("Clear").clicked() {
                    self.search_query.clear();
                }
            });

            ui.add_space(4.0);

            let query = self.search_query.trim().to_lowercase();
            let matching_addons: Vec<&'static AddOnItem> = DEFAULT_ADDONS
                .iter()
                .filter(|a| {
                    query.is_empty()
                        || a.title.to_lowercase().contains(&query)
                        || a.description.to_lowercase().contains(&query)
                        || a.category.to_lowercase().contains(&query)
                })
                .collect();

            ui.columns(2, |cols| {
                // Left Column: List of Add-ons
                cols[0].vertical(|ui| {
                    ui.label(
                        RichText::new(format!("Catalog ({})", matching_addons.len()))
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ScrollArea::vertical()
                        .id_salt("addons_list")
                        .max_height(ui.available_height() - 10.0)
                        .show(ui, |ui| {
                            for addon in &matching_addons {
                                let is_selected = self
                                    .selected_addon_id
                                    .as_deref()
                                    .map(|id| id == addon.id)
                                    .unwrap_or(false);

                                let is_installed =
                                    self.installed_ids.contains(&addon.id.to_string());
                                let status_badge = if addon.is_builtin {
                                    " [Built-in]"
                                } else if is_installed {
                                    " [Installed]"
                                } else {
                                    " [Available]"
                                };

                                let label =
                                    format!("{} v{}{}", addon.title, addon.version, status_badge);
                                ui.push_id(addon.id, |ui| {
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        self.selected_addon_id = Some(addon.id.to_string());
                                    }
                                });
                            }
                        });
                });

                // Right Column: Details and actions
                cols[1].vertical(|ui| {
                    if let Some(selected_id) = &self.selected_addon_id {
                        if let Some(addon) =
                            DEFAULT_ADDONS.iter().find(|a| a.id == selected_id.as_str())
                        {
                            let is_installed = self.installed_ids.contains(&addon.id.to_string());

                            ScrollArea::vertical()
                                .id_salt("addon_detail")
                                .max_height(ui.available_height() - 10.0)
                                .show(ui, |ui| {
                                    ui.heading(
                                        RichText::new(addon.title).color(theme.accent).size(18.0),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("Version: {}", addon.version))
                                                .size(11.0),
                                        );
                                        ui.separator();
                                        ui.label(
                                            RichText::new(format!("Category: {}", addon.category))
                                                .size(11.0),
                                        );
                                        ui.separator();
                                        ui.label(
                                            RichText::new(format!("Author: {}", addon.author))
                                                .size(11.0),
                                        );
                                    });
                                    ui.add_space(8.0);

                                    ui.label(RichText::new("Description").strong());
                                    ui.label(
                                        RichText::new(addon.description).color(theme.text_primary),
                                    );
                                    ui.add_space(12.0);

                                    // Action buttons
                                    ui.horizontal(|ui| {
                                        if addon.is_builtin {
                                            if ui
                                                .button(RichText::new("🚀 Launch App").strong())
                                                .clicked()
                                            {
                                                *on_launch_app = Some(addon.id);
                                            }
                                        } else if is_installed {
                                            if ui
                                                .button(RichText::new("🚀 Open").strong())
                                                .clicked()
                                            {
                                                *on_launch_app = Some(addon.id);
                                            }
                                            if ui.button("Uninstall").clicked() {
                                                self.installed_ids.retain(|id| id != addon.id);
                                            }
                                        } else if ui
                                            .button(
                                                RichText::new("⬇ Install Package")
                                                    .color(Color32::from_rgb(100, 200, 255)),
                                            )
                                            .clicked()
                                        {
                                            self.installed_ids.push(addon.id.to_string());
                                        }
                                    });
                                });
                        }
                    } else {
                        ui.label(
                            RichText::new(
                                "Select an add-on or package from the catalog to view details.",
                            )
                            .italics()
                            .color(theme.text_secondary),
                        );
                    }
                });
            });
        });
    }
}
