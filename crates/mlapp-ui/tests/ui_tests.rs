use mlapp_ui::{
    figure_to_top_left_geometry, matlab_to_top_left_geometry, parse_app_xml,
    top_left_to_matlab_geometry, ComponentKind, PropertyValue, Rect,
};

#[test]
fn test_top_level_figure_geometry_exact_values() {
    // In MATLAB, a figure might have Position="100 100 800 600" on desktop display
    let fig_rect = Rect::new(100.0, 100.0, 800.0, 600.0);

    // 1. In window-local mode, the client area root is exact (0, 0, 800, 600)
    let local = figure_to_top_left_geometry(fig_rect, None);
    assert_eq!(local.x, 0.0);
    assert_eq!(local.y, 0.0);
    assert_eq!(local.width, 800.0);
    assert_eq!(local.height, 600.0);

    // 2. In screen desktop mode with screen height 1080
    // top_y = 1080 - (100 + 600) = 380.0 exact
    let screen_1080 = figure_to_top_left_geometry(fig_rect, Some(1080.0));
    assert_eq!(screen_1080.x, 100.0);
    assert_eq!(screen_1080.y, 380.0);
    assert_eq!(screen_1080.width, 800.0);
    assert_eq!(screen_1080.height, 600.0);

    // 3. In screen desktop mode with screen height 1440
    // top_y = 1440 - (100 + 600) = 740.0 exact
    let screen_1440 = figure_to_top_left_geometry(fig_rect, Some(1440.0));
    assert_eq!(screen_1440.x, 100.0);
    assert_eq!(screen_1440.y, 740.0);
    assert_eq!(screen_1440.width, 800.0);
    assert_eq!(screen_1440.height, 600.0);
}

#[test]
fn test_coordinate_transformation_bidirectional() {
    let parent_height = 600.0;
    // Component at bottom-left (50, 40) with width 100, height 30
    let matlab_rect = Rect::new(50.0, 40.0, 100.0, 30.0);

    let top_left = matlab_to_top_left_geometry(matlab_rect, parent_height);
    // top_y = 600 - (40 + 30) = 530
    assert_eq!(top_left.x, 50.0);
    assert_eq!(top_left.y, 530.0);
    assert_eq!(top_left.width, 100.0);
    assert_eq!(top_left.height, 30.0);

    // Invert back to MATLAB bottom-left coordinates
    let restored = top_left_to_matlab_geometry(top_left, parent_height);
    assert_eq!(restored.x, matlab_rect.x);
    assert_eq!(restored.y, matlab_rect.y);
    assert_eq!(restored.width, matlab_rect.width);
    assert_eq!(restored.height, matlab_rect.height);
}

#[test]
fn test_parse_app_xml_component_tree() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<appdata>
  <UIFigure Name="UIFigure" Position="[100, 100, 800, 600]">
    <Panel Name="ControlPanel" Position="20, 20, 260, 560">
      <Button Name="CalculateBtn" Text="Calculate" Position="20, 450, 100, 30" ButtonPushedFcn="CalculateBtnPushed" />
      <NumericEditField Name="FrequencyField" Value="5.0" Position="20, 400, 100, 22" />
      <Slider Name="GainSlider" Position="20, 350, 150, 3" />
      <Switch Name="PowerSwitch" Position="20, 300, 45, 20" />
    </Panel>
    <UIAxes Name="PlotAxes" Position="300, 100, 480, 480" />
    <FutureExoticWidget Name="CustomGadget" Position="10, 10, 50, 50" CustomProp="123" />
  </UIFigure>
</appdata>"#;

    let mut root = parse_app_xml(xml).expect("parse app.xml");
    assert_eq!(root.kind, ComponentKind::UIFigure);
    assert_eq!(root.name, "UIFigure");
    assert_eq!(root.position, Rect::new(100.0, 100.0, 800.0, 600.0));

    // Find panel
    let panel = root
        .find_by_name("ControlPanel")
        .expect("find ControlPanel");
    assert_eq!(panel.kind, ComponentKind::Panel);
    assert_eq!(panel.children.len(), 4);

    // Find button inside panel
    let btn = root
        .find_by_name("CalculateBtn")
        .expect("find CalculateBtn");
    assert_eq!(btn.kind, ComponentKind::Button);
    assert_eq!(
        btn.callbacks.get("buttonpushedfcn").map(|s| s.as_str()),
        Some("CalculateBtnPushed")
    );

    // Find unknown component and verify it does NOT crash
    let custom = root
        .find_by_name("CustomGadget")
        .expect("find CustomGadget");
    assert_eq!(
        custom.kind,
        ComponentKind::Unknown("FutureExoticWidget".to_string())
    );

    // Test property mutation
    let freq_field = root
        .find_by_name_mut("FrequencyField")
        .expect("find FrequencyField");
    assert_eq!(freq_field.kind, ComponentKind::NumericEditField);
    freq_field
        .set_property("Value", PropertyValue::String("10.0".to_string()))
        .expect("set property");
    assert_eq!(
        freq_field.get_property("Value"),
        Some(&PropertyValue::String("10.0".to_string()))
    );
}

#[test]
fn test_xml_nesting_depth_limit() {
    use mlapp_ui::UiError;

    // Construct deeply nested XML with 10,000 opening tags: "<Panel><Panel>..."
    let n = 2_000;
    let mut xml =
        String::from(r#"<?xml version="1.0" encoding="utf-8"?><UIFigure Name="UIFigure">"#);
    for i in 0..n {
        xml.push_str(&format!(r#"<Panel Name="P{i}">"#));
    }
    for _i in (0..n).rev() {
        xml.push_str("</Panel>");
    }
    xml.push_str("</UIFigure>");

    let res = parse_app_xml(&xml);
    assert!(res.is_err(), "deeply nested XML must return an error");
    match res.unwrap_err() {
        UiError::MaxDepthExceeded(depth) => {
            assert!(depth >= 1000);
        }
        other => panic!("expected MaxDepthExceeded, got {other:?}"),
    }
}

#[test]
fn test_dos_xml_depth_999_small_stack() {
    // Construct XML with exactly 999 nested panels inside UIFigure (stack depth 999)
    let n = 998; // 1 UIFigure + 998 nested Panels = depth 999
    let mut xml =
        String::from(r#"<?xml version="1.0" encoding="utf-8"?><UIFigure Name="UIFigure">"#);
    for i in 0..n {
        xml.push_str(&format!(r#"<Panel Name="P{i}">"#));
    }
    for _i in (0..n).rev() {
        xml.push_str("</Panel>");
    }
    xml.push_str("</UIFigure>");

    // Execute parsing and dropping on a dedicated thread with a small 128 KiB stack
    let builder = std::thread::Builder::new().stack_size(128 * 1024);
    let handle = builder
        .spawn(move || {
            let res = parse_app_xml(&xml);
            assert!(res.is_ok(), "depth 999 must succeed within depth limit");
            let comp = res.unwrap();
            drop(comp); // Iterative drop must not overflow the 128 KiB stack
        })
        .expect("spawn small-stack thread");

    handle.join().expect("thread join must not panic or abort");
}
