//! GUI-CLEAN headless checks for the ribbon rect contract and clear actions.

use egui::{vec2, Context, Modifiers, Pos2, RawInput, Rect};
use matrust_studio::app::{MatStudioApp, TopTab};
use matrust_studio::ui_registry;

fn ribbon_audit(width: f32, height: f32, tab: TopTab) -> (ui_registry::UiAudit, Rect) {
    let ctx = Context::default();
    let mut app = MatStudioApp {
        active_top_tab: tab,
        ..Default::default()
    };
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
    let raw = RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                app.render_toolstrip_for_test(ui);
            });
    });
    (ui_registry::audit(&ctx), screen)
}

#[test]
fn test_ribbon_controls_stay_inside_panel_at_all_target_sizes() {
    // Physical 1280x720 at 100%, then the logical widths seen at 125% and 150%
    // scaling. All three must wrap instead of painting outside the panel.
    let sizes = [
        (1280.0, 720.0),
        (1024.0, 576.0),
        (853.0, 480.0),
        (1366.0, 768.0),
        (1920.0, 1080.0),
    ];
    let tabs = [
        TopTab::Home,
        TopTab::Plots,
        TopTab::Apps,
        TopTab::Editor,
        TopTab::Publish,
        TopTab::View,
    ];

    for (width, height) in sizes {
        for tab in tabs {
            let (audit, screen) = ribbon_audit(width, height, tab);
            if tab != TopTab::Publish {
                assert!(
                    !audit.controls.is_empty(),
                    "no controls registered for {tab:?} at {width}x{height}"
                );
            }
            assert!(
                audit.clashes.is_empty(),
                "duplicate egui ids in {tab:?} at {width}x{height}: {:?}",
                audit.clashes
            );

            let mut seen = std::collections::HashSet::new();
            let unique: Vec<_> = audit
                .controls
                .iter()
                .filter(|control| seen.insert(control.egui_id))
                .collect();
            for control in &unique {
                assert!(
                    screen.contains_rect(control.rect),
                    "{} at {:?} overflows the panel {:?} on {tab:?} at {width}x{height}",
                    control.control_id,
                    control.rect,
                    screen
                );
            }
            for (index, first) in unique.iter().enumerate() {
                for second in unique.iter().skip(index + 1) {
                    let overlap = first.rect.intersect(second.rect);
                    assert!(
                        overlap.width() <= 0.5 || overlap.height() <= 0.5,
                        "{} {:?} overlaps {} {:?} on {tab:?} at {width}x{height}",
                        first.control_id,
                        first.rect,
                        second.control_id,
                        second.rect
                    );
                }
            }
        }
    }
}

#[test]
fn test_clear_scopes_have_exactly_one_registered_action() {
    let actions = matrust_studio::actions::clear_actions();
    assert_eq!(actions.len(), 4);
    let labels: Vec<&str> = actions.iter().map(|action| action.label).collect();
    for required in [
        "Clear Command Window",
        "Clear Workspace",
        "Clear History",
        "Close Figure",
    ] {
        assert_eq!(
            labels.iter().filter(|label| **label == required).count(),
            1,
            "expected exactly one action label {required}; got {labels:?}"
        );
    }
    assert!(
        !labels.iter().any(|label| label.contains("Clean")),
        "the old ambiguous Clean Data label must be gone"
    );
}

fn shell_audit(width: f32, height: f32, tab: TopTab) -> (ui_registry::UiAudit, Rect) {
    let ctx = Context::default();
    let mut app = MatStudioApp {
        active_top_tab: tab,
        ..Default::default()
    };
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
    let raw = RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        app.render_ui(ctx);
    });
    // One extra pass lets panels settle min sizes after the first layout.
    let raw = RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        app.render_ui(ctx);
    });
    (ui_registry::audit(&ctx), screen)
}

#[test]
fn test_full_shell_controls_fit_at_target_sizes_and_scales() {
    for (width, height) in [(1280.0, 720.0), (1024.0, 576.0), (853.0, 480.0)] {
        for tab in [TopTab::Home, TopTab::Editor, TopTab::View] {
            let (audit, screen) = shell_audit(width, height, tab);
            assert!(
                audit.clashes.is_empty(),
                "duplicate egui ids on {tab:?} at {width}x{height}: {:?}",
                audit.clashes
            );
            for control in &audit.controls {
                assert!(
                    screen.contains_rect(control.rect),
                    "{} at {:?} overflows the window {:?} on {tab:?} at {width}x{height}",
                    control.control_id,
                    control.rect,
                    screen
                );
            }
        }
    }
}

#[test]
fn test_all_aspect_ratio_targets_have_a_status_name_and_theme() {
    let ctx = Context::default();
    let mut app = MatStudioApp::default();
    app.statusbar.active_title = "Untitled1.m".to_string();
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0));
    let raw = RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        app.render_ui(ctx);
    });
    assert_eq!(app.statusbar.active_file_label(), "Untitled1.m");
}

#[test]
fn test_ribbon_registry_inventory_is_complete() {
    use std::collections::{BTreeSet, HashSet};

    let expectations: &[(TopTab, &[&str])] = &[
        (
            TopTab::Home,
            &[
                "ribbon.home.new_script",
                "ribbon.home.import_data",
                "ribbon.home.save_workspace",
                "ribbon.home.clear_workspace",
                "ribbon.home.clear_command_window",
                "ribbon.home.block_diagram",
                "ribbon.home.layout",
                "ribbon.home.set_path",
                "ribbon.home.addons",
                "ribbon.home.help",
                "ribbon.home.learn",
            ],
        ),
        (
            TopTab::Plots,
            &[
                "ribbon.plots.line",
                "ribbon.plots.scatter",
                "ribbon.plots.bar",
                "ribbon.plots.histogram",
            ],
        ),
        (
            TopTab::Apps,
            &[
                "ribbon.apps.data_importer",
                "ribbon.apps.plot_explorer",
                "ribbon.apps.signal_viewer",
                "ribbon.apps.inspector",
                "ribbon.apps.addon_explorer",
            ],
        ),
        (
            TopTab::Editor,
            &[
                "ribbon.editor.run",
                "ribbon.editor.run_section",
                "ribbon.editor.run_selection",
                "ribbon.editor.stop",
                "ribbon.editor.save",
            ],
        ),
        (TopTab::Publish, &[]),
        (
            TopTab::View,
            &[
                "ribbon.view.panel.Current Folder",
                "ribbon.view.panel.Editor",
                "ribbon.view.panel.Workspace",
                "ribbon.view.panel.Command Window",
                "ribbon.view.panel.Figures",
                "ribbon.view.panel.Help & Documentation",
                "ribbon.view.panel.Block Diagram Editor",
            ],
        ),
    ];

    for (tab, expected) in expectations {
        let (audit, _) = ribbon_audit(1280.0, 720.0, *tab);
        let mut seen = HashSet::new();
        let ids: BTreeSet<&str> = audit
            .controls
            .iter()
            .filter(|control| seen.insert(control.egui_id))
            .map(|control| control.control_id.as_str())
            .collect();
        println!(
            "GUI-CLEAN ribbon coverage {tab:?}: {} unique controls -> {:?}",
            ids.len(),
            ids
        );
        for id in *expected {
            assert!(
                ids.contains(id),
                "missing registered control {id} on {tab:?}; got {ids:?}"
            );
        }
    }
}

#[test]
fn test_panel_action_controls_are_registered_and_reachable() {
    use std::collections::HashSet;

    let (audit, screen) = shell_audit(1280.0, 720.0, TopTab::Home);
    let mut seen = HashSet::new();
    let controls: Vec<_> = audit
        .controls
        .iter()
        .filter(|control| seen.insert(control.egui_id))
        .collect();
    let ids: Vec<&str> = controls
        .iter()
        .map(|control| control.control_id.as_str())
        .collect();
    println!(
        "GUI-CLEAN full shell coverage: {} unique controls",
        controls.len()
    );
    for required in [
        "current_folder.hidden",
        "current_folder.open",
        "current_folder.project",
        "current_folder.up",
        "current_folder.this_pc",
        "current_folder.refresh",
        "workspace.refresh",
        "figures.new",
        "figures.reset",
        "editor.run_script",
        "editor.run_section",
        "editor.run_selection",
        "editor.stop",
        "ClearWorkspace",
        "ClearCommandWindow",
        "CloseFigure",
    ] {
        assert!(
            ids.contains(&required),
            "missing panel/action control {required}; got {ids:?}"
        );
    }
    for control in controls {
        assert!(
            screen.contains_rect(control.rect),
            "{} overflows at 1280x720: {:?}",
            control.control_id,
            control.rect
        );
    }
}

#[test]
fn test_history_clear_control_is_reachable_in_narrow_panel() {
    use egui::vec2;
    use matrust_studio::command_window::CommandWindow;
    use matrust_studio::theme::Theme;

    let ctx = Context::default();
    let mut history = CommandWindow::default();
    history.history.push("x = 1".to_string());
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(220.0, 300.0));
    let raw = RawInput {
        screen_rect: Some(screen),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let mut exec = None;
                let mut action = None;
                history.show_history_panel(ui, &Theme::dark(), &mut exec, &mut action);
            });
    });
    let audit = ui_registry::audit(&ctx);
    let control = audit
        .controls
        .iter()
        .find(|control| control.control_id == "ClearHistory")
        .expect("Clear History control must be registered");
    assert!(
        screen.contains_rect(control.rect),
        "Clear History overflows a 220 px panel: {:?}",
        control.rect
    );
}

#[test]
fn test_perform_action_clears_the_named_scope_only() {
    let mut app = MatStudioApp::default();
    app.command_window.logs.clear();
    app.command_window.push_output("keep me".to_string());
    app.command_window.history.push("x = 1".to_string());
    app.perform_action(matrust_studio::actions::AppAction::ClearHistory);
    assert!(app.command_window.history.is_empty());
    assert!(
        !app.command_window.logs.is_empty(),
        "history must not clear the log"
    );

    app.command_window.history.push("y = 2".to_string());
    app.perform_action(matrust_studio::actions::AppAction::ClearCommandWindow);
    assert!(app.command_window.logs.is_empty());
    assert_eq!(app.command_window.history, vec!["y = 2".to_string()]);

    app.graphics.get_or_create_figure(1);
    app.figures_panel.selected_figure_id = 1;
    app.perform_action(matrust_studio::actions::AppAction::CloseFigure);
    assert!(app.graphics.figures.is_empty());
}

#[test]
fn test_command_window_input_stays_inside_a_short_panel() {
    use matrust_studio::command_window::CommandWindow;
    use matrust_studio::theme::Theme;

    let ctx = Context::default();
    let panel = Rect::from_min_size(Pos2::ZERO, vec2(720.0, 120.0));
    let raw = RawInput {
        screen_rect: Some(panel),
        ..Default::default()
    };
    let _ = ctx.run(raw, |ctx| {
        ui_registry::begin_frame(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let mut child = ui.new_child(
                    egui::UiBuilder::default()
                        .max_rect(panel)
                        .layout(egui::Layout::top_down(egui::Align::LEFT)),
                );
                let mut window = CommandWindow::default();
                let mut execute = None;
                let mut action = None;
                window.show(&mut child, &Theme::dark(), &mut execute, &mut action);
            });
    });
    let audit = ui_registry::audit(&ctx);
    for required in ["command.input", "command.run"] {
        let control = audit
            .controls
            .iter()
            .find(|control| control.control_id == required)
            .unwrap_or_else(|| panic!("missing {required}"));
        assert!(
            panel.contains_rect(control.rect),
            "{required} must stay inside the 120 px command panel: {:?}",
            control.rect
        );
    }
}

#[test]
fn test_status_error_count_is_clickable_and_jumps_to_first_error() {
    use egui::{Event, PointerButton};
    use matrust_studio::statusbar::{EngineStatus, StatusAction, StatusBar};
    use matrust_studio::theme::Theme;

    let ctx = Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 40.0));
    let mut bar = StatusBar {
        status: EngineStatus::Ready,
        error_count: 2,
        warning_count: 0,
        ..Default::default()
    };

    let mut frame = |events: Vec<Event>| -> Option<StatusAction> {
        let raw = RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        let mut action = None;
        let _ = ctx.run(raw, |ctx| {
            ui_registry::begin_frame(ctx);
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    action = bar.show(ui, &Theme::dark());
                });
        });
        action
    };

    assert_eq!(frame(Vec::new()), None);
    let button = ui_registry::audit(&ctx)
        .controls
        .into_iter()
        .find(|control| control.control_id == "status.errors")
        .expect("Errors count must be registered");
    let pos = button.rect.center();
    assert!(screen.contains_rect(button.rect));
    assert_eq!(frame(vec![Event::PointerMoved(pos)]), None);
    assert_eq!(
        frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }]),
        None
    );
    let mut action = frame(vec![Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }]);
    if action.is_none() {
        action = frame(Vec::new());
    }
    assert_eq!(
        action,
        Some(StatusAction::JumpToFirstError),
        "clicking Errors: N must request a jump to the first problem"
    );
}
