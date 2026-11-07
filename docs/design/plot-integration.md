# PLOT-1 Design Note — Plot Integration

**Repo:** `<repo>`
**Branch:** `gui/polish-1` (owner instruction: same branch; owner also allowed `plot/int-1` from it — not created, stayed on `gui/polish-1`)
**Phase:** PLOT-1 — design note only, NO code changes
**Based exclusively on:** PLOT-0 discovery (`docs/plot-int/discovery.md`) — every structural claim cites `file:line`.

---

## 1. Scope

**IN SCOPE (extend, do not rewrite):**

1. Extend `crates/rustlab-plot` model (`Figure`, `Axes`, `Series`, `PlotRegistry`) with missing fields/variants only (zlabel, subplot position, 3D camera, surf mesh, colorbar, hold color cycle).
2. Add missing plotting builtins (`peaks`, `surf`, `zlabel`, `colorbar`, `colormap`, `shading`, `view`, `axis`, `ishold`, `gcf`, `gca`, `clf`, `cla`, `close`, plus `hold` toggle / `hold(ax,'on')` / single-vector `plot`).
3. Consolidate the two plot paths onto ONE engine (`rustlab-plot::PlotRegistry`); remove the parallel desktop `GraphicsState` plot path (`execution.rs` GUI builtins) — no second plotting engine.
4. Figure tabs: one tab per Figure; subplot axes laid out inside the tab.
5. Deterministic SVG renderer (existing `svg.rs`) extended for 3D + colorbar; byte-equal snapshot tests.
6. Fix call-expression span bug (`parser.rs:712-716`) — separate commit S1 + regression test.
7. Docs + help + error-catalog entries for every new builtin (docs coverage test must stay green).
8. Tests S1–S12 per the milestone spec.

**OUT OF SCOPE:**

- Rewrite the app, interpreter, or plot engine.
- A second plotting engine (the desktop `GraphicsState` path is consolidated, not duplicated).
- Hardcoding outputs for test programs.
- GUI beyond figure-tab grouping (no egui shell rewrite).
- 3D interactive mouse handling in this phase's design beyond the view-model camera math (S7).

---

## 2. Ownership (adapted to real structs found — no parallel types)

| Concept | Real struct | file:line | Notes |
|---|---|---|---|
| FigureManager | `rustlab-plot::PlotRegistry` | `crates/rustlab-plot/src/state.rs:7-9`, `lib.rs:281-335` | `figures: HashMap<usize, Figure>`, `current_id: Option<usize>`, `next_id: usize`; thread-local `RefCell`. |
| Figure | `rustlab-plot::Figure` | `crates/rustlab-plot/src/lib.rs:243-249` | `id`, `title`, `axes: Vec<Axes>`, `current_axes: Option<usize>`, `visible`. |
| Axes | `rustlab-plot::Axes` | `crates/rustlab-plot/src/lib.rs:274-294` | `id,title,xlabel,ylabel,zlabel,xlim,ylim,zlim,xscale,yscale,zscale,grid,hold,colormap,series,texts,legend,colorbar,camera`. |
| Series | `rustlab-plot::Series` | `crates/rustlab-plot/src/lib.rs:41-49` | `Line/Scatter/Bar/Histogram`; **Surface added at `:137`**. |
| Session (desktop) | `execution::Session` | `apps/desktop/src/execution.rs:174` | owns `graphics: Arc<Mutex<GraphicsState>>` today — to be consolidated onto `rustlab-plot::PlotRegistry`. |
| GraphicsState (desktop, parallel engine to remove) | `execution::GraphicsState` | `apps/desktop/src/execution.rs:120` | `figures: Vec<FigureState>`, `current_figure`; flat `items` — no axes grid/hold/legend. |
| UI panel | `desktop::FiguresPanel` | `apps/desktop/src/figures.rs:7,24` | renders one tab per `FigureState` (`figures.rs:43-61`). |

**Ownership rule (adapted):** interpreter/CLI path already mutates `PlotRegistry` via `with_plot_state` (`state.rs:15`). Desktop `Session` (`execution.rs:174`) must use the SAME `rustlab-plot::PlotRegistry` functions for plotting builtins instead of writing to parallel `GraphicsState`. `FiguresPanel` reads figure state from `PlotRegistry` (or snapshot through `GraphicsState` for non-plot fields only). Extends `rustlab-plot`; does not create a parallel type.

> **UNVERIFIED:** whether `PlotRegistry` thread-local state leaks across `Session` lifetimes (`execution.rs:190`).

---

## 3. Model extensions (fields added to EXISTING structs only)

| Struct | New field | Purpose | Existing anchor |
|---|---|---|---|
| `Axes` | `zlabel: String` | 3D z-label | `lib.rs:279` |
| `Axes` | `position: (usize, usize, usize)` | subplot (row, col, span) | `lib.rs:275` |
| `Axes` | `kind: AxesKind { TwoD, ThreeD }` | 2D vs 3D axes | `lib.rs:275` |
| `Axes` | `camera: Option<Camera { azimuth, elevation }>` | 3D view | `lib.rs:293` |
| `Axes` | `color_cycle_index: usize` | hold color cycling | `lib.rs:287` |
| `Axes` | `colorbar: Option<Colorbar>` | current-axes colorbar | `lib.rs:292` |
| `Figure` | `layout: (usize, usize)` | subplot grid rows/cols | `lib.rs:392` |
| `Figure` | `dirty: bool` | tab dot indicator | `lib.rs:393` |
| `Series` | `Surface { … }` | surf quad mesh | `lib.rs:137-159` |

No new top-level types; `AxesKind`, `Camera`, `Colorbar` nested as fields on existing structs.

---

## 4. MATLAB semantics to implement (each with source or UNVERIFIED)

| Semantics | Source / status |
|---|---|
| `plot` with no figure creates Figure 1 and Axes 1 | UNVERIFIED (MATLAB doc not opened) |
| `hold on`: current axes only; subsequent `plot` appends series + cycles color order | UNVERIFIED |
| `hold off`: next `plot` replaces series | UNVERIFIED |
| `hold` alone toggles; `hold(ax,'on')` form; query via `ishold` | UNVERIFIED |
| `figure`: new figure, makes current; `figure(n)` selects/creates n | UNVERIFIED |
| `close`, `close all`, `gcf`, `gca`, `clf`, `cla` | UNVERIFIED |
| `subplot(m,n,p)`: select/create axes at grid cell in current figure; reuse same cell; never create figure unless none exists | UNVERIFIED |
| `subplot(mnp)` three-digit | UNVERIFIED |
| `title`/`xlabel`/`ylabel`/`legend`/`grid`/`hold` act on current axes only | UNVERIFIED |
| `grid on/off/toggle`: current axes only | UNVERIFIED |
| `legend(names...)`: current axes, series order | UNVERIFIED |
| `zlabel`, `view(az,el)`, `axis equal/tight/limits`, `colormap(name)`, `colorbar` (current axes), `shading flat|interp` | UNVERIFIED |
| `meshgrid` (1 or 2 vectors); X varies along columns | **VERIFIED** (`nd_ops::meshgrid`, `lib.rs:82`; `meshgrid.out`) |
| `peaks(n)` default n=49; `peaks(x,y)`; formula | **PARTIALLY** (`peaks(5)` → 5×5, contains 0.9810 = golden; formula UNVERIFIED vs Octave) |

**peaks formula:** `z = 3*(1-x).^2.*exp(-(x.^2)-(y+1).^2) - 10*(x/5 - x.^3 - y.^5).*exp(-x.^2-y.^2) - (1/3)*exp(-(x+1).^2+y.^2)`; `peaks(n) ~ meshgrid(linspace(-3,3,n))`; golden `peaks(0,0) = 0.9810` (4 d.p.).

**Colormap:** own perceptual colormap, generated by our code, documented construction, own name; do NOT copy MathWorks table (parula).

---

## 5. Rendering

- 2D SVG: `rustlab-plot::to_svg` (`svg.rs:16`) — deterministic, 2-decimal fmt, stable ids. Extend for 3D + colorbar; byte-equal snapshots.
- 3D: orthographic/perspective quad mesh + painter's algorithm (depth by quad centre); lives in `rustlab-plot` so SVG is deterministic/snapshot-testable; GUI (egui `FiguresPanel`) only draws.
- Tabs: one Figure = one tab (`figures.rs:43-61`); tab key = FigureId; subplot axes in tab; editor tab separate; execution never changes active tab unless user setting "focus figure on plot" on (default: dot on tab).

---

## 6. Events

Builtins mutate `rustlab-plot::PlotRegistry`, emit `FigureChanged(FigureId)`. UI reacts only to events. No UI calls from builtins.

---

## 7. Error texts and ids

| Id | Meaning | Source |
|---|---|---|
| `RUSTLAB:plot:badArgs` | wrong arg count | `plot_builtins.rs:59` |
| `RUSTLAB:plot:noFigure` | no current figure | `plot_builtins.rs:74` |
| `RUSTLAB:plot:emptyData` | X/Y mismatch | `plot_builtins.rs:67` |
| `RUSTLAB:plot:noAxes` | no current axes | `plot_builtins.rs:84` |
| surf size mismatch | `RUSTLAB:plot:badArgs: surf(X, Y, Z) requires X and Y to be Z's size (rows*cols).` | `plot_builtins.rs:752` (surf) — **UNVERIFIED vs MATLAB** |
| hold bad state | `RUSTLAB:plot:badArgs: hold requires 1 input ('on' or 'off').` | `plot_builtins.rs:185` |
| Call-expression span bug | `Expr::Index.span` starts at `(` → caret at `(` (Column 12 for `nonexistent(1,2)`; Column 6 for `surf(x,y,z)`) | `parser.rs:712-716`; `docs/plot-int/nonexistent.out` |

---

## 8. Test plan and file list (from PLOT-0)

| Step | File(s) | Test |
|---|---|---|
| S1 | `crates/rustlab-parse/src/parser.rs` | regression: `z = peaks(x, y);` caret at correct column |
| S2 | `crates/rustlab-plot/src/lib.rs` | plot twice/no-hold→1 series; hold→2; hold off→replace; hold toggle; ishold; per-axes hold (two subplots) |
| S3 | `crates/rustlab-stdlib/src/plot_builtins.rs`, help, builtins.txt, error catalog | hold, ishold, gcf, gca, clf, cla, close, figure(n), subplot, zlabel, grid per axes |
| S4 | `apps/desktop/src/*` | tab grouping view-model tests (Scenarios 1-3) |
| S5 | `crates/rustlab-stdlib/src/nd_ops.rs` | meshgrid 31×31; peaks golden; surface data |
| S6 | `crates/rustlab-plot/src/svg.rs` | 3D SVG + colorbar + snapshot |
| S7 | `crates/rustlab-plot/src/svg.rs`, `apps/desktop/src/figures.rs` | camera view-model test |
| S8 | `apps/desktop/src/*`, editor | editor/graph independence |
| S9 | GUI action layer + screenshots | Scenarios 1-6 |
| S10 | regression tests | tests 19/20 still pass; count increases; manifest updated |
| S11 | perf | surf 201×201 ≥30 fps release |
| S12 | REA | our own release binary |

---

## 9. Risks

- Consolidating desktop `GraphicsState` onto `PlotRegistry` is cross-crate (desktop ↔ plot) — large surface.
- 3D SVG rendering with colorbar/colormap determinism (snapshot stability).
- Thread-local `PlotRegistry` lifetime across sessions (UNVERIFIED).
- Feature flags must not change core semantics (R13).

---

*End of PLOT-1 design note. Phase PLOT-1 = design only, NO code changes.*

---

**STOPPED at end of Phase PLOT-1 — waiting for owner approval of the PLOT-2 implementation plan.**

*(Next action per milestone procedure: PLOT-2 implementation begins with S1 span fix + regression test.)*
