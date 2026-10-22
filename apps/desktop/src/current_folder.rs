//! Current Folder panel: filesystem browser over the working directory.
//! Files are only modified through explicit user actions (save,
//! new-script-then-save, one confirmed delete).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Default)]
pub struct FolderState {
    pub cwd: PathBuf,
    entries: Vec<Entry>,
    pub selected: Option<PathBuf>,
    /// Directory-change error (permissions, missing…), shown inline.
    pub error: Option<String>,
    /// File pending a confirmed delete.
    pub confirm_delete: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum FolderAction {
    None,
    /// Open a script file in the editor.
    Open {
        path: PathBuf,
    },
    /// Run the "new script" flow rooted at this directory.
    NewScript,
    /// Delete file after the user confirmed.
    Delete {
        path: PathBuf,
    },
    /// Copy the path to the clipboard.
    CopyPath {
        path: PathBuf,
    },
}

impl FolderState {
    pub fn with_directory(cwd: PathBuf) -> Self {
        let mut s = Self {
            cwd,
            ..Default::default()
        };
        s.refresh();
        s
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        match std::fs::read_dir(&self.cwd) {
            Ok(read) => {
                for item in read.flatten() {
                    let path = item.path();
                    let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let name = item.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue; // hidden files stay out of the panel
                    }
                    self.entries.push(Entry { path, name, is_dir });
                }
                self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                });
                self.error = None;
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
    }

    /// `cd(path)` — only directories are accepted.
    pub fn navigate_to(&mut self, path: &Path) {
        match std::fs::canonicalize(resolve_relative(&self.cwd, path)) {
            Ok(canon) if canon.is_dir() => {
                self.cwd = canon;
                self.selected = None;
                self.refresh();
            }
            Ok(_) => {
                self.error = Some(format!("Not a directory: {}", path.display()));
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.cwd.parent().map(Path::to_path_buf) {
            self.navigate_to(&parent);
        }
    }

    /// The `.m` icon marker used in the list.
    fn icon_of(entry: &Entry) -> &'static str {
        Self::icon(entry)
    }

    /// The `.m` icon marker used in the list.
    fn icon(entry: &Entry) -> &'static str {
        if entry.is_dir {
            "▸"
        } else if has_matlabish_extension(&entry.name) {
            "m"
        } else {
            "·"
        }
    }
}

impl FolderAction {
    /// Combine actions from two calls in one frame (app-level toolbar plus
    /// listing interactions).
    pub fn merge(self, other: FolderAction) -> FolderAction {
        match other {
            FolderAction::None => self,
            other => other,
        }
    }
}

pub fn has_matlabish_extension(name: &str) -> bool {
    Path::new(name)
        .extension()
        .map(|e| e.eq_ignore_ascii_case("m"))
        .unwrap_or(false)
}

fn resolve_relative(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// Render the panel body: current-directory line and the file listing.
/// The app owns the toolbar, scroll container and header; this function must
/// not be wrapped in its own scroll area by the caller when listing is large —
/// the caller (left panel) places it inside one.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut FolderState,
    kind: crate::theme::ThemeKind,
) -> FolderAction {
    let mut action = FolderAction::None;

    // One-line, truncated path (tooltip carries the full path).
    let cwd = state.cwd.to_string_lossy().to_string();
    let resp =
        ui.add(egui::Label::new(egui::RichText::new(cwd.clone()).small().monospace()).truncate());
    resp.on_hover_text(cwd);

    if let Some(err) = &state.error {
        ui.colored_label(crate::theme::error_color(kind), format!("⚠ {}", err));
    }

    for entry in state.entries.clone() {
        let icon = FolderState::icon_of(&entry);
        let selected = state.selected.as_deref() == Some(entry.path.as_path());
        let label = egui::RichText::new(format!("{} {}", icon, entry.name)).monospace();
        let label = if entry.is_dir || selected {
            label.color(crate::theme::text_color(kind))
        } else {
            label.color(crate::theme::text_color_faint(kind))
        };
        let fill = if selected {
            crate::theme::tint(crate::theme::accent(kind), 0.45)
        } else {
            egui::Color32::TRANSPARENT
        };
        let btn = egui::Button::new(label).fill(fill);
        let resp = ui.add(btn);
        if resp.clicked() {
            state.selected = Some(entry.path.clone());
        }
        if resp.double_clicked() {
            if entry.is_dir {
                state.navigate_to(&entry.path);
            } else {
                action = FolderAction::Open {
                    path: entry.path.clone(),
                };
            }
        }
        resp.context_menu(|menu| {
            if !entry.is_dir && menu.button("Open").clicked() {
                action = FolderAction::Open {
                    path: entry.path.clone(),
                };
                menu.close();
            }
            if menu.button("Copy path").clicked() {
                action = FolderAction::CopyPath {
                    path: entry.path.clone(),
                };
                menu.close();
            }
            if !entry.is_dir && menu.button("Delete…").clicked() {
                state.confirm_delete = Some(entry.path.clone());
                menu.close();
            }
        });
    }
    // Every directory change re-sorts entries; selected may dangle.
    let dangling = state
        .selected
        .as_ref()
        .map(|sel| !sel.exists())
        .unwrap_or(false);
    if dangling {
        state.selected = None;
    }
    action
}

/// The delete-confirmation dialog, if one is pending. Returns the path whose
/// deletion was accepted (the app performs the fs write itself).
pub fn show_delete_confirm(ctx: &egui::Context, state: &mut FolderState) -> Option<PathBuf> {
    let Some(path) = state.confirm_delete.clone() else {
        return None;
    };
    let mut accepted = None;
    let mut open = true;
    egui::Window::new("Delete file?")
        .id(egui::Id::unique("confirm_delete_file"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "Move '{}' to the trash? This cannot be undone.",
                path.display()
            ));
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    accepted = Some(path.clone());
                    state.confirm_delete = None;
                }
                if ui.button("Cancel").clicked() {
                    state.confirm_delete = None;
                }
            });
        });
    if !open {
        state.confirm_delete = None;
    }
    accepted
}
