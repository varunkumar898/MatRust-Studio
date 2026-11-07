//! LAYOUT contract (master prompt v1, Phase L2 — infrastructure).
//!
//! C1–C12 are the desktop layout contract. This module is the pure, headless
//! home of the two pieces L2 builds first:
//!   - `PANEL_MIN`: the single source of panel minimum sizes (C2).
//!   - `LayoutProbe` + `check_layout`: the rect-ownership probe and the pure
//!     invariant checker (C1, C2, C7).
//!
//! No panel is fixed here; panels are wired to these in later L2/L3 steps.

use crate::dock::PanelId;
use egui::{Rect, Ui};

/// Minimum panel sizes from the layout contract (C2), one row per panel.
pub const PANEL_MIN: &[(PanelId, f32, f32)] = &[
    (PanelId::CurrentFolder, 220.0, 160.0),
    (PanelId::Workspace, 220.0, 140.0),
    (PanelId::Editor, 360.0, 240.0),
    (PanelId::BlockDiagram, 400.0, 300.0),
    (PanelId::CommandWindow, 360.0, 160.0),
    (PanelId::Help, 280.0, 200.0),
    (PanelId::Figures, 280.0, 200.0),
];

/// Look up the minimum size for one panel. Panels not in the table have no
/// declared minimum yet and return `(0.0, 0.0)`.
pub fn panel_min(panel_id: PanelId) -> (f32, f32) {
    PANEL_MIN
        .iter()
        .find(|(id, _, _)| *id == panel_id)
        .map(|(_, w, h)| (*w, *h))
        .unwrap_or((0.0, 0.0))
}

/// C1 container: use the panel's own body rect as the authoritative clip.
///
/// Call this at the top of a panel body, then use [`PanelClip::probe`] for
/// child controls. It does not draw or allocate anything; it only intersects
/// egui's clip with the panel body so a child cannot paint into a neighbour.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelClip {
    pub panel_id: PanelId,
    pub panel_rect: Rect,
    pub clip_rect: Rect,
}

impl PanelClip {
    pub fn new(ui: &mut Ui, panel_id: PanelId) -> Self {
        let panel_rect = ui.max_rect();
        let clip_rect = ui.clip_rect().intersect(panel_rect);
        ui.set_clip_rect(clip_rect);
        Self {
            panel_id,
            panel_rect,
            clip_rect,
        }
    }

    /// Build a probe for a child control using this panel's authoritative clip.
    pub fn probe(&self, control_id: &str, rect: Rect) -> LayoutProbe {
        LayoutProbe::new(self.panel_id, control_id, rect, self.clip_rect)
    }

    /// C2 check for the panel body itself.
    pub fn check_panel(&self) -> Vec<LayoutViolation> {
        check_panel(self.panel_id, self.panel_rect)
    }
}

/// A rect reported by a panel or control for the layout invariant checker.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutProbe {
    pub panel_id: PanelId,
    pub control_id: String,
    pub rect: Rect,
    pub clip_rect: Rect,
}

impl LayoutProbe {
    pub fn new(panel_id: PanelId, control_id: &str, rect: Rect, clip_rect: Rect) -> Self {
        Self {
            panel_id,
            control_id: control_id.to_string(),
            rect,
            clip_rect,
        }
    }
}

/// A layout invariant violation (rule id + location).
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutViolation {
    pub rule: String,
    pub panel_id: PanelId,
    pub control_id: String,
}

/// V1 (C1): a control must paint inside its clip rect.
pub fn check_control(probe: &LayoutProbe) -> Vec<LayoutViolation> {
    let inside = probe.rect.min.x >= probe.clip_rect.min.x
        && probe.rect.min.y >= probe.clip_rect.min.y
        && probe.rect.max.x <= probe.clip_rect.max.x
        && probe.rect.max.y <= probe.clip_rect.max.y;
    if inside {
        Vec::new()
    } else {
        vec![LayoutViolation {
            rule: "V1".to_string(),
            panel_id: probe.panel_id,
            control_id: probe.control_id.clone(),
        }]
    }
}

/// V4 (C2): a panel rect must not be smaller than its declared minimum.
pub fn check_panel(panel_id: PanelId, rect: Rect) -> Vec<LayoutViolation> {
    let (min_w, min_h) = panel_min(panel_id);
    let too_small = rect.width() < min_w - 0.5 || rect.height() < min_h - 0.5;
    if too_small {
        vec![LayoutViolation {
            rule: "V4".to_string(),
            panel_id,
            control_id: panel_id.title().to_string(),
        }]
    } else {
        Vec::new()
    }
}

/// Run the pure layout checks over a set of probes (V1) plus panel minimums.
pub fn check_layout(probes: &[LayoutProbe]) -> Vec<LayoutViolation> {
    let mut out = Vec::new();
    for p in probes {
        out.extend(check_control(p));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, Context, Pos2, RawInput};

    fn make_probe(panel: PanelId, control: &str, rect: Rect, clip: Rect) -> LayoutProbe {
        LayoutProbe::new(panel, control, rect, clip)
    }

    #[test]
    fn v1_passes_when_control_inside_clip() {
        let p = make_probe(
            PanelId::Editor,
            "run-button",
            Rect::from_min_max(pos2(10.0, 10.0), pos2(80.0, 40.0)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 200.0)),
        );
        assert!(check_control(&p).is_empty());
    }

    #[test]
    fn v1_fails_when_control_outside_clip() {
        let p = make_probe(
            PanelId::Editor,
            "run-button",
            Rect::from_min_max(pos2(150.0, 150.0), pos2(300.0, 300.0)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 200.0)),
        );
        let v = check_control(&p);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "V1");
        assert_eq!(v[0].control_id, "run-button");
    }

    #[test]
    fn check_layout_reports_only_out_of_clip_probes() {
        let ok = make_probe(
            PanelId::Figures,
            "canvas",
            Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 100.0)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 100.0)),
        );
        assert!(check_layout(&[ok]).is_empty());
    }

    #[test]
    fn v4_panel_min_sizes_are_positive() {
        assert!(PANEL_MIN.len() >= 7);
        for (id, w, h) in PANEL_MIN {
            assert!(*w > 0.0, "panel {:?} min width must be positive", id);
            assert!(*h > 0.0, "panel {:?} min height must be positive", id);
        }
    }

    #[test]
    fn v4_editor_below_minimum_is_flagged() {
        let bad = Rect::from_min_max(pos2(0.0, 0.0), Pos2::new(200.0, 100.0)); // 200x100 < 360x240
        let v = check_panel(PanelId::Editor, bad);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "V4");
    }

    #[test]
    fn v4_editor_at_minimum_is_clean() {
        let good = Rect::from_min_max(pos2(0.0, 0.0), Pos2::new(360.0, 240.0));
        assert!(check_panel(PanelId::Editor, good).is_empty());
    }

    #[test]
    fn c1_panel_clip_owns_child_probes() {
        let ctx = Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 300.0));
        let raw = RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let _ = ctx.run(raw, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let clip = PanelClip::new(ui, PanelId::Editor);
                    assert!(clip.check_panel().is_empty());
                    assert_eq!(clip.panel_rect, clip.panel_rect.intersect(clip.clip_rect));

                    let inside = clip.probe("inside", clip.clip_rect.shrink(1.0));
                    assert!(check_control(&inside).is_empty());

                    let outside = clip.probe(
                        "mutated-outside",
                        Rect::from_min_max(
                            pos2(clip.clip_rect.max.x + 1.0, clip.clip_rect.min.y),
                            pos2(clip.clip_rect.max.x + 30.0, clip.clip_rect.min.y + 20.0),
                        ),
                    );
                    let violations = check_control(&outside);
                    assert_eq!(violations.len(), 1);
                    assert_eq!(violations[0].rule, "V1");
                    assert_eq!(violations[0].control_id, "mutated-outside");
                });
        });
    }

    #[test]
    fn c2_panel_minimum_lookup_is_exact() {
        assert_eq!(panel_min(PanelId::Editor), (360.0, 240.0));
        assert_eq!(panel_min(PanelId::CommandWindow), (360.0, 160.0));
        // Panels without a declared minimum are explicitly zero, not guessed.
        assert_eq!(panel_min(PanelId::History), (0.0, 0.0));
    }

    #[test]
    fn c2_minimum_table_has_no_duplicate_panels() {
        let mut seen = std::collections::HashSet::new();
        for (id, _, _) in PANEL_MIN {
            assert!(seen.insert(*id), "duplicate minimum for {id:?}");
        }
    }
}
