# RustLab Desktop GUI Architecture Specification

**Document:** `docs/gui/GUI-ARCHITECTURE.md`  
**Target Milestone:** APP / MATSTUDIO Native Desktop Shell  
**Author:** RustLab Systems Architecture  
**Status:** DESIGN PROPOSAL & ARCHITECTURE BLUEPRINT  

---

## 1. Architectural Philosophy & Framework Selection

RustLab's mission is to provide a clean-room, MATLAB-compatible scientific computing platform written purely in safe, modern Rust. The graphical desktop interface must mirror this mission by being:
- **Fast and lightweight:** Sub-100ms cold startup, 60+ FPS rendering, minimal RAM overhead.
- **Self-contained:** Zero external web runtime or heavy C++ DLL dependencies (unlike Chromium/CEF).
- **Responsive under numerical load:** Computations execute asynchronously on background worker threads without stuttering the UI.

### 1.1 Comparative Framework Evaluation

| Criteria | `egui` / `eframe` | `Slint` | `Iced` | `Tauri` (Webview) |
| :--- | :--- | :--- | :--- | :--- |
| **Paradigm** | Immediate Mode | Retained Mode | Elm-style Reactive | Web / Retained DOM |
| **Language** | 100% Pure Rust | Rust + Slint DSL | 100% Pure Rust | Rust + JS/HTML/CSS |
| **Dependencies** | Minimal (wgpu/glow) | Slint compiler crate | WGPU / iced_core | OS WebView2 / Edge |
| **Windows ARM64** | Native Tier-1 | Good | Good | Depends on system WebView2 |
| **Plotting Integration** | Native (`egui_plot`) | Custom Canvas | `iced_canvas` | Chart.js / Canvas |
| **Docking / Layout** | `egui_dock` or split | Built-in grid | Flexible layout | CSS Grid / Flexbox |
| **Engine Coupling** | Clean decoupled state | Property bindings | Messages / Commands | JSON IPC over IPC bridge |
| **Suitability Score** | **9.5 / 10 (Selected)** | 7.5 / 10 | 7.0 / 10 | 6.0 / 10 |

### 1.2 Justification for `egui` / `eframe`
1. **Immediate Mode Simplicity:** Ideal for technical applications with frequent state updates (variable inspection tables, dynamically changing console output, animated simulation plots).
2. **Deterministic UI State:** Eliminates complex synchronization bugs between retained DOM trees and internal Rust data structures.
3. **Pure Rust Pipeline:** Fully compilable in standard Rust environments on Windows ARM64 without Node.js, C++ toolchains, or WebView dependencies.
4. **Rich Scientific Ecosystem:** Native support for `egui_plot` for real-time line/scatter/bar plots and mathematical graphing.

---

## 2. System Architecture & Threading Model

```
┌────────────────────────────────────────────────────────────────────────┐
│                        MAIN THREAD (UI / RENDER)                       │
│                                                                        │
│   eframe Native Window ──> egui Event Loop ──> MatStudioApp            │
│      ├── MenuBar & ToolBar                                             │
│      ├── Top: Script Editor (Multi-tab syntax highlighter)             │
│      ├── Left: Current Folder / Project Browser                        │
│      ├── Right: Workspace Variable Inspector & Command History         │
│      ├── Bottom: Interactive Command Window (REPL Console)             │
│      └── Dock / Floating: Scientific Plot Window (egui_plot)           │
└──────────────────┬─────────────────────────────────▲───────────────────┘
                   │                                 │
         Command/Script Action                Execution Event /
        (Sender<ExecutionRequest>)        (Receiver<ExecutionResponse>)
                   │                                 │
┌──────────────────▼─────────────────────────────────┴───────────────────┐
│                    BACKGROUND WORKER (NUMERICAL ENGINE)                │
│                                                                        │
│   Worker Thread ──> std::sync::mpsc ──> RustLab Interpreter            │
│      ├── Lexer (rustlab-parse)                                         │
│      ├── Parser & AST (rustlab-parse)                                  │
│      ├── Environment & Scopes (rustlab-eval)                           │
│      ├── Standard Library & Generators (rustlab-stdlib)                │
│      ├── Dense/Sparse Matrix Operations (rustlab-math)                 │
│      └── Statistics & CSV Matrix IO (rustlab-data)                     │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Concurrency & Execution Decoupling
To strictly uphold Rule 8 (Responsiveness) and prevent long numerical calculations from freezing the interface:
- **UI Thread:** Dedicated solely to handling input events (mouse, keyboard), rendering frames via `eframe` (60 FPS), and updating panel layouts.
- **Execution Worker Thread:** A persistent worker thread initialized with `new_interpreter()`. It receives `ExecutionRequest` messages across an unbounded MPSC channel, executes `interp.eval_script(&source)`, drains `interp.output_log`, extracts updated workspace variable metadata, and sends an `ExecutionResponse` back to the UI thread.
- **Cancellation:** An `Arc<AtomicBool>` interrupt flag allows the UI thread to signal immediate halt for loops and long reductions that check cancellation tokens.

---

## 3. Data Structures & Application State Model

```rust
pub struct MatStudioApp {
    // Subsystem States
    pub editor: EditorState,
    pub console: ConsoleState,
    pub workspace: WorkspaceState,
    pub project: ProjectState,
    pub history: HistoryState,
    pub plot: PlotState,
    pub theme: ThemeState,

    // Channels
    pub tx_request: std::sync::mpsc::Sender<ExecutionRequest>,
    pub rx_response: std::sync::mpsc::Receiver<ExecutionResponse>,

    // Active Engine Status
    pub execution_status: EngineStatus, // Idle | Running(Duration) | Error
}
```

### 3.1 Panel State Definitions

1. **`EditorState`**:
   - Manages an open set of `Document` tabs.
   - Tracks `path: Option<PathBuf>`, `content: String`, `is_dirty: bool`, `cursor_pos: (usize, usize)`.
   - Supports Open, Save, Save As, Close Tab, Run Script, Run Selection.
2. **`ConsoleState`**:
   - Manages the scrollable output history (`Vec<ConsoleEntry>`) separating stdout, stderr, and prompt echoes.
   - Maintains current interactive input line `input_buffer: String`.
   - Command history navigation via Up/Down arrow keys.
3. **`WorkspaceState`**:
   - Cached variable table: `Vec<VariableRecord>`.
   - `VariableRecord` stores: `name`, `data_type`, `dimensions`, `numel`, `bytes`, `preview_string`.
   - Provides search filtering and double-click variable inspection.
4. **`ProjectState`**:
   - Active working directory (`current_dir: PathBuf`).
   - File listing with extensions, sizes, and last modified timestamps.
   - Quick double-click to open `.m`, `.mat`, `.csv` files.
5. **`PlotState`**:
   - Active figures: line curves, scatter points, bar charts.
   - Titles, x-labels, y-labels, grid toggles.

---

## 4. Error Isolation & Panic Safety

1. **Worker Isolation:** If a numerical calculation encounters an unexpected panic, the worker thread catches it via `std::panic::catch_unwind`, restores an uncorrupted interpreter state, and transmits a clear diagnostic error message to the console rather than crashing the desktop application.
2. **Syntax Caret Diagnostics:** Evaluator errors (`EvalError`) are converted into exact line and column caret annotations using `rustlab_cli::format::format_eval_error` and rendered with monospaced highlighting in both the Console and the Editor gutter.
