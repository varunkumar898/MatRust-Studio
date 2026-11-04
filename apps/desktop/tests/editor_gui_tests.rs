//! GUI-EDITOR integration tests: layout model, gutter alignment, click focus,
//! egui id hygiene and control-registry coverage for the code editor panel.
//!
//! These tests run egui headlessly. Every test supplies the exact panel rect
//! that the editor body receives in the real window, so the rect assertions are
//! about the same code path used by the desktop app.

use egui::{
    vec2, Align, Context, Event, Key, Layout, Modifiers, MouseWheelUnit, PointerButton, Pos2,
    RawInput, Rect, UiBuilder, Vec2,
};
use matrust_studio::editor::{
    compute_editor_layout, logical_line_count, EditorCallbacks, EditorPanel,
    EDITOR_DEFAULT_FONT_SIZE, EDITOR_MIN_PANEL_HEIGHT,
};
use matrust_studio::theme::Theme;
use matrust_studio::ui_registry;

fn panel_with(content: &str) -> EditorPanel {
    let mut panel = EditorPanel::default();
    if let Some(doc) = panel.tabs.get_mut(0) {
        doc.content = content.to_string();
    }
    panel
}

fn long_script(lines: usize) -> String {
    (1..=lines)
        .map(|i| format!("x{i} = {i}; % line {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn raw_input(screen: Vec2, events: Vec<Event>, modifiers: Modifiers) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
        events,
        modifiers,
        ..Default::default()
    }
}

fn run_editor_frame(
    ctx: &Context,
    panel: &mut EditorPanel,
    panel_rect: Rect,
    events: Vec<Event>,
    modifiers: Modifiers,
) -> matrust_studio::editor::EditorLayout {
    let raw = raw_input(panel_rect.size(), events, modifiers);
    let mut rendered = None;
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let mut child = ui.new_child(
                    UiBuilder::default()
                        .max_rect(panel_rect)
                        .layout(Layout::top_down(Align::LEFT)),
                );
                rendered = Some(panel.show_content(&mut child, panel_rect, &Theme::dark()));
            });
    });
    rendered.expect("editor frame should run")
}

fn run_full_editor_frame(
    ctx: &Context,
    panel: &mut EditorPanel,
    panel_rect: Rect,
    events: Vec<Event>,
) {
    let raw = raw_input(panel_rect.size(), events, Modifiers::NONE);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let mut child = ui.new_child(
                    UiBuilder::default()
                        .max_rect(panel_rect)
                        .layout(Layout::top_down(Align::LEFT)),
                );
                let mut run_script = None;
                let mut run_selection = None;
                let mut run_section = None;
                let mut stop = false;
                panel.show(
                    &mut child,
                    &Theme::dark(),
                    &[],
                    EditorCallbacks {
                        run_script: &mut run_script,
                        run_selection: &mut run_selection,
                        run_section: &mut run_section,
                        stop: &mut stop,
                    },
                );
            });
    });
}

fn click_at(
    ctx: &Context,
    panel: &mut EditorPanel,
    panel_rect: Rect,
    pos: Pos2,
) -> matrust_studio::editor::EditorLayout {
    let _ = run_editor_frame(
        ctx,
        panel,
        panel_rect,
        vec![Event::PointerMoved(pos)],
        Modifiers::NONE,
    );
    let _ = run_editor_frame(
        ctx,
        panel,
        panel_rect,
        vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    run_editor_frame(
        ctx,
        panel,
        panel_rect,
        vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    )
}

#[test]
fn test_editor_content_rect_matches_panel_at_1280_and_1920() {
    let sizes = [(1280.0, 720.0), (1920.0, 1080.0)];
    let contents = [String::new(), long_script(500)];

    for (width, height) in sizes {
        for content in &contents {
            let ctx = Context::default();
            let mut panel = panel_with(content);
            let panel_rect = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
            let layout =
                run_editor_frame(&ctx, &mut panel, panel_rect, Vec::new(), Modifiers::NONE);

            let min_delta = (layout.content_rect.min - panel_rect.min).length();
            let max_delta = (layout.content_rect.max - panel_rect.max).length();
            assert!(
                min_delta <= 1.0 && max_delta <= 1.0,
                "editor content rect {:?} must equal panel rect {:?} at {width}x{height} (deltas {min_delta}, {max_delta})",
                layout.content_rect,
                panel_rect
            );
            assert_eq!(layout.panel_rect, panel_rect);
            assert!(layout.gutter_rect.width() > 0.0);
            assert!(layout.text_rect.width() > 1.0);
            assert!(layout.text_rect.height() > 1.0);
            assert!(layout.content_rect.height() >= EDITOR_MIN_PANEL_HEIGHT.min(height));
        }
    }
}

#[test]
fn test_editor_gutter_rows_match_visible_text_rows() {
    let ctx = Context::default();
    let mut panel = panel_with(&long_script(30));
    let panel_rect = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let layout = run_editor_frame(&ctx, &mut panel, panel_rect, Vec::new(), Modifiers::NONE);

    assert_eq!(layout.rows.len(), layout.visible_rows);
    assert_eq!(layout.rows.first().map(|r| r.line_no), Some(1));
    assert_eq!(layout.first_visible_line, 1);
    assert_eq!(
        layout.line_count,
        logical_line_count(&panel.tabs[0].content)
    );

    let galley_rows = panel
        .last_galley_row_tops
        .as_ref()
        .expect("TextEdit galley rows are recorded");
    assert_eq!(galley_rows.len(), layout.line_count);
    let comparable_rows = galley_rows.len().min(layout.rows.len());
    assert!(comparable_rows > 0);
    for (index, row) in layout.rows.iter().take(comparable_rows).enumerate() {
        let text_row_top = layout
            .text_row_top(index)
            .expect("visible text row index is in range");
        assert!(
            (row.top - text_row_top).abs() <= 0.01,
            "gutter row {index} top {} != model text row top {text_row_top}",
            row.top
        );
        let galley_top = galley_rows[index];
        assert!(
            (row.top - galley_top).abs() <= 1.0,
            "gutter row {index} top {} != real galley row top {galley_top}",
            row.top
        );
    }

    // Short file: the gutter still fills the viewport, but rows past EOF are dim.
    let ctx = Context::default();
    let mut short_panel = panel_with("a = 1;");
    let short_layout = run_editor_frame(
        &ctx,
        &mut short_panel,
        panel_rect,
        Vec::new(),
        Modifiers::NONE,
    );
    assert_eq!(short_layout.rows.len(), short_layout.visible_rows);
    let eof_row = short_layout.rows.iter().find(|r| r.line_no == 2).unwrap();
    assert!(!eof_row.has_text);
    let last_visible = short_layout.rows.last().unwrap();
    assert!(last_visible.line_no > short_layout.line_count);
    assert!(last_visible.top > eof_row.top);
}

#[test]
fn test_editor_click_in_empty_area_focuses_and_puts_cursor_at_end() {
    let ctx = Context::default();
    let content = "a = 1;\nb = 2;";
    let mut panel = panel_with(content);
    let panel_rect = Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 600.0));

    let first = run_editor_frame(&ctx, &mut panel, panel_rect, Vec::new(), Modifiers::NONE);
    assert_eq!(first.line_count, 2);
    let text_id = panel.last_text_id.expect("editor text id recorded");
    let click_pos = Pos2::new(panel_rect.center().x, panel_rect.bottom() - 40.0);

    let layout = click_at(&ctx, &mut panel, panel_rect, click_pos);
    assert!(
        ctx.memory(|memory| memory.has_focus(text_id)),
        "clicking the empty editor area must focus the TextEdit"
    );
    let cursor = panel.last_cursor.expect("cursor recorded");
    assert_eq!(cursor.char_index, content.chars().count());
    assert_eq!(cursor.line, 2);
    assert_eq!(cursor.column, 7);
    assert_eq!(layout.line_count, 2);

    // Typing must work on the next frame without another click.
    let _ = run_editor_frame(
        &ctx,
        &mut panel,
        panel_rect,
        vec![Event::Text("x".to_string())],
        Modifiers::NONE,
    );
    assert_eq!(panel.tabs[0].content, "a = 1;\nb = 2;x");
    assert!(panel.tabs[0].is_dirty);
}

#[test]
fn test_editor_ids_are_unique_per_file_and_panel() {
    let ctx = Context::default();
    let mut first = panel_with("a = 1;");
    let mut second = panel_with("b = 2;");
    if let Some(doc) = first.tabs.get_mut(0) {
        doc.title = "first.m".to_string();
    }
    if let Some(doc) = second.tabs.get_mut(0) {
        doc.title = "second.m".to_string();
    }
    let left = Rect::from_min_size(Pos2::ZERO, vec2(640.0, 400.0));
    let right = Rect::from_min_size(Pos2::new(640.0, 0.0), vec2(640.0, 400.0));

    let mut rendered = Vec::new();
    let raw = raw_input(vec2(1280.0, 400.0), Vec::new(), Modifiers::NONE);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                for (panel, rect) in [(&mut first, left), (&mut second, right)] {
                    let mut child = ui.new_child(
                        UiBuilder::default()
                            .max_rect(rect)
                            .layout(Layout::top_down(Align::LEFT)),
                    );
                    rendered.push(panel.show_content(&mut child, rect, &Theme::dark()));
                }
            });
    });
    assert_eq!(rendered.len(), 2);
    let first_scroll = first.last_scroll_id.expect("first scroll id");
    let first_text = first.last_text_id.expect("first text id");
    let second_scroll = second.last_scroll_id.expect("second scroll id");
    let second_text = second.last_text_id.expect("second text id");
    assert_ne!(first_scroll, second_scroll, "ScrollArea ids must differ");
    assert_ne!(first_text, second_text, "TextEdit ids must differ");
    assert_ne!(first_scroll, first_text);
    assert_ne!(second_scroll, second_text);

    let audit = ui_registry::audit(&ctx);
    assert!(
        audit.clashes.is_empty(),
        "two different editor documents must not share egui ids: {:?}",
        audit.clashes
    );

    // Negative control: the audit really does detect a deliberate duplicate.
    let ctx_duplicate = Context::default();
    let mut duplicate_a = panel_with("same = 1;");
    let mut duplicate_b = panel_with("same = 2;");
    if let Some(doc) = duplicate_a.tabs.get_mut(0) {
        doc.title = "same.m".to_string();
    }
    if let Some(doc) = duplicate_b.tabs.get_mut(0) {
        doc.title = "same.m".to_string();
    }
    let raw = raw_input(vec2(1280.0, 400.0), Vec::new(), Modifiers::NONE);
    let _ = ctx_duplicate.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                for (panel, rect) in [(&mut duplicate_a, left), (&mut duplicate_b, right)] {
                    let mut child = ui.new_child(
                        UiBuilder::default()
                            .max_rect(rect)
                            .layout(Layout::top_down(Align::LEFT)),
                    );
                    panel.show_content(&mut child, rect, &Theme::dark());
                }
            });
    });
    let duplicate_audit = ui_registry::audit(&ctx_duplicate);
    assert!(
        !duplicate_audit.clashes.is_empty(),
        "the id audit must catch two panels using the same file key"
    );
}

#[test]
fn test_editor_font_shortcuts_and_tab_inserts_spaces() {
    let ctx = Context::default();
    let mut panel = EditorPanel::default();
    assert_eq!(panel.font_size, EDITOR_DEFAULT_FONT_SIZE);

    let command = Modifiers {
        command: true,
        ctrl: true,
        ..Modifiers::NONE
    };
    let key_event = |key| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: command,
    };
    let raw = raw_input(vec2(800.0, 600.0), vec![key_event(Key::Equals)], command);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        panel.handle_font_shortcuts(ctx);
    });
    assert_eq!(panel.font_size, EDITOR_DEFAULT_FONT_SIZE + 1.0);

    let raw = raw_input(vec2(800.0, 600.0), vec![key_event(Key::Minus)], command);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        panel.handle_font_shortcuts(ctx);
    });
    assert_eq!(panel.font_size, EDITOR_DEFAULT_FONT_SIZE);

    let raw = raw_input(vec2(800.0, 600.0), vec![key_event(Key::Num0)], command);
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        panel.handle_font_shortcuts(ctx);
    });
    assert_eq!(panel.font_size, EDITOR_DEFAULT_FONT_SIZE);

    // Focus via click, then press Tab and verify four spaces (no tab char).
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    let _ = click_at(&ctx, &mut panel, rect, Pos2::new(400.0, 300.0));
    assert_eq!(panel.tabs[0].content, "");
    let text_id = panel.last_text_id.expect("text id after focus");
    assert!(
        ctx.memory(|memory| memory.has_focus(text_id)),
        "click should focus the empty editor"
    );
    // One idle frame lets TextEdit install its focus filter (tab will indent),
    // exactly as it would before a real user can press the next key.
    let _ = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);
    let tab_event = Event::Key {
        key: Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    let _ = run_editor_frame(&ctx, &mut panel, rect, vec![tab_event], Modifiers::NONE);
    assert_eq!(panel.tabs[0].content, "    ");
    assert!(!panel.tabs[0].content.contains('\t'));
}

#[test]
fn test_editor_gutter_width_grows_from_99_to_100_lines() {
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let layout_99 = compute_editor_layout(
        rect,
        &long_script(99),
        EDITOR_DEFAULT_FONT_SIZE,
        18.0,
        8.0,
        Vec2::ZERO,
        false,
    );
    let layout_100 = compute_editor_layout(
        rect,
        &long_script(100),
        EDITOR_DEFAULT_FONT_SIZE,
        18.0,
        8.0,
        Vec2::ZERO,
        false,
    );
    assert_eq!(layout_99.line_count, 99);
    assert_eq!(layout_100.line_count, 100);
    assert!(
        layout_100.gutter_width > layout_99.gutter_width,
        "gutter width must grow when the line number gains a digit: {} -> {}",
        layout_99.gutter_width,
        layout_100.gutter_width
    );
    assert_eq!(
        layout_100.gutter_width - layout_99.gutter_width,
        8.0,
        "one extra monospace digit at char width 8"
    );
}

#[test]
fn test_editor_controls_are_registered() {
    let ctx = Context::default();
    let mut panel = panel_with("a = 1;");
    let panel_rect = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    run_full_editor_frame(&ctx, &mut panel, panel_rect, Vec::new());

    let audit = ui_registry::audit(&ctx);
    let registered: Vec<&str> = audit
        .controls
        .iter()
        .map(|control| control.control_id.as_str())
        .collect();
    for required in [
        "editor.tab",
        "editor.new_script",
        "editor.run_script",
        "editor.run_selection",
        "editor.run_section",
        "editor.stop",
        "editor.wrap_toggle",
        "editor.font_decrease",
        "editor.font_increase",
        "editor.font_reset",
        "editor.scroll",
        "editor.text",
    ] {
        assert!(
            registered.contains(&required),
            "control {required} was not registered; got {registered:?}"
        );
        assert!(
            ui_registry::editor_control(required).is_some(),
            "control {required} has no central registry spec"
        );
    }

    let specs = ui_registry::editor_controls();
    let mut ids = std::collections::HashSet::new();
    for spec in specs {
        assert!(ids.insert(spec.id), "duplicate registry spec {}", spec.id);
        assert_eq!(spec.panel, "Editor");
        assert!(spec.id.starts_with("editor."));
    }
}

#[test]
fn test_editor_background_diagnostics_and_first_error_jump() {
    use std::time::{Duration, Instant};

    let ctx = Context::default();
    let mut panel = panel_with("x = (1 + 2;\ny = 4;\n");
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 400.0));

    // First frame starts the 300 ms quiet period; no parsing on the UI thread.
    let _ = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);
    assert_eq!(panel.diagnostics().len(), 0);

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && panel.diagnostics().is_empty() {
        std::thread::sleep(Duration::from_millis(40));
        let _ = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);
    }
    let diagnostics = panel.diagnostics();
    assert_eq!(diagnostics.len(), 1, "one parser error expected");
    let expected_line = diagnostics[0].line;
    let expected_column = diagnostics[0].column;
    assert_eq!(expected_line, 1);
    assert!(expected_column >= 1);
    assert!(diagnostics[0].message.contains("Syntax error"));
    assert!(panel.tabs[0].error_line.is_some());

    panel.jump_to_first_error(&ctx);
    let _ = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);
    let cursor = panel.last_cursor.expect("cursor after jump");
    assert_eq!(
        cursor.line, expected_line,
        "jump must land on the diagnostic line"
    );
    assert_eq!(cursor.column, expected_column);
}

#[test]
fn test_section_marker_is_visible_and_runs_only_that_section() {
    let ctx = Context::default();
    let content = "%% First section\nx = 1;\n%% Second section\ny = 2;\n";
    let mut panel = panel_with(content);
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 400.0));
    let layout = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);

    let audit = ui_registry::audit(&ctx);
    let marker = audit
        .controls
        .iter()
        .find(|control| control.control_id == "editor.run_section_marker.1")
        .expect("section marker must be registered in the gutter")
        .clone();
    assert!(
        layout.gutter_rect.contains_rect(marker.rect),
        "section marker must live in the gutter: {:?}",
        marker.rect
    );

    let _ = click_at(&ctx, &mut panel, rect, marker.rect.center());
    let section = panel
        .section_run_request
        .take()
        .expect("clicking the marker must request a section run");
    assert!(
        section.contains("x = 1;"),
        "first section should run: {section:?}"
    );
    assert!(
        !section.contains("y = 2;"),
        "the next section must not run: {section:?}"
    );
}

#[test]
fn test_editor_toolbar_has_exactly_one_run_control_set() {
    use std::collections::HashMap;

    let ctx = Context::default();
    let mut panel = panel_with("x = 1;");
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 400.0));
    run_full_editor_frame(&ctx, &mut panel, rect, Vec::new());
    let mut counts: HashMap<String, usize> = HashMap::new();
    for control in ui_registry::audit(&ctx).controls {
        *counts.entry(control.control_id).or_default() += 1;
    }
    for required in [
        "editor.run_script",
        "editor.run_section",
        "editor.run_selection",
        "editor.stop",
    ] {
        assert_eq!(
            counts.get(required).copied().unwrap_or(0),
            1,
            "editor toolbar must contain exactly one {required}: {counts:?}"
        );
    }
    assert!(
        !counts.contains_key("editor.run_script_duplicate"),
        "no legacy duplicate Run control may remain"
    );
}

#[test]
fn test_editor_mouse_wheel_scrolls_a_long_document() {
    let ctx = Context::default();
    let mut panel = panel_with(&long_script(500));
    let rect = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 400.0));
    let first = run_editor_frame(&ctx, &mut panel, rect, Vec::new(), Modifiers::NONE);
    assert_eq!(first.first_visible_line, 1);

    let scroll_pos = rect.center();
    let _ = run_editor_frame(
        &ctx,
        &mut panel,
        rect,
        vec![Event::PointerMoved(scroll_pos)],
        Modifiers::NONE,
    );
    let scrolled = run_editor_frame(
        &ctx,
        &mut panel,
        rect,
        vec![Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: vec2(0.0, -8.0),
            modifiers: Modifiers::NONE,
        }],
        Modifiers::NONE,
    );
    assert!(
        scrolled.first_visible_line > 1 && scrolled.scroll_offset.y > 0.0,
        "wheel input must scroll the editor: first line {}, offset {:?}",
        scrolled.first_visible_line,
        scrolled.scroll_offset
    );
}
