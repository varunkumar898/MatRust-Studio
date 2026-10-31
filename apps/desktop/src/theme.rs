//! Visual themes and scientific color styling for MatRust Studio.

use egui::{Color32, Stroke, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub background: Color32,
    pub panel_bg: Color32,
    pub editor_bg: Color32,
    pub gutter_bg: Color32,
    pub gutter_fg: Color32,
    pub border: Color32,
    pub active_tab: Color32,
    pub inactive_tab: Color32,
    pub accent: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,

    // Syntax Highlighting
    pub syn_keyword: Color32,
    pub syn_builtin: Color32,
    pub syn_string: Color32,
    pub syn_comment: Color32,
    pub syn_number: Color32,
    pub syn_operator: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            background: Color32::from_rgb(0x18, 0x18, 0x18),
            panel_bg: Color32::from_rgb(0x20, 0x20, 0x20),
            editor_bg: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            gutter_bg: Color32::from_rgb(0x25, 0x25, 0x26),
            gutter_fg: Color32::from_rgb(0x85, 0x85, 0x85),
            border: Color32::from_rgb(0x38, 0x38, 0x38),
            active_tab: Color32::from_rgb(0x2D, 0x2D, 0x2D),
            inactive_tab: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            accent: Color32::from_rgb(0x00, 0x7A, 0xCC), // MATLAB / VS Code Blue
            text_primary: Color32::from_rgb(0xDD, 0xDD, 0xDD),
            text_secondary: Color32::from_rgb(0x9E, 0x9E, 0x9E),
            success: Color32::from_rgb(0x4E, 0xC9, 0xB0),
            warning: Color32::from_rgb(0xCE, 0x91, 0x78),
            error: Color32::from_rgb(0xF4, 0x47, 0x47),

            // Syntax colors
            syn_keyword: Color32::from_rgb(0x56, 0x9C, 0xD6), // Keyword blue
            syn_builtin: Color32::from_rgb(0x4E, 0xC9, 0xB0), // Builtin teal
            syn_string: Color32::from_rgb(0xCE, 0x91, 0x78),  // String salmon
            syn_comment: Color32::from_rgb(0x6A, 0x99, 0x55), // Comment green
            syn_number: Color32::from_rgb(0xB5, 0xCE, 0xA8),  // Number light green
            syn_operator: Color32::from_rgb(0xD4, 0xD4, 0xD4), // Operator neutral
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            background: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            panel_bg: Color32::from_rgb(0xF8, 0xF8, 0xF8),
            editor_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            gutter_bg: Color32::from_rgb(0xEB, 0xEB, 0xEB),
            gutter_fg: Color32::from_rgb(0x6E, 0x6E, 0x6E),
            border: Color32::from_rgb(0xDD, 0xDD, 0xDD),
            active_tab: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            inactive_tab: Color32::from_rgb(0xEC, 0xEC, 0xEC),
            accent: Color32::from_rgb(0x00, 0x5F, 0xB8),
            text_primary: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            text_secondary: Color32::from_rgb(0x61, 0x61, 0x61),
            success: Color32::from_rgb(0x10, 0x7C, 0x10),
            warning: Color32::from_rgb(0x79, 0x5E, 0x26),
            error: Color32::from_rgb(0xA8, 0x00, 0x00),

            // Syntax colors
            syn_keyword: Color32::from_rgb(0x00, 0x00, 0xFF),
            syn_builtin: Color32::from_rgb(0x26, 0x7F, 0x99),
            syn_string: Color32::from_rgb(0xA3, 0x15, 0x15),
            syn_comment: Color32::from_rgb(0x00, 0x80, 0x00),
            syn_number: Color32::from_rgb(0x09, 0x86, 0x58),
            syn_operator: Color32::from_rgb(0x00, 0x00, 0x00),
        }
    }

    pub fn apply_to_visuals(&self, visuals: &mut Visuals) {
        match self.mode {
            ThemeMode::Dark => {
                visuals.dark_mode = true;
                visuals.panel_fill = self.panel_bg;
                visuals.window_fill = self.background;
                visuals.widgets.noninteractive.bg_fill = self.panel_bg;
                visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;
                visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border);
            }
            ThemeMode::Light => {
                visuals.dark_mode = false;
                visuals.panel_fill = self.panel_bg;
                visuals.window_fill = self.background;
                visuals.widgets.noninteractive.bg_fill = self.panel_bg;
                visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;
                visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.border);
            }
        }
    }
}
