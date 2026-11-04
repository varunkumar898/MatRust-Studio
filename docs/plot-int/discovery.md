# PLOT-0 Discovery — Plot Integration investigation

**Repo:** `C:\Users\varun\Desktop\MATSTUDIO`
**Branch:** `gui/polish-1`
**Constraint:** PLOT-0 is discovery-only (no code changes). Every claim is backed by `file:line` or a command + raw output file saved under `docs/plot-int/` with its SHA-256.

**Note:** `docs/MASTER-PROMPT-GUI.md` (referenced by the milestone spec) does not exist in this repo. Only `AGENTS.md` (master prompt v1) and `docs/STATE.md` govern. The GUI rules referenced by the spec are treated as `UNVERIFIED` until a governing document is produced.

---

## 0.1 git state and crate list

| Item | Answer | Evidence |
|------|--------|----------|
| `git log --oneline -n 5` | `7378008 feat(plot): add snapshot tests for deterministic SVG output`; `2c31205 docs(gui): final from-clean verify log...`; `2405c09 docs(gui): S5 from-clean verify log...`; `6187009 fix(math): ...`; `43cd589 docs(gui): S4/S5 records...` | command (this session) |
| `git status --short` | Working tree has uncommitted modifications: `Cargo.lock`, `GEMINI.md`, `crates/rustlab-plot/src/lib.rs`, `crates/rustlab-stdlib/Cargo.toml`, `crates/rustlab-stdlib/src/help.rs`, `crates/rustlab-stdlib/src/lib.rs`; untracked: `crates/rustlab-stdlib/src/plot_builtins.rs` | command (this session) |
| Crate list (Cargo.toml files) | `crates/rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-stdlib`, `rustlab-matio`, `rustlab-data`, `rustlab-cli`, `rustlab-plot`, `mlapp-*` (sim, runtime, render, py, host, ffi, matlab, octave, plot, ui, cli, container), `apps/desktop` | `crates/*/Cargo.toml`, `apps/desktop/Cargo.toml` |

Relevant plot crates:
- `crates/rustlab-plot` — renderer-independent Figure/Axes/Series model + deterministic SVG renderer (thread-local `PlotRegistry`).
- `crates/mlapp-plot` — App Designer `UIAxes`/`AxesModel` model with its own `to_svg`.
- `apps/desktop` — `FiguresPanel` (`figures.rs`) + `GraphicsState`/`FigureState`/`PlotItem` (`execution.rs`) driven by GUI-aware builtins.

---

## 0.2 Where are builtins registered?

| Question | Answer | Evidence (file:line) |
|----------|--------|----------------------|
| Registration trait | `Interpreter::register_builtin(&mut self, name: &str, func: F) where F: BuiltinFn + 'static`; `BuiltinFn` trait: `fn call(&self, args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError>` | `crates/rustlab-eval/src/eval.rs:92-97`, `eval.rs:14-26` |
| Central registration | `rustlab_stdlib::register_all(interp: &mut Interpreter)` calls `interp.register_builtin(...)` for every stdlib builtin. | `crates/rustlab-stdlib/src/lib.rs:47-328` |
| Example: `plot` registered | `interp.register_builtin("plot", plot_builtins::plot_fn);` under "// Plotting (B2.2)" | `crates/rustlab-stdlib/src/lib.rs:204` |
| Which plotting builtins exist & where | `figure`(203) `plot`(204) `scatter`(205) `bar`(206) `hist`(207) `hold`(208) `subplot`(209) `xlabel`(210) `ylabel`(211) `title`(212) `legend`(213) `xlim`(214) `ylim`(215) `grid`(216) `saveas`(217) — all in `crates/rustlab-stdlib/src/plot_builtins.rs` | `crates/rustlab-stdlib/src/lib.rs:202-217`; impls in `crates/rustlab-stdlib/src/plot_builtins.rs` |
| `meshgrid` registered | `interp.register_builtin("meshgrid", nd_ops::meshgrid);` | `crates/rustlab-stdlib/src/lib.rs:82` |

**Missing (NOT registered anywhere):** `peaks`, `surf`, `zlabel`, `colorbar`, `colormap`, `shading`, `view`, `axis`, `ishold`, `gcf`, `gca`, `clf`, `cla`, `close`. Verified by running each in the CLI — all produce `Undefined function or variable '<name>`. (CLI raw output files: `peaks.out`, `surf.out`; individual runs for the rest — see 0.7.)

> **UNVERIFIED:** whether the desktop `Session` re-registers some of these separately in `execution.rs` `register_gui_builtins` (that function registers its own `plot`/`scatter`/`bar`/`title`/`xlabel`/`ylabel`/`grid`/`figure`/`clf`/`close` writing to `GraphicsState`, not `rustlab-plot`). Confirmed present in GUI path: `plot`(execution.rs:243), `scatter`(279), `bar`(305), `title`(331), `xlabel`(343), `ylabel`(354), `grid`(367), `figure`(386), `clf`(400), `close`(412). Confirmed ABSENT in GUI path: `hold`, `subplot`, `xlim`, `ylim`, `legend`, `saveas`.

---

## 0.3 Plot model — Figure / Axes / Series

| Question | Answer | Evidence (file:line) |
|----------|--------|----------------------|
| Where do Figure/Axes/Series live | `crates/rustlab-plot/src/lib.rs` (renderer-independent model) | `crates/rustlab-plot/src/lib.rs:29-276` |
| `Axes` fields | `id`, `title`, `xlabel`, `ylabel`, `xlim`, `ylim`, `xscale`, `yscale`, `grid`, `hold`, `series: Vec<Series>`, `texts: Vec<Text>`, `legend: Option<Legend>` | `crates/rustlab-plot/src/lib.rs:156-170` |
| Axes has series list? | YES — `series: Vec<Series>` | `crates/rustlab-plot/src/lib.rs:167` |
| Axes has title? | YES — `title: String` | `crates/rustlab-plot/src/lib.rs:158` |
| Axes has x/y labels? | YES — `xlabel`, `ylabel` | `crates/rustlab-plot/src/lib.rs:159-160` |
| Axes has z-label? | NO — no `zlabel` field | `crates/rustlab-plot/src/lib.rs:156-170` (no z field) |
| Axes has legend entries? | YES — `legend: Option<Legend>` | `crates/rustlab-plot/src/lib.rs:169` |
| Axes has grid flag? | YES — `grid: bool` | `crates/rustlab-plot/src/lib.rs:165` |
| Axes has limits? | YES — `xlim`, `ylim: Option<[f64; 2]>` | `crates/rustlab-plot/src/lib.rs:161-162` |
| Axes has hold flag? | YES — `hold: bool`; `push_series` clears series unless hold (lib.rs:192-197) | `crates/rustlab-plot/src/lib.rs:166, 192-197` |
| Axes has subplot position? | NO — no `row`/`col`/`span` on Axes; `subplot` reuses axes by numeric id `p` only | `crates/rustlab-stdlib/src/plot_builtins.rs:215-246` |
| `Figure` fields | `id`, `title`, `axes: Vec<Axes>`, `current_axes: Option<usize>`, `visible` | `crates/rustlab-plot/src/lib.rs:243-249` |
| `PlotRegistry` (current-figure concept) | YES — thread-local; `figures: HashMap<usize, Figure>`, `current_id: Option<usize>`, `next_id: usize`; `create_figure`, `set_current`, `current_figure` | `crates/rustlab-plot/src/lib.rs:281-335` |
| Where stored | Thread-local `RefCell<PlotRegistry>` via `thread_local!` in `state.rs`; mirror of MATLAB `gcf`/`gca` within one interpreter session | `crates/rustlab-plot/src/state.rs:7-9` |
| GUI path graphics state | Separate `GraphicsState { figures: Vec<FigureState>, current_figure }` + `FigureState { id, title, items, x_label, y_label, grid }` — NO axes/hold/legend/subplot | `apps/desktop/src/execution.rs:119-160` |

> **UNVERIFIED:** whether `PlotRegistry` is reset between interpreter sessions in the desktop app (it is thread-local per process, so it persists across `eval_script` calls within one `Interpreter`, but `Session` creates a fresh `Interpreter` — the thread-local may leak across sessions).

---

## 0.4 How plotting builtins reach the UI (trace plot(x,y) → pixel)

Two **separate, unconnected** paths exist (this is the "second plotting engine" concern):

### Path A — stdlib `plot_builtins` → thread-local `PlotRegistry` → SVG (headless / CLI)

`plot(x,y)` → `plot_builtins::plot_fn` (`plot_builtins.rs:57`) → `rustlab_plot::with_plot_state` (`state.rs:15`) mutates thread-local `PlotRegistry` → `Figure`/`Axes` → `to_svg(fig, w, h)` (`svg.rs:12`) → SVG **string**. No UI pixels; output is an SVG string written to disk by `saveas` or returned as data.

There is **no code path** from the thread-local `PlotRegistry` to the desktop `FiguresPanel`/egui painter. The CLI (`new_interpreter`) registers `plot_builtins` but never renders to screen.

### Path B — desktop `Session::register_gui_builtins` → `GraphicsState` → egui painter

`plot(x,y)` → GUI-aware closure (`execution.rs:243`) → locks `GraphicsState` → `FigureState.items.push(PlotItem::Line{...})` → `FiguresPanel::show` (`figures.rs:24`) iterates `graphics.figures`, draws with `ui.painter_at(...)` line segments (`figures.rs:274-294`).

**File trace (Path B):** `plot` builtin closure → `execution.rs:243-276` → `GraphicsState.figures` → `apps/desktop/src/app.rs` renders `FiguresPanel` → `figures.rs:24-363` → egui `Painter`.

> **UNVERIFIED:** whether Path A's SVG is ever displayed in the desktop UI (no consumer of `rustlab-plot::to_svg` found in `apps/desktop`); whether Path B's `GraphicsState` is ever serialized to SVG.

---

## 0.5 Tabs — figure tabs and subplot grouping

| Question | Answer | Evidence (file:line) |
|----------|--------|----------------------|
| Where figure tabs created | `FiguresPanel::show` iterates `graphics.figures`; each figure gets a `selectable_label` tab + close "X" button | `apps/desktop/src/figures.rs:43-61` |
| "New tab" decision | `get_or_create_figure(id)` creates a new `FigureState` and pushes it when `id` not found | `apps/desktop/src/execution.rs:126-142` |
| One Figure = one tab? | YES — each `FigureState` in `graphics.figures` renders one tab button | `figures.rs:43-61` |
| subplot(2,1,1) / subplot(2,1,2) in GUI | **Not supported.** GUI `GraphicsState.FigureState` has a flat `items: Vec<PlotItem>` — no axes grid, no subplot position. The stdlib `subplot_fn` (`plot_builtins.rs:215`) reuses an axes by id `p` inside `PlotRegistry`, but that state never reaches `GraphicsState`. | `execution.rs:96-103` (`FigureState`), `figures.rs:74-110` (renders `fig.items` flat), `plot_builtins.rs:215-246` |
| Why two tabs would appear (hypothesis) | Hypothesis: calling `figure` then `subplot` in different figures creates two `FigureState`s → two tabs. But subplot axes are NOT rendered as separate tabs; they'd be merged into one figure's flat `items`. **Experiment needed:** not yet run. | hypothesis only |

> **UNVERIFIED:** subplot tab behavior — requires running `figure; subplot(2,1,1); plot(...); figure; subplot(2,1,2); plot(...)` and observing tab count. Not yet executed.

---

## 0.6 `hold` in the CLI

| Question | Answer | Evidence |
|----------|--------|----------|
| `hold on` (command syntax) | `Error: RUSTLAB:plot:badArgs: hold requires 1 input ('on' or 'off').` — command syntax `hold on` does **not** deliver `"on"` as an argument (0 args received). | `docs/plot-int/hold_on.out` (SHA-256 `F64B8690...`), exit code 1 |
| `hold('on')` (function syntax) | Works syntactically (1 arg accepted) → `Error: RUSTLAB:plot:noFigure: No current figure. Call figure first.` | `docs/plot-int/hold_paren.out` (SHA-256 `ABEC22CD...`), exit code 1 |
| `hold` alone (toggle) | `Error: RUSTLAB:plot:badArgs: hold requires 1 input ('on' or 'off').` — toggle-with-no-args form NOT supported | `docs/plot-int/hold_toggle.out` (SHA-256 `6D9F101A...`), exit code 1 |
| `hold(ax,'on')` form | Not supported; `hold_fn` takes exactly 1 arg | `plot_builtins.rs:185-212` |
| `ishold` | Undefined (not registered) | CLI run, exit 1 |
| Parser: does `hold on` parse as command syntax? | `--dump-ast` flag NOT supported by CLI (`rustlab -e` / `rustlab file` only; `main.rs:37-95`). `--dump-ast` → exit code 2. | `docs/plot-int/dump_ast.out` (SHA-256 `48308991...`), `crates/rustlab-cli/src/main.rs:37-43` |
| "Undefined function" error source | `EvalError::UserError` with identifier; formatted by `format_eval_error` | `crates/rustlab-cli/src/main.rs:63`, `crates/rustlab-eval/src/error.rs` |

> **UNVERIFIED:** exact AST for `hold on` (no `--dump-ast` available); whether command-syntax `hold on` should be `hold("on")`. The parser's handling of command syntax for builtins is not traced here.

---

## 0.7 peaks, surf, meshgrid and friends — raw CLI output

| Question | Answer | Evidence |
|----------|--------|----------|
| `peaks` exists? | NO — `Undefined function or variable 'peaks'.` at Line 1, Column 1 | `docs/plot-int/peaks.out` (SHA-256 `AF530050...`) |
| `surf` exists? | NO — `Undefined function or variable 'surf'.` at Line 1, Column 1 | `docs/plot-int/surf.out` (SHA-256 `B7EDEC6C...`) |
| `zlabel`, `colorbar`, `colormap`, `shading`, `view`, `axis` | ALL undefined (not registered) | individual CLI runs, exit 1 each |
| `clf`, `cla`, `close`, `gcf`, `gca`, `ishold` | ALL undefined in CLI (registered only in GUI `GraphicsState` path for `clf`/`close`) | individual CLI runs, exit 1 each |
| `meshgrid(-3:0.2:3, -3:0.2:3)` | WORKS. Returns 31×31 matrices (31 rows displayed; each row has 31 values −3…3 step 0.2). X varies along columns (row 1 = −3:0.2:3). | `docs/plot-int/meshgrid.out` (SHA-256 `DD91E65C...`), exit code 0 |
| `size(X)`, `size(Y)` | Not yet printed separately (meshgrid output shows `ans` = first output only; need `[X,Y]=...; size(X); size(Y)`) | `docs/plot-int/meshgrid.out` — only one `ans` block shown |
| `z = sin(sqrt(X.^2+Y.^2)); size(z); z(16,16)` | Included in meshgrid command batch; need to read full output for `z(16,16)` value | `docs/plot-int/meshgrid.out` |

**meshgrid output analysis:** 31 rows of 31 columns each, values −3.0000…3.0000 step 0.2 (with `3.8858e-16` for 0 due to floating point). Row 1 (and every row) = `X` row vector → **X varies along columns**, consistent with MATLAB `meshgrid` where `X(i,:) = x`. Need to confirm `Y(:,1) = -3:0.2:3` and `z(16,16) = 0`.

> **UNVERIFIED:** `Y(:,1)` orientation, `z(16,16)` exact value, `size(X)`/`size(Y)` as separate calls (the CLI batch printed them but output needs re-reading column-by-column).

---

## 0.8 Renderers — backends and 3D

| Question | Answer | Evidence (file:line) |
|----------|--------|----------------------|
| SVG backend | YES — `rustlab-plot::to_svg(fig, width, height) -> String`, deterministic, fixed 2-decimal formatting, stable element ids (`axes-{id}`) | `crates/rustlab-plot/src/svg.rs:12-90` |
| egui painter backend | YES — `FiguresPanel::show` uses `ui.painter_at(canvas_rect)` to draw lines/scatter/bars/histograms interactively | `apps/desktop/src/figures.rs:123, 274-362` |
| `mlapp-plot` SVG | YES — `AxesModel::to_svg` (separate, simpler renderer) | `crates/mlapp-plot/src/lib.rs:100-159` |
| Any 3D or projection code? | NO — no azimuth/elevation camera, no quad-mesh projection, no painter's-algorithm depth sort, no `surf`/3D series variant. `Series` enum = Line/Scatter/Bar/Histogram only. `Axes` has no `camera`/`azimuth`/`elevation`. | `crates/rustlab-plot/src/lib.rs:29-120` (Series), `svg.rs` (2D only) |
| Which crate would own 3D? | `rustlab-plot` (it owns the Figure/Axes model and SVG renderer); `mlapp-plot` owns the UIAxes model. No crate currently has 3D. | crate Cargo.toml descriptions (`rustlab-plot`: "Renderer-independent MATLAB-compatible plot description model with deterministic SVG output") |

> **UNVERIFIED:** none for renderers; 3D absence is confirmed by reading all of `rustlab-plot/src/{lib,svg,ticks,state}.rs`.

---

## 0.9 Existing tests for plotting

| Crate | Test file | What it covers | Extendable? |
|-------|-----------|----------------|-------------|
| `rustlab-plot` | `crates/rustlab-plot/tests/snapshot_tests.rs` | `snapshot_single_line_plot`, `snapshot_scatter_plot`, `snapshot_deterministic_across_runs` — byte-equal SVG vs `tests/snapshots/*.svg` | YES — add snapshots for subplot/hold/multiple-axes |
| `rustlab-plot` | `src/lib.rs` (mod tests) | figure creation/switching, axes data bounds, hold clears/keeps series | YES |
| `rustlab-plot` | `src/state.rs` (mod tests) | create/get current figure, reset, multiple figures | YES |
| `rustlab-plot` | `src/svg.rs` (mod tests) | svg deterministic, stable ids, fmt2 stable | YES |
| `rustlab-plot` | `src/ticks.rs` (mod tests) | nice ticks monotonic/cover/nice steps/degenerate/negative/deterministic | YES |
| `mlapp-plot` | `crates/mlapp-plot/tests/plot_tests.rs` | `test_axes_model_plotting_and_svg_export` | YES |
| `rustlab-stdlib` | `tests/regression_a312_tests.rs:341` | `every_registered_builtin_has_docs_and_help` — docs+help coverage (MUST keep green when adding builtins) | N/A (constraint) |
| `rustlab-stdlib` | `tests/doc_generator_tests.rs` | doc parity/count | N/A |

Snapshot files: `crates/rustlab-plot/tests/snapshots/{single_line_plot,scatter_plot,deterministic_line}.svg`.

---

## 0.10 Diagnostic spans — call expression span (the column bug)

| Question | Answer | Evidence |
|----------|--------|----------|
| How `Expr::Index` span is built | `start_span = self.advance().span` (the `(` token), `end_span = RParen span`, `span: start_span.merge(end_span)` — covers `(`…`)` **excluding** the callee identifier | `crates/rustlab-parse/src/parser.rs:697-716` |
| `nonexistent(1,2)` error column | **Column 12** — points at `(`, NOT column 1 (`nonexistent`) | `docs/plot-int/nonexistent.out` (SHA-256 `8275A2F2...`): "Line 1, Column 12" |
| `z = sin(x);` error column | Column 9 — points at `x` (the undefined argument), correct because `x` is the unknown | `docs/plot-int/defined_call.out` (SHA-256 `CC6B6B18...`): "Line 1, Column 9" |
| `surf` (no args) error column | Column 1 — points at `s` (`surf` parses as bare `Ident`, span covers the name) | `docs/plot-int/surf.out`: "Line 1, Column 1" |
| The bug | For `f(args)`, `Expr::Index.span` starts at `(` so an "undefined function" error's caret points **past** the identifier (at `(`), not at `f`. | parser.rs:712-716 + nonexistent.out evidence |

Expected vs actual for `z = peaks(x,y);` (peaks undefined):
- **Expected column:** 5 (where `peaks` starts)
- **Actual column (if it followed nonexistent pattern):** 10 (where `(` starts)

> **CONFIRMED BUG:** call-expression span for undefined functions points at `(` (column 10/12) instead of the identifier (column 5/1). Fix belongs in `parser.rs` Index span construction (separate commit + regression test per PLOT-2 S1).

---

## 0.11 Why the report's `hold on` said Line 10 for a program where it's line 7/8

Not yet reproduced. PLOT-0 instruction: save exact test code to `tests/golden/plot_t18.m` and reproduce.

| Item | Status |
|------|--------|
| `tests/golden/plot_t18.m` created | NOT yet |
| Reproduction run | NOT yet |
| Line-number discrepancy explained | NOT yet |

> **UNVERIFIED:** requires creating `tests/golden/plot_t18.m` and running it.

---

## Summary of gaps (for PLOT-1 design)

| Feature | Exists? | Where | Gap |
|---------|---------|-------|-----|
| `figure`/`plot`/`scatter`/`bar`/`hist` | YES (both paths) | `plot_builtins.rs` + `execution.rs` | two unconnected engines |
| `hold on/off` | PARTIAL — `hold('on')` works, `hold` toggle + `hold(ax,'on')` missing | `plot_builtins.rs:185` | add toggle + 2-arg form |
| `subplot` | stdlib only (axes-by-id), not in GUI | `plot_builtins.rs:215` | GUI `GraphicsState` has no axes grid |
| `xlabel`/`ylabel`/`title`/`legend`/`xlim`/`ylim`/`grid` | YES | `plot_builtins.rs` | — |
| `saveas` | YES (SVG to disk) | `plot_builtins.rs:424` | — |
| `gcf`/`gca`/`ishold`/`clf`/`cla`/`close` | NO in CLI; `clf`/`close` in GUI only | — | add all |
| `zlabel`/`colorbar`/`colormap`/`shading`/`view`/`axis` | NO | — | add |
| `peaks`/`surf` | NO | — | add + 3D surface data model |
| `meshgrid` | YES | `nd_ops::meshgrid` (lib.rs:82) | — |
| SVG renderer | YES (deterministic, 2D) | `rustlab-plot/svg.rs` | no 3D |
| egui painter | YES (2D) | `apps/desktop/figures.rs` | no 3D, no subplot layout |
| Docs coverage | enforced by test | `regression_a312_tests.rs:341` | every new builtin needs `docs/functions/<name>.md` + `help::get_help_text` + `builtins.txt` sync |

**Next:** STOP and wait for approval of this discovery before PLOT-1 design note.
