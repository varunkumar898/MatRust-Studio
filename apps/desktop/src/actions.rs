//! Named application actions and their single registry entries.
//!
//! GUI-CLEAN requires exactly one action name per clear scope. Menus, ribbon
//! buttons and panel buttons all stay on the same `AppAction` variant, so the
//! label and the executed operation cannot drift apart.

use crate::ui_registry;

/// Clear-scope actions that must have exactly one registered name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppAction {
    /// Clear only the Command Window log.
    ClearCommandWindow,
    /// Clear workspace variables, after confirmation when non-empty.
    ClearWorkspace,
    /// Clear only the command history list.
    ClearHistory,
    /// Close the selected figure.
    CloseFigure,
}

/// One registry record for an application action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionSpec {
    /// Stable machine id used by the control registry.
    pub id: &'static str,
    /// User-visible label. Each clear scope has exactly one.
    pub label: &'static str,
    /// Owning panel.
    pub panel: &'static str,
}

/// Four clear actions, four specs, one label each.
pub const CLEAR_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: "ClearCommandWindow",
        label: "Clear Command Window",
        panel: "Command Window",
    },
    ActionSpec {
        id: "ClearWorkspace",
        label: "Clear Workspace",
        panel: "Workspace",
    },
    ActionSpec {
        id: "ClearHistory",
        label: "Clear History",
        panel: "Command History",
    },
    ActionSpec {
        id: "CloseFigure",
        label: "Close Figure",
        panel: "Figures",
    },
];

impl AppAction {
    /// Registry spec for this action.
    pub fn spec(self) -> &'static ActionSpec {
        match self {
            Self::ClearCommandWindow => &CLEAR_ACTIONS[0],
            Self::ClearWorkspace => &CLEAR_ACTIONS[1],
            Self::ClearHistory => &CLEAR_ACTIONS[2],
            Self::CloseFigure => &CLEAR_ACTIONS[3],
        }
    }

    /// Register the actual egui id of a control that dispatches this action.
    pub fn register_control(self, ctx: &egui::Context, id: egui::Id, rect: egui::Rect) {
        let _ = ui_registry::register(ctx, self.spec().id, id, rect);
    }
}

/// All clear-scope specs, for tests and the control audit.
pub fn clear_actions() -> &'static [ActionSpec] {
    CLEAR_ACTIONS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn clear_actions_have_one_unique_label_per_panel() {
        let mut seen = HashSet::new();
        for action in clear_actions() {
            assert!(seen.insert((action.panel, action.label)));
            assert!(action.label.is_ascii(), "label must be plain ASCII");
        }
        assert_eq!(clear_actions().len(), 4);
    }

    #[test]
    fn every_clear_scope_maps_to_a_distinct_action() {
        let specs = clear_actions();
        assert_eq!(AppAction::ClearCommandWindow.spec(), &specs[0]);
        assert_eq!(AppAction::ClearWorkspace.spec(), &specs[1]);
        assert_eq!(AppAction::ClearHistory.spec(), &specs[2]);
        assert_eq!(AppAction::CloseFigure.spec(), &specs[3]);
    }
}
