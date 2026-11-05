//! Central UI-control registry and per-pass egui id audit (GUI rule R19).
//!
//! Every interactive editor control declares a stable spec (id, panel, label,
//! shortcut), then registers the real `egui::Id` and rect while it is built.
//! The audit is stored in egui temporary data, so it is per-`Context` and safe
//! for the headless tests to inspect after a pass.
//!
//! Registration is deliberately cheap: a short string and two rect comparisons.
//! It does not change layout or input handling, which keeps R13 (feature flags
//! and core semantics) untouched.

use crate::dock::PanelId;
use egui::{Context, Id, Rect};
use std::collections::HashMap;

/// A control that is part of the documented UI inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiControl {
    /// Stable dotted id, for example `editor.text`.
    pub id: &'static str,
    /// Owning panel, for example `Editor`.
    pub panel: &'static str,
    /// Human-readable purpose.
    pub label: &'static str,
    /// Keyboard shortcut, if the control has one.
    pub shortcut: Option<&'static str>,
}

/// Every interactive control owned by the code editor panel.
pub const EDITOR_CONTROLS: &[UiControl] = &[
    UiControl {
        id: "editor.tab",
        panel: "Editor",
        label: "Document tab",
        shortcut: None,
    },
    UiControl {
        id: "editor.tab_close",
        panel: "Editor",
        label: "Close document tab",
        shortcut: None,
    },
    UiControl {
        id: "editor.new_script",
        panel: "Editor",
        label: "New script",
        shortcut: Some("Ctrl+N"),
    },
    UiControl {
        id: "editor.run_script",
        panel: "Editor",
        label: "Run script",
        shortcut: Some("F5"),
    },
    UiControl {
        id: "editor.run_selection",
        panel: "Editor",
        label: "Run selection",
        shortcut: Some("F9"),
    },
    UiControl {
        id: "editor.run_section",
        panel: "Editor",
        label: "Run section",
        shortcut: Some("Ctrl+Enter"),
    },
    UiControl {
        id: "editor.stop",
        panel: "Editor",
        label: "Stop execution",
        shortcut: Some("Ctrl+C"),
    },
    UiControl {
        id: "editor.wrap_toggle",
        panel: "Editor",
        label: "Word wrap",
        shortcut: None,
    },
    UiControl {
        id: "editor.font_decrease",
        panel: "Editor",
        label: "Decrease editor font size",
        shortcut: Some("Ctrl+-"),
    },
    UiControl {
        id: "editor.font_increase",
        panel: "Editor",
        label: "Increase editor font size",
        shortcut: Some("Ctrl+="),
    },
    UiControl {
        id: "editor.font_reset",
        panel: "Editor",
        label: "Reset editor font size",
        shortcut: Some("Ctrl+0"),
    },
    UiControl {
        id: "editor.scroll",
        panel: "Editor",
        label: "Editor scroll area",
        shortcut: None,
    },
    UiControl {
        id: "editor.text",
        panel: "Editor",
        label: "Code text edit",
        shortcut: Some("Tab inserts 4 spaces"),
    },
    UiControl {
        id: "editor.gutter_row",
        panel: "Editor",
        label: "Line-number gutter row",
        shortcut: None,
    },
    UiControl {
        id: "editor.completion_close",
        panel: "Editor",
        label: "Close completion popup",
        shortcut: Some("Esc"),
    },
    UiControl {
        id: "editor.completion_filter",
        panel: "Editor",
        label: "Completion filter",
        shortcut: None,
    },
    UiControl {
        id: "editor.completion_item",
        panel: "Editor",
        label: "Completion candidate",
        shortcut: None,
    },
    UiControl {
        id: "editor.completion_scroll",
        panel: "Editor",
        label: "Completion candidate list",
        shortcut: None,
    },
];

/// Return the static editor control inventory.
pub fn editor_controls() -> &'static [UiControl] {
    EDITOR_CONTROLS
}

/// Look up one editor control spec by its stable id.
pub fn editor_control(id: &str) -> Option<&'static UiControl> {
    EDITOR_CONTROLS.iter().find(|control| control.id == id)
}

/// One control observed during the current pass.
#[derive(Clone, Debug)]
pub struct RegisteredControl {
    /// Registry spec id (for example `editor.text`).
    pub control_id: String,
    /// Real id passed to egui.
    pub egui_id: Id,
    /// Screen rect where the control was built.
    pub rect: Rect,
}

/// One dock panel body observed during the current pass.
///
/// This is intentionally separate from the interactive-control inventory: a
/// panel is a container, while [`RegisteredControl`] is a leaf the user can
/// operate. The layout contract uses both to check clipping and C2 minimums.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisteredPanel {
    /// Which dock panel owns this body.
    pub panel_id: PanelId,
    /// Body rect after the tab strip.
    pub body_rect: Rect,
    /// egui clip rect in force when the panel body was built.
    pub clip_rect: Rect,
}

/// Two controls in one pass used the same egui id at different positions.
#[derive(Clone, Debug)]
pub struct IdClash {
    /// The duplicated egui id.
    pub egui_id: Id,
    /// Rect of the first registration.
    pub first: Rect,
    /// Rect of the duplicate registration.
    pub second: Rect,
    /// Registry spec id of the first registration.
    pub first_control: String,
    /// Registry spec id of the duplicate registration.
    pub second_control: String,
}

/// Snapshot of the control registry for the current pass.
#[derive(Clone, Debug, Default)]
pub struct UiAudit {
    /// Pass sequence number in this context.
    pub pass: u64,
    /// Controls built in this pass.
    pub controls: Vec<RegisteredControl>,
    /// Dock panel bodies built in this pass.
    pub panels: Vec<RegisteredPanel>,
    /// Duplicate egui ids detected in this pass.
    pub clashes: Vec<IdClash>,
}

#[derive(Clone, Debug, Default)]
struct RegistryState {
    pass: u64,
    started: bool,
    audit: UiAudit,
    seen: HashMap<Id, RegisteredControl>,
}

fn state_id() -> Id {
    Id::new("matrust_ui_registry_state")
}

/// Start a new audit pass. Call once at the top of an app update or test frame.
pub fn begin_frame(ctx: &Context) {
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<RegistryState>(state_id());
        state.pass = state.pass.wrapping_add(1);
        state.started = true;
        state.audit.pass = state.pass;
        state.audit.controls.clear();
        state.audit.panels.clear();
        state.audit.clashes.clear();
        state.seen.clear();
    });
}

/// Register a real egui id with its logical registry control id.
///
/// Returns `false` when the same egui id was already used at a different rect
/// in this pass. The duplicate is also passed to `Context::check_for_id_clash`,
/// so egui's own debug overlay reports it during normal development.
pub fn register(ctx: &Context, control_id: &str, egui_id: Id, rect: Rect) -> bool {
    let label = editor_control(control_id)
        .map(|control| control.label)
        .unwrap_or("UI control");
    ctx.check_for_id_clash(egui_id, rect, label);

    let control = RegisteredControl {
        control_id: control_id.to_string(),
        egui_id,
        rect,
    };

    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<RegistryState>(state_id());
        if !state.started {
            state.pass = state.pass.wrapping_add(1);
            state.started = true;
            state.audit.pass = state.pass;
        }

        let mut ok = true;
        if let Some(previous) = state.seen.get(&egui_id) {
            let same_place = (previous.rect.min - rect.min).length() <= 0.5
                && (previous.rect.max - rect.max).length() <= 0.5;
            if !same_place {
                ok = false;
                state.audit.clashes.push(IdClash {
                    egui_id,
                    first: previous.rect,
                    second: rect,
                    first_control: previous.control_id.clone(),
                    second_control: control.control_id.clone(),
                });
            }
        } else {
            state.seen.insert(egui_id, control.clone());
        }
        state.audit.controls.push(control);
        ok
    })
}

/// Register a dock panel body for the current pass.
///
/// Called by the dock immediately before it builds the active panel body. It
/// does not allocate a widget or affect layout; it only records the two rects
/// needed by [`crate::layout_contract`].
pub fn register_panel(ctx: &Context, panel_id: PanelId, body_rect: Rect, clip_rect: Rect) {
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<RegistryState>(state_id());
        if !state.started {
            state.pass = state.pass.wrapping_add(1);
            state.started = true;
            state.audit.pass = state.pass;
        }
        state.audit.panels.push(RegisteredPanel {
            panel_id,
            body_rect,
            clip_rect,
        });
    });
}

/// Read the current pass audit. The returned value is a clone, so the caller
/// can assert on it after `Context::run` has finished.
pub fn audit(ctx: &Context) -> UiAudit {
    ctx.data(|data| {
        data.get_temp::<RegistryState>(state_id())
            .map(|state| state.audit)
            .unwrap_or_default()
    })
}
