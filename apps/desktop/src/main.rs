//! Entry point for the MATSTUDIO desktop GUI.

use matrust_studio::app::MatStudioApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("MATSTUDIO"),
        ..Default::default()
    };

    eframe::run_native(
        "MATSTUDIO",
        native_options,
        Box::new(|_cc| Ok(Box::new(MatStudioApp::default()))),
    )
}
