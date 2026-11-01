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

#[test]
fn test_autotest_script_json_deserialization() {
    let json_wrapped = r#"{
        "actions": [
            { "type": "wait_ms", "duration_ms": 50 },
            { "type": "set_value", "component": "FreqField", "value": "10" },
            { "type": "expect_property", "component": "FreqField", "property": "Value", "expected": "10" },
            { "type": "click", "component": "PlotBtn" },
            { "type": "screenshot", "path": "target/test_screenshot.png" },
            { "type": "close" }
        ]
    }"#;
    let script = mlapp_render::AutotestScript::from_json_str(json_wrapped)
        .expect("deserialize wrapped script");
    assert_eq!(script.actions.len(), 6);

    let json_array = r#"[
        { "type": "wait_ms", "duration_ms": 10 },
        { "type": "close" }
    ]"#;
    let script2 =
        mlapp_render::AutotestScript::from_json_str(json_array).expect("deserialize array script");
    assert_eq!(script2.actions.len(), 2);
}

#[test]
fn test_autotest_headless_e2e_with_plot_and_label_mutation() {
    use mlapp_host::{AppSession, HostConfig};
    use std::io::Cursor;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", options)
            .unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>AutotestDemoApp</dc:title>
    <cp:version>1.0.0</cp:version>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", options).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 640, 480]">
        <Label Name="StatusLabel" Text="InitialReady" Position="20, 400, 200, 30" />
        <Button Name="PlotBtn" Text="Run Plot" Position="20, 350, 100, 30" ButtonPushedFcn="PlotBtnPushed" />
        <UIAxes Name="PlotAxes" Position="150, 50, 450, 350" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", options).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef AutotestDemoApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            StatusLabel
            PlotBtn
            PlotAxes
        end
        methods (Access = private)
            function PlotBtnPushed(app, event)
                t = linspace(0, 1, 50);
                y = sin(2 * 3.14159 * 5 * t);
                plot(app.PlotAxes, t, y);
                app.StatusLabel.Text = 'PlotGenerated:50pts';
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mut session = AppSession::launch(Cursor::new(buf), HostConfig::default())
        .expect("launch AutotestDemoApp session");

    let candidate_dirs = [
        std::path::PathBuf::from("docs/screenshots/c4a0ff9"),
        std::path::PathBuf::from("../../docs/screenshots/c4a0ff9"),
    ];
    let mut screenshot_dir = candidate_dirs[0].clone();
    for d in &candidate_dirs {
        if let Some(parent) = d.parent() {
            if parent.exists() {
                screenshot_dir = d.clone();
                break;
            }
        }
    }
    let _ = std::fs::create_dir_all(&screenshot_dir);
    let screenshot_path = screenshot_dir.join("gui_window_autotest.png");

    let script = mlapp_render::AutotestScript {
        actions: vec![
            mlapp_render::AutotestAction::ExpectProperty {
                component: "StatusLabel".to_string(),
                property: "Text".to_string(),
                expected: "InitialReady".to_string(),
            },
            mlapp_render::AutotestAction::Click {
                component: "PlotBtn".to_string(),
            },
            mlapp_render::AutotestAction::WaitMs { duration_ms: 100 },
            mlapp_render::AutotestAction::ExpectProperty {
                component: "StatusLabel".to_string(),
                property: "Text".to_string(),
                expected: "PlotGenerated:50pts".to_string(),
            },
            mlapp_render::AutotestAction::ExpectProperty {
                component: "PlotAxes".to_string(),
                property: "HasPlottedData".to_string(),
                expected: "true".to_string(),
            },
            mlapp_render::AutotestAction::Screenshot {
                path: screenshot_path.clone(),
            },
            mlapp_render::AutotestAction::Close,
        ],
    };

    mlapp_render::run_autotest_headless(&mut session, &script)
        .expect("headless autotest execution failed");

    assert!(
        screenshot_path.exists(),
        "Screenshot must be created at {:?}",
        screenshot_path
    );
    let bytes = std::fs::read(&screenshot_path).expect("read screenshot");
    assert!(bytes.len() > 1000, "Screenshot size must be > 1000 bytes");
}
