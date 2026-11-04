# Milestone B2.2 — rustlab-plot: renderer-independent plot model + deterministic SVG

**Branch:** `b2/plot` (to be created from `main`) · **Scope:** new core crate `crates/rustlab-plot` + plot builtins in `rustlab-stdlib`.

## 1. Problem
MATLAB-compatible plotting needs a renderer-independent description of a figure (axes, series, limits, legend, text) plus a deterministic way to render it to SVG. The existing `mlapp-plot` crate is App Designer–specific and coupled to `egui_plot` rendering in the desktop GUI. B2.2 creates a core `rustlab-plot` crate with no GUI dependency, usable under `--no-default-features`, and registers MATLAB-compatible plot builtins in `rustlab-stdlib`.

## 2. What this milestone changes

**New crate `crates/rustlab-plot`** (renderer-independent model):
- `Figure { id, title, axes, current_axes, visible }`
- `Axes { id, title, xlabel, ylabel, xlim, ylim, xscale, yscale, grid, hold, series, texts, legend }`
- `Series` enum: `Line`, `Scatter`, `Bar`, `Histogram`
- `Line { name, x, y, color, width }`, `Scatter { name, x, y, color, size }`, `Bar { name, x, y, color }`, `Histogram { name, values, bins, color }`
- `Text { x, y, content }`, `Legend { location }`
- `Scale` enum: `Linear` (now), `Log` (reserved, not implemented)
- `nice_ticks(min, max, count) -> Vec<f64>` — "nice ticks" rule: step snapped to 1/2/5 × 10ⁿ so tick labels fall on round numbers
- `to_svg(&self, width, height) -> String` — deterministic SVG with fixed 2-decimal coordinate formatting, stable element ids, no timestamps/randomness

**Builtins registered in `rustlab-stdlib`** (via a thread-local `PlotRegistry`):
- `figure` — create/switch figure, returns handle
- `plot(x, y)` — line series on current axes
- `scatter(x, y)` — scatter series
- `bar(x, y)` — bar series
- `hist(y, nbins)` — histogram series
- `hold('on'|'off')` — toggle hold on current axes
- `subplot(m, n, p)` — create axes in an m×n grid at position p
- `xlabel`, `ylabel`, `title` — set axis/figure text
- `legend(loc)` — set legend location
- `xlim([lo hi])`, `ylim([lo hi])` — axis limits
- `grid('on'|'off')` — grid toggle
- `saveas(handle, 'file.svg')` — write SVG to disk

**Tests:**
- Unit tests for `nice_ticks` (nice-number snapping, monotonic output)
- Unit tests for `to_svg` determinism (same input → same bytes)
- Snapshot tests: stored `.svg` files compared byte-equal
- Builtin tests (golden + edge: empty input, NaN, wrong arg count/type)

**Scope OUT:** PNG rendering (resvg, later), 3D, timetables, GUI integration (desktop keeps its own `GraphicsState`/`egui_plot` for now), interactive zooming.

## 3. Error ids
All `RUSTLAB:` prefixed, catalogued in `docs/spec/error-catalog.md`:
- `RUSTLAB:plot:badArgs` — wrong argument count/type
- `RUSTLAB:plot:noFigure` — no current figure exists
- `RUSTLAB:plot:saveFail` — could not write SVG file
- `RUSTLAB:plot:emptyData` — x/y empty or length mismatch
- `RUSTLAB:hist:badBins` — histogram bin count must be positive

## 4. State management
Interpreter builtins are pure functions; figure state is held in a `thread_local!` `RefCell<PlotRegistry>` inside `rustlab-plot`, reset between tests via `plot_state_reset()`. Matches MATLAB's current-figure (`gcf`) semantics within one session.

## 5. Risks
- Thread-local state is not shared across interpreter threads — acceptable for single-threaded REPL/script mode.
- SVG float formatting must be fixed precision (2 decimals) to guarantee byte-identical snapshots.
- `nice_ticks` must handle degenerate ranges (min == max) without division by zero.