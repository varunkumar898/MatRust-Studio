# Linux Build & Cross-Platform GUI Verification Report

**Component:** Linux Compatibility & Headless Verification  
**Package:** `matrust_studio` & RustLab Workspace  
**Status:** **VERIFIED (Architecture & Headless Automated Test Suite)**

---

## 1. Cross-Platform GUI Architecture

The desktop application package `matrust_studio` is architected using standard cross-platform Rust GUI primitives:
- **Application Framework:** `eframe 0.31.1` (via `egui-winit` + `egui_glow`)
- **Windowing & Event Pump:** `winit 0.30` (providing native abstractions over Windows Win32, Linux Wayland, and Linux X11)
- **Rendering Backend:** `glow 0.16` (OpenGL ES / Desktop OpenGL 3.3+ via EGL or GLX)

### Linux Runtime Windowing Backends
On Linux, `winit` automatically negotiates the active display protocol at runtime:
1. **Wayland:** If `WAYLAND_DISPLAY` is set in the environment, `winit` binds directly to the Wayland compositor protocol via `wayland-client`.
2. **X11:** If `DISPLAY` is set, `winit` connects via Xlib / XCB.

### System Dependency Footprint on Linux
To compile and link `matrust_studio` on Debian/Ubuntu Linux distributions:
```bash
sudo apt-get install -y \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxinerama-dev \
    libgl1-mesa-dev \
    libxkbcommon-dev \
    libwayland-dev
```

---

## 2. Headless & CI Pipeline Verification

In Linux server environments, continuous integration runners (such as GitHub Actions or Docker containers) operate without a physical GPU or display server attached.

### Numerical Engine Decoupling
The core numerical computing engine (`rustlab-parse`, `rustlab-math`, `rustlab-eval`, `rustlab-stdlib`, `rustlab-data`, `rustlab-matio`, `rustlab-cli`) contains **zero GUI dependencies**. It compiles and executes with full functionality on pure headless Linux servers.

### Headless Verification of Desktop Logic
Crucially, all 12 integration tests in `apps/desktop/tests/desktop_tests.rs`:
- `test_dock_tree_presets_and_json_persistence`
- `test_dock_tree_move_panel_and_tab_operations`
- `test_code_folding_block_detection`
- `test_breakpoint_execution_pause_inspect_and_continue`
- `test_breakpoint_execution_stop`
- `test_worker_panic_recovery_and_session_reset`
- `test_worker_client_async_execution`
- `test_session_basic_arithmetic_and_matrices`
- `test_session_command_window_disp`
- `test_session_plotting_builtins_and_state`
- `test_session_cancellation_flag`
- `test_session_csv_import`

exercise the complete docking tree logic, code folding parser, statement-level debugger synchronization, CSV data ingestion, and worker panic recovery **without requiring an active X11 or Wayland display server**.

Running `cargo test -p matrust_studio --offline` succeeds 100% in headless environments. For graphical end-to-end testing of `matrust_studio.exe` in headless CI, standard tools like `xvfb-run` provide virtual framebuffer acceleration.
