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

/// Canonical snapshot record for a single active panel in the dock layout.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanelSnapshot {
    pub panel_id: String,
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
    pub min_width_required: f32,
    pub min_height_required: f32,
}

/// Canonical snapshot record for a tested window resolution.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResolutionSnapshot {
    pub resolution: [f32; 2],
    pub dock_rect: [f32; 4], // [min_x, min_y, width, height]
    pub panels: Vec<PanelSnapshot>,
}

/// Canonical layout snapshot manifest stored on disk as a regression gate (Wave 0 Task L4).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutSnapshotManifest {
    pub version: u32,
    pub schema: String,
    pub approved_reason: String,
    pub resolutions: Vec<ResolutionSnapshot>,
}

/// Builds the canonical layout snapshot manifest from the active dock tree across supported resolutions:
/// 900x600, 1280x800, and 1920x1080.
pub fn build_current_layout_snapshot(dock: &DockTree) -> LayoutSnapshotManifest {
    let supported_sizes: &[[f32; 2]] = &[[900.0, 600.0], [1280.0, 800.0], [1920.0, 1080.0]];
    let top_bar_height = 115.0f32;
    let status_bar_height = 25.0f32;

    let mut resolutions = Vec::new();
    for &res in supported_sizes {
        let dock_h: f32 = (res[1] - top_bar_height - status_bar_height).max(10.0);
        let dock_rect = Rect::from_min_size(Pos2::new(0.0, top_bar_height), vec2(res[0], dock_h));
        let panels_raw = dock.collect_panel_rects(dock_rect);

        let mut panels = Vec::new();
        for (id, rect) in panels_raw {
            let (req_w, req_h) = panel_min(id);
            panels.push(PanelSnapshot {
                panel_id: format!("{:?}", id),
                min_x: rect.min.x,
                min_y: rect.min.y,
                width: rect.width(),
                height: rect.height(),
                min_width_required: req_w,
                min_height_required: req_h,
            });
        }

        resolutions.push(ResolutionSnapshot {
            resolution: res,
            dock_rect: [
                dock_rect.min.x,
                dock_rect.min.y,
                dock_rect.width(),
                dock_rect.height(),
            ],
            panels,
        });
    }

    LayoutSnapshotManifest {
        version: 1,
        schema: "matrust-layout-snapshot-v1".to_string(),
        approved_reason:
            "Wave 0 Task L4 canonical baseline for supported sizes (900x600, 1280x800, 1920x1080)"
                .to_string(),
        resolutions,
    }
}

/// Verifies that all panel records in the snapshot satisfy per-panel minimum floors, container
/// bounds, and pairwise non-overlapping geometry.
pub fn verify_snapshot_integrity(snapshot: &LayoutSnapshotManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if snapshot.resolutions.is_empty() {
        errors.push("Snapshot contains zero resolutions".to_string());
        return Err(errors);
    }

    for res in &snapshot.resolutions {
        if res.panels.is_empty() {
            errors.push(format!(
                "Resolution {:?} contains zero panels",
                res.resolution
            ));
            continue;
        }

        let [dock_min_x, dock_min_y, dock_w, dock_h] = res.dock_rect;
        let dock_max_x = dock_min_x + dock_w;
        let dock_max_y = dock_min_y + dock_h;

        // 1. Verify per-panel minimum rect floors and container bounds
        for p in &res.panels {
            if p.width < p.min_width_required - 0.5 {
                errors.push(format!(
                    "Panel {} width {:.1} is below minimum {:.1} at {:?}",
                    p.panel_id, p.width, p.min_width_required, res.resolution
                ));
            }
            if p.height < p.min_height_required - 0.5 {
                errors.push(format!(
                    "Panel {} height {:.1} is below minimum {:.1} at {:?}",
                    p.panel_id, p.height, p.min_height_required, res.resolution
                ));
            }

            let max_x = p.min_x + p.width;
            let max_y = p.min_y + p.height;
            if p.min_x < dock_min_x - 0.5
                || max_x > dock_max_x + 0.5
                || p.min_y < dock_min_y - 0.5
                || max_y > dock_max_y + 0.5
            {
                errors.push(format!(
                    "Panel {} rect [min=({:.1}, {:.1}), max=({:.1}, {:.1})] clipped outside dock container [min=({:.1}, {:.1}), max=({:.1}, {:.1})] at {:?}",
                    p.panel_id, p.min_x, p.min_y, max_x, max_y, dock_min_x, dock_min_y, dock_max_x, dock_max_y, res.resolution
                ));
            }
        }

        // 2. Verify no two panels overlap
        for i in 0..res.panels.len() {
            for j in (i + 1)..res.panels.len() {
                let a = &res.panels[i];
                let b = &res.panels[j];

                let a_max_x = a.min_x + a.width;
                let a_max_y = a.min_y + a.height;
                let b_max_x = b.min_x + b.width;
                let b_max_y = b.min_y + b.height;

                let overlap_x = (a_max_x.min(b_max_x) - a.min_x.max(b.min_x)).max(0.0);
                let overlap_y = (a_max_y.min(b_max_y) - a.min_y.max(b.min_y)).max(0.0);

                if overlap_x > 0.5 && overlap_y > 0.5 {
                    errors.push(format!(
                        "Panels {} and {} overlap by {:.1}x{:.1} at {:?}",
                        a.panel_id, b.panel_id, overlap_x, overlap_y, res.resolution
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Compares actual live snapshot against expected canonical snapshot within numerical tolerance.
pub fn compare_snapshots(
    actual: &LayoutSnapshotManifest,
    expected: &LayoutSnapshotManifest,
    tolerance: f32,
) -> Result<(), Vec<String>> {
    let mut diffs = Vec::new();

    if actual.resolutions.len() != expected.resolutions.len() {
        diffs.push(format!(
            "Resolution count mismatch: actual {} vs expected {}",
            actual.resolutions.len(),
            expected.resolutions.len()
        ));
        return Err(diffs);
    }

    for (act_res, exp_res) in actual.resolutions.iter().zip(expected.resolutions.iter()) {
        if act_res.resolution != exp_res.resolution {
            diffs.push(format!(
                "Resolution mismatch: actual {:?} vs expected {:?}",
                act_res.resolution, exp_res.resolution
            ));
            continue;
        }

        if act_res.panels.len() != exp_res.panels.len() {
            diffs.push(format!(
                "Panel count mismatch at {:?}: actual {} vs expected {}",
                act_res.resolution,
                act_res.panels.len(),
                exp_res.panels.len()
            ));
            continue;
        }

        for (act_p, exp_p) in act_res.panels.iter().zip(exp_res.panels.iter()) {
            if act_p.panel_id != exp_p.panel_id {
                diffs.push(format!(
                    "Panel ID sequence mismatch at {:?}: actual {} vs expected {}",
                    act_res.resolution, act_p.panel_id, exp_p.panel_id
                ));
                continue;
            }

            let dx = (act_p.min_x - exp_p.min_x).abs();
            let dy = (act_p.min_y - exp_p.min_y).abs();
            let dw = (act_p.width - exp_p.width).abs();
            let dh = (act_p.height - exp_p.height).abs();

            if dx > tolerance || dy > tolerance || dw > tolerance || dh > tolerance {
                diffs.push(format!(
                    "Panel {} rect drift at {:?}: actual [x={:.1}, y={:.1}, w={:.1}, h={:.1}] vs expected [x={:.1}, y={:.1}, w={:.1}, h={:.1}] (diffs: dx={:.2}, dy={:.2}, dw={:.2}, dh={:.2})",
                    act_p.panel_id,
                    act_res.resolution,
                    act_p.min_x, act_p.min_y, act_p.width, act_p.height,
                    exp_p.min_x, exp_p.min_y, exp_p.width, exp_p.height,
                    dx, dy, dw, dh
                ));
            }
        }
    }

    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs)
    }
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

        // Across all 3 supported resolutions, responsive split clamping satisfies all panel minimums
        assert!(
            report[0].report.is_clean(),
            "900x600 standard layout must meet all declared minimums under responsive split clamping (got {:?})",
            report[0].report.panel_violations
        );
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
    fn three_size_geometry_panels_satisfy_bounds_and_no_overlaps() {
        let dock = DockTree::standard();
        let supported_sizes: &[[f32; 2]] = &[[900.0, 600.0], [1280.0, 800.0], [1920.0, 1080.0]];

        for &res in supported_sizes {
            let dock_h: f32 = (res[1] - 115.0 - 25.0).max(10.0);
            let dock_rect = Rect::from_min_size(Pos2::new(0.0, 115.0), egui::vec2(res[0], dock_h));
            let panels = dock.collect_panel_rects(dock_rect);

            assert!(
                !panels.is_empty(),
                "Dock must have active panels at {:?}",
                res
            );

            // 1. Each panel satisfies its declared minimum size
            for &(panel_id, rect) in &panels {
                let (min_w, min_h) = panel_min(panel_id);
                assert!(
                    rect.width() >= min_w - 0.5,
                    "Panel {:?} width {:.1} < min {:.1} at {:?}",
                    panel_id,
                    rect.width(),
                    min_w,
                    res
                );
                assert!(
                    rect.height() >= min_h - 0.5,
                    "Panel {:?} height {:.1} < min {:.1} at {:?}",
                    panel_id,
                    rect.height(),
                    min_h,
                    res
                );

                // 2. Panel is fully within container dock bounds
                assert!(
                    rect.min.x >= dock_rect.min.x - 0.5
                        && rect.max.x <= dock_rect.max.x + 0.5
                        && rect.min.y >= dock_rect.min.y - 0.5
                        && rect.max.y <= dock_rect.max.y + 0.5,
                    "Panel {:?} rect {:?} clipped outside container {:?}",
                    panel_id,
                    rect,
                    dock_rect
                );
            }

            // 3. No two distinct panels overlap
            for i in 0..panels.len() {
                for j in (i + 1)..panels.len() {
                    let (id_a, rect_a) = panels[i];
                    let (id_b, rect_b) = panels[j];
                    let intersection = rect_a.intersect(rect_b);
                    let overlap_w = intersection.width().max(0.0);
                    let overlap_h = intersection.height().max(0.0);
                    assert!(
                        overlap_w <= 0.0 || overlap_h <= 0.0,
                        "Panels {:?} and {:?} overlap by {:.1}x{:.1} at {:?}",
                        id_a,
                        id_b,
                        overlap_w,
                        overlap_h,
                        res
                    );
                }
            }
        }
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

    #[test]
    fn layout_snapshot_matches_canonical_and_satisfies_minimums() {
        use std::path::Path;

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let snapshot_path = manifest_dir.join("../../docs/snapshots/layout-geometry.json");

        let dock = DockTree::standard();
        let current_snapshot = build_current_layout_snapshot(&dock);

        // 1. Verify integrity of current live layout
        if let Err(errors) = verify_snapshot_integrity(&current_snapshot) {
            panic!(
                "Current layout violates minimum rect or geometry constraints: {:?}",
                errors
            );
        }

        // 2. Support updating snapshot if explicitly requested
        if std::env::var("UPDATE_LAYOUT_SNAPSHOTS").as_deref() == Ok("1") {
            if let Some(parent) = snapshot_path.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create snapshot directory");
            }
            let serialized = serde_json::to_string_pretty(&current_snapshot)
                .expect("Failed to serialize snapshot");
            std::fs::write(&snapshot_path, serialized).expect("Failed to write snapshot file");
            println!("Updated layout snapshot at {:?}", snapshot_path);
            return;
        }

        assert!(
            snapshot_path.exists(),
            "Canonical layout snapshot missing at {:?}. Run with UPDATE_LAYOUT_SNAPSHOTS=1 to generate.",
            snapshot_path
        );

        let snapshot_data =
            std::fs::read_to_string(&snapshot_path).expect("Failed to read snapshot file");
        let expected_snapshot: LayoutSnapshotManifest = serde_json::from_str(&snapshot_data)
            .expect("Failed to parse canonical layout snapshot JSON");

        if let Err(diffs) = compare_snapshots(&current_snapshot, &expected_snapshot, 0.5) {
            panic!(
                "Layout regression detected! Live layout deviates from canonical snapshot in docs/snapshots/layout-geometry.json:\n{}\n\nIf this layout change is intentional and approved, update the snapshot with UPDATE_LAYOUT_SNAPSHOTS=1.",
                diffs.join("\n")
            );
        }
    }

    #[test]
    fn layout_snapshot_negative_test_fails_when_panel_leaves_minimum_rect() {
        let dock = DockTree::standard();
        let mut mutated_snapshot = build_current_layout_snapshot(&dock);

        assert!(!mutated_snapshot.resolutions.is_empty());
        let res0 = &mut mutated_snapshot.resolutions[0];
        assert!(!res0.panels.is_empty());
        // Deliberately force a panel width to 50px (below required minimum)
        res0.panels[0].width = 50.0;

        let result = verify_snapshot_integrity(&mutated_snapshot);
        assert!(
            result.is_err(),
            "verify_snapshot_integrity must fail when a panel leaves its minimum rect"
        );
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.contains("width") && e.contains("below minimum")));
    }

    #[test]
    fn layout_snapshot_negative_test_detects_geometry_drift() {
        let dock = DockTree::standard();
        let base_snapshot = build_current_layout_snapshot(&dock);
        let mut mutated_snapshot = base_snapshot.clone();

        assert!(!mutated_snapshot.resolutions.is_empty());
        let res0 = &mut mutated_snapshot.resolutions[0];
        assert!(!res0.panels.is_empty());
        // Mutate panel x position by 15px
        res0.panels[0].min_x += 15.0;

        let result = compare_snapshots(&mutated_snapshot, &base_snapshot, 0.5);
        assert!(
            result.is_err(),
            "compare_snapshots must detect geometry drift"
        );
        let diffs = result.unwrap_err();
        assert!(diffs.iter().any(|d| d.contains("rect drift")));
    }
}
