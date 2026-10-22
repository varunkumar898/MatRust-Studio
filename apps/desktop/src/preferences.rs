//! User preferences and window-layout persistence (eframe storage).

use crate::theme::ThemeKind;
use serde::{Deserialize, Serialize};

pub const STORAGE_KEY: &str = "matrust_prefs_v2";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutPrefs {
    #[serde(default = "yes")]
    pub show_left: bool,
    /// Figures panel; hidden by default so an empty panel never reserves
    /// space. Auto-opens when the session creates a figure.
    #[serde(default)]
    pub show_right: bool,
    #[serde(default = "yes")]
    pub show_command_window: bool,
    /// Editor-focused vs standard preset display density, etc.
    pub left_width: Option<f32>,
    pub right_width: Option<f32>,
    pub command_height: Option<f32>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub theme: ThemeKind,
    pub font_size: f32,
    pub working_dir: Option<String>,
    /// Paths of script tabs to reopen (files that still exist).
    pub open_files: Vec<String>,
    pub active_file: Option<String>,
    pub layout: LayoutPrefs,
    /// Last `n` successfully submitted command-window commands.
    pub command_history: Vec<String>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: ThemeKind::Dark,
            font_size: 13.5,
            working_dir: None,
            open_files: Vec::new(),
            active_file: None,
            layout: LayoutPrefs {
                show_left: true,
                show_right: false,
                show_command_window: true,
                left_width: None,
                right_width: None,
                command_height: None,
            },
            command_history: Vec::new(),
        }
    }
}

impl Prefs {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|s| eframe::get_value::<Prefs>(s, STORAGE_KEY))
            .unwrap_or_default()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, self);
    }
}
