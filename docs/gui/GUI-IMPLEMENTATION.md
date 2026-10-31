# RustLab Desktop GUI Implementation Plan & Specification

**Document:** `docs/gui/GUI-IMPLEMENTATION.md`  
**Target Milestone:** APP / MATSTUDIO Native Desktop Shell  
**Status:** SPECIFICATION & IMPLEMENTATION ROADMAP  

---

## 1. Modular Source Organization

The GUI shell is architected as an independent, modular crate within the Cargo workspace: `crates/rustlab-gui` (or integrated behind a feature flag in `rustlab-cli`). This guarantees that core numerical crates (`rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-stdlib`, `rustlab-data`, `rustlab-matio`) remain clean, lightweight, and completely free of GUI dependencies.

### Directory Layout

```text
crates/rustlab-gui/
├── Cargo.toml
└── src/
    ├── lib.rs                 # Public entry point: run_matstudio_gui()
    ├── app.rs                 # MatStudioApp struct implementing eframe::App
    ├── state.rs               # Coherent state structures & action events
    ├── worker.rs              # Asynchronous numerical engine worker thread
    ├── theme.rs               # Professional scientific dark/light palettes
    ├── actions.rs             # Unified action dispatcher (Menu + Toolbar)
    └── panels/
        ├── mod.rs
        ├── menu_bar.rs        # Top menu bar (File, Edit, View, Run, Tools, Help)
        ├── toolbar.rs         # Quick-action icon toolbar
        ├── editor.rs          # Multi-document script editor with gutter & tabs
        ├── command_window.rs  # Interactive REPL console with history navigation
        ├── workspace.rs       # Live variable table (Name, Size, Bytes, Class, Value)
        ├── current_folder.rs  # Filesystem browser and project tree
        ├── history.rs         # Command history list with rerun capability
        ├── plot_view.rs       # Embedded scientific canvas using egui_plot
        └── status_bar.rs      # Engine status, cursor coordinates, working dir
```

---

## 2. Core Panel Implementation Specifications

### 2.1 Main Window Layout
The desktop layout uses a professional, adjustable 4-pane docking layout:
1. **Top Menu & Toolbar:** Persistent at the top, exposing common actions.
2. **Left Panel (Current Folder):** Collapsible file tree showing `.m`, `.mat`, `.csv` files.
3. **Center-Top Panel (Script Editor):** Tabbed interface with line numbering and syntax-aware formatting.
4. **Center-Bottom Panel (Command Window):** Monospaced console executing commands against the background interpreter.
5. **Right-Top Panel (Workspace):** Real-time table of workspace variables.
6. **Right-Bottom Panel (Command History):** Chronological log of past executions.
7. **Bottom Status Bar:** Displays execution state (`READY`, `RUNNING (1.2s)`, `ERROR`), active file, and working directory.

### 2.2 Visual Theme & Styling (Restrained Desktop Palette)
Strictly adhering to Section 11 of the Master Prompt, the visual styling avoids decorative neon gradients or glassmorphism, employing a clean scientific palette:
- **Background Base:** Dark Slate `#1E1E1E` (Dark Mode) / Soft Neutral `#F3F3F3` (Light Mode).
- **Panel Surfaces:** Monochromatic `#252526` / `#FFFFFF`.
- **Borders & Dividers:** Subtle `#3E3E42` (1px width).
- **Accents:** Restrained MATLAB Amber/Blue `#007ACC` / `#D7BA7D`.
- **Typography:** System monospace (`Consolas`, `Cascadia Code`, `SF Mono`, or `Menlo`) for code, editor, and console.

---

## 3. Step-by-Step Implementation Sequence

### Stage 1: Crate Initialization & Backend Channels
1. Add `crates/rustlab-gui/Cargo.toml` with `eframe`, `egui`, `egui_plot`, and workspace dependencies (`rustlab-stdlib`, `rustlab-eval`, `rustlab-math`, `rustlab-parse`).
2. Implement `src/worker.rs` spawning a persistent background thread holding `new_interpreter()`.
3. Establish bidirectional message passing:
   - `ExecutionRequest::EvalScript(String)`
   - `ExecutionRequest::EvalCommand(String)`
   - `ExecutionRequest::ClearWorkspace`
   - `ExecutionResponse::Output(String)`
   - `ExecutionResponse::Error(String)`
   - `ExecutionResponse::WorkspaceUpdated(Vec<VariableRecord>)`

### Stage 2: Command Window & REPL Console
1. Implement interactive console with `>> ` prompt.
2. Connect Enter key to `ExecutionRequest::EvalCommand`.
3. Display streaming output from `interp.output_log` without blocking the frame renderer.
4. Implement Up/Down arrow key command history recall.

### Stage 3: Script Editor & Document Management
1. Multi-tab document manager supporting open, edit, save, and close.
2. Gutter rendering with line numbers and current line highlight.
3. Run Script button dispatching full file content to the worker thread.

### Stage 4: Workspace Variable Inspector
1. On each execution completion, extract `interp.env.locals()`.
2. Format variable records: Name, Dimension String, Byte Count, Class Name, Preview.
3. Double-click inspection displaying a structured matrix viewer.

### Stage 5: Scientific Plot Window
1. Integrate `egui_plot` canvas.
2. Provide interactive pan, zoom, and coordinate hover probes.
3. Map plotting data generated by script evaluations to real curves.
