//! Visual theme: dense, dark-neutral workspace (default) with a light variant.
//! No gradients, restrained blue accent, thin separators.

use egui::{Color32, FontId, TextStyle, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeKind {
    Dark,
    Light,
}

impl Default for ThemeKind {
    fn default() -> Self {
        ThemeKind::Dark
    }
}

/// Syntax-highlighting palette shared by the editor and command window.
#[derive(Debug, Clone, Copy)]
pub struct SyntaxPalette {
    pub keyword: Color32,
    pub builtin: Color32,
    pub number: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub ident: Color32,
    pub operator: Color32,
    pub gutter: Color32,
    pub gutter_active: Color32,
    pub line_highlight: Color32,
    pub error_bg: Color32,
    pub selection_bg: Color32,
    pub caret: Color32,
}

impl SyntaxPalette {
    pub fn dark() -> Self {
        Self {
            keyword: Color32::from_rgb(0xd1, 0x9a, 0x66),
            builtin: Color32::from_rgb(0x61, 0xaf, 0xef),
            number: Color32::from_rgb(0x56, 0xb6, 0xc2),
            string: Color32::from_rgb(0x98, 0xc3, 0x79),
            comment: Color32::from_rgb(0x6e, 0x76, 0x82),
            ident: Color32::from_rgb(0xd8, 0xda, 0xdf),
            operator: Color32::from_rgb(0xb2, 0xb8, 0xc2),
            gutter: Color32::from_rgb(0x5e, 0x64, 0x6e),
            gutter_active: Color32::from_rgb(0xa5, 0xac, 0xb8),
            line_highlight: Color32::from_rgb(0x23, 0x25, 0x2a),
            error_bg: Color32::from_rgb(0x4b, 0x1f, 0x22),
            selection_bg: Color32::from_rgb(0x2c, 0x44, 0x66),
            caret: Color32::from_rgb(0xe8, 0xec, 0xf1),
        }
    }

    pub fn light() -> Self {
        Self {
            keyword: Color32::from_rgb(0xa1, 0x56, 0x0d),
            builtin: Color32::from_rgb(0x1b, 0x62, 0xa8),
            number: Color32::from_rgb(0x0a, 0x6e, 0x7c),
            string: Color32::from_rgb(0x29, 0x6e, 0x2b),
            comment: Color32::from_rgb(0x8a, 0x93, 0x9e),
            ident: Color32::from_rgb(0x26, 0x29, 0x2e),
            operator: Color32::from_rgb(0x50, 0x56, 0x60),
            gutter: Color32::from_rgb(0xa5, 0xab, 0xb5),
            gutter_active: Color32::from_rgb(0x44, 0x4a, 0x54),
            line_highlight: Color32::from_rgb(0xe8, 0xec, 0xf2),
            error_bg: Color32::from_rgb(0xf6, 0xd7, 0xd9),
            selection_bg: Color32::from_rgb(0xbd, 0xd4, 0xf0),
            caret: Color32::from_rgb(0x1a, 0x1c, 0x20),
        }
    }
}

pub fn text_color(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0xcf, 0xd2, 0xd8),
        ThemeKind::Light => Color32::from_rgb(0x23, 0x26, 0x2b),
    }
}

pub fn text_color_faint(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0x8d, 0x93, 0x9d),
        ThemeKind::Light => Color32::from_rgb(0x77, 0x7e, 0x88),
    }
}

pub fn accent(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0x4a, 0x8f, 0xd4),
        ThemeKind::Light => Color32::from_rgb(0x2b, 0x6c, 0xb0),
    }
}

pub fn error_color(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0xe0, 0x72, 0x74),
        ThemeKind::Light => Color32::from_rgb(0xb2, 0x3b, 0x3e),
    }
}

pub fn success_color(kind: ThemeKind) -> Color32 {
    match kind {
        ThemeKind::Dark => Color32::from_rgb(0x87, 0xc3, 0x8b),
        ThemeKind::Light => Color32::from_rgb(0x2f, 0x7d, 0x35),
    }
}

/// Scale a color's RGB channels (keeps alpha). Used for translucent fills.
pub fn tint(color: Color32, factor: f32) -> Color32 {
    let scale = |v: u8| ((v as f32 * factor) as u32).clamp(0, 255) as u8;
    Color32::from_rgba_unmultiplied(
        scale(color.r()),
        scale(color.g()),
        scale(color.b()),
        color.a(),
    )
}

/// Base visuals per theme, with the shared dense styling applied.
pub fn visuals(kind: ThemeKind) -> Visuals {
    let mut v = match kind {
        ThemeKind::Dark => Visuals::dark(),
        ThemeKind::Light => Visuals::light(),
    };
    match kind {
        ThemeKind::Dark => {
            v.panel_fill = Color32::from_rgb(0x1a, 0x1b, 0x1e);
            v.window_fill = Color32::from_rgb(0x20, 0x21, 0x25);
            v.extreme_bg_color = Color32::from_rgb(0x14, 0x15, 0x19);
            v.faint_bg_color = Color32::from_rgb(0x23, 0x25, 0x2a);
        }
        ThemeKind::Light => {
            v.panel_fill = Color32::from_rgb(0xf1, 0xf2, 0xf4);
            v.window_fill = Color32::from_rgb(0xf7, 0xf8, 0xfa);
            v.extreme_bg_color = Color32::from_rgb(0xff, 0xff, 0xff);
            v.faint_bg_color = Color32::from_rgb(0xe9, 0xec, 0xf0);
        }
    }

    let accent = accent(kind);
    v.selection.bg_fill = tint(accent, 0.55);
    v.selection.stroke = egui::Stroke::new(1.0, accent);

    let (widget_bg, widget_hover, widget_active) = match kind {
        ThemeKind::Dark => (
            Color32::from_rgb(0x26, 0x27, 0x2c),
            Color32::from_rgb(0x2e, 0x30, 0x37),
            Color32::from_rgb(0x37, 0x3a, 0x42),
        ),
        ThemeKind::Light => (
            Color32::from_rgb(0xe4, 0xe6, 0xea),
            Color32::from_rgb(0xdc, 0xdf, 0xe5),
            Color32::from_rgb(0xd3, 0xd7, 0xde),
        ),
    };
    v.widgets.inactive.bg_fill = widget_bg;
    v.widgets.hovered.bg_fill = widget_hover;
    v.widgets.active.bg_fill = widget_active;
    v.widgets.inactive.weak_bg_fill = widget_bg;
    v.widgets.hovered.weak_bg_fill = widget_hover;
    v.widgets.active.weak_bg_fill = widget_active;

    let border = egui::Stroke::new(
        1.0,
        match kind {
            ThemeKind::Dark => Color32::from_rgb(0x2c, 0x2e, 0x34),
            ThemeKind::Light => Color32::from_rgb(0xd7, 0xda, 0xdf),
        },
    );
    v.widgets.noninteractive.bg_stroke = border;
    v.widgets.inactive.bg_stroke = border;
    v.widgets.hovered.bg_stroke = border;
    v.widgets.active.bg_stroke = border;

    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_color(kind));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_color(kind));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text_color(kind));
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, text_color(kind));

    v
}

/// Apply theme + typography + density to the context. Called when prefs change.
pub fn apply(ctx: &egui::Context, kind: ThemeKind, font_size: f32) {
    ctx.set_theme(match kind {
        ThemeKind::Dark => egui::Theme::Dark,
        ThemeKind::Light => egui::Theme::Light,
    });
    let visuals = visuals(kind);
    ctx.all_styles_mut(|style| {
        style.visuals = visuals.clone();
        style.visuals.dark_mode = matches!(kind, ThemeKind::Dark);
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.menu_margin = egui::Margin::symmetric(8, 4);
        style.spacing.icon_width_inner = 12.0;
        style.spacing.icon_width = 14.0;
        style.spacing.indent = 14.0;
        style.text_styles = [
            (TextStyle::Body, FontId::proportional(font_size)),
            (TextStyle::Monospace, FontId::monospace(font_size)),
            (TextStyle::Button, FontId::proportional(font_size - 0.5)),
            (TextStyle::Small, FontId::proportional(font_size - 3.5)),
            (TextStyle::Heading, FontId::proportional(font_size + 5.0)),
        ]
        .into();
    });
}

/// The plot-series palette (restrained order, no purple dominance).
pub fn plot_series_color(index: usize, kind: ThemeKind) -> Color32 {
    const DARK: [[u8; 3]; 5] = [
        [0x4a, 0x8f, 0xd4], // blue
        [0xd9, 0x8a, 0x3a], // orange
        [0x59, 0xa1, 0x6d], // green
        [0xd0, 0x6a, 0x6c], // red
        [0x77, 0x7e, 0x94], // slate
    ];
    const LIGHT: [[u8; 3]; 5] = [
        [0x2b, 0x6c, 0xb0],
        [0xaf, 0x5e, 0x0e],
        [0x2e, 0x76, 0x45],
        [0xa8, 0x45, 0x48],
        [0x55, 0x5a, 0x74],
    ];
    let palette = match kind {
        ThemeKind::Dark => DARK,
        ThemeKind::Light => LIGHT,
    };
    let c = palette[index % 5];
    Color32::from_rgb(c[0], c[1], c[2])
}
