//! Current Folder integration tests: registry coverage and drag-and-drop routing.

use egui::{vec2, Context, DroppedFile, Pos2, RawInput, Rect};
use matrust_studio::app::MatStudioApp;
use matrust_studio::current_folder::CurrentFolderPanel;
use matrust_studio::theme::Theme;
use matrust_studio::ui_registry;
use std::fs;

fn unique_temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rustlab-cf-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("create temporary folder");
    dir
}

fn raw_frame(screen: Rect, dropped: Vec<std::path::PathBuf>) -> RawInput {
    RawInput {
        screen_rect: Some(screen),
        dropped_files: dropped
            .into_iter()
            .map(|path| DroppedFile {
                path: Some(path),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

#[test]
fn test_current_folder_registers_navigation_controls_in_a_narrow_panel() {
    let ctx = Context::default();
    let mut panel = CurrentFolderPanel {
        quick_locations: vec![("Test".to_string(), std::env::temp_dir().to_path_buf())],
        ..Default::default()
    };
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(240.0, 300.0));
    let raw = raw_frame(screen, Vec::new());
    let mut open_file = None;
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                panel.show(ui, &Theme::dark(), &mut open_file);
            });
    });
    let audit = ui_registry::audit(&ctx);
    assert!(
        audit.clashes.is_empty(),
        "current folder produced duplicate egui ids: {:?}",
        audit.clashes
    );
    for required in [
        "current_folder.open",
        "current_folder.project",
        "current_folder.up",
        "current_folder.this_pc",
        "current_folder.quick",
        "current_folder.refresh",
        "current_folder.hidden",
    ] {
        let control = audit
            .controls
            .iter()
            .find(|control| control.control_id == required)
            .unwrap_or_else(|| panic!("missing current-folder control {required}"));
        assert!(
            screen.contains_rect(control.rect),
            "{required} overflows a 240x300 dock column: {:?}",
            control.rect
        );
    }
}

#[test]
fn test_dropping_a_folder_opens_it_as_the_project() {
    let dir = unique_temp_dir("drop-folder");
    let ctx = Context::default();
    let mut app = MatStudioApp::default();
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let raw = raw_frame(screen, vec![dir.clone()]);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        app.render_ui(ctx);
    });
    assert_eq!(app.current_folder.project_root, dir);
    assert_eq!(app.current_folder.current_dir, dir);
    assert!(
        app.command_window
            .logs
            .iter()
            .any(|line| line.text.contains("Opened project folder")),
        "drop should be confirmed in the command window"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_dropping_a_script_opens_it_in_the_editor() {
    let dir = unique_temp_dir("drop-script");
    let script = dir.join("dropped.m");
    fs::write(&script, "x = 42;\n").expect("write dropped script");
    let ctx = Context::default();
    let mut app = MatStudioApp::default();
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let raw = raw_frame(screen, vec![script.clone()]);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        app.render_ui(ctx);
    });
    assert_eq!(app.current_folder.current_dir, dir);
    let active = app.editor.active_doc().expect("active editor document");
    assert_eq!(active.path.as_deref(), Some(script.as_path()));
    assert_eq!(active.content, "x = 42;\n");
    let _ = fs::remove_dir_all(&dir);
}
