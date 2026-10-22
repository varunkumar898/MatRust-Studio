//! MatRust Studio — native desktop application entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod command_window;
mod current_folder;
mod editor;
mod execution;
mod figures;
mod preferences;
mod statusbar;
mod theme;
mod workspace_panel;

use eframe::egui;

/// 32x32 RGBA window icon drawn procedurally: dark tile, blue "M" mark,
/// thin orange accent bar. No external assets required.
fn app_icon() -> egui::IconData {
    const W: usize = 32;
    const H: usize = 32;
    let mut rgba = vec![0u8; W * H * 4];

    let set = |rgba: &mut [u8], x: usize, y: usize, c: [u8; 4]| {
        if x < W && y < H {
            let i = (y * W + x) * 4;
            rgba[i..i + 3].copy_from_slice(&c[..3]);
            rgba[i + 3] = c[3];
        }
    };

    let bg: [u8; 4] = [0x17, 0x18, 0x1b, 0xff];
    let blue: [u8; 4] = [0x4a, 0x8f, 0xd4, 0xff];
    let orange: [u8; 4] = [0xd9, 0x8a, 0x3a, 0xff];

    for y in 0..H {
        for x in 0..W {
            set(&mut rgba, x, y, bg);
        }
    }

    // An "M" built from three bar strokes.
    for y in 9..24 {
        for x in 7..10 {
            set(&mut rgba, x, y, blue);
        }
        for x in 22..25 {
            set(&mut rgba, x, y, blue);
        }
    }
    // Diagonals of the M.
    for k in 0..6 {
        let y = 10 + k;
        let x = 10 + k;
        set(&mut rgba, x, y, blue);
        set(&mut rgba, 21 - k, y, blue);
    }
    // Accent bar.
    for x in 7..25 {
        set(&mut rgba, x, 27, orange);
    }

    egui::IconData {
        width: W as u32,
        height: H as u32,
        rgba,
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MatRust Studio")
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_app_id("studio.matrust")
            .with_icon(app_icon()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "MatRust Studio",
        options,
        Box::new(|cc| {
            let prefs = preferences::Prefs::load(cc.storage);
            Ok(Box::new(app::MatRustApp::new(cc, prefs)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_opaque_32x32() {
        let icon = app_icon();
        assert_eq!(icon.width, 32);
        assert_eq!(icon.height, 32);
        assert_eq!(icon.rgba.len(), 32 * 32 * 4);
        // fully opaque background
        assert!(icon.rgba.iter().skip(3).step_by(4).all(|&a| a == 255));
    }
}
