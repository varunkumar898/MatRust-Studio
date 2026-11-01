# GUI Interactive Window Manual Verification Checklist

This checklist documents the manual testing procedure for interactive desktop window verification of App Designer packages via `mlapp-cli --gui`.

Automated CI environments run headless without a display server; while automated headless rendering tests verify offscreen rasterization and widget tree positioning (see `crates/mlapp-render/tests/render_tests.rs` and the exported snapshot `docs/assets/gui_window_sample.png`), real-window event loop execution, hardware window rendering, and physical mouse-click callback dispatch must be verified through this manual interactive checklist.

---

## 1. Prerequisites

- OS with active window server: Windows 10/11, macOS, or Linux with X11 / Wayland.
- Rust stable toolchain (`cargo`, `rustc`).
- Clean repository working copy.

---

## 2. Verification Procedure

### Step 1: Build and Launch Sample App

From the repository root, run:

```bash
cargo run -p mlapp-cli -- run --gui crates/mlapp-container/fixtures/sample_apps/r2021a_sample.mlapp
```

### Step 2: Window Initialization & Layout Checks

Verify the following upon application launch:
- [ ] **Window Creation**: A graphical desktop window opens without crashing or throwing panics to `stderr`.
- [ ] **Window Dimensions**: Window size matches the `UIFigure` specification (default: 800x600 pixels).
- [ ] **Component Positioning**:
  - Left side: Panel containing a button labeled **"Calculate"** (`StartBtn`) and a text label reading **"Ready"** (`StatusLabel`).
  - Right side: Coordinate axes box (`UIAxes`) with axis borders and grid/ticks.
- [ ] **Coordinate Integrity**: Components are right-side-up (converted correctly from MATLAB bottom-left origin to top-left desktop origin).

### Step 3: Interactive Event Dispatch & Callback Verification

Perform the following user interactions:
- [ ] **Button Click**: Hover over and click the **"Calculate"** (`StartBtn`) button with the mouse.
- [ ] **Callback Execution**: Verify that the background worker thread executes the MATLAB callback without blocking the UI rendering thread.
- [ ] **UI State Mutation**:
  - The status label text transitions from **"Ready"** to **"Done"**.
- [ ] **Plot Update**:
  - The `UIAxes` area updates to draw the calculated series (e.g. sine wave curve from 0 to $2\pi$).
  - Plot title and axis labels are updated if specified by the callback.

### Step 4: Clean Window Exit

- [ ] **Window Close**: Close the window by clicking the window close button (`X`) or pressing Alt+F4.
- [ ] **Process Termination**: Verify that the CLI process exits cleanly with exit code `0` and background threads terminate without hanging.

---

## 3. Verification Record

| Date | Tester | OS Platform | Window Backend | Result | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 2026-11-01 | Automated | Windows 11 ARM64 | eframe 0.31.1 (winit + glow) | Window opened, screenshot captured, exit 0 | `scripts/capture-gui-window.ps1` launched `mlapp-cli run --gui` with `MLAPP_GUI_AUTOCLOSE_MS=5000`; `PrintWindow` captured the 814x638 native window to `docs/assets/gui_window_live.png` (SHA-256 `06CB900F5C02FD8028DA5D3B1B6FA7339657A999B94E9C1CB2E95A4E8D3F8A4B`). Button click and callback mutation are still **not** automated; the checklist below remains a manual step. |

## 4. Reproduce the Automated Window Capture (Windows)

```powershell
cargo build -p mlapp-cli
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/capture-gui-window.ps1 `
  -Package fixtures/sample_apps/r2021a_sample.mlapp `
  -Output docs/assets/gui_window_live.png `
  -Title FixtureApp_R2021a `
  -AutoCloseMs 5000
```

The script sets `MLAPP_GUI_AUTOCLOSE_MS` for the child process, waits for the native window title, captures the window with Win32 `PrintWindow`, and fails if the window or screenshot is missing. It does not click buttons. CI is headless and only runs the offscreen screenshot test; the native-window capture is a desktop check.
