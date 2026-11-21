# Agent Session Handoff Register (docs/HANDOFF.md)

This document provides a structured handoff ledger between agent sessions, recording session deliverables, verification command evidence, active state, and immediate next steps.

---

## Session Handoff Entry: Bootstrap & Governance Baseline

- **Date:** 2026-11-14
- **Branch:** `docs/a2-bootstrap`
- **Target PR Base:** `main` (commit `0d54a52`, PR #44)
- **Session Focus:** Documentation & Governance Bootstrap (Addenda A2 & A3)

### 1. Completed in This Session
1. **Master Workplan Restoration (`docs/WORKPLAN.md`)**: Restored Waves 0–2 verbatim (Wave 0: H1, L1..L4; Wave 1: P1..P5; Wave 2: S1..S4). Formed separate "Completed Work" section documenting Milestones M1–M12 with verified merge SHAs and PR numbers. Removed undefined terminology ("Zero-Copilot") and converted absolute links to relative paths.
2. **Phase Catalog Roadmap (`docs/WORKPLAN-PHASES.md`)**: Full itemization across Phases F1–F16 (281 catalog rows) with size decomposition, release trains R1–R5, and audit step 1e status mappings.
3. **Octave & Julia Governance**: Created `docs/CLEAN-ROOM-OCTAVE.md` (mirrored as P14 in `docs/AGENT-PROTOCOL.md`) and added Phases F15 (Octave Conformance) and F16 (Julia Bridge).
4. **Agent Prompt Stubs & Legacy Preservation**: Preserved full legacy master prompt in `docs/legacy/AGENTS-legacy.md`. Installed identical 12-line stubs in `AGENTS.md`, `GEMINI.md`, `MASTER-PROMPT.md`, and `docs/MASTER-PROMPT.md`. Added Protocol rule P15.
5. **Path & Secret Privacy**: Refactored `scripts/check_docs.py` to read custom denylist tokens from untracked `.privacy-denylist` or `PRIVACY_DENYLIST` env var; generic checks verify user directories and private keys. Added test suite `scripts/test_check_docs.py`. Zero hits across tracked tree.
6. **Storage & Footprint Auditing**: Created `scripts/storage-report.ps1` and `scripts/storage-report.sh`. Recorded baseline audits in `docs/audit/truth.md`, `docs/audit/storage.md`, and `docs/audit/build-size.md`.
7. **Architectural Decision Records**: Added `docs/adr/0001-record-decisions.md` and `docs/adr/0002-storage-and-build-hygiene.md`.

### 2. Verified Baseline State
- **Workspace Build:** `cargo check --workspace` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Privacy Test Suite:** `python scripts/test_check_docs.py` passes all 3 tests.
- **Storage Budgets:** Tracked size 9.48 MB (budget <= 50 MB); max file 218.1 KB (threshold <= 500 KB); release binary 10.20 MB (budget <= 13.0 MB); free disk 100.96 GB (minimum >= 20 GB).

### 3. Immediate Next Steps for Next Session
1. Await owner command to merge PR for `docs/a2-bootstrap`.
2. Following merge, initiate Wave 0 / Planning Task `PLAN-F1` as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).

---

## Session Handoff Entry: Wave 0 Task H1 - Repository Hygiene & Truth Audit

- **Date:** 2026-11-14
- **Branch:** `task/h1-hygiene-truth-audit`
- **Target PR Base:** `main` (commit `4f52800`, PR #48)
- **Session Focus:** Wave 0 Task H1 (Automated repository hygiene and truth audit)

### 1. Completed in This Session
1. **Verification Harness Gating**:
   - `scripts/verify.ps1`: Added Step 10 executing `python scripts/check_docs.py` to gate documentation coverage, privacy hygiene, and link integrity before workspace cleanliness checks.
   - `scripts/verify.sh`: Added Step 7 executing `check_docs.py` across Linux/macOS environments via `python3` or `python`.
2. **Capability Claim Truth Audit**:
   - Synchronized `README.md` test count claim from 503 to active verified count of 599.
3. **Master Test Manifest Audit**:
   - Audited `docs/test-manifest.txt` against `cargo test --workspace -- --list`; exact 1:1 match verified (599 named tests).
4. **Audit Ledgers & Documentation Update**:
   - Updated `docs/audit/truth.md` with execution evidence and verification gate mappings.
   - Updated `docs/STATE.md` with active Task H1 status and branch details.

### 2. Verified Baseline State
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Privacy Validator Test Suite:** `python scripts/test_check_docs.py` passes all 3 tests.
- **Test Suite Verification:** 599 named tests in workspace match 599 entries in `docs/test-manifest.txt`.

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task H1 PR.
2. Following merge, proceed with Wave 0 Task L1 (Layout audit - read-only) as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).

---

## Session Handoff Entry: Wave 0 Task L1 - Layout Audit (Read-Only)

- **Date:** 2026-11-14
- **Branch:** `task/l1-layout-audit`
- **Target PR Base:** `main` (commit `4950614`, PR #49)
- **Session Focus:** Wave 0 Task L1 (Desktop layout audit against contracts C1–C12)

### 1. Completed in This Session
1. **Real-Window Captures at Supported Sizes**:
   - `docs/screenshots/l1/desktop_900x600.png` (103,181 B, SHA-256 `AE97E3D11C508D147CD17B3661D930B50437444BBC95F56022ACADC9F4F6F560`)
   - `docs/screenshots/l1/desktop_1280x800.png` (151,397 B, SHA-256 `FE1548E6061A159DAC40C631AE731AE455F6840A5215E7A641B108349091788F`)
   - `docs/screenshots/l1/desktop_1920x1080.png` (207,773 B, SHA-256 `3F7C60D4FA23D14EC00BD3AD225B048D0D37A8A8C28FD841DECBFD1A83CDA836`)
2. **Contracts C1–C12 Complete Table**:
   - Updated `docs/layout-audit.md` with complete verdicts, concrete line references, and visual analysis across all three resolutions.
   - Identified critical violations: C5 (HOME ribbon overflow without wrapping), C2 (unenforced panel minimum sizes in runtime dock), C6 (search box and help header clipping), C8 (inconsistent CWD state representation), C9 (bracket prefixes in tab titles), C10 (Figures empty-state clipping), C12 (unresponsive at 900×600).
3. **Audit Ledgers & Session Records**:
   - Updated `docs/STATE.md` with active Task L1 status.
   - Preserved read-only contract: zero source code in `apps/**` or `crates/**` modified.

### 2. Verified Baseline State
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Privacy Validator Test Suite:** `python scripts/test_check_docs.py` passes all 3 tests.
- **Storage Budget Check:** All capture files individually <= 500 KB; total added footprint ~452 KB (well below 50 MB budget).

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task L1 PR.
2. Following merge, proceed with Wave 0 Task L2 (Layout probe and contract checks) as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).

---

## Session Handoff Entry: Wave 0 Task L2 - Layout Probe & Contract Checks

- **Date:** 2026-11-14
- **Branch:** `task/l2-layout-probe-checks`
- **Target PR Base:** `main` (commit `6793ca1`, PR #50)
- **Session Focus:** Wave 0 Task L2 (Read-only layout probe and automated contract checks)

### 1. Completed in This Session
1. **Layout Probe & Contract Checking Infrastructure**:
   - `apps/desktop/src/layout_contract.rs`:
     - `LayoutProbe`: Lightweight rect ownership probe tracking panel ID, control ID, control rect, and authoritative clip rect.
     - `check_layout` & `check_layout_full`: Aggregated invariant checker returning structured `LayoutReport`.
     - `PanelClip`: C1 container check enforcing panel body clip intersection to prevent child paint bleed.
     - `PANEL_MIN`: Centralized minimum panel dimensions table for 7 core panels (`panel_min` lookup).
     - `check_mutation`: L-E mutation checker proving perturbation across all 4 quadrants ($+x, -x, +y, -y$) fails deterministically.
     - `run_size_sweep` & `generate_capture_report`: L-C size sweep evaluating dock layout across resolutions.
2. **Pure Dock Rect Geometry Extraction**:
   - `apps/desktop/src/dock.rs`: Added read-only `DockNode::collect_leaf_rects` and `DockTree::collect_panel_rects` for headless geometry evaluation without UI mutation.
3. **Automated Unit & Contract Test Suite**:
   - Added 5 new tests in `layout_contract::tests` (+5 net tests, bringing workspace total from 599 to 604):
     - `c2_all_panel_min_table_rows_exercised_below_threshold`: Exercises every row in `PANEL_MIN` with under-sized rects (proves V4 detection).
     - `c2_all_panel_min_table_rows_exercised_at_or_above_threshold`: Exercises every row in `PANEL_MIN` at/above threshold (proves clean pass).
     - `le_mutation_check_detects_all_quadrant_shifts`: Exercises L-E mutation checks in $+x, -x, +y, -y$.
     - `lc_size_sweep_exercises_supported_sizes`: Evaluates 900×600, 1280×800, and 1920×1080 against standard dock tree.
     - `check_layout_full_aggregates_both_controls_and_panels`: Verifies dual-mode aggregation.
4. **Manifest and Documentation Synchronization**:
   - Synchronized `docs/test-manifest.txt` to 604 tests (1:1 match with `cargo test --workspace -- --list`).
   - Synchronized `README.md` test counter claim to 604 tests.
   - Updated `docs/audit/truth.md` and `docs/STATE.md`.
5. **Findings Only / No Panel Fixes**:
   - Strict adherence to task boundary: zero panel layout fixes implemented (reserved for Task L3).

### 2. Verified Baseline State
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Test Suite Verification:** 604/604 named tests in workspace match `docs/test-manifest.txt`.
- **All 14 Layout Contract Tests Pass Cleanly.**

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task L2 PR.
2. Following merge, proceed with Wave 0 Task L3 (Responsive layout fixes) as governed by `docs/WORKPLAN.md`.

---

## Session Handoff Entry: Wave 0 Task L3 - Responsive Layout Fixes

- **Date:** 2026-11-14
- **Branch:** `task/l3-responsive-layout-fixes`
- **Target PR Base:** `main` (commit `5fb3c49`, PR #51)
- **Session Focus:** Wave 0 Task L3 (Responsive dock layout fixes, C2/C12 resolution, three-size geometry testing)

### 1. Completed in This Session
1. **Dock Split Clamping Logic**:
   - `apps/desktop/src/dock.rs`:
     - Added `DockNode::min_width(&self) -> f32` and `DockNode::min_height(&self) -> f32` to compute aggregate minimum dimension floors across dock leaves and subtrees.
     - Added `DockNode::split_h_sizes` and `DockNode::split_v_sizes` clamping split allocations to children's minimum dimensions, preventing panel collapse below minimum bounds.
     - Integrated split clamping into both headless geometry extraction (`collect_leaf_rects`) and live UI rendering (`render_node`).
2. **Typography and Header Polishing**:
   - `apps/desktop/src/dock.rs`: Cleaned up bracket prefix tags (`[D]`, `[E]`) from tab headers and floating drag preview chips.
   - `apps/desktop/src/app.rs`: Cleaned up bracket prefixes in View menu panel toggle checkboxes (`panel.title()`).
   - `apps/desktop/src/app.rs`: Made the global search box responsive by clamping width dynamically to `(avail_w - 10.0).clamp(100.0, 260.0)` and truncating hint text on narrow widths.
3. **Automated Three-Size Geometry Verification**:
   - `apps/desktop/src/layout_contract.rs`:
     - Added `three_size_geometry_panels_satisfy_bounds_and_no_overlaps`: Verifies per-panel minimums satisfied, panels contained within container bounds, and pairwise disjoint panel rects (zero overlap) across 900×600, 1280×800, and 1920×1080.
     - Updated `lc_size_sweep_exercises_supported_sizes` to assert clean pass across all 3 supported resolutions.
4. **Real-Window Captures at Supported Sizes**:
   - Captured fresh screenshots into `docs/screenshots/l3/`:
     - `desktop_900x600.png` (101,112 B, SHA-256 `C5012E16E42F2130361704496953B20DA71C9AF62D512708DF39144E3BBD15ED`)
     - `desktop_1280x800.png` (151,526 B, SHA-256 `A2F85AD3BE37B3451C39E60F3DF5E5918FFBA2896E5388243F19D97DA12C6E6B`)
     - `desktop_1920x1080.png` (206,126 B, SHA-256 `B30D32D9CB2240F18BBDEDCF02A9F1C6645DF2DD2A3C5529CF865899F9E58B07`)
   - All files <= 500 KB; total added footprint ~458 KB (within 50 MB budget).
5. **Manifest & Accounting Synchronization**:
   - Synchronized `docs/test-manifest.txt` to 605 tests (+1 test: `three_size_geometry_panels_satisfy_bounds_and_no_overlaps`).
   - Synchronized `README.md` test counter claim to 605 tests.
   - Updated `docs/audit/truth.md` and `docs/STATE.md`.

### 2. Verified Baseline State
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Test Suite Verification:** 605/605 named tests in workspace match `docs/test-manifest.txt`.
- **Desktop Test Suite:** All 104 desktop tests pass cleanly (15 layout contract tests, 50 unit tests, 28 desktop integration tests, 13 editor tests, 10 clean GUI tests, 3 current folder tests).

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task L3 PR.
2. Following merge, proceed with Wave 0 Task L4 (Layout regression gate) as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).

---

## Session Handoff Entry: Wave 0 Task L4 - Layout Regression Gate (Auto)

- **Date:** 2026-11-14
- **Branch:** `task/l4-layout-regression-gate`
- **Target PR Base:** `main` (commit `fe235b4`, PR #52)
- **Session Focus:** Wave 0 Task L4 (Layout regression gate, canonical snapshot storage, unapproved regression prevention)

### 1. Completed in This Session
1. **Canonical Layout Snapshot Manifest**:
   - `docs/snapshots/layout-geometry.json`: Recorded canonical baseline geometry (all panel rect coordinates, minimum dimensions, containment bounds, and zero-overlap guarantees) for supported sizes: 900×600, 1280×800, and 1920×1080.
2. **Snapshot Infrastructure & Verification Logic**:
   - `apps/desktop/src/layout_contract.rs`:
     - Added `PanelSnapshot`, `ResolutionSnapshot`, and `LayoutSnapshotManifest` with Serde derive.
     - Implemented `build_current_layout_snapshot` extracting live dock panel rects.
     - Implemented `verify_snapshot_integrity` asserting per-panel minimum floors and boundary containment.
     - Implemented `compare_snapshots` detecting geometric rect drift against stored canonical snapshots.
3. **Automated Unit & Regression Tests**:
   - Added 3 new tests in `layout_contract::tests` (+3 net tests, bringing workspace total from 605 to 608):
     - `layout_snapshot_matches_canonical_and_satisfies_minimums`: Verifies live layout matches `docs/snapshots/layout-geometry.json` within 0.5px tolerance and satisfies all minimum floors (supports `UPDATE_LAYOUT_SNAPSHOTS=1` regeneration).
     - `layout_snapshot_negative_test_fails_when_panel_leaves_minimum_rect`: Proves check fails deterministically when a panel leaves its declared minimum rect.
     - `layout_snapshot_negative_test_detects_geometry_drift`: Proves check fails on unauthorized geometric rect drift.
4. **Automated Layout Regression Gate Checker**:
   - Added `scripts/check_layout_gate.py` with `--self-test` coverage.
   - Enforces snapshot integrity and guards against unapproved snapshot modifications without an explicit `[layout-approved]` commit rationale or `approved_reason`.
5. **CI & Verification Harnesses Integration**:
   - `.github/workflows/ci.yml`: Added `Layout Regression Gate (Wave 0 Task L4)` running on every pull request.
   - `scripts/verify.ps1`: Step 11 runs `python scripts/check_layout_gate.py`.
   - `scripts/verify.sh`: Step 8 runs `check_layout_gate.py`.
6. **Manifest & Accounting Synchronization**:
   - Synchronized `docs/test-manifest.txt` to 608 tests.
   - Synchronized `README.md` test counter claim to 608 tests.
   - Updated `docs/audit/truth.md` and `docs/STATE.md`.

### 2. Verified Baseline State
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Layout Gate Checks:** `python scripts/check_layout_gate.py --self-test` and `python scripts/check_layout_gate.py` pass.
- **Test Suite Verification:** 608/608 named tests in workspace match `docs/test-manifest.txt`.
- **Desktop Test Suite:** All 107 desktop tests pass cleanly (18 layout contract tests, 50 unit tests, 28 desktop integration tests, 13 editor tests, 10 clean GUI tests, 3 current folder tests).

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task L4 PR.
2. Following merge of Task L4 (completing Wave 0: Hygiene and Layout Foundations: H1, L1, L2, L3, L4), proceed to **Wave 1: Plotting and Visualization (Task P1: Subplot rendering)** as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).

---

## Session Handoff Entry: Wave 1 Task P1 - Subplot Rendering

- **Date:** 2026-11-14
- **Branch:** `task/p1-subplot-rendering`
- **Target PR Base:** `main` (commit `a35d047`, PR #53)
- **Session Focus:** Wave 1 Task P1 (Subplot rendering, grid positioning, spanning cells, non-overlapping rects, and golden SVG snapshots)

### 1. Completed in This Session
1. **Subplot Position & Core Modeling**:
   - `crates/rustlab-plot/src/lib.rs`:
     - Added `SubplotPosition` struct with `single`, `spanning`, `primary_index`, `grid_bounds`, and `compute_rect`.
     - Added `subplot_pos`, `shared_x_with`, and `shared_y_with` on `Axes`.
     - Added `Figure::deterministic_axes(&self) -> Vec<&Axes>` sorting axes deterministically by `(min_row, min_col, id)`.
     - Added `Figure::subplot(&mut self, m, n, p) -> &mut Axes` and `Figure::subplot_spanning(&mut self, m, n, indices) -> &mut Axes`.
2. **Deterministic SVG Multi-Axes Layout Engine**:
   - `crates/rustlab-plot/src/svg.rs`:
     - Updated `to_svg` to layout subplots across available figure canvas with 60px gutters and non-overlapping cell bounding rects.
     - Preserved 100% byte-exact backwards compatibility for single default axes (`fig.axes.len() == 1 && fig.axes[0].subplot_pos.is_none()`).
     - Implemented `resolve_limits` honoring shared axis synchronization (`shared_x_with`, `shared_y_with`).
     - Implemented `grid_lines_subplot` with boundary tick clipping to prevent grid lines or ticks bleeding into neighboring subplots.
3. **MATLAB Built-in & Desktop Integration**:
   - `crates/rustlab-stdlib/src/plot_builtins.rs`:
     - Updated `subplot_fn` to support 1-argument (`subplot(211)`), 3-argument scalar (`subplot(2, 2, 3)`), and 3-argument vector spanning (`subplot(2, 2, [1, 2])`).
     - Added validation for dimension boundaries ($1 \le p \le m \times n$).
   - `apps/desktop/src/execution.rs`:
     - Extended desktop subplot builtin to handle scalar and vector spanning arguments.
   - `apps/desktop/src/figures.rs`:
     - Extended `FiguresPanel::show` to render all axes for multi-axes figures into tiled grid positions.
4. **Golden SVG Snapshots & Unit Tests**:
   - `crates/rustlab-plot/tests/snapshot_tests.rs`:
     - `test_subplot_223_places_axes_in_bottom_left_cell`: Asserts `subplot(2, 2, 3)` lands at bottom-left (row 1, col 0) with exact coordinates.
     - `test_empty_subplot_no_bleeding_or_collapsing`: Asserts empty subplots preserve positive width/height without collapsing or bleeding.
     - `snapshot_subplot_1x1_at_1280x800`: 1x1 subplot snapshot.
     - `snapshot_subplot_2x1_at_1280x800`: 2x1 grid subplot snapshot.
     - `snapshot_subplot_2x2_at_1280x800`: 2x2 grid subplot snapshot.
     - `snapshot_subplot_spanning_at_1280x800`: Spanning top-row subplot snapshot (`[1, 2]`) with 2 bottom subplots.
     - `snapshot_subplot_empty_at_1280x800`: 2x1 subplot snapshot with second subplot empty.
   - Committed 5 golden reference SVG files to `crates/rustlab-plot/tests/snapshots/`.
5. **Accounting & Verification Synchronization**:
   - Synchronized `docs/test-manifest.txt` to 623 tests (+4 net tests in Task P3).
   - Synchronized `README.md` test counter claim to 623 tests.
   - Updated `docs/STATE.md` and `docs/audit/truth.md`.

### 2. Verified Baseline State (Wave 1 Task P3)
- **Formatting:** `cargo fmt --all -- --check` passes cleanly.
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (242 registered / 249 documented, 0 privacy leaks, 100% link resolution).
- **Layout Gate Checks:** `python scripts/check_layout_gate.py` passes.
- **Plotting & Snapshot Test Suite:** All 41 tests in `rustlab-plot` pass (23 unit tests + 18 integration and regression tests).
- **Stdlib Test Suite:** All tests in `rustlab-stdlib` pass.
- **Desktop Test Suite:** All 107 tests in `matrust_studio` pass.
- **Workspace Master Test Manifest:** 623/623 named tests in workspace match `docs/test-manifest.txt`.

### 3. Immediate Next Steps for Next Session
1. Await owner review and approval for Task P3 PR.
2. Following merge of Task P3, proceed to **Wave 1 Task P4: 3D plotting** as governed by `docs/WORKPLAN.md`.
3. Adhere strictly to Protocol P6 (one task per session).


---

## Session Handoff Entry: Task FIX-MAIN-CI & Phase F17 Math Gallery Audit

- **Date:** 2026-11-21
- **Branch:** `task/fix-main-ci`
- **Target PR Base:** `main` (commit `a141145`)
- **Session Focus:** CI failure reproduction, clippy repairs, lockfile drift reconciliation, and Math Gallery status ledger

### 1. Math Gallery Task Status Ledger (MG0..MG10)

> [!IMPORTANT]
> **Phase F17 Completion Status:** **NOT COMPLETE**.
> Under project governance rules (Charter & Protocol P6), work must be reviewed and merged via PRs with all CI gates green. All 7 commits since `8e567c5` were pushed directly to `main` without PR numbers, and essential dock/clippy fixes currently reside in local uncommitted state. Phase F17 cannot be marked complete until every row is merged via approved PR.

| Task | Description | Status | Commit / Location | Named Tests / Evidence | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **MG0** | Probe and inventory | Pushed directly to main | `2549b90` | `docs/audit/math-probe.md` | Probed 99 names (47 works, 52 missing); no PR |
| **MG1** | Calculus built-ins | Pushed directly to main | `aafebe4` | `test_del2_1d_and_2d`, `test_divergence_and_curl_2d`, `test_quadrature_1d_integral_quad_quadgk`, `test_quadrature_2d_and_3d` | `del2`, `divergence`, `curl`, `integral[2,3]`, `quad[gk]`; no PR |
| **MG2** | ODE solver suite | Pushed directly to main | `aafebe4` | `test_odeset_options_struct`, `test_ode45_and_ode23_with_options`, `test_ode15s_stiff_system`, `test_ode15s_vanderpol_oscillator`, `test_expm_and_interp1_integration` | `ode45`, `ode23`, `ode15s`, `odeset`, `expm`, `interp1`; no PR |
| **MG3** | Signal processing suite | Pushed directly to main | `aafebe4` | `test_interpolation_spline_and_pchip`, `test_spectral_pwelch_and_spectrogram`, `test_signal_deconv_and_freqz`, `test_signal_residue` | `deconv`, `freqz`, `residue`, `polyfit`, `spline`, `pchip`, `pwelch`, `spectrogram`; no PR |
| **MG4** | Optimization suite | Pushed directly to main | `cee478c` | `test_optimset_options_management`, `test_fminbnd_scalar_minimization`, `test_fminunc_multivariable_bfgs`, `test_fsolve_nonlinear_system`, `test_lsqnonlin_and_lsqcurvefit`, `test_linprog_two_phase_simplex` | `fzero`, `fminbnd`, `fminsearch`, `fminunc`, `fsolve`, `lsqnonlin`, `lsqcurvefit`, `linprog`, `optimset`; no PR |
| **MG5** | Plotting 2D/3D graphics | Pushed directly to main (local clippy fix) | `916492a` / local | `test_plot_types_mesh`, `test_plot_types_stem`, `test_plot_types_log_scales`, `test_plot_types_polar`, `test_plot_types_quiver`, `test_plot_types_contour_and_imagesc`, `test_plot_types_area_and_fill`, `test_plot_types_view_and_axis` | Clippy error in `imagesc_fn` fixed locally; no PR |
| **MG6** | Math Gallery UI & Catalog | Pushed directly to main (local layout overhaul) | `007314c`, `a141145` / local | `test_math_gallery_dock_placement_and_migration` | Central dock migration, collapsible_match clippy fixes, responsive cards, zero dead space; no PR |
| **MG7** | Gallery sets: Calculus & ODE | Pushed directly to main | `007314c`, `a141145` | `test_gallery_adaptive_quadrature`, `test_gallery_laplacian_del2`, `test_gallery_helmholtz_gradient`, `test_gallery_damped_pendulum`, `test_gallery_vector_curl`, `test_gallery_van_der_pol` | 6 calculus/ODE scripts; no PR |
| **MG8** | Gallery sets: Linalg & Signal | Pushed directly to main | `007314c`, `a141145` | `test_gallery_qr_least_squares`, `test_gallery_spline_interpolation`, `test_gallery_svd_image_compression`, `test_gallery_spectral_pwelch`, `test_gallery_matrix_exponential`, `test_gallery_eigen_modes`, `test_gallery_spectrogram_chirp`, `test_gallery_butterworth_filter` | 8 linalg/signal scripts; no PR |
| **MG9** | Gallery sets: Optim & Fractals | Pushed directly to main | `007314c`, `a141145` | `test_gallery_rosenbrock_fminunc`, `test_gallery_nonlinear_fsolve`, `test_gallery_simplex_linprog`, `test_gallery_curve_fitting_lsq`, `test_gallery_barnsley_fern`, `test_gallery_julia_set`, `test_gallery_mandelbrot_set` | 8 optim/fractal scripts; no PR |
| **MG10** | End-to-end audit & manual pass | Pushed directly to main | `6423081`, `007314c`, `docs/audit/math-gallery-audit.md` | 24 verified runnable models | Audited and verified across CLI/GUI; no PR |

### 2. Verified Baseline State (Local Gate Reproduction)
- **Formatting:** `cargo fmt --all -- --check` passes cleanly (code 0).
- **Clippy (--all-features):** `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes cleanly (code 0).
- **Clippy (--no-default-features):** `cargo clippy --workspace --all-targets --no-default-features -- -D warnings` passes cleanly (code 0).
- **Workspace Tests:** `cargo test --workspace` passes all tests (code 0).
- **Documentation & Privacy Checks:** `python scripts/check_docs.py` passes (code 0).
- **Layout Gate Checks:** `python scripts/check_layout_gate.py` passes across all 3 resolutions (code 0).
- **Fuzz Corpus Smoke Suite:** `cargo test --manifest-path fuzz/Cargo.toml --test corpus_smoke` passes all 5 tests (code 0).
