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
