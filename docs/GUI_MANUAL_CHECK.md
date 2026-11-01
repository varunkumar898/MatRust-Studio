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
| *Pending* | User | Windows / Linux / macOS | eframe (winit + glow/wgpu) | *Pending User Run* | Real interactive run pending manual execution |
