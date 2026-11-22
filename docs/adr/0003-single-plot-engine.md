# ADR 0003: Single Plot Engine & UI Synchronization Architecture

- **Status:** Proposed (Milestone FIG, Task PLAN-FIG)
- **Date:** 2026-11-22
- **Deciders:** Project Owner, AI Engineering Team
- **Consulted:** `docs/design/plot-integration.md`, `docs/plot-int/discovery.md`, FIG-GUI diagnosis

---

## 1. Context and Problem Statement

The MatRust Studio desktop environment currently suffers from a dual-engine architectural fracture:

1. **Engine Model (`rustlab-plot`)**: Standard library builtins (`contour`, `quiver`, `surf`, `mesh`, `stem`, `polar`, `area`, `imagesc`, `plot3`, `legend`, `saveas`, etc.) construct a rich scene model (`Figure`, `Axes`, `Series`, `Camera`, `Colorbar`) tracked inside a thread-local `PlotRegistry` (`crates/rustlab-plot/src/state.rs:7-10`). This engine powers CLI rendering, script automation, and deterministic SVG output (`crates/rustlab-plot/src/svg.rs`).
2. **GUI Model (`GraphicsState`)**: The desktop session (`apps/desktop/src/execution.rs:264-578`) intercepts and overrides a small subset of builtins (`plot`, `scatter`, `bar`, `title`, `xlabel`, `ylabel`, `grid`, `figure`, `clf`, `cla`, `hold`, `subplot`, `close`), pushing flat `PlotItem` entries into a parallel `GraphicsState` (`apps/desktop/src/execution.rs:202-225`).
3. **GUI Painter (`figures.rs`)**: The GUI figures canvas (`apps/desktop/src/figures.rs:115-420`) draws exclusively from `GraphicsState` and only knows how to paint 4 series types: `Line`, `Scatter`, `Bar`, and `Histogram`.

When running examples such as the Math Gallery's *"Helmholtz Potential & Vector Gradient"* (`contour`, `quiver`, `hold on`, `title`, `legend`):
- `figure(1)` creates a tab in `GraphicsState`.
- `title` sets the tab title in `GraphicsState`.
- `contour` and `quiver` are not intercepted by `execution.rs`. They fall back to `rustlab-stdlib`, writing to `rustlab_plot::PlotRegistry` on the worker thread.
- The GUI's `GraphicsState.axes[0].items` remains length 0 (`total_series() == 0`).
- The Figures panel displays `Series: 0` and early-returns the empty placeholder *"This panel shows plots from your workspace."*

Furthermore, extensive interactive functionality previously developed for `rustlab_plot::Figure` (such as `hit_test_axes`, `zoom_subplot`, `pan_subplot`, and linked axes synchronization in `crates/rustlab-plot/src/lib.rs:818-936`) is orphaned in `rustlab-plot` and never called by the running desktop application.

---

## 2. Decision

We establish **`rustlab_plot::Figure` as the single authoritative source of truth** across the entire project. The desktop GUI becomes a pure, reactive viewer over `rustlab_plot::Figure` snapshots:

1. **Single Source of Truth**: All plotting builtins in all execution environments (desktop worker thread, CLI, and integration tests) execute identically through `rustlab-stdlib` and write to `rustlab_plot::PlotRegistry`.
2. **Decommission Duplicate GUI Builtins**: Remove the parallel plot interceptors in `apps/desktop/src/execution.rs` (`plot`, `scatter`, `bar`, `figure`, `hold`, `subplot`, `title`, `xlabel`, `ylabel`, `grid`, `clf`, `cla`, `close`).
3. **Snapshot Transport**: Upon completion of script or statement execution on the background worker thread, `Session::exec()` extracts an immutable snapshot of all active figures (`Arc<PlotRegistry>` or `HashMap<usize, Arc<Figure>>`) and transmits it via `WorkerResponse::ExecutionFinished { result, .. }` to the GUI thread.
4. **No Per-Frame Cloning**: The GUI thread holds `Arc<Figure>`. Rendering frames in `egui` borrow data directly from the `Arc<Figure>` without cloning point vectors or surface meshes.
5. **Shared Projection Math**: 3D projection, depth sorting, and camera calculations are implemented once as pure functions in `rustlab-plot` and shared between `svg.rs` and the desktop painter in `figures.rs`.

---

## 3. Alternatives Considered

### Alternative A: Keep Both Engines and Mirror Builtins (Status Quo Extended)
- **Concept**: Add GUI versions of `contour`, `quiver`, `surf`, `mesh`, `stem`, `polar`, `area`, `imagesc`, `legend`, etc. into `execution.rs`, translating them into expanded `PlotItem` variants in `GraphicsState`.
- **Rejection**: Violates Engineering Charter Rule 1 (DRY / Single Responsibility). Doubles maintenance burden for every new plot feature. Causes subtle behavioral divergence between CLI/SVG output and desktop visualization (e.g. limit calculations, color cycling, camera rotations).

### Alternative B: Bidirectional Synchronization Adapter
- **Concept**: Maintain both `PlotRegistry` and `GraphicsState`, using a listener or delta-sync mechanism to replicate changes between them.
- **Rejection**: Introduces distributed state reconciliation issues, cycle detection complexity, and lock contention between the worker thread and the UI rendering thread.

### Alternative C: GUI-Owned Canvas Model (egui_plot / ImGui Immediate Mode)
- **Concept**: Move plotting entirely to the desktop application crate; standard library scripts would call desktop callbacks.
- **Rejection**: Completely breaks headless execution, `--no-default-features` builds, automated testing, CI pipelines, and standalone CLI SVG exports.

---

## 4. Detailed Data Flow & Architecture

```
[ Worker Thread: rustlab-worker ]
      │
      ├─► Session::exec(code)
      │      └─► Interpreter eval_script()
      │            └─► Builtins (plot, contour, quiver, surf, legend, etc.)
      │                  └─► Mutates thread-local rustlab_plot::PlotRegistry
      │
      ├─► Execution finishes
      │      └─► Snapshot: registry.figures -> HashMap<usize, Arc<Figure>>
      │      └─► tx.send(WorkerResponse::ExecutionFinished { ..., figures })
      ▼
[ Channel Boundary: std::sync::mpsc ]
      ▼
[ Main / GUI Thread: eframe / egui ]
      │
      ├─► App::update() -> poll_worker()
      │      └─► rx.try_recv() receives Arc<Figure> snapshots
      │      └─► Stores in App::figures_state
      │
      └─► FiguresPanel::show(ui, &figures_state)
             └─► Reads Arc<Figure> without copying data points
             └─► Evaluates layout & axes bounds
             └─► Paints shapes using egui::Painter
```

### 4.1 Lifecycle & Thread-Safety
- The background thread `rustlab-worker` (`apps/desktop/src/execution.rs:1039`) runs continuously for the lifetime of the interactive session.
- `rustlab_plot::PlotRegistry` lives in the worker thread's thread-local storage (`crates/rustlab-plot/src/state.rs:7-10`).
- Because the worker thread persists across commands, `figure(1)` retains its axes and series across multiple command window executions, faithfully replicating MATLAB's stateful figure semantics.
- Figures sent across the channel are wrapped in `Arc<Figure>`. Since `Figure`, `Axes`, `Series`, and their numeric vectors implement `Send + Sync`, transfer across threads is completely thread-safe and lock-free.

### 4.2 Interaction State vs. Model State
- **Problem**: When a user drags or zooms an axis in the GUI, does it mutate the worker's `PlotRegistry`? If a script reruns, does manual zoom get wiped?
- **Decision**:
  - The model `Figure` contains *authoritative computed limits* (`xlim: Option<[f64; 2]>`, `ylim`, `zlim`) established by MATLAB script commands (`xlim([0 10])`) or data autoscaling.
  - Interactive camera angles (3D orbit) and pan/zoom offsets are tracked in a lightweight UI view-state per axes: `AxesViewState { manual_xlim: Option<[f64; 2]>, manual_ylim: Option<[f64; 2]>, camera_offset: Vec2 }`.
  - When a script re-executes, if the script explicitly set `xlim`/`ylim` or called `clf`/`cla`, the manual view-state resets. If the script simply appends data under `hold on`, the user's zoom/pan viewpoint is preserved.
  - Double-clicking or clicking "Reset View" clears `AxesViewState`, smoothly returning to the model's auto-resolved limits.

### 4.3 Memory Bounds & Large Data Safety
- **Budget**: 100×100 surface ($10^4$ vertices = $\approx 240\text{ KB}$ for $X, Y, Z$) and $10^5$ line points ($\approx 1.6\text{ MB}$ for $X, Y$) fit easily within a sub-5 MB memory envelope.
- **Cap**: Any single series exceeding $10^6$ points or 3D surface exceeding $500 \times 500$ will log a diagnostic warning and apply level-of-detail decimation during GUI polygon construction, preventing GPU command-buffer stall while keeping the raw data in `Figure` intact.

---

## 5. Consequences

### Positive
- **100% Behavioral Parity**: Exactly what the MATLAB standard library computes is what the desktop GUI paints.
- **Zero Duplicate Code**: Eliminates ~300 lines of fragile builtin interception glue in `execution.rs`.
- **Support for All 17 Plot Types**: Math Gallery examples (including Helmholtz, Lorenz, Rosenbrock, Julia, Mandelbrot, Butterworth, Spectrogram) immediately show their figures in the desktop GUI.
- **Reactivates P1–P3 Interaction Math**: `hit_test_axes`, `zoom_subplot`, and linked axes in `rustlab-plot` become the underlying calculation engine for GUI mouse interactions.

### Negative / Trade-offs
- GUI painting code in `figures.rs` must implement egui polygon painters for surfaces, 3D projected lines, and NaN-delimited polylines.
- Tests that asserted against the old `GraphicsState.items` must be updated to inspect `Figure.axes.series`.
