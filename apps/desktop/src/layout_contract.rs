//! LAYOUT contract (master prompt v1, Phase L2 — infrastructure).
//!
//! C1–C12 are the desktop layout contract. This module is the pure, headless
//! home of the infrastructure built for Task L2:
//!   - `PANEL_MIN`: the single source of panel minimum sizes (C2).
//!   - `LayoutProbe` + `check_layout`: the rect-ownership probe and invariant checker (C1, C2, C7).
//!   - `PanelClip`: C1 container boundary enforcement.
//!   - `check_mutation`: L-E mutation check detecting perturbations across all quadrants.
//!   - `run_size_sweep` & `generate_capture_report`: L-C size sweep evaluating layout across resolutions.
//!
//! Findings only; no panel layout fixes are made here (fixes belong to L3).

use crate::dock::{DockTree, PanelId};
use egui::{vec2, Pos2, Rect, Ui};

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

/// Overall layout check report for controls and panels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutReport {
    pub control_violations: Vec<LayoutViolation>,
    pub panel_violations: Vec<LayoutViolation>,
}

impl LayoutReport {
    pub fn is_clean(&self) -> bool {
        self.control_violations.is_empty() && self.panel_violations.is_empty()
    }

    pub fn total_violations(&self) -> usize {
        self.control_violations.len() + self.panel_violations.len()
    }
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

/// Run the pure layout checks over a set of control probes (V1).
pub fn check_layout(probes: &[LayoutProbe]) -> Vec<LayoutViolation> {
    let mut out = Vec::new();
    for p in probes {
        out.extend(check_control(p));
    }
    out
}

/// Run layout contract checks over both control probes and panel rects.
pub fn check_layout_full(probes: &[LayoutProbe], panels: &[(PanelId, Rect)]) -> LayoutReport {
    let mut control_violations = Vec::new();
    for p in probes {
        control_violations.extend(check_control(p));
    }
    let mut panel_violations = Vec::new();
    for &(id, rect) in panels {
        panel_violations.extend(check_panel(id, rect));
    }
    LayoutReport {
        control_violations,
        panel_violations,
    }
}

/// L-E mutation check: verify that perturbing a probe rect by delta produces an out-of-clip violation.
pub fn check_mutation(probe: &LayoutProbe, delta: egui::Vec2) -> Vec<LayoutViolation> {
    let mut mutated = probe.clone();
    mutated.rect = mutated.rect.translate(delta);
    check_control(&mutated)
}

/// Evaluation result for one resolution step in an L-C size sweep.
#[derive(Clone, Debug, PartialEq)]
pub struct SweepEvaluation {
    pub resolution: [f32; 2],
    pub panel_count: usize,
    pub report: LayoutReport,
}

/// L-C size sweep: evaluate layout across a sequence of window resolutions against the dock tree.
pub fn run_size_sweep(
    sizes: &[[f32; 2]],
    dock: &DockTree,
    top_bar_height: f32,
    status_bar_height: f32,
) -> Vec<SweepEvaluation> {
    sizes
        .iter()
        .map(|&res| {
            let dock_h = (res[1] - top_bar_height - status_bar_height).max(10.0);
            let dock_rect =
                Rect::from_min_size(Pos2::new(0.0, top_bar_height), vec2(res[0], dock_h));
            let panels = dock.collect_panel_rects(dock_rect);
            let report = check_layout_full(&[], &panels);
            SweepEvaluation {
                resolution: res,
                panel_count: panels.len(),
                report,
            }
        })
        .collect()
}

/// Generate the documented layout report for the three supported capture sizes:
/// 900x600, 1280x800, and 1920x1080.
pub fn generate_capture_report(dock: &DockTree) -> Vec<SweepEvaluation> {
    let supported_sizes = &[[900.0, 600.0], [1280.0, 800.0], [1920.0, 1080.0]];
    // Standard IDE vertical chrome: ~115px top ribbon/toolstrip, ~25px status bar
    run_size_sweep(supported_sizes, dock, 115.0, 25.0)
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

    #[test]
    fn c2_all_panel_min_table_rows_exercised_below_threshold() {
        // Exercise every single row in PANEL_MIN to prove check_panel flags V4
        for &(id, min_w, min_h) in PANEL_MIN {
            // Width below minimum
            let bad_w = Rect::from_min_size(Pos2::ZERO, egui::vec2(min_w - 5.0, min_h + 10.0));
            let v_w = check_panel(id, bad_w);
            assert_eq!(v_w.len(), 1, "panel {id:?} below min_w must produce V4");
            assert_eq!(v_w[0].rule, "V4");

            // Height below minimum
            let bad_h = Rect::from_min_size(Pos2::ZERO, egui::vec2(min_w + 10.0, min_h - 5.0));
            let v_h = check_panel(id, bad_h);
            assert_eq!(v_h.len(), 1, "panel {id:?} below min_h must produce V4");
            assert_eq!(v_h[0].rule, "V4");
        }
    }

    #[test]
    fn c2_all_panel_min_table_rows_exercised_at_or_above_threshold() {
        // Exercise every single row in PANEL_MIN to prove check_panel passes at exact minimum and above
        for &(id, min_w, min_h) in PANEL_MIN {
            let exact = Rect::from_min_size(Pos2::ZERO, egui::vec2(min_w, min_h));
            assert!(
                check_panel(id, exact).is_empty(),
                "panel {id:?} at exact minimum must pass"
            );

            let above = Rect::from_min_size(Pos2::ZERO, egui::vec2(min_w + 50.0, min_h + 50.0));
            assert!(
                check_panel(id, above).is_empty(),
                "panel {id:?} above minimum must pass"
            );
        }
    }

    #[test]
    fn le_mutation_check_detects_all_quadrant_shifts() {
        let clip = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 300.0));
        let probe = LayoutProbe::new(
            PanelId::Editor,
            "target-control",
            Rect::from_min_max(pos2(120.0, 120.0), pos2(280.0, 280.0)),
            clip,
        );

        // Baseline inside clip: clean
        assert!(check_control(&probe).is_empty());

        // Shift right (+x beyond clip.max.x)
        let v_right = check_mutation(&probe, egui::vec2(30.0, 0.0));
        assert_eq!(v_right.len(), 1);
        assert_eq!(v_right[0].rule, "V1");

        // Shift left (-x beyond clip.min.x)
        let v_left = check_mutation(&probe, egui::vec2(-30.0, 0.0));
        assert_eq!(v_left.len(), 1);
        assert_eq!(v_left[0].rule, "V1");

        // Shift down (+y beyond clip.max.y)
        let v_down = check_mutation(&probe, egui::vec2(0.0, 30.0));
        assert_eq!(v_down.len(), 1);
        assert_eq!(v_down[0].rule, "V1");

        // Shift up (-y beyond clip.min.y)
        let v_up = check_mutation(&probe, egui::vec2(0.0, -30.0));
        assert_eq!(v_up.len(), 1);
        assert_eq!(v_up[0].rule, "V1");
    }

    #[test]
    fn lc_size_sweep_exercises_supported_sizes() {
        let dock = DockTree::standard();
        let report = generate_capture_report(&dock);

        assert_eq!(report.len(), 3);
        assert_eq!(report[0].resolution, [900.0, 600.0]);
        assert_eq!(report[1].resolution, [1280.0, 800.0]);
        assert_eq!(report[2].resolution, [1920.0, 1080.0]);

        // At 900x600, standard layout compresses left column (w=179 < 220) and right column (w=200 < 280)
        assert!(
            !report[0].report.is_clean(),
            "900x600 layout must detect panel minimum violations (C2/C12)"
        );
        let v_panels: Vec<_> = report[0]
            .report
            .panel_violations
            .iter()
            .map(|v| v.panel_id)
            .collect();
        assert!(v_panels.contains(&PanelId::CurrentFolder));
        assert!(v_panels.contains(&PanelId::Workspace));
        assert!(v_panels.contains(&PanelId::Help));
        assert!(v_panels.contains(&PanelId::Figures));

        // At 1280x800 and 1920x1080, all standard layout panels exceed their declared minimums
        assert!(
            report[1].report.is_clean(),
            "1280x800 standard preset must meet all declared minimums"
        );
        assert!(
            report[2].report.is_clean(),
            "1920x1080 standard preset must meet all declared minimums"
        );
    }

    #[test]
    fn check_layout_full_aggregates_both_controls_and_panels() {
        let clip = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 200.0));
        let bad_control = LayoutProbe::new(
            PanelId::Editor,
            "overflow-btn",
            Rect::from_min_max(pos2(150.0, 150.0), pos2(250.0, 250.0)),
            clip,
        );
        let bad_panel = (
            PanelId::Workspace,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0)), // 100x100 < 220x140
        );

        let report = check_layout_full(&[bad_control], &[bad_panel]);
        assert_eq!(report.total_violations(), 2);
        assert_eq!(report.control_violations.len(), 1);
        assert_eq!(report.control_violations[0].rule, "V1");
        assert_eq!(report.panel_violations.len(), 1);
        assert_eq!(report.panel_violations[0].rule, "V4");
    }
}
