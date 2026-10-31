//! User preferences, layout presets, and panel visibility states.

use crate::theme::ThemeMode;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutPreset {
    Standard,
    EditorFocus,
    DataAnalysis,
    Plotting,
}

#[derive(Clone, Debug)]
pub struct Preferences {
    pub theme_mode: ThemeMode,
    pub current_preset: LayoutPreset,

    // Panel Visibilities
    pub show_current_folder: bool,
    pub show_editor: bool,
    pub show_workspace: bool,
    pub show_command_window: bool,
    pub show_figures: bool,
    pub show_history: bool,

    // Panel Split Dimensions
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,

    // Editor Font Size
    pub font_size: f32,

    // Recent Files
    pub recent_files: Vec<PathBuf>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            current_preset: LayoutPreset::Standard,

            show_current_folder: true,
            show_editor: true,
            show_workspace: true,
            show_command_window: true,
            show_figures: true,
            show_history: true,

            left_panel_width: 230.0,
            right_panel_width: 280.0,
            bottom_panel_height: 220.0,

            font_size: 13.0,
            recent_files: Vec::new(),
        }
    }
}

impl Preferences {
    pub fn apply_preset(&mut self, preset: LayoutPreset) {
        self.current_preset = preset;
        match preset {
            LayoutPreset::Standard => {
                self.show_current_folder = true;
                self.show_editor = true;
                self.show_workspace = true;
                self.show_command_window = true;
                self.show_figures = true;
                self.show_history = true;
                self.left_panel_width = 230.0;
                self.right_panel_width = 280.0;
                self.bottom_panel_height = 220.0;
            }
            LayoutPreset::EditorFocus => {
                self.show_current_folder = false;
                self.show_editor = true;
                self.show_workspace = false;
                self.show_command_window = true;
                self.show_figures = false;
                self.show_history = false;
                self.bottom_panel_height = 140.0;
            }
            LayoutPreset::DataAnalysis => {
                self.show_current_folder = true;
                self.show_editor = false;
                self.show_workspace = true;
                self.show_command_window = true;
                self.show_figures = true;
                self.show_history = true;
                self.right_panel_width = 380.0;
                self.bottom_panel_height = 280.0;
            }
            LayoutPreset::Plotting => {
                self.show_current_folder = false;
                self.show_editor = true;
                self.show_workspace = true;
                self.show_command_window = true;
                self.show_figures = true;
                self.show_history = false;
                self.right_panel_width = 460.0;
            }
        }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }
}
