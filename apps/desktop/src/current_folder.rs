//! Project-aware filesystem browser.
//!
//! The panel keeps a *project root* separate from the folder currently being
//! browsed. Double-clicking a subfolder changes the view without losing the
//! project root; the Project button returns to it. Any local folder can be
//! opened with the native folder picker, dropped onto the panel, or reached
//! through This PC / Quick access. Detected cloud-sync folders (OneDrive,
//! Dropbox, Google Drive, iCloud Drive) appear as ordinary local folders, so
//! selecting them needs no network code or account credentials.

use crate::theme::Theme;
use egui::{RichText, ScrollArea, Ui};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CurrentFolderPanel {
    pub current_dir: PathBuf,
    /// Root of the project the user opened. It does not change while browsing
    /// subfolders, so Project always has a stable meaning.
    pub project_root: PathBuf,
    pub selected_file: Option<PathBuf>,
    /// G5/G15-priced toggle from BACKLOG 6.2: dot-entries (".git",
    /// ".github", ".re-ghidra", ...) are hidden by default, matching the
    /// common IDE file-browser default.
    pub show_hidden: bool,
    /// Virtual view listing local drives and synced cloud folders.
    pub show_this_pc: bool,
    /// Cached quick locations. Refreshed on startup, on This PC, and whenever
    /// the user asks for a fresh list. Avoids probing drive letters every frame.
    pub quick_locations: Vec<(String, PathBuf)>,
    /// First detected cloud-sync folder, shown as a one-click Cloud button.
    pub cloud_root: Option<PathBuf>,
    /// True shows the whole project as an expandable tree rooted at the
    /// project root; false shows the classic flat list of the open folder.
    pub view_tree: bool,
    /// Folders the user expanded in the tree view. Collapsed folders are
    /// never read from disk, so a large project stays fast.
    pub expanded: HashSet<PathBuf>,
    /// Whole-project totals (folders, files, truncated-flag) under the
    /// project root, respecting the Hidden toggle. `None` means stale;
    /// recomputed lazily so a save or refresh costs one bounded walk.
    pub totals: Option<(usize, usize, bool)>,
}

impl Default for CurrentFolderPanel {
    fn default() -> Self {
        let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let quick_locations = quick_locations();
        let cloud_root = quick_locations
            .iter()
            .find(|(label, _)| label.starts_with("Cloud:"))
            .map(|(_, path)| path.clone());
        Self {
            current_dir: start.clone(),
            project_root: start,
            selected_file: None,
            show_hidden: false,
            show_this_pc: false,
            quick_locations,
            cloud_root,
            view_tree: true,
            expanded: HashSet::new(),
            totals: None,
        }
    }
}

impl CurrentFolderPanel {
    /// Browse another directory without changing the project root.
    pub fn set_directory(&mut self, dir: PathBuf) {
        if dir.is_dir() {
            self.current_dir = dir;
            self.show_this_pc = false;
            self.selected_file = None;
        }
    }

    /// Start a new project rooted at `dir`.
    pub fn open_project(&mut self, dir: PathBuf) {
        if dir.is_dir() {
            self.project_root = dir.clone();
            self.set_directory(dir);
            self.totals = None;
            self.expanded.clear();
            // A fresh project starts fully collapsed; the root row is always
            // drawn, so the user expands what they need.
            self.expanded.insert(self.project_root.clone());
        }
    }

    /// Mark the whole-project totals stale. Called after the app itself
    /// writes a file (editor Save) and by the Refresh button, so a file
    /// created inside the app is counted on the next frame.
    pub fn invalidate_totals(&mut self) {
        self.totals = None;
    }

    /// Toggle a tree node. Returns the new state; true means expanded.
    pub fn toggle_tree_node(&mut self, dir: &Path) -> bool {
        if self.expanded.contains(dir) {
            self.expanded.remove(dir);
            false
        } else {
            self.expanded.insert(dir.to_path_buf());
            true
        }
    }

    /// Whole-project totals: visible folders and files under the project
    /// root, walking breadth-first with hard caps so a huge or cyclic tree
    /// cannot hang the UI. The caps are: at most 65,536 entries and at most
    /// 24 levels deep; `true` in the third slot means the walk was truncated
    /// and the real numbers are larger. Symlink-like entries are counted but
    /// never descended into.
    pub fn project_totals(&mut self) -> std::io::Result<(usize, usize, bool)> {
        if let Some(cached) = self.totals {
            return Ok(cached);
        }
        const MAX_ENTRIES: usize = 65_536;
        const MAX_DEPTH: usize = 24;
        let mut folders = 0usize;
        let mut files = 0usize;
        let mut truncated = false;
        let mut visited = 0usize;
        let mut stack: Vec<(PathBuf, usize)> = vec![(self.project_root.clone(), 0)];
        while let Some((dir, depth)) = stack.pop() {
            for entry in fs::read_dir(&dir)? {
                visited += 1;
                if visited > MAX_ENTRIES {
                    truncated = true;
                    break;
                }
                let entry = entry?;
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                if !self.show_hidden && is_hidden(name) {
                    continue;
                }
                // file_type avoids a second stat per entry on Windows.
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let is_link = entry.file_type().map(|t| t.is_symlink()).unwrap_or(false);
                if is_dir && !is_link {
                    folders += 1;
                    if depth < MAX_DEPTH {
                        stack.push((path, depth + 1));
                    } else {
                        truncated = true;
                    }
                } else {
                    files += 1;
                }
            }
            if truncated {
                break;
            }
        }
        let result = (folders, files, truncated);
        self.totals = Some(result);
        Ok(result)
    }

    /// Visible immediate children of `dir`, folders first then names, the
    /// same ordering as the flat list. Shared by the list and tree bodies so
    /// both views always agree.
    pub fn sorted_children(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        let read = fs::read_dir(dir)?;
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
        items.retain(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            self.show_hidden || !is_hidden(name)
        });
        Ok(items)
    }

    /// A dropped directory opens as the new project. Returns false for files
    /// so the app can route them to the editor/importer instead.
    pub fn handle_dropped_path(&mut self, path: &Path) -> bool {
        if path.is_dir() {
            self.open_project(path.to_path_buf());
            true
        } else {
            false
        }
    }

    /// Re-probe This PC, drives and cloud-sync folders.
    pub fn refresh_locations(&mut self) {
        self.quick_locations = quick_locations();
        self.cloud_root = self
            .quick_locations
            .iter()
            .find(|(label, _)| label.starts_with("Cloud:"))
            .map(|(_, path)| path.clone());
    }

    pub fn project_name(&self) -> String {
        self.project_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Project")
            .to_string()
    }

    /// Visible immediate children as (folders, files) for the summary line.
    /// The Hidden toggle is respected so the number always matches the list.
    pub fn directory_counts(&self) -> std::io::Result<(usize, usize)> {
        let mut folders = 0;
        let mut files = 0;
        for entry in fs::read_dir(&self.current_dir)? {
            let path = entry?.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !self.show_hidden && is_hidden(name) {
                continue;
            }
            if path.is_dir() {
                folders += 1;
            } else {
                files += 1;
            }
        }
        Ok((folders, files))
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, open_file_request: &mut Option<PathBuf>) {
        ui.vertical(|ui| {
            // Header bar. Wrapping keeps every control reachable in a narrow
            // dock column (for example at 125% or 150% display scaling).
            ui.horizontal_wrapped(|ui| {
                ui.heading("Current Folder");
                let hidden = ui.checkbox(&mut self.show_hidden, "Hidden");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.hidden",
                    hidden.id,
                    hidden.rect,
                );
                let view = ui
                    .button(if self.view_tree {
                        "List view"
                    } else {
                        "Tree view"
                    })
                    .on_hover_text(if self.view_tree {
                        "Switch to the flat list of this folder"
                    } else {
                        "Switch to the whole-project tree (folders expand in place)"
                    });
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.view",
                    view.id,
                    view.rect,
                );
                if view.clicked() {
                    self.view_tree = !self.view_tree;
                }
            });

            let mut next_dir: Option<PathBuf> = None;
            let mut pick_folder = false;
            ui.horizontal_wrapped(|ui| {
                // Compact button metrics keep all navigation controls in two
                // rows in the 250 px dock column at 1280x720.
                ui.spacing_mut().button_padding = egui::vec2(4.0, 2.0);
                ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                let open = ui
                    .button("Open Folder...")
                    .on_hover_text("Choose any local or synced folder");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.open",
                    open.id,
                    open.rect,
                );
                if open.clicked() {
                    pick_folder = true;
                }
                let project = ui
                    .button("Project")
                    .on_hover_text("Return to the project root");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.project",
                    project.id,
                    project.rect,
                );
                if project.clicked() {
                    next_dir = Some(self.project_root.clone());
                }
                let up = ui
                    .button("Up")
                    .on_hover_text("Navigate to parent directory");
                let _ = crate::ui_registry::register(ui.ctx(), "current_folder.up", up.id, up.rect);
                if up.clicked() {
                    if let Some(parent) = self.current_dir.parent() {
                        next_dir = Some(parent.to_path_buf());
                    }
                }
                let this_pc = ui
                    .button("This PC")
                    .on_hover_text("Show local folders, drives and synced cloud folders");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.this_pc",
                    this_pc.id,
                    this_pc.rect,
                );
                if this_pc.clicked() {
                    self.show_this_pc = true;
                    self.refresh_locations();
                }
                if let Some(cloud) = self.cloud_root.clone() {
                    let cloud_btn = ui
                        .button("Cloud")
                        .on_hover_text(cloud.display().to_string());
                    let _ = crate::ui_registry::register(
                        ui.ctx(),
                        "current_folder.cloud",
                        cloud_btn.id,
                        cloud_btn.rect,
                    );
                    if cloud_btn.clicked() {
                        next_dir = Some(cloud);
                    }
                }
                let quick = quick_access_menu(ui, &self.quick_locations, &mut next_dir);
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.quick",
                    quick.response.id,
                    quick.response.rect,
                );
                let refresh = ui
                    .button("Refresh")
                    .on_hover_text("Refresh files, drives and cloud folders");
                let _ = crate::ui_registry::register(
                    ui.ctx(),
                    "current_folder.refresh",
                    refresh.id,
                    refresh.rect,
                );
                if refresh.clicked() {
                    self.refresh_locations();
                }
            });

            if pick_folder {
                let start = if self.current_dir.is_dir() {
                    self.current_dir.clone()
                } else {
                    self.project_root.clone()
                };
                if let Some(folder) = rfd::FileDialog::new().set_directory(start).pick_folder() {
                    next_dir = Some(folder);
                }
            }
            if let Some(dir) = next_dir {
                self.open_project(dir);
            }

            // Path / virtual-location breadcrumb.
            let path_label = if self.show_this_pc {
                "This PC".to_string()
            } else {
                self.current_dir.display().to_string()
            };
            ui.add(
                egui::Label::new(
                    RichText::new(path_label)
                        .small()
                        .color(theme.text_secondary),
                )
                .truncate(),
            );

            let hovered_drop = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
            if hovered_drop {
                ui.colored_label(theme.accent, "Drop a folder here to open it");
            }

            if self.show_this_pc {
                self.show_this_pc_body(ui, theme);
                return;
            }

            // Read the entries once through the shared helper so the list,
            // the tree and the summary line can never disagree.
            let entries = match self.sorted_children(&self.current_dir) {
                Ok(items) => items,
                Err(err) => {
                    ui.colored_label(theme.error, format!("Cannot read folder: {err}"));
                    return;
                }
            };
            let folder_count = entries.iter().filter(|path| path.is_dir()).count();
            let file_count = entries.len().saturating_sub(folder_count);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(
                    RichText::new(format!("Project: {}", self.project_name()))
                        .small()
                        .strong(),
                );
                let counts = ui.label(
                    RichText::new(format!("{folder_count} folders, {file_count} files here"))
                        .small()
                        .color(theme.text_secondary),
                );
                counts.on_hover_text("Counts are the immediate children of this folder");
                let totals = self.project_totals();
                if let Ok((folders, files, truncated)) = totals {
                    let plus = if truncated { "+" } else { "" };
                    let totals_label = ui.label(
                        RichText::new(format!(
                            "|  project: {folders}{plus} folders, {files}{plus} files"
                        ))
                        .small()
                        .color(theme.text_secondary),
                    );
                    totals_label.on_hover_text(
                        "Whole-project totals under the project root, capped at 65,536 entries",
                    );
                }
            });
            ui.separator();

            if self.view_tree {
                self.show_tree_body(ui, theme, open_file_request);
                return;
            }

            ScrollArea::vertical()
                .id_salt("current_folder_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut navigated = false;
                    for path in entries {
                        // G5: stable per-row id keyed by the path itself. The
                        // key is computed before the closure so the path can
                        // be moved into the navigation handler below.
                        let row_key = path.as_os_str().to_os_string();
                        ui.push_id(row_key, |ui| {
                            let file_name =
                                path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            let (icon, color) = file_icon(file_name, path.is_dir(), theme);
                            let is_selected = self.selected_file.as_ref() == Some(&path);
                            let label_text = format!("{icon} {file_name}");
                            let resp = ui.selectable_label(
                                is_selected,
                                RichText::new(label_text).color(color),
                            );
                            if resp.clicked() {
                                self.selected_file = Some(path.clone());
                            }
                            if resp.double_clicked() {
                                if path.is_dir() {
                                    self.current_dir = path;
                                    navigated = true;
                                } else {
                                    *open_file_request = Some(path);
                                }
                            }
                        });
                        if navigated {
                            break;
                        }
                    }
                });
        });
    }

    fn show_this_pc_body(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.label(
            RichText::new("Local folders, drives and synced cloud folders")
                .small()
                .color(theme.text_secondary),
        );
        ui.separator();
        let mut next_dir = None;
        ScrollArea::vertical()
            .id_salt("this_pc_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (label, path) in &self.quick_locations {
                    let display = if label.starts_with("Cloud:") {
                        format!("[C] {label}")
                    } else {
                        format!("[D] {label}")
                    };
                    let response = ui.selectable_label(false, display);
                    if response.clicked() || response.double_clicked() {
                        next_dir = Some(path.clone());
                    }
                }
            });
        if let Some(dir) = next_dir {
            self.open_project(dir);
        }
    }

    /// The whole-project tree. The root row is always visible; a folder's
    /// children are read from disk only while it is expanded, so a large
    /// project never costs more reads than the folders the user opened.
    fn show_tree_body(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        open_file_request: &mut Option<PathBuf>,
    ) {
        if !self.project_root.is_dir() {
            ui.colored_label(
                theme.error,
                format!("Project folder is missing: {}", self.project_root.display()),
            );
            return;
        }
        ScrollArea::vertical()
            .id_salt("current_folder_tree_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.show_tree_dir(ui, theme, &self.project_root.clone(), 0, open_file_request);
            });
    }

    /// One tree level: the folder's visible children, folders first. Returns
    /// after drawing; recursion happens through the expand toggles below.
    fn show_tree_dir(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        dir: &Path,
        depth: usize,
        open_file_request: &mut Option<PathBuf>,
    ) {
        // Hard depth cap mirrors project_totals so a pathological tree
        // cannot build an unbounded widget stack.
        if depth > 24 {
            ui.weak("...");
            return;
        }
        let children = match self.sorted_children(dir) {
            Ok(children) => children,
            Err(err) => {
                ui.colored_label(theme.error, format!("Cannot read folder: {err}"));
                return;
            }
        };
        for path in children {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let is_dir = path.is_dir();
            ui.push_id(path.as_os_str().to_os_string(), |ui| {
                ui.horizontal(|ui| {
                    if is_dir {
                        let expanded = self.expanded.contains(&path);
                        let toggle = ui.selectable_label(
                            false,
                            RichText::new(if expanded { "[-]" } else { "[+]" })
                                .color(theme.text_secondary),
                        );
                        let _ = crate::ui_registry::register(
                            ui.ctx(),
                            "current_folder.tree_toggle",
                            toggle.id,
                            toggle.rect,
                        );
                        if toggle.clicked() {
                            self.toggle_tree_node(&path);
                        }
                    } else {
                        ui.label("    ");
                    }
                    let (icon, color) = file_icon(file_name, is_dir, theme);
                    let resp = ui.selectable_label(
                        self.selected_file.as_ref() == Some(&path),
                        RichText::new(format!("{icon} {file_name}")).color(color),
                    );
                    // A double-click also reports clicked, so the toggle must
                    // live only here; otherwise a folder would open and close
                    // in the same event.
                    if resp.clicked() {
                        self.selected_file = Some(path.clone());
                    }
                    if resp.double_clicked() {
                        if is_dir {
                            self.toggle_tree_node(&path);
                        } else {
                            *open_file_request = Some(path.clone());
                        }
                    }
                });
                if is_dir && self.expanded.contains(&path) {
                    ui.indent(path.as_os_str().to_os_string(), |ui| {
                        self.show_tree_dir(ui, theme, &path, depth + 1, open_file_request);
                    });
                }
            });
        }
    }
}

/// Quick-location menu: one compact control instead of a tall row of buttons.
fn quick_access_menu(
    ui: &mut Ui,
    locations: &[(String, PathBuf)],
    next_dir: &mut Option<PathBuf>,
) -> egui::InnerResponse<Option<()>> {
    ui.menu_button("Quick", |ui| {
        for (label, path) in locations {
            if ui
                .button(label)
                .on_hover_text(path.display().to_string())
                .clicked()
            {
                *next_dir = Some(path.clone());
                ui.close_menu();
            }
        }
    })
}

fn file_icon(file_name: &str, is_dir: bool, theme: &Theme) -> (&'static str, egui::Color32) {
    if is_dir {
        ("[D]", theme.text_primary)
    } else if file_name.ends_with(".m") {
        ("[M]", theme.accent)
    } else if file_name.ends_with(".csv") {
        ("[C]", theme.warning)
    } else if file_name.ends_with(".mat") {
        ("[T]", theme.syn_builtin)
    } else if file_name.ends_with(".svg") {
        ("[S]", theme.accent)
    } else {
        ("[F]", theme.text_secondary)
    }
}

/// Directories and cloud roots offered by This PC and Quick access.
fn quick_locations() -> Vec<(String, PathBuf)> {
    let mut locations: Vec<(String, PathBuf)> = Vec::new();
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    if let Some(home) = &home {
        for (label, child) in [
            ("Home", ""),
            ("Desktop", "Desktop"),
            ("Documents", "Documents"),
            ("Downloads", "Downloads"),
        ] {
            let path = if child.is_empty() {
                home.clone()
            } else {
                home.join(child)
            };
            if path.is_dir() {
                locations.push((label.to_string(), path));
            }
        }
    }

    for candidate in cloud_candidates(home.as_deref()) {
        if candidate.is_dir() {
            let label = candidate
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Cloud")
                .to_string();
            locations.push((format!("Cloud: {label}"), candidate));
        }
    }

    #[cfg(windows)]
    for letter in b'C'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let path = PathBuf::from(&drive);
        if path.is_dir() {
            locations.push((format!("{}:", letter as char), path));
        }
    }
    #[cfg(not(windows))]
    locations.push(("Root".to_string(), PathBuf::from("/")));

    locations
}

fn cloud_candidates(home: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for key in ["OneDrive", "OneDriveCommercial", "OneDriveConsumer"] {
        if let Some(path) = std::env::var_os(key) {
            candidates.push(PathBuf::from(path));
        }
    }
    if let Some(home) = home {
        for child in [
            "OneDrive",
            "Dropbox",
            "Google Drive",
            "GoogleDrive",
            "iCloudDrive",
        ] {
            candidates.push(home.join(child));
        }
    }
    candidates
}

/// Dot-entries and generated/build folders are hidden by default. The Hidden
/// toggle reveals all of them; nothing is deleted or renamed.
fn is_hidden(file_name: &str) -> bool {
    file_name.starts_with('.')
        || matches!(
            file_name,
            "target" | "re-ghidra" | "re" | "node_modules" | "build" | "dist"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_files_have_a_visible_ascii_icon() {
        assert_eq!(file_icon("notes.txt", false, &Theme::dark()).0, "[F]");
        assert_eq!(file_icon("script.m", false, &Theme::dark()).0, "[M]");
        assert_eq!(file_icon("data.csv", false, &Theme::dark()).0, "[C]");
        assert_eq!(file_icon("data.mat", false, &Theme::dark()).0, "[T]");
        assert_eq!(file_icon("plot.svg", false, &Theme::dark()).0, "[S]");
    }

    #[test]
    fn hidden_filter_covers_build_and_dot_entries() {
        assert!(is_hidden(".git"));
        assert!(is_hidden(".github"));
        assert!(is_hidden(".re-ghidra"));
        assert!(is_hidden("target"));
        assert!(is_hidden("re-ghidra"));
        assert!(is_hidden("node_modules"));
        assert!(is_hidden("build"));
        assert!(!is_hidden("main.rs"));
        assert!(!is_hidden("hello.m"));
        assert!(!is_hidden("a.txt"));
    }

    #[test]
    fn opening_a_folder_sets_project_and_browse_root() {
        let base = std::env::temp_dir().join(format!(
            "rustlab-cf-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let child = base.join("child");
        fs::create_dir_all(&child).expect("create temporary test folders");
        let mut panel = CurrentFolderPanel {
            current_dir: base.clone(),
            project_root: base.clone(),
            selected_file: None,
            show_hidden: false,
            show_this_pc: false,
            quick_locations: Vec::new(),
            cloud_root: None,
            view_tree: false,
            expanded: HashSet::new(),
            totals: None,
        };
        assert!(panel.handle_dropped_path(&child));
        assert_eq!(panel.project_root, child);
        panel.set_directory(base.clone());
        assert_eq!(panel.current_dir, base);
        assert_eq!(
            panel.project_root, child,
            "browsing a parent must not move the project root"
        );
        assert!(!panel.handle_dropped_path(&base.join("file.m")));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn directory_counts_respect_hidden_toggle() {
        let base = std::env::temp_dir().join(format!(
            "rustlab-cf-counts-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(base.join("sub")).expect("create subfolder");
        fs::create_dir_all(base.join("target")).expect("create build folder");
        fs::write(base.join("a.m"), "x = 1;\n").expect("write script");
        fs::write(base.join("b.csv"), "x\n1\n").expect("write csv");
        fs::write(base.join(".secret"), "x\n").expect("write dotfile");
        let mut panel = CurrentFolderPanel {
            current_dir: base.clone(),
            project_root: base.clone(),
            selected_file: None,
            show_hidden: false,
            show_this_pc: false,
            quick_locations: Vec::new(),
            cloud_root: None,
            view_tree: false,
            expanded: HashSet::new(),
            totals: None,
        };
        let (folders, files) = panel.directory_counts().expect("count visible entries");
        assert_eq!((folders, files), (1, 2));
        panel.show_hidden = true;
        let (folders, files) = panel.directory_counts().expect("count all entries");
        assert_eq!((folders, files), (2, 3));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cloud_candidates_include_onedrive_environment_slot() {
        let candidates = cloud_candidates(Some(Path::new("C:/Users/test")));
        assert!(candidates.iter().any(|path| path.ends_with("OneDrive")));
    }

    /// Minimal panel for logic tests (no egui needed).
    fn test_panel(root: PathBuf) -> CurrentFolderPanel {
        CurrentFolderPanel {
            current_dir: root.clone(),
            project_root: root,
            selected_file: None,
            show_hidden: false,
            show_this_pc: false,
            quick_locations: Vec::new(),
            cloud_root: None,
            view_tree: true,
            expanded: HashSet::new(),
            totals: None,
        }
    }

    fn unique_tree_dir(tag: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "rustlab-cf-tree-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&base).expect("create temporary project");
        base
    }

    #[test]
    fn tree_totals_count_visible_entries_and_invalidate_on_new_files() {
        let base = unique_tree_dir("totals");
        fs::create_dir_all(base.join("src/deep")).expect("create nested folders");
        fs::write(base.join("src/main.m"), "x = 1;\n").expect("write script");
        fs::write(base.join("src/deep/inner.csv"), "v\n1\n").expect("write csv");
        fs::write(base.join(".hidden.rs"), "fn x() {}\n").expect("write dotfile");

        let mut panel = test_panel(base.clone());
        let (folders, files, truncated) = panel.project_totals().expect("walk project");
        assert_eq!(
            (folders, files),
            (2, 2),
            "2 folders (src, deep) and 2 files"
        );
        assert!(!truncated);

        // A file created inside the app (editor Save) must be counted after
        // the invalidation hook fires — this is the auto-refresh contract.
        fs::write(base.join("src/extra.m"), "y = 2;\n").expect("write new file");
        panel.invalidate_totals();
        let (folders, files, _) = panel.project_totals().expect("re-walk project");
        assert_eq!((folders, files), (2, 3), "new file appears in totals");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn tree_totals_respect_hidden_toggle_and_expand_state_resets() {
        let base = unique_tree_dir("hidden");
        fs::create_dir_all(base.join("target")).expect("create build folder");
        fs::create_dir_all(base.join("src")).expect("create src");
        fs::write(base.join("target/artifact.bin"), "x").expect("write artifact");

        let mut panel = test_panel(base.clone());
        let (folders, _, truncated) = panel.project_totals().expect("walk project");
        assert_eq!((folders, truncated), (1, false), "target hidden by default");

        panel.show_hidden = true;
        panel.invalidate_totals();
        let (folders, _, _) = panel.project_totals().expect("walk with hidden");
        assert_eq!(folders, 2, "target counts when Hidden is on");

        // Opening a different project resets expansion to just the new root.
        let other = unique_tree_dir("other");
        panel.open_project(other.clone());
        assert_eq!(panel.expanded, std::iter::once(other).collect());
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(panel.project_root);
    }

    #[test]
    fn toggle_tree_node_flips_expansion() {
        let base = unique_tree_dir("toggle");
        let sub = base.join("sub");
        fs::create_dir_all(&sub).expect("create subfolder");
        let mut panel = test_panel(base);
        assert!(panel.toggle_tree_node(&sub), "first toggle expands");
        assert!(panel.expanded.contains(&sub));
        assert!(!panel.toggle_tree_node(&sub), "second toggle collapses");
        assert!(!panel.expanded.contains(&sub));
        let _ = fs::remove_dir_all(panel.project_root);
    }
}
