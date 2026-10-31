//! Entry point for MatRust Studio Desktop GUI.

use matrust_studio::app::MatStudioApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("MatRust Studio - MATLAB-Compatible Scientific Computing IDE"),
        ..Default::default()
    };

    eframe::run_native(
        "MatRust Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(MatStudioApp::default()))),
    )
}
