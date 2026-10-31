# Repository Reconciliation Report: MATSTUDIO / MatRust Studio

## 1. Executive Summary

This report establishes the baseline source code state, environment, and dependency configuration for **MATSTUDIO / MatRust Studio** as required by Phase Zero.

The canonical implementation repository is located locally at `C:\Users\varun\Desktop\MATSTUDIO`.

---

## 2. Git Environment & Verification Facts

- **Repository Root:** `C:\Users\varun\Desktop\MATSTUDIO`
- **Active Branch:** `master`
- **HEAD Commit Hash:** `ca3c32580a2617dcad44b2a2aebc34445747dd91`
- **HEAD Commit Message:** `G1-FIX: correct benchmark verdicts, add reproducibility evidence`
- **Tags Present:** `a3.1.4`, `b2.1`, `b2.1.1`, `g0`, `g0.1`, `g0.1-verified`, `g1-fix`, `g1-verified`, `m0`, `r0`
- **Remote Configuration (`git remote -v`):** None configured locally.
- **Remote URL Mentioned in Prompt:** `https://github.com/varunkumar898/MatRust-Studio` (public remote repository; not connected as a git remote in this sandbox workspace).
- **Rust Toolchain:** `rustc 1.99.1 (LLVM 21.1.5)`, `cargo 1.99.1`
- **Target Platform:** `aarch64-pc-windows-gnullvm` (Windows 11 Enterprise ARM64)

---

## 3. Architecture & Dependency Reconciliation

### 3.1 Historical vs. Local Crate Architecture

The user prompt notes that external or alternative specifications referenced:
- `matlab_runtime`
- `matlab_values`
- `matlab_diagnostics`
- `eframe 0.37`
- `egui 0.37`
- `egui_plot 0.38`
- `egui_dock`

In contrast, our canonical repository structure contains the clean-room RustLab modular architecture:
- `crates/rustlab-parse`: Lexer, tokens, parser, AST with spans.
- `crates/rustlab-math`: N-D arrays, dense column-major matrices, CSC sparse matrices, `Value` enum.
- `crates/rustlab-eval`: Tree-walking interpreter, variable environment, output log, indexing contexts.
- `crates/rustlab-stdlib`: MATLAB-compatible standard library (elementary math, linear algebra, SVD, eig, QR, FFT, statistics, I/O formatting).
- `crates/rustlab-data`: Numerical statistics, covariance, correlation, histograms, sliding window metrics.
- `crates/rustlab-matio`: Level 5 binary `.mat` file reader/writer.
- `crates/rustlab-cli`: REPL console and benchmark harness.
- `apps/desktop`: Package `matrust_studio`, providing the native graphical user interface.

### 3.2 Dependency Versions

The local `.cargo/registry/cache` contains pre-fetched, offline-accessible dependencies:
- `eframe = "0.31.1"` (features: `default_fonts`, `glow`)
- `egui = "0.31.1"`
- `egui_glow = "0.31.1"`
- `winit = "0.30.13"`

Neither `egui_dock` nor `egui_plot` exists in the local cargo cache. In standard sandboxed environments without network access, adding an external dependency that is not pre-cached would fail. 

**Decision:**
To satisfy all docking requirements (draggable split docking, tabbed panels, resizable regions, layout presets, and layout persistence) without introducing incompatible version conflicts or network blockers, we implement a full-featured, native `DockTree` docking engine (`apps/desktop/src/dock.rs`) within `matrust_studio`. This guarantees 100% offline compilation and zero external dependency risk while providing native drag-and-drop tab docking.

---

## 4. Source Inventory of GUI Capabilities

| Module | Source File | Implemented Functionality |
| :--- | :--- | :--- |
| **App Shell** | `apps/desktop/src/app.rs` | Main `eframe::App`, top menu bar, toolstrip, modal dialogs, global keyboard shortcuts. |
| **Docking Engine** | `apps/desktop/src/dock.rs` | Tree-based binary split layout, tabbed regions, drag-and-drop docking, layout persistence. |
| **Script Editor** | `apps/desktop/src/editor.rs` | Multi-tab document manager, syntax styling, gutter breakpoints, error indicators, Ctrl+Space autocompletion, code folding. |
| **Command Window** | `apps/desktop/src/command_window.rs` | Interactive `>> ` REPL console, history navigation (Up/Down), output formatting, `clc` command. |
| **Current Folder** | `apps/desktop/src/current_folder.rs` | Directory tree explorer, path breadcrumbs, specialized icons (`.m`, `.mat`, `.csv`), double-click file opening. |
| **Workspace Panel** | `apps/desktop/src/workspace_panel.rs` | Real-time variable browser table (Name, Value, Size, Bytes, Class), search filter, variable inspector modal. |
| **Figures Canvas** | `apps/desktop/src/figures.rs` | Multi-figure tabs, hardware-accelerated Line, Scatter, Bar, and Histogram plots, pan, zoom, grid toggles. |
| **Execution Worker** | `apps/desktop/src/execution.rs` | Actor-pattern background execution thread (`rustlab-worker`), typed message passing, panic isolation (`catch_unwind`), cancellation flag. |
| **Themes & Presets** | `apps/desktop/src/theme.rs`, `apps/desktop/src/preferences.rs` | Dark/Light themes, layout presets (Standard, Editor Focus, Data Analysis, Plotting, Reset), user preferences persistence. |

---

## 5. Verification Baseline

- All 218 existing tests (212 workspace library tests + 6 desktop integration tests) compile and pass cleanly via `cargo test --workspace --offline`.
- Zero compiler warnings with `cargo check -p matrust_studio --offline`.
- `cargo clippy -p matrust_studio --offline -- -D warnings` passes with 0 warnings.
