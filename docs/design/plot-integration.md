# PLOT-1 Design Note — Plot Integration

**Repo:** `C:\Users\varun\Desktop\MATSTUDIO`
**Branch:** `gui/polish-1` (or new branch `plot/int-1` from it)
**Phase:** PLOT-1 — design note only, NO code changes
**Based exclusively on:** PLOT-0 discovery (`docs/plot-int/discovery.md`) — every structural claim cites `file:line`.

---

## 1. Scope

**IN SCOPE (extend, do not rewrite):**
- Extend `crates/rustlab-plot` model (`Figure`, `Axes`, `Series`, `PlotRegistry`) with missing fields/variants (zlabel, subplot position, 3D camera, surf mesh, colorbar).
- Add missing plotting builtins (`peaks`, `surf`, `zlabel`, `colorbar`, `colormap`, `shading`, `view`, `axis`, `ishold`, `gcf`, `gca`, `clf`, `cla`, `close`, plus `hold` toggle/`hold(ax,'on')` forms).
- Consolidate the two plot paths onto ONE engine (`rustlab-plot::PlotRegistry`); remove the parallel desktop `GraphicsState` plot path (`execution.rs` GUI builtins) so there is no second plotting engine.
- Figure tabs: one tab per Figure; subplot axes laid out inside the tab.
- Deterministic SVG renderer (existing `svg.rs`) extended for 3D + colorbar; byte-equal snapshot tests.
- Fix call-expression span bug (`parser.rs:712-716`) — separate commit S1 + regression test.
- Docs + help + error-catalog entries for every new builtin (docs coverage test must stay green).
- Tests S1–S12 per the milestone spec.

**OUT OF SCOPE:**
- Rewrite the app, interpreter, or plot engine.
- A second plotting engine (the desktop `GraphicsState` path is consolidated, not duplicated).
- Hardcoding outputs for test programs.
- GUI beyond figure-tab grouping (no egui shell rewrite).
- 3D interactive mouse handling in this phase's design beyond the view-model camera math (S7).

---

## 2. Ownership (adapted to real structs found — no parallel types)

| Concept | Real struct | file:line | Notes |
|---------|-------------|-----------|-------|
| FigureManager | `rustlab-plot::PlotRegistry` | `crates/rustlab-plot/src/state.rs:7-9`, `lib.rs:281-335` | `figures: HashMap<usize, Figure>`, `current_id: Option<usize>`, `next_id: usize`. Thread-local `RefCell` (`state.rs:7-9`). |
| Figure | `rustlab-plot::Figure` | `crates/rustlab-plot/src/lib.rs:243-249` | `id`, `title`, `axes: Vec<Axes>`, `current_axes: Option<usize>`, `visible`. |
| Axes | `rustlab-plot::Axes` | `crates/rustlab-plot/src/lib.rs:156-170` | `id`, `title`, `xlabel`, `ylabel`, `xlim`, `ylim`, `xscale`, `yscale`, `grid`, `hold`, `series`, `texts`, `legend`. |
| Series | `rustlab-plot::Series` | `crates/rustlab-plot/src/lib.rs:29-120` | `Line`, `Scatter`, `Bar`, `Histogram`. |
| Session (desktop) | `execution::Session` | `apps/desktop/src/execution.rs:174` | owns `graphics: Arc<Mutex<GraphicsState>>` today — to be consolidated onto `rustlab-plot::PlotRegistry`. |
| GraphicsState (desktop, parallel engine to remove) | `execution::GraphicsState` | `apps/desktop/src/execution.rs:120` | `figures: Vec<FigureState>`, `current_figure`. Flat `FigureState.items` — no axes grid/hold/legend. |
| UI panel | `desktop::FiguresPanel` | `apps/desktop/src/figures.rs:7` | renders one tab per `FigureState` (`figures.rs:43-61`). |

**Ownership rule (adapted):** the interpreter/CLI path already mutates `PlotRegistry` via `with_plot_state` (`state.rs:15`). The desktop `Session` (`execution.rs:174`) must use the SAME `rustlab-plot::PlotRegistry` functions for its plotting builtins instead of writing to the parallel `GraphicsState`. `FiguresPanel` reads figure state from `PlotRegistry` (or a snapshot passed through `GraphicsState` non-plot fields only). This extends `rustlab-plot`; it does not create a parallel type.

> **UNVERIFIED:** whether `PlotRegistry` thread-local state leaks across `Session` lifetimes (it is thread-local per process; `Session` creates a fresh `Interpreter` — `execution.rs:190`).

---

## 3. Model extensions (fields added to EXISTING structs only)

| Struct | New field | Purpose | file:line (existing anchor) |
|--------|-----------|---------|------------------------------|
| `Axes` | `zlabel: String` | z-axis label for 3D | `lib.rs:160` (ylabel anchor) |
| `Axes` | `position: (usize, usize, usize)` | subplot (row, col, span) | `lib.rs:156` (id anchor) |
| `Axes` | `kind: AxesKind { TwoD, ThreeD }` | 2D vs 3D axes | `lib.rs:156` |
| `Axes` | `camera: Option<Camera { azimuth: f64, elevation: f64 }>` | 3D view | `lib.rs:156` |
| `Axes` | `color_cycle_index: usize` | hold color cycling | `lib.rs:166` (hold anchor) |
| `Axes` | `colorbar: Option<Colorbar>` | current-axes colorbar | `lib.rs:169` (legend anchor) |
| `Figure` | `layout: (usize, usize)` | subplot grid rows/cols | `lib.rs:243` (id anchor) |
| `Figure` | `dirty: bool` | tab dot indicator | `lib.rs:243` |
| `Series` | `Surface { .. }` quad-mesh variant | surf data | `lib.rs:31` (Line anchor) |

No new top-level types are introduced; `AxesKind`, `Camera`, `Colorbar` are small enums/structs nested as fields.

---

## 4. MATLAB semantics to implement (each with source or UNVERIFIED)

Per R3, since Octave is not installed on this machine and no MathWorks documentation URL was opened in this session, every MATLAB-behavior claim below is **UNVERIFIED** pending verification against a MathWorks doc or Octave output. The existing RustLab behavior (where it exists) is cited from code.

| Semantics | Source | Verdict |
|-----------|--------|---------|
| `plot` with no figure creates Figure 1 and Axes 1 | MATLAB public doc (not opened) | UNVERIFIED |
| `hold on`: current axes only; subsequent `plot` appends series and cycles color order | MATLAB public doc (not opened) | UNVERIFIED |
| `hold off`: next `plot` replaces series | MATLAB public doc (not opened) | UNVERIFIED |
| `hold` alone toggles; `hold(ax,'on')` form; query via `ishold` | MATLAB public doc (not opened) | UNVERIFIED |
| `figure`: new figure, makes it current; `figure(n)` selects/creates n | MATLAB public doc (not opened) | UNVERIFIED |
| `close`, `close all`, `gcf`, `gca`, `clf`, `cla` | MATLAB public doc (not opened) | UNVERIFIED |
| `subplot(m,n,p)`: select/create axes at grid cell inside current figure; reuse same cell; never creates figure unless none exists | MATLAB public doc (not opened) | UNVERIFIED |
| `subplot(mnp)` three-digit form | MATLAB public doc (not opened) | UNVERIFIED |
| `title`/`xlabel`/`ylabel`/`legend`/`grid`/`hold` act on current axes only | MATLAB public doc (not opened) | UNVERIFIED |
| `grid on` / `grid off` / `grid` (toggle): current axes only | MATLAB public doc (not opened) | UNVERIFIED |
| `legend(names...)`: current axes, series order | MATLAB public doc (not opened) | UNVERIFIED |
| `zlabel`, `view(az,el)`, `axis equal/tight/limits`, `colormap(name)`, `colorbar` (current axes), `shading flat|interp` | MATLAB public doc (not opened) | UNVERIFIED |
| `meshgrid` (1 or 2 vectors); `X` varies along columns | VERIFIED against code + CLI | `nd_ops::meshgrid` (`lib.rs:82`); `meshgrid.out` (row 1 = −3:0.2:3) |
| `peaks(n)` default n=49; `peaks(x,y)`; formula | public docs (not opened); Octave not installed | UNVERIFIED |
| `surf(X,Y,Z)` / `surf(Z)`; sizes match; NaN cells skipped; color from Z via colormap scaled to [min(Z),max(Z)] | MATLAB public doc (not opened) | UNVERIFIED |

**peaks formula** (public, documented; numeric check by hand — UNVERIFIED against Octave since Octave is absent):
```
z = 3*(1-x).^2.*exp(-(x.^2)-(y+1).^2)
  - 10*(x/5 - x.^3 - y.^5).*exp(-x.^2-y.^2)
  - (1/3)*exp(-(x+1).^2-y.^2)
```
`peaks(n)` uses `meshgrid` of `linspace(-3,3,n)`. Golden: `peaks(0,0) = (3 - 1/3)*exp(-1) = 0.98101` (4 d.p. `0.9810`).

**Colormap:** implement our own perceptual colormap, generated by code we write, documented construction, with our own name. Do NOT copy any MathWorks colormap table (parula is a MathWorks asset).

---

## 5. Rendering

- **2D SVG renderer:** existing `rustlab-plot::to_svg` (`svg.rs:12`) — deterministic, fixed 2-decimal formatting, stable element ids (`axes-{id}`). Extended for 3D surface + colorbar; byte-equal snapshot tests.
- **3D projection:** orthographic or perspective projection of the quad mesh; painter's algorithm (depth sort by quad centre); rotation by azimuth and elevation. Lives in `rustlab-plot` so the SVG renderer is deterministic and snapshot-testable; the GUI (egui `FiguresPanel`) only draws.
- **GUI interaction:** mouse drag rotate, wheel zoom, double-click reset — view-model camera math tested (S7); actual egui paint in `figures.rs` (existing, extended).
- **Tabs:** one Figure = one tab (`figures.rs:43-61` already does one tab per `FigureState`). Tab key = `FigureId`. Subplot axes are laid out inside the tab by the figure layout. Editor tab is separate, never replaced; execution never changes the active tab unless user setting "focus figure on plot" is on (default: show a dot on the figure tab — `Figure.dirty`).

---

## 6. Events (no UI calls from builtins)

Builtins mutate `rustlab-plot::PlotRegistry` and emit `FigureChanged(FigureId)`. The UI only reacts to events. No UI calls from builtins. (Current desktop GUI builtins in `execution.rs:243-428` write directly to `GraphicsState` — to be replaced by `PlotRegistry` mutation + event.)

---

## 7. Error texts and ids

Listed for the catalog (`docs/spec/error-catalog.md`), each cited or UNVERIFIED:
- `RUSTLAB:plot:badArgs` — wrong argument count (exists, `plot_builtins.rs:59`)
- `RUSTLAB:plot:noFigure` — no current figure (exists, `plot_builtins.rs:74`)
- `RUSTLAB:plot:emptyData` — X/Y length mismatch (exists, `plot_builtins.rs:67`)
- New: `RUSTLAB:plot:noAxes`, `RUSTLAB:surf:sizeMismatch`, `RUSTLAB:subplot:badDims`, `RUSTLAB:hold:badState`, etc. — UNVERIFIED text.
- Call-expression span fix: `parser.rs:712-716` (`Expr::Index.span` covers `(`…`)`, not callee) — confirmed bug via `nonexistent(1,2)` → Column 12 (`docs/plot-int/nonexistent.out`).

---

## 8. Test plan and file list (one commit per step)

| Step | File(s) to change | Test |
|------|-------------------|------|
| S1 Span fix | `crates/rustlab-parse/src/parser.rs` + new `tests/golden/plot_t18.m` | `Expr::Index.span` starts at callee ident, not `(`; regression test fails before, passes after |
| S2 State | `crates/rustlab-plot/src/lib.rs` (Axes/Figure fields) | plot twice without hold → 1 series; with hold → 2; hold off → replace; hold toggle; `ishold`; hold per axes (two subplots independent) |
| S3 Builtins | `crates/rustlab-stdlib/src/plot_builtins.rs`, `docs/functions/<name>.md`, `crates/rustlab-stdlib/src/help.rs`, `docs/spec/error-catalog.md`, `docs/builtins.txt` | hold/ishold/gcf/gca/clf/cla/close/figure(n)/subplot both forms/zlabel/grid per axes; docs+help coverage test green |
| S4 Tabs | `apps/desktop/src/figures.rs`, `execution.rs` | Scenario 2: 1 figure tab containing 2 axes; Scenario 3: 2 tabs; Scenario 1: 1 tab with 2 series; 10 plots without hold → 1 tab |
| S5 Data | `crates/rustlab-nd_ops` / `meshgrid` (`lib.rs:82`), `peaks`, `surf` data | meshgrid 31×31; `x(1,:) == -3:0.2:3`, `y(:,1) == (-3:0.2:3)'`; `peaks(0,0)=0.9810`; `size(peaks(49)) = 49×49`; `z(16,16)=0`; `max|z|<=1` |
| S6 SVG 3D | `crates/rustlab-plot/src/svg.rs` + `tests/snapshots/*.svg` | surf + colormap + colorbar + view + axis; byte-equal snapshots; fixed number formatting; deterministic depth sort with stable tie-break |
| S7 GUI 3D | `apps/desktop/src/figures.rs` | camera math view-model test: azimuth/elevation → projection of a known point |
| S8 Editor independence | `apps/desktop/src/execution.rs` | worker-thread script execution; editor tab stays editable; switching tabs during execution; figure tabs persist; closing a figure tab removes only that figure; re-running reuses figure numbers (no tab growth) |
| S9 Scenarios | `tests/golden/*.m` + `docs/plot-int/*.out` + screenshots | Scenarios 1-6 executed through action layer; saved transcripts + 1280x720/1920x1080 screenshots (SHA-256); three-axes check: grid on after second subplot affects only axes 2 |
| S10 Regression | all test files | Tests 19 & 20 from report still pass; existing plot tests unchanged; test count increases; manifest updated; `git status --porcelain` empty after verify.ps1 |
| S11 Perf | release build | surf 201×201 renders at 30 fps or better (machine + build stated); MISSED first if not met |
| S12 REA | own release binary only | R-check per AGENTS.md section 3; CALL-LOG rows |

---

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Two parallel engines (`rustlab-plot` vs desktop `GraphicsState`) | Consolidate desktop GUI builtins onto `PlotRegistry`; delete GUI plot path from `GraphicsState` |
| Thread-local `PlotRegistry` vs Session ownership | Keep thread-local (existing mechanism); both CLI and desktop call the same `with_plot_state` functions |
| Docs coverage test (`regression_a312_tests.rs:341`) must stay green | Every new builtin gets `docs/functions/<name>.md` + `help::get_help_text` + `builtins.txt` entry in the same commit |
| `builtins.txt` drift fails the test | Update `docs/builtins.txt` deliberately when adding builtins |
| 3D has no existing code | Built from scratch in `rustlab-plot` (projection + painter's algorithm), no MathWorks assets copied |
| No Octave / no MathWorks doc opened | All MATLAB semantics marked UNVERIFIED; verified against code behavior where it exists |

---

## 10. Decision needed

This design note adapts names to real structs (`PlotRegistry`, `Figure`, `Axes`, `Series`, `Session`, `GraphicsState`, `FiguresPanel`) and adds no parallel types. It consolidates the desktop `GraphicsState` plot path onto `rustlab-plot` (no second engine).

**STOP — waiting for approval of this plan before PLOT-2 (S1 span fix + implementation).**
