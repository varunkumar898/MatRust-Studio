//! Real multi-panel docking system with draggable tabs, split regions, presets, and persistence.

use crate::theme::Theme;
use egui::{
    vec2, Align, Color32, CursorIcon, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use serde::{Deserialize, Serialize};

/// Identifiers for all dockable panels in the MatRust Studio IDE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    CurrentFolder,
    Editor,
    Workspace,
    CommandWindow,
    Figures,
    History,
}

impl PanelId {
    pub fn title(&self) -> &'static str {
        match self {
            PanelId::CurrentFolder => "Current Folder",
            PanelId::Editor => "Editor",
            PanelId::Workspace => "Workspace",
            PanelId::CommandWindow => "Command Window",
            PanelId::Figures => "Figures",
            PanelId::History => "Command History",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PanelId::CurrentFolder => "📁",
            PanelId::Editor => "📝",
            PanelId::Workspace => "📊",
            PanelId::CommandWindow => "💻",
            PanelId::Figures => "📈",
            PanelId::History => "🕒",
        }
    }
}

/// Orientation of binary split in the dock tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitOrientation {
    Horizontal, // Left | Right
    Vertical,   // Top / Bottom
}

/// Drop zone target during tab drag-and-drop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZone {
    Left,
    Right,
    Top,
    Bottom,
    Tab,
}

/// Binary dock node. Either a leaf containing tabbed panels or a split node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DockNode {
    Leaf {
        tabs: Vec<PanelId>,
        active_tab: usize,
    },
    Split {
        orientation: SplitOrientation,
        fraction: f32,
        left_or_top: Box<DockNode>,
        right_or_bottom: Box<DockNode>,
    },
}

impl DockNode {
    pub fn leaf(panel: PanelId) -> Self {
        DockNode::Leaf {
            tabs: vec![panel],
            active_tab: 0,
        }
    }

    pub fn leaf_multi(tabs: Vec<PanelId>) -> Self {
        DockNode::Leaf {
            tabs,
            active_tab: 0,
        }
    }

    pub fn split_h(fraction: f32, left: DockNode, right: DockNode) -> Self {
        DockNode::Split {
            orientation: SplitOrientation::Horizontal,
            fraction: fraction.clamp(0.05, 0.95),
            left_or_top: Box::new(left),
            right_or_bottom: Box::new(right),
        }
    }

    pub fn split_v(fraction: f32, top: DockNode, bottom: DockNode) -> Self {
        DockNode::Split {
            orientation: SplitOrientation::Vertical,
            fraction: fraction.clamp(0.05, 0.95),
            left_or_top: Box::new(top),
            right_or_bottom: Box::new(bottom),
        }
    }

    pub fn contains_panel(&self, panel: PanelId) -> bool {
        match self {
            DockNode::Leaf { tabs, .. } => tabs.contains(&panel),
            DockNode::Split {
                left_or_top,
                right_or_bottom,
                ..
            } => left_or_top.contains_panel(panel) || right_or_bottom.contains_panel(panel),
        }
    }

    pub fn remove_panel(&mut self, panel: PanelId) -> bool {
        match self {
            DockNode::Leaf { tabs, active_tab } => {
                if let Some(pos) = tabs.iter().position(|&p| p == panel) {
                    tabs.remove(pos);
                    if *active_tab >= tabs.len() && !tabs.is_empty() {
                        *active_tab = tabs.len() - 1;
                    }
                    true
                } else {
                    false
                }
            }
            DockNode::Split {
                left_or_top,
                right_or_bottom,
                ..
            } => left_or_top.remove_panel(panel) || right_or_bottom.remove_panel(panel),
        }
    }

    pub fn prune_empty(&mut self) -> bool {
        match self {
            DockNode::Leaf { tabs, .. } => tabs.is_empty(),
            DockNode::Split {
                left_or_top,
                right_or_bottom,
                ..
            } => {
                let left_empty = left_or_top.prune_empty();
                let right_empty = right_or_bottom.prune_empty();

                if left_empty && right_empty {
                    true
                } else if left_empty {
                    *self = *right_or_bottom.clone();
                    false
                } else if right_empty {
                    *self = *left_or_top.clone();
                    false
                } else {
                    false
                }
            }
        }
    }
}

/// Dock tree holding the root layout and transient drag-drop interaction state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockTree {
    pub root: DockNode,
    #[serde(skip)]
    pub dragged_tab: Option<PanelId>,
    #[serde(skip)]
    pub drop_target: Option<(usize, DropZone)>,
    #[serde(skip)]
    pub next_leaf_id: usize,
}

impl Default for DockTree {
    fn default() -> Self {
        Self::standard()
    }
}

impl DockTree {
    /// Preset: Standard balanced scientific layout.
    pub fn standard() -> Self {
        // Left column: Current Folder (top) / History (bottom)
        let left = DockNode::split_v(
            0.6,
            DockNode::leaf(PanelId::CurrentFolder),
            DockNode::leaf(PanelId::History),
        );

        // Center column: Editor (top) / Command Window (bottom)
        let center = DockNode::split_v(
            0.65,
            DockNode::leaf(PanelId::Editor),
            DockNode::leaf(PanelId::CommandWindow),
        );

        // Right column: Workspace (top) / Figures (bottom)
        let right = DockNode::split_v(
            0.5,
            DockNode::leaf(PanelId::Workspace),
            DockNode::leaf(PanelId::Figures),
        );

        let center_right = DockNode::split_h(0.72, center, right);
        let root = DockNode::split_h(0.18, left, center_right);

        Self {
            root,
            dragged_tab: None,
            drop_target: None,
            next_leaf_id: 0,
        }
    }

    /// Preset: Maximized Editor focus layout.
    pub fn editor_focus() -> Self {
        let left = DockNode::leaf(PanelId::CurrentFolder);
        let center = DockNode::split_v(
            0.82,
            DockNode::leaf(PanelId::Editor),
            DockNode::leaf_multi(vec![PanelId::CommandWindow, PanelId::History]),
        );
        let right = DockNode::leaf_multi(vec![PanelId::Workspace, PanelId::Figures]);

        let center_right = DockNode::split_h(0.8, center, right);
        let root = DockNode::split_h(0.15, left, center_right);

        Self {
            root,
            dragged_tab: None,
            drop_target: None,
            next_leaf_id: 0,
        }
    }

    /// Preset: Data Analysis layout focusing on Workspace & Tables.
    pub fn data_analysis() -> Self {
        let left = DockNode::split_v(
            0.5,
            DockNode::leaf(PanelId::CurrentFolder),
            DockNode::leaf(PanelId::Workspace),
        );
        let center = DockNode::split_v(
            0.5,
            DockNode::leaf(PanelId::Editor),
            DockNode::leaf(PanelId::CommandWindow),
        );
        let right = DockNode::leaf(PanelId::Figures);

        let center_right = DockNode::split_h(0.65, center, right);
        let root = DockNode::split_h(0.25, left, center_right);

        Self {
            root,
            dragged_tab: None,
            drop_target: None,
            next_leaf_id: 0,
        }
    }

    /// Preset: Plotting layout emphasizing Figures and visualization.
    pub fn plotting() -> Self {
        let left = DockNode::split_v(
            0.5,
            DockNode::leaf(PanelId::Editor),
            DockNode::leaf(PanelId::CommandWindow),
        );
        let right = DockNode::split_v(
            0.7,
            DockNode::leaf(PanelId::Figures),
            DockNode::leaf_multi(vec![PanelId::Workspace, PanelId::CurrentFolder]),
        );

        let root = DockNode::split_h(0.35, left, right);

        Self {
            root,
            dragged_tab: None,
            drop_target: None,
            next_leaf_id: 0,
        }
    }

    /// Checks if a panel is currently present in the tree.
    pub fn is_open(&self, panel: PanelId) -> bool {
        self.root.contains_panel(panel)
    }

    /// Opens a panel if not present, or brings its tab to focus if already present.
    pub fn open_panel(&mut self, panel: PanelId) {
        if self.is_open(panel) {
            self.focus_panel(panel);
            return;
        }

        // Insert into most appropriate leaf
        let target_leaf = match panel {
            PanelId::Figures | PanelId::Workspace => PanelId::Workspace,
            PanelId::CommandWindow | PanelId::History => PanelId::CommandWindow,
            PanelId::CurrentFolder => PanelId::CurrentFolder,
            PanelId::Editor => PanelId::Editor,
        };

        if !self.insert_tab_next_to(target_leaf, panel) {
            // Append as tab to the first available leaf
            self.append_to_first_leaf(panel);
        }
    }

    pub fn focus_panel(&mut self, panel: PanelId) {
        fn rec(node: &mut DockNode, panel: PanelId) -> bool {
            match node {
                DockNode::Leaf { tabs, active_tab } => {
                    if let Some(pos) = tabs.iter().position(|&p| p == panel) {
                        *active_tab = pos;
                        true
                    } else {
                        false
                    }
                }
                DockNode::Split {
                    left_or_top,
                    right_or_bottom,
                    ..
                } => rec(left_or_top, panel) || rec(right_or_bottom, panel),
            }
        }
        rec(&mut self.root, panel);
    }

    fn insert_tab_next_to(&mut self, neighbor: PanelId, panel: PanelId) -> bool {
        fn rec(node: &mut DockNode, neighbor: PanelId, panel: PanelId) -> bool {
            match node {
                DockNode::Leaf { tabs, active_tab } => {
                    if tabs.contains(&neighbor) {
                        tabs.push(panel);
                        *active_tab = tabs.len() - 1;
                        true
                    } else {
                        false
                    }
                }
                DockNode::Split {
                    left_or_top,
                    right_or_bottom,
                    ..
                } => rec(left_or_top, neighbor, panel) || rec(right_or_bottom, neighbor, panel),
            }
        }
        rec(&mut self.root, neighbor, panel)
    }

    fn append_to_first_leaf(&mut self, panel: PanelId) {
        fn rec(node: &mut DockNode, panel: PanelId) -> bool {
            match node {
                DockNode::Leaf { tabs, active_tab } => {
                    tabs.push(panel);
                    *active_tab = tabs.len() - 1;
                    true
                }
                DockNode::Split { left_or_top, .. } => rec(left_or_top, panel),
            }
        }
        rec(&mut self.root, panel);
    }

    /// Serializes dock tree layout to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes dock tree layout from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Closes a specific tab in a leaf node.
    pub fn close_tab(&mut self, target_leaf_idx: usize, tab_idx: usize) {
        fn rec(node: &mut DockNode, target_idx: usize, current_idx: &mut usize, tab_idx: usize) {
            match node {
                DockNode::Leaf { tabs, active_tab } => {
                    if *current_idx == target_idx {
                        if tab_idx < tabs.len() {
                            tabs.remove(tab_idx);
                            if *active_tab >= tabs.len() && !tabs.is_empty() {
                                *active_tab = tabs.len() - 1;
                            }
                        }
                    } else {
                        *current_idx += 1;
                    }
                }
                DockNode::Split {
                    left_or_top,
                    right_or_bottom,
                    ..
                } => {
                    rec(left_or_top, target_idx, current_idx, tab_idx);
                    rec(right_or_bottom, target_idx, current_idx, tab_idx);
                }
            }
        }
        let mut idx = 0;
        rec(&mut self.root, target_leaf_idx, &mut idx, tab_idx);
    }

    pub fn apply_drop(&mut self, target_leaf_idx: usize, zone: DropZone, panel: PanelId) {
        fn rec(
            node: &mut DockNode,
            target_idx: usize,
            current_idx: &mut usize,
            zone: DropZone,
            panel: PanelId,
        ) -> bool {
            match node {
                DockNode::Leaf { tabs, active_tab } => {
                    if *current_idx == target_idx {
                        match zone {
                            DropZone::Tab => {
                                tabs.push(panel);
                                *active_tab = tabs.len() - 1;
                            }
                            DropZone::Left => {
                                let old_leaf = node.clone();
                                *node = DockNode::split_h(0.35, DockNode::leaf(panel), old_leaf);
                            }
                            DropZone::Right => {
                                let old_leaf = node.clone();
                                *node = DockNode::split_h(0.65, old_leaf, DockNode::leaf(panel));
                            }
                            DropZone::Top => {
                                let old_leaf = node.clone();
                                *node = DockNode::split_v(0.35, DockNode::leaf(panel), old_leaf);
                            }
                            DropZone::Bottom => {
                                let old_leaf = node.clone();
                                *node = DockNode::split_v(0.65, old_leaf, DockNode::leaf(panel));
                            }
                        }
                        true
                    } else {
                        *current_idx += 1;
                        false
                    }
                }
                DockNode::Split {
                    left_or_top,
                    right_or_bottom,
                    ..
                } => {
                    if rec(left_or_top, target_idx, current_idx, zone, panel) {
                        true
                    } else {
                        rec(right_or_bottom, target_idx, current_idx, zone, panel)
                    }
                }
            }
        }

        let mut idx = 0;
        rec(&mut self.root, target_leaf_idx, &mut idx, zone, panel);
    }

    pub fn render<F>(&mut self, ui: &mut Ui, theme: &Theme, mut render_panel: F)
    where
        F: FnMut(&mut Ui, PanelId),
    {
        let available_rect = ui.available_rect_before_wrap();
        let mut leaf_counter = 0;
        let mut new_drop_target = None;
        let mut tab_closed = None;

        Self::render_node(
            &mut self.root,
            ui,
            theme,
            available_rect,
            &mut leaf_counter,
            self.dragged_tab,
            &mut new_drop_target,
            &mut self.dragged_tab,
            &mut tab_closed,
            &mut render_panel,
        );

        if let Some((leaf_idx, tab_idx)) = tab_closed {
            self.close_tab(leaf_idx, tab_idx);
            self.root.prune_empty();
        }

        self.drop_target = new_drop_target;

        // Pointer release: commit drag and drop
        if ui.input(|i| i.pointer.any_released()) {
            if let Some(dragged) = self.dragged_tab.take() {
                if let Some((target_leaf, zone)) = self.drop_target.take() {
                    self.root.remove_panel(dragged);
                    self.apply_drop(target_leaf, zone, dragged);
                    self.root.prune_empty();
                }
            }
        }

        // Draw floating drag preview chip
        if let Some(panel) = self.dragged_tab {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let chip_rect = Rect::from_min_size(pos + vec2(12.0, 12.0), vec2(130.0, 26.0));
                let painter = ui.painter();
                painter.rect_filled(chip_rect, 4.0, theme.panel_bg);
                painter.rect_stroke(
                    chip_rect,
                    4.0,
                    Stroke::new(1.5_f32, theme.accent),
                    StrokeKind::Inside,
                );
                painter.text(
                    chip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {}", panel.icon(), panel.title()),
                    egui::FontId::proportional(12.0),
                    theme.text_primary,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_node<F>(
        node: &mut DockNode,
        ui: &mut Ui,
        theme: &Theme,
        rect: Rect,
        leaf_counter: &mut usize,
        dragged_tab: Option<PanelId>,
        drop_target: &mut Option<(usize, DropZone)>,
        dragged_tab_out: &mut Option<PanelId>,
        tab_closed: &mut Option<(usize, usize)>,
        render_panel: &mut F,
    ) where
        F: FnMut(&mut Ui, PanelId),
    {
        if rect.width() <= 1.0 || rect.height() <= 1.0 {
            return;
        }

        match node {
            DockNode::Split {
                orientation,
                fraction,
                left_or_top,
                right_or_bottom,
            } => {
                let divider_size = 5.0_f32;
                match orientation {
                    SplitOrientation::Horizontal => {
                        let total_w = (rect.width() - divider_size).max(10.0);
                        let w_left = (total_w * *fraction).round();
                        let w_right = total_w - w_left;

                        let r_left = Rect::from_min_size(rect.min, vec2(w_left, rect.height()));
                        let r_div = Rect::from_min_size(
                            Pos2::new(rect.min.x + w_left, rect.min.y),
                            vec2(divider_size, rect.height()),
                        );
                        let r_right = Rect::from_min_size(
                            Pos2::new(rect.min.x + w_left + divider_size, rect.min.y),
                            vec2(w_right, rect.height()),
                        );

                        // Interactive divider
                        let div_id = ui.id().with(format!("div_h_{}", *leaf_counter));
                        let resp = ui.interact(r_div, div_id, Sense::click_and_drag());
                        if resp.hovered() || resp.dragged() {
                            ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                        }
                        if resp.dragged() {
                            let delta = resp.drag_delta().x / total_w;
                            *fraction = (*fraction + delta).clamp(0.05, 0.95);
                        }
                        ui.painter().rect_filled(r_div, 0.0, theme.border);

                        Self::render_node(
                            left_or_top,
                            ui,
                            theme,
                            r_left,
                            leaf_counter,
                            dragged_tab,
                            drop_target,
                            dragged_tab_out,
                            tab_closed,
                            render_panel,
                        );
                        Self::render_node(
                            right_or_bottom,
                            ui,
                            theme,
                            r_right,
                            leaf_counter,
                            dragged_tab,
                            drop_target,
                            dragged_tab_out,
                            tab_closed,
                            render_panel,
                        );
                    }
                    SplitOrientation::Vertical => {
                        let total_h = (rect.height() - divider_size).max(10.0);
                        let h_top = (total_h * *fraction).round();
                        let h_bottom = total_h - h_top;

                        let r_top = Rect::from_min_size(rect.min, vec2(rect.width(), h_top));
                        let r_div = Rect::from_min_size(
                            Pos2::new(rect.min.x, rect.min.y + h_top),
                            vec2(rect.width(), divider_size),
                        );
                        let r_bottom = Rect::from_min_size(
                            Pos2::new(rect.min.x, rect.min.y + h_top + divider_size),
                            vec2(rect.width(), h_bottom),
                        );

                        // Interactive divider
                        let div_id = ui.id().with(format!("div_v_{}", *leaf_counter));
                        let resp = ui.interact(r_div, div_id, Sense::click_and_drag());
                        if resp.hovered() || resp.dragged() {
                            ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
                        }
                        if resp.dragged() {
                            let delta = resp.drag_delta().y / total_h;
                            *fraction = (*fraction + delta).clamp(0.05, 0.95);
                        }
                        ui.painter().rect_filled(r_div, 0.0, theme.border);

                        Self::render_node(
                            left_or_top,
                            ui,
                            theme,
                            r_top,
                            leaf_counter,
                            dragged_tab,
                            drop_target,
                            dragged_tab_out,
                            tab_closed,
                            render_panel,
                        );
                        Self::render_node(
                            right_or_bottom,
                            ui,
                            theme,
                            r_bottom,
                            leaf_counter,
                            dragged_tab,
                            drop_target,
                            dragged_tab_out,
                            tab_closed,
                            render_panel,
                        );
                    }
                }
            }
            DockNode::Leaf { tabs, active_tab } => {
                let my_leaf_idx = *leaf_counter;
                *leaf_counter += 1;

                let tab_bar_height = 28.0_f32;
                let tab_bar_rect =
                    Rect::from_min_size(rect.min, vec2(rect.width(), tab_bar_height));
                let body_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.min.y + tab_bar_height),
                    vec2(rect.width(), (rect.height() - tab_bar_height).max(0.0)),
                );

                // Tab Bar Background & Border
                ui.painter().rect_filled(tab_bar_rect, 0.0, theme.panel_bg);
                ui.painter().line_segment(
                    [
                        Pos2::new(tab_bar_rect.min.x, tab_bar_rect.max.y),
                        Pos2::new(tab_bar_rect.max.x, tab_bar_rect.max.y),
                    ],
                    Stroke::new(1.0_f32, theme.border),
                );

                // Render tabs
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::default()
                        .max_rect(tab_bar_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );

                for (idx, &tab) in tabs.iter().enumerate() {
                    let is_active = idx == *active_tab;
                    let tab_label = format!("{} {}", tab.icon(), tab.title());

                    let resp = child_ui.selectable_label(
                        is_active,
                        RichText::new(tab_label).strong().color(if is_active {
                            theme.accent
                        } else {
                            theme.text_primary
                        }),
                    );

                    if resp.clicked() {
                        *active_tab = idx;
                    }

                    // Drag tab to undock / move
                    if resp.drag_started() {
                        *dragged_tab_out = Some(tab);
                    }

                    // Close button on active tab
                    if is_active
                        && tabs.len() > 1
                        && child_ui
                            .small_button("✕")
                            .on_hover_text("Close tab")
                            .clicked()
                    {
                        *tab_closed = Some((my_leaf_idx, idx));
                    }
                }

                // Check drop zone if dragging a tab over this leaf
                if let Some(_dragged) = dragged_tab {
                    if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                        if rect.contains(ptr) {
                            let rel_x = (ptr.x - rect.min.x) / rect.width();
                            let rel_y = (ptr.y - rect.min.y) / rect.height();

                            let zone = if rel_y < 0.25 {
                                DropZone::Top
                            } else if rel_y > 0.75 {
                                DropZone::Bottom
                            } else if rel_x < 0.25 {
                                DropZone::Left
                            } else if rel_x > 0.75 {
                                DropZone::Right
                            } else {
                                DropZone::Tab
                            };

                            *drop_target = Some((my_leaf_idx, zone));

                            // Draw visual drop zone highlight
                            let highlight_rect = match zone {
                                DropZone::Left => Rect::from_min_size(
                                    rect.min,
                                    vec2(rect.width() * 0.4, rect.height()),
                                ),
                                DropZone::Right => Rect::from_min_size(
                                    Pos2::new(rect.max.x - rect.width() * 0.4, rect.min.y),
                                    vec2(rect.width() * 0.4, rect.height()),
                                ),
                                DropZone::Top => Rect::from_min_size(
                                    rect.min,
                                    vec2(rect.width(), rect.height() * 0.4),
                                ),
                                DropZone::Bottom => Rect::from_min_size(
                                    Pos2::new(rect.min.x, rect.max.y - rect.height() * 0.4),
                                    vec2(rect.width(), rect.height() * 0.4),
                                ),
                                DropZone::Tab => body_rect,
                            };

                            ui.painter().rect_filled(
                                highlight_rect,
                                2.0,
                                Color32::from_rgba_unmultiplied(0, 114, 189, 45),
                            );
                            ui.painter().rect_stroke(
                                highlight_rect,
                                2.0,
                                Stroke::new(2.0_f32, theme.accent),
                                StrokeKind::Inside,
                            );
                        }
                    }
                }

                // Render Active Panel Body
                if let Some(&active_panel_id) = tabs.get(*active_tab) {
                    let mut body_ui = ui.new_child(
                        egui::UiBuilder::default()
                            .max_rect(body_rect)
                            .layout(Layout::top_down(Align::LEFT)),
                    );
                    render_panel(&mut body_ui, active_panel_id);
                } else if tabs.is_empty() {
                    ui.painter().text(
                        body_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Empty Panel (Open panels via View menu)",
                        egui::FontId::proportional(13.0),
                        theme.text_secondary,
                    );
                }
            }
        }
    }
}
