use mlapp_host::UiEvent;
use mlapp_render::{HeadlessRenderer, RenderBackend};
use mlapp_ui::{Component, ComponentKind, Rect};

#[test]
fn test_headless_renderer_tree_walk_and_event_dispatch() {
    let mut root = Component::new(ComponentKind::UIFigure, "UIFigure");
    root.position = Rect::new(0.0, 0.0, 800.0, 600.0);

    let mut panel = Component::new(ComponentKind::Panel, "MainPanel");
    panel.position = Rect::new(10.0, 10.0, 200.0, 580.0);

    let mut btn = Component::new(ComponentKind::Button, "CalculateBtn");
    btn.position = Rect::new(20.0, 500.0, 100.0, 30.0);

    panel.children.push(btn);
    root.children.push(panel);

    let mut renderer = HeadlessRenderer::new();
    renderer.queue_click("CalculateBtn");

    let events = renderer.render_tree(&root);
    assert_eq!(renderer.name(), "headless");
    assert_eq!(events.len(), 1);
    match &events[0] {
        UiEvent::ButtonClick { component_name } => {
            assert_eq!(component_name, "CalculateBtn");
        }
        _ => panic!("Expected ButtonClick event"),
    }

    assert_eq!(renderer.rendered_log.len(), 3);
    assert!(renderer.rendered_log[0].contains("UIFigure"));
    assert!(renderer.rendered_log[0].contains("at (0,0 w=800 h=600)"));
    assert!(renderer.rendered_log[1].contains("MainPanel"));
    assert!(renderer.rendered_log[2].contains("CalculateBtn"));
}

#[test]
fn test_headless_screenshot_render() {
    let mut root = Component::new(ComponentKind::UIFigure, "UIFigure");
    root.position = Rect::new(0.0, 0.0, 800.0, 600.0);

    let mut panel = Component::new(ComponentKind::Panel, "ControlPanel");
    panel.position = Rect::new(20.0, 20.0, 250.0, 560.0);

    let mut btn = Component::new(ComponentKind::Button, "ActionBtn");
    btn.position = Rect::new(30.0, 480.0, 120.0, 40.0);

    let mut axes = Component::new(ComponentKind::UIAxes, "PlotAxes");
    axes.position = Rect::new(300.0, 50.0, 460.0, 500.0);

    panel.children.push(btn);
    root.children.push(panel);
    root.children.push(axes);

    let renderer = HeadlessRenderer::new();
    let width = 800;
    let height = 600;
    let rgba = renderer.capture_rgba_buffer(&root, width, height);

    // Assert exact buffer size: 800 * 600 * 4 = 1,920,000 bytes
    assert_eq!(rgba.len(), (width * height * 4) as usize);

    // Verify non-trivial rendering by checking different pixel samples:
    // Sample inside UIFigure background (x=10, y=10) should be #F0F0F0 -> [240, 240, 240, 255]
    let fig_idx = ((10 * width + 10) * 4) as usize;
    assert_eq!(&rgba[fig_idx..fig_idx + 4], &[240, 240, 240, 255]);

    // Check that MATLAB blue plot line [0, 114, 189, 255] was rasterized onto the axes canvas
    let mut found_plot_line = false;
    for &chunk in rgba.as_chunks::<4>().0 {
        if chunk == [0, 114, 189, 255] {
            found_plot_line = true;
            break;
        }
    }
    assert!(
        found_plot_line,
        "Expected rasterized plot line with MATLAB blue color in UIAxes screenshot"
    );
}

#[test]
fn test_gui_window_screenshot_capture_and_export() {
    let mut root = Component::new(ComponentKind::UIFigure, "UIFigure");
    root.position = Rect::new(0.0, 0.0, 640.0, 480.0);

    let mut panel = Component::new(ComponentKind::Panel, "ControlPanel");
    panel.position = Rect::new(10.0, 10.0, 200.0, 460.0);

    let mut btn = Component::new(ComponentKind::Button, "ActionBtn");
    btn.position = Rect::new(20.0, 400.0, 100.0, 30.0);

    let mut axes = Component::new(ComponentKind::UIAxes, "PlotAxes");
    axes.position = Rect::new(230.0, 30.0, 390.0, 420.0);

    panel.children.push(btn);
    root.children.push(panel);
    root.children.push(axes);

    let candidate_paths = [
        std::path::PathBuf::from("docs/assets/gui_window_sample.png"),
        std::path::PathBuf::from("../../docs/assets/gui_window_sample.png"),
    ];

    let mut out_path = candidate_paths[0].clone();
    for p in &candidate_paths {
        if let Some(parent) = p.parent() {
            if parent.exists() {
                out_path = p.clone();
                break;
            }
        }
    }

    mlapp_render::save_screenshot_png(&root, &out_path).expect("save_screenshot_png failed");

    assert!(
        out_path.exists(),
        "Screenshot file was not generated: {:?}",
        out_path
    );
    let bytes = std::fs::read(&out_path).expect("read screenshot file failed");
    assert!(
        bytes.len() > 1000,
        "Screenshot file is unexpectedly small: {} bytes",
        bytes.len()
    );

    // Verify PNG magic bytes
    let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(&bytes[0..8], &png_header, "Invalid PNG header");
}
