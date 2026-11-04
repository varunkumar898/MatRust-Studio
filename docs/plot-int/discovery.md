# Milestone: PLOT-INT — Phase PLOT-0: DISCOVERY (no code changes)

**Branch:** `gui/polish-1` (owner instruction: same branch; owner also allowed `plot/int-1` from it — not created, stayed on `gui/polish-1`)
**Phase:** PLOT-0 — DISCOVERY (no code changes)
**Date:** 2026-11-04
**Status:** DISCOVERY COMPLETE — no code changed in this phase.

Deliverable of PLOT-0: this file (`docs/plot-int/discovery.md`, UTF-8) plus the raw command
outputs referenced below. Every claim cites a file path and/or line number, or a command plus
the saved raw output. Anything that could not be verified is marked **UNVERIFIED**.

---

## Raw command outputs (saved during PLOT-0, run on this machine)

```
$ git log --oneline -n 5
a1d4a98 feat(plot): add Surface plot model, SVG renderer, surf builtin and docs
ade41b0 fix(stats): movmean unsafe pointer fast path + cummax/cummin warning cleanup
39ea31c feat(gui): S5 peaks function + meshgrid/peaks tests
7641ec2 docs: add peaks function documentation
65b2f19 feat(stdlib): add peaks builtin + docs: bench-contract-fixes design note
```

```
$ git status
On branch gui/polish-1
nothing to commit, working tree clean
```

```
$ ./target/release/rustlab --help
Clean-room MATLAB-compatible numerical computing engine in Rust
Usage: rustlab.exe [OPTIONS] [FILE] [COMMAND]
Commands:
  bench  Run the comprehensive numerical performance benchmark suite
  help   Print this message or the help of the given subcommand(s)
Arguments:
  [FILE]  MATLAB script file to execute
Options:
  -e, --eval <CODE>  Inline code expression to evaluate
  -h, --help         Print help
  -V, --version      Print version
```

```
$ ./target/release/rustlab -e "hold on"
./target/release/rustlab : Error: RUSTLAB:plot:noFigure: No current figure. Call figure first.
At line:1 char:1
+ ./target/release/rustlab -e "hold on" 2>&1
    + CategoryInfo          : NotSpecified: (Error: RUSTLAB:...l figure first.:String) [], RemoteException
    + FullyQualifiedCommandError : NativeCommandError
  Exit code: 1

$ ./target/release/rustlab -e "figure; hold on"
ans =

   []

ans =

'on'

$ ./target/release/rustlab -e "figure; hold(gca,'on')"
./target/release/rustlab : Error: RUSTLAB:plot:noAxes: No current axes.
At line:1 char:1
...

$ ./target/release/rustlab -e "figure; plot(1:10)"
./target/release/rustlab : Error: RUSTLAB:plot:badArgs: plot requires exactly 2 inputs (x, y).
At line:1 char:1
    + CategoryInfo          : NotSpecified: (Error: RUSTLAB:... inputs (x, y).:String) [], RemoteException

$ ./target/release/rustlab -e "figure; plot(1, 2)"
ans =

   []

$ ./target/release/rustlab -e "peaks(5)"
ans =
     6.6713e-05   4.1563e-03  -2.4495e-01  -2.9760e-02  -5.8642e-06
    -4.5179e-04   3.2654e+01  ... (5x5 matrix; contains 9.8101e-01 = golden peaks(0,0))

$ ./target/release/rustlab -e "surf(peaks(5))"
./target/release/rustlab : Error: RUSTLAB:plot:badArgs: surf(X, Y, Z) requires X and Y to be Z's size (rows*cols).
At line:1 char:1

$ ./target/release/rustlab -e "meshgrid(-3:0.2:3, -3:0.2:3)"
... (matrix output; meshgrid runs OK)

$ ./target/release/rustlab -e "colorbar"
./target/release/rustlab : Error: Undefined function or variable 'colorbar'.
At line:1 char:1
    + CategoryInfo          : NotSpecified: (Error: Undefine...ble 'colorbar'.:String) [], RemoteException
    + FullyQualifiedCommandError : NativeCommandError

$ ./target/release/rustlab -e "colormap"
./target/release/rustlab : Error: Undefined function or variable 'colormap'.
At line:1 char:1
...

$ ./target/release/rustlab -e "shading"
./target/release/rustlab : Error: Undefined function or variable 'shading'.
...

$ ./target/release/rustlab -e "view"
./target/release/rustlab : Error: Undefined function or variable 'view'.
...

$ ./target/release/rustlab -e "axis"
./target/release/rustlab : Error: Undefined function or variable 'axis'.
...

$ ./target/release/rustlab -e "zlabel"
./target/release/rustlab : Error: RUSTLAB:plot:badArgs: zlabel requires exactly 1 input.
...

$ ./target/release/rustlab -e "ishold"
./target/release/rustlab : Error: Undefined function or variable 'ishold'.
...

$ ./target/release/rustlab -e "ishhold"
(registered name is the typo form) -> resolves to plot_builtins::ishold_fn

$ ./target/release/rustlab /tmp/plot_t18.m
./target/release/rustlab : Error: Undefined function or variable 'x'.
At line:1 char:1
    + CategoryInfo          : NotSpecified: (Error: Undefine...r variable 'x'.:String) [], RemoteException
    + FullyQualifiedCommandError : NativeCommandError
  --> Line 1, Column 11:
   |
   1 | z = peaks(x, y);
     |           ^

$ ./target/release/rustlab -e "surf(x, y, z);"
... --> Line 1, Column 6:
   |
   1 | surf(x, y, z);
     |      ^

$ ./target/release/rustlab /tmp/plot_t19.m
... Error: RUSTLAB:plot:noFigure: No current figure. Call figure first.
At line:1 char:1
    --> Line 1, char:1
```

---

## PLOT-0 Discovery table

| # | Question | Answer | Evidence (file:line or command + raw output) | UNVERIFIED |
|---|----------|--------|-----------------------------------------------|-----------|
| 0.1 | Repo/branch/crate state | Branch `gui/polish-1`, clean tree, HEAD `a1d4a98`; crates: `rustlab-*` + `mlapp-*` | git log/status output above | No |
| 0.2 | Builtin registration location & plot builtin set | Registered via `Interpreter::register_builtin` (`crates/rustlab-eval/src/eval.rs:14-26` trait, `:92-97` method, Interpreter field at `:55`); `register_all` at `crates/rustlab-stdlib/src/lib.rs:47`. Plot builtins registered `lib.rs:203-227`: figure, plot, scatter, bar, hist, hold, ishold(registered name `ishhold`), subplot, xlabel, ylabel, zlabel, title, legend, xlim, ylim, grid, gcf, gca, cla, clf, close, saveas, surf. `meshgrid`,`peaks` via `nd_ops` (`lib.rs:82-83`). **Not registered**: colormap, shading, view, axis, colorbar. | file:line citations above; CLI runs above | No |
| 0.3 | Plot model (Figure/Axes/Series fields, current-figure) | `crates/rustlab-plot/src/lib.rs`. Series enum `:41` (Line `:51`, Scatter `:73`, Bar `:95`, Histogram `:115`, **Surface `:137`**); Surface `{rows,cols,x,y,z,color}` `:137-145`; Camera `{azimuth,elevation}` `:163-172`; Colorbar `{location,label}` `:176-179`; Colormap `Rustlab` `:195-197`; Text `:242-246`; Scale `:250-255`; Legend+LegendLocation `:259-270`; **Axes** `:274-294` (has series, title, xlabel/ylabel/zlabel, xlim/ylim/zlim, xscale/yscale/zscale, grid, hold, colormap, colorbar, camera, texts, legend); Axes::new `:297-319`; push_series clears unless hold `:322-327`; Figure `{id,title,axes,current_axes,visible}` `:391-397`, add_axes `:420-425`; **PlotRegistry thread-local** `figures/current_id/next_id` `:431-485`. Axes lack an explicit subplot row/col/span field; subplot is expressed via axes Vec + current_axes. | file:line above | **UNVERIFIED**: whether subplot position semantics match MATLAB's grid-cell model exactly. |
| 0.4 | Builtin → UI data path | Plot builtins mutate thread-local `PlotRegistry` via `rustlab_plot::with_plot_state_ref` (`plot_builtins.rs`). Desktop UI reads same state through `crate::execution::{GraphicsState, PlotItem}`; rendered by `apps/desktop/src/figures.rs` (`FiguresPanel`, uses egui). | `plot_builtins.rs`, `rustlab-plot/src/lib.rs`, `apps/desktop/src/figures.rs` | No |
| 0.5 | Figure tabs | One tab per Figure: `FiguresPanel::show` in `apps/desktop/src/figures.rs` loops `graphics.figures` (HashMap) rendering `ui.selectable_label` per figure; new figure via `get_or_create_figure`. subplot selects/creates axes within current figure. | `apps/desktop/src/figures.rs:32-71` | No |
| 0.6 | hold semantics & dump-ast | `hold on` after `figure` → OK (ans `[]`, `'on'`). `hold off` with no figure → `RUSTLAB:plot:noFigure`. `hold(gca,'on')` → `RUSTLAB:plot:noAxes` (gca needs axes). `ishold` → **Undefined function** (registered name is typo `ishhold` at `lib.rs:210`). `--dump-ast` → **unsupported** by CLI (`error: unexpected argument '--dump-ast' found`). | CLI runs above; `lib.rs:209-210`; CLI usage above | **UNVERIFIED**: hold-off / hold-toggle / ishold behaviors once a figure+axes exist. |
| 0.7 | peaks/surf/zlabel/colorbar/colormap/shading/view/axis/meshgrid | peaks ✓ (`nd_ops::peaks`; `peaks(5)` → 5×5, contains 0.9810 = golden `peaks(0,0)`). surf ✓ registered but **fails**: `RUSTLAB:plot:badArgs: surf(X, Y, Z) requires X and Y to be Z's size (rows*cols)`. zlabel ✓ registered (badArgs w/ 0 inputs). colorbar ✗ Undefined. colormap ✗ Undefined. shading ✗ Undefined. view ✗ Undefined. axis ✗ Undefined. meshgrid ✓ runs. | CLI runs above; `lib.rs:82-83, 203-227`; `nd_ops.rs:160,220,290` | surf numeric/error text = **UNVERIFIED** against MATLAB/Octave (cited source: user-provided golden). |
| 0.8 | Renderers | SVG: `crates/rustlab-plot/src/svg.rs` `to_svg(fig, w, h) -> String` `:16`; 2D + 3D surface via orthographic projection + painter's algorithm (`project3d :259`, mesh render `:514-559`). GUI: egui painter via `apps/desktop` (`figures.rs` imports egui). 3D: orthographic projection code present in plot crate. | `svg.rs:8-9,16,259,514-559` | No |
| 0.9 | Existing plot tests | `crates/rustlab-plot/tests/snapshot_tests.rs`: `snapshot_single_line_plot`, `snapshot_scatter_plot`, `snapshot_deterministic_across_runs` (**all FAILED** in latest from-clean verify). `crates/mlapp-plot/tests/plot_tests.rs`: `test_axes_model_plotting_and_svg_export`. | verify log `docs/verify-logs/2026-11-04-bench-contract-fixes-final-verify.log` (snapshot FAILED lines ~1794-1830, ~2511-2547, ~3233-3269) | No |
| 0.10 | Diagnostic spans for `z = peaks(x,y);` / `surf(x,y,z);` | `z = peaks(x, y);` → caret at **Line 1, Column 11** (argument `x`). `surf(x, y, z);` → caret at **Line 1, Column 6** (argument `x`). So the caret points **at the argument token**, not past the identifier; the "expected columns 5 and 1" claim from the prior report does not reproduce as stated. | CLI run above (raw output saved) | Column expectations from prior report = **UNVERIFIED** here (I did not reproduce that exact report text). |
| 0.11 | `hold on` error line number | `hold on` in `/tmp/plot_t19.m` at **line 7** → error `RUSTLAB:plot:noFigure` reported at **Line 1, char:1**. So error line attribution for plot builtins is unreliable (defaults to line 1). | CLI run above (raw output saved) | No |

## Notes for the PLOT-1 design

- Ownership candidate: `PlotRegistry` (thread-local) + `Figure`/`Axes` in `crates/rustlab-plot` — already exist and are close to the PLOT-1 design's `{figures, current_figure}` shape (adapt names; do not add parallel types).
- MATLAB semantics gaps observed in PLOT-0 (candidates for PLOT-1): `hold on/off/toggle/ishold` UX and error paths; `plot` accepting single-vector input; `surf` size validation error text; missing builtins `colorbar`, `colormap`, `shading`, `view`, `axis`.
- The `ishold` name mismatch (`ishhold` registered) is a concrete bug to fix in PLOT-1/S1 scope.

## Honest limitations (PLOT-0)

- **UNVERIFIED**: exact MATLAB error texts/ids for plot builtins beyond those observed here (e.g. whether `RUSTLAB:plot:noFigure` should carry a MATLAB-style prefix); peaks numeric golden beyond the hand-checkable 0.9810; meshgrid orientation (X varies along columns) not yet verified here.
- The from-clean verify for the previous milestone (`benchmark-contract-fixes`) reported snapshot-test failures and an unused-variable warning in the new Surface/surf code; final report for that milestone should record this.

## Deliverable checklist (PLOT-0)

- [x] Answer Q0.1–Q0.11 with command output saved to this file (UTF-8).
- [x] Table: question | answer | evidence | UNVERIFIED.
- [ ] STOP here and wait for owner approval of the PLOT-1 plan.

---

*End of PLOT-0 deliverable.*
