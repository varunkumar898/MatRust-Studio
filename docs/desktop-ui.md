# MatRust Studio — desktop IDE

`apps/desktop` (crate `matrust_studio`) is a native desktop IDE for the
interpreter, built on `eframe`/`egui` 0.37 (`wgpu` renderer, Wayland/X11) and
`egui_plot` 0.38 for figures. It is a thin shell around the runtime crates —
the GUI creates one persistent
interpreter `Session` and talks to it from a dedicated worker thread,
exactly like the CLI does. There is no second interpreter.

Launch: `cargo run -p matrust_studio` (binary name `matrust-studio`).

## Status: what was verified on a live Wayland session (2026-10-22)

The window **did launch** on this machine (Hyprland / Wayland, screenshots
captured with `grim`); what follows was observed, not extrapolated:

- Toolstrip with `File/Edit/Window/Help` menu bar and `Home/Plots/Apps` tabs.
  `Home` exposes New Script, Open…, Save, Save As…, Run, Run Selection,
  Stop, Import Data…, Set Current Folder, Clear Workspace, Layout, and
  Preferences. `Stop` renders disabled while idle; disabled items are the
  ones not implemented yet (`Import Data…` shows a *Planned* note).
- Current Folder panel: directory listing with navigation (`Up/Ref/+ New`
  header buttons), drive-by double-click, rename?—no, context menu
  (Open / Copy path / New Script / Delete… with confirmation).
- Workspace panel: variable table (`Name / Value / Size / Type`), selection
  highlight, `Inspect…` (variable viewer window) and `Clear` wired to the
  real session (`clear` command equivalent). Empty state text when no
  variables exist.
- Script editor with per-tab documents: line numbers, MATLAB syntax
  highlighting (comments, strings, numbers, keywords), click-and-drag
  selection, Home/End/arrow navigation, undo/redo, editable find bar
  (`find_next` wraps), snippets (`standalone workbook buttons` on the toolstrip
  are planned — the editor itself is complete for editing).
- Command Window docked at the bottom: `>>` prompt, persistent environment
  shared with the editor, up/down history (arrow keys), `;` suppression,
  MATLAB-style diagnostics (e.g. `at line 1, column 5: Unrecognized function
  or variable 'A'.` for a fresh session), `who` / `clear` / `clc` / `exit`
  handling, `Clear` and `Hide` buttons, per-line coloring (echo blue, out
  white, error red, info faint).
- Figures panel on the right: tab per figure (`Figure 1 …`), plot with
  axes/grid/legend, pan & zoom with the mouse, `Close All`. Runtime builtins
  `plot/scatter/bar/histogram/title/xlabel/ylabel/legend/grid/clf/close/
  figure` populate it.
- Status bar: ready/running/error dot, current directory, active file,
  Ln/Col, `last run <duration>`, theme indicator.
- End-to-end run verified live: typed-equivalent selection run of
  `A = [1 2; 3 4]; B = [5 6; 7 8]; C = A * B; disp(C); …plot(xs, ys)…`
  produced `19 22` / `43 50` in the Command Window, filled the Workspace
  table with `A/B/C/xs/ys`, created `Figure 1` with a titled grid plot, and
  updated the status bar. Error runs produce red status + inline diagnostics.
- Preferences persistence verified across restarts (open files, active tab,
  cwd, command history, layout, theme) via eframe storage
  (`~/.local/share/studio.matrust`, key `matrust_prefs_v2`).
- Theme: dark charcoal by default, light theme switchable in Preferences;
  no purple styling, restrained blue accent, monospace editor font.

## Renderer note (important)

eframe/egui 0.37 ships two renderers. The **glow** renderer misbehaves on this
system (Mesa + winit on Hyprland): panel backgrounds render partially
transparent (wallpaper bleed-through), `ScrollArea` widget contents get
clipped shortly after their first rows, and a second bottom panel paints
outside its panel. The same symptoms reproduce in a minimal three-line
panel layout with default styling, so it is not app code. **This app ships
with `Renderer::Wgpu` and the `wgpu` feature**, which renders all of the
above correctly on this machine. If you build without `wgpu` you should
switch `apps/desktop/Cargo.toml` back to `glow` explicitly rather than
relying on defaults.

## Architecture

- `main.rs` — entry point, procedural app icon, native options.
- `app.rs` — `MatRustApp`: panel orchestration, toolstrip, dialogs, prefs
  save/load wiring, editor documents (`Doc`), command submissions.
- `execution.rs` — worker thread. Request channel (`Exec { id, source,
  origin }`, `ClearWorkspace`) / reply channel (`ExecDone { id, origin,
  result, duration }`, `WorkspaceCleared`). `Arc<AtomicBool>` cancel flag
  checked between statements. GUI-side polls via `try_recv` once per frame;
  no locks, no task blocking, so the GUI cannot freeze on long scripts.
- `theme.rs` — charcoal palette, dense styling, per-theme syntax palette.
- `editor.rs` — custom code editor widget (canonical `Vec<String>` store,
  char-based cursor, snapshots/undo coalescing, gutter painting, error-line
  marker hook, find highlight). Unit-tested.
- `command_window.rs`, `workspace_panel.rs`, `current_folder.rs`,
  `figures.rs`, `statusbar.rs` — the five panels.
- `preferences.rs` — `Prefs` (serde) + `eframe` storage round-trip.

Runtime-facing API: `matlab_runtime::Session { env, graphics }` +
`Session::exec(source, Option<&AtomicBool>) -> RunResult` (fields `env`,
`outputs`, `stdout`, `figures`, `interrupted`). Legacy `run_source` /
`exec_source` / `exec_program_env` functions remain for the CLI and tests.
Graphics builtins (`plot`, `figure`, …) return an empty matrix so they can
be chained; the CLI prints that empty result as before, and the GUI filters
an unlabeled empty `ans` to keep transcripts clean.

A scripted smoke test is available for CI-style verification:
`MATRUST_AUTORUN="<source>" ./target/debug/matrust-studio` echoes the source
and runs it through the real worker path on startup (used for the screenshot
verification below; it never executes unless the environment variable is
set).

## Limitations & known gaps

- **Stop execution** sets the cancel flag and is honored **between
  statements**. A long single statement (e.g. a huge matrix multiply) is not
  preemptible inside the runtime.
- No IME text-input support in the custom editor (input is character-based;
  the desktop does not yet consume `im` events). If you need CJK/cyrillic
  input, enter it through the command window (winit handles IME there) or a
  file opened from disk.
- The editor stores its text per document and does not persist unsaved
  drafts across restarts (unsaved buffers reload empty with a `*`).
- Resizable splitters (folders/workspace divider, panel gaps) are wired but
  only verified via code + document above; equal pixel-perfect drag behavior
  has not been audited for accessibility sizes.
- Command Window line buffer is capped at 2000 lines (older lines drop);
  very long one-line outputs are not paginated yet.
- `Import Data…`, `fit`, `readtable` and other *Planned* toolstrip entries
  are intentionally disabled with tooltips.
- Un-docked / floating windows (dockable panels as in R2026b) are not
  implemented; the figures panel cannot be popped out into a native window.
- Performance is adequate for expected script sizes (< 1000 line documents,
  < 5k-row workspace tables). No benchmark suite yet.

## What is next

- IME-aware input handling in the editor.
- Docking controls (pop-out figures window, move panels between regions).
- Editor: code folding, breakpoints in the gutter (for the future debugger).
- Workspace panel: inline header search, in-place numeric editing.
