# MatRust Studio / RustLab Project State

**Current branch:** `gui/polish-1` (polish milestone branch; owner instruction: branch of `main` @ `425f54e`)
**Repository:** `https://github.com/varunkumar898/MatRust-Studio.git`
**Current code commit:** `4ae9c687e0ea5e89d81373305a415ff648aa0c31` (PR #20 merged)
**Date:** 2026-11-03
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`)
**Local stable toolchain:** Rust 1.99.1
**Declared MSRV:** Rust 1.88.0
**Verification status:** 
- Local default, all-features, and no-default-features test runs pass at **495/495/495**.
- Local clippy (all-features, no-default-features, per-crate no-default-features) passes with 0 warnings.
- `cargo fmt --all -- --check` passes cleanly.
- `scripts/verify.ps1` runs from cargo clean and exits 0 with all verification steps passing.
- GitHub Actions CI matrix: 100% green on all 14 jobs on PR #20 (run `41584278404`) and running on `main` (run `41592329702`).

## Current work: GUI polish milestone `gui/polish-1`

- Branch `gui/polish-1` created from `main` @ `425f54e` (master prompt v2, S-step plan).
- **S1 GUI inventory + audit COMPLETE** (no code changed): `docs/gui-audit.md` documents
  the full panel/control inventory of `apps/desktop`, a real-window capture walk
  (`docs/screenshots/before/*.png`), runtime findings (duplicate egui ScrollArea id
  `B61C`, editor tab-bar duplicates, gutter desync, toolstrip overflow, trademark window
  title, missing run-example and 38/225 help coverage, no-Stop cancellation path in plain
  runs, %TEMP% layout persistence, and one repo-hygiene bug: workspace tests rewrite
  `docs/builtins.txt` and re-zip a fixture). Baseline verification from this step: 351/351/351
  pass (verification log not retained in this repository; CI is the source of truth).
- **S2@C1 test hygiene COMPLETE** (commit `9ea305c`): all six test-suite writers that
  touched repo files (`docs/builtins.txt`, `docs/SUPPORTED_FUNCTIONS.md` on UPDATE_DOCS=1,
  `fixtures/sample_models/simple_feedback.slx`, the four `fixtures/sample_apps/*.mlapp`,
  and the T-ENTROPY screenshot) now write only to `%TEMP%`; `docs/builtins.txt` is verified
  read-only against the live registry (225 names under full defaults; drift is a loud test
  failure instead of a silent rewrite). `scripts/verify.ps1` gained step 10: the run fails
  on any working-tree dirt. From-clean log summarized in
  `docs/VERIFICATION.md` (historical SHA-256
  `09287F07129071D3FE34ADBA131D984FB4E4273150F1078B2C76FE7E417F571E`), still 351/351/351
  and all 351 manifest entries present.
- **S2@C2 audit reconciliation COMPLETE**: six owner-screenshot items reconciled in
  `docs/gui-audit.md` section 12 (constants NOT reproduced — engine truth: `true`/`false`
  builtins only; "No active file" confirmed as a PATH-based condition bug; dot-entries,
  Help clip, Figures hint recorded) and BACKLOG 6.1-6.6 appended.
- **S2@C3 code steps COMPLETE**: C3a rename (`774d6f1`), C3b watermark/title (`5d0b301`),
  C3c demo-content removal (`fcdc88d`). Final from-clean verification: steps 5/6/7
  pass-sums 351/351/351, all 351 manifest entries present, working-tree gate green
  (verification log not retained in this repository; CI is the source of truth).
  After-screenshot captured at 1280x720 (`docs/screenshots/after/desktop_1280x720.png`,
  SHA-256 `CFB1018FE5CE9F9461D5FC8CFBD8E8C868E86C2B24FF6CE63B9EEA4BD5D63C82`): title
  "MATSTUDIO", empty editor, single-line command-window hint, no watermark overlays;
  remaining known red ribbons (ScrollArea B61C duplicates) belong to S3 as documented.
- **S3 COMPLETE** (owner approved S2 with "proceed"): id hygiene + dot-entry
  policy. Commits: `af930a2` (G5 id hygiene: salted both unsalted ScrollAreas —
  the B61C pair — per-row `push_id` in Workspace/Current Folder/Help/Add-Ons
  lists, station-scoped `.id()` on the block-params modal; note: egui 0.31's
  `Window` takes `.id(Id)`, ScrollArea takes `.id_salt(...)`), `f153d9c`
  (Current Folder hides dot-entries by default with a per-session `Hidden`
  checkbox, BACKLOG 6.2; +1 test `hidden_filter_is_dotfile_only`; manifest
  regenerated: 351 -> 352 names), `67f9126` (capture-script fix: window find
  now filters by the launched process id because the terminal window's title
  also starts with "MATSTUDIO"; first S3 capture had grabbed — and moved — the
  wrong window; + runtime evidence shot).
  Runtime evidence pair (both read in-session): BEFORE
  `docs/screenshots/after/desktop_1280x720.png` (99036 B, SHA-256
  `CFB1018FE5CE9F9461D5FC8CFBD8E8C868E86C2B24FF6CE63B9EEA4BD5D63C82`) shows red
  "First use of ScrollArea ID B61C" in Current Folder and "Second use of
  ScrollArea ID B61C" in Command Window over `.git/.github/.re-ghidra` rows;
  AFTER `docs/screenshots/s3-ids/desktop_1280x720.png` (93120 B, SHA-256
  `B2E79E0D110C3DD79AD0A51FD3F8CDB8C8DD6A937E50987EC0D47979BD6FD931`) shows zero
  ribbons, no dot-entries (list starts at `apps`), and the Hidden checkbox.
  egui 0.31.1 findings (recorded in audit section 5 + BACKLOG 6.4): duplicate-id
  warnings are `warn_on_id_clash` debug-build paint only
  (`egui-0.31.1/src/memory/mod.rs:278-281`), no `Context::warnings()` API and no
  log channel — a logger wiring cannot capture them; screenshots of debug
  builds are the verification channel, and the hover-visited sweep across
  panels/overlays moves to the W-suite step (needs interactive input).
  Desktop-crate test runs during S3: 19 -> 20 (lines 1+0+19+0 per binary, then
  +1 new unit test), build/fmt clean; workspace count now 352 (manifest
  snapshot `docs/test-manifest.txt`).
- **Takeover session (2026-11-04 13:46-14:0x, this block):** the owner asked to
  resume, open the app, and confirm everything works. During verification,
  file-hash probes caught a SECOND writer (the parallel Antigravity/Gemini
  agent) actively editing sources (editor.rs content flip mid-read;
  current_folder.rs grew 23 KB -> 37 KB across three hashes; one torn save
  failed to compile). Owner closed it; after 14:05:21 no further foreign
  writes were observed (30 s stability windows, then 2+ min quiet). The
  takeover session verified: desktop tests 61/61
  (`docs/VERIFICATION.md`), clippy
  `-D warnings` clean, manifest exact parity 393/393, and captured real-window
  screenshots at 1280x720 / 1920x1080
  (`docs/screenshots/gui-editor/after/`, SHA-256 prefixes `CCA4ADC994589B05` /
  `2F7A97262C88BD56`) confirming the single-window, all-panels-visible, ASCII
  clean layout (verification log not retained in this repository; CI is the source of truth).
- Next: proposal — S4 = editor duplicated Run-Section blocks + emoji sweep +
  toolstrip overflow; input-handling fixes (Command-Window focus steal, global
  Ctrl+C intercept, completion popup arrows, gutter row keys) are a separate
  step needing interactive verification (currently recorded in audit section 5).
  STOPPED for owner approval of the S3 report.
- **S4 prior-session work STAGED (this session, commits `4809d9b`, `ecc7baa`,
  `3059f72`)**: the working tree was found with a large coherent uncommitted
  batch (editor rewrite + `actions`/`syntax`/`ui_registry` modules, ribbon
  registry audit, project-aware Current Folder, `editor_gui_tests` /
  `gui_clean_tests` suites, rfd folder-picker dependency, evidence logs).
  Per R14 honesty: this was NOT written this session; it was verified green
  this session (fmt, clippy `-p matrust_studio` 0 warnings, full workspace
  `cargo test` exit 0) and committed as its own labeled commits so later work
  sits on a green base. gui-editor milestone note at `docs/milestones/gui-editor.md`.
- **S5 file tree + cloud dialog COMPLETE (commit `73495c2`)**, owner-requested
  feature step on `gui/polish-1`:
  - Current Folder gained a **Tree view** (default on) with a `Tree view` /
    `List view` toggle button (`current_folder.view` control): whole-project
    structure rooted at the project root, folders expand in place via `[-]`/
    `[+]` toggles, children are read from disk only while expanded, hard caps
    (depth 24) shared with the totals walk; double-click opens files in the
    editor, single click selects. Flat list behavior preserved exactly.
  - **Whole-project totals** line: `project: N folders, M files` under the
    project root, respecting the Hidden toggle, capped at 65,536 entries
    (`+` suffix when truncated), cached and recomputed lazily.
  - **Auto-refresh on save**: all three editor save sites (Ctrl+S, File menu,
    EDITOR ribbon) now route through `MatStudioApp::save_active_doc`, which
    invalidates the totals cache; the list itself re-reads the disk every
    frame, so files created inside the app appear immediately.
  - **Cloud Drives dialog** (File menu → `Cloud Drives...`): OneDrive,
    Google Drive, Dropbox, iCloud Drive rows with detected-sync-folder
    status, one-click Open, official sign-in/download hyperlinks (browser),
    and an `Open any folder...` picker for custom locations (covers Google
    Drive desktop's mounted drive letter). SECURITY MODEL: MATSTUDIO never
    sees or stores passwords/tokens; sign-in happens in the provider's own
    app or browser; synced folders are read as ordinary local folders.
    New module `apps/desktop/src/cloud_accounts.rs` with pure, tested
    detection (`detect_sync_folder` takes an injected env lookup).
  - +6 new lib tests (3 `current_folder` tree/totals, 3 `cloud_accounts`
    detection). Tests never decrease: manifest regenerated **352 -> 393**
    (the +41 delta = +6 this session, +35 staged prior-session tests).
  - In-app OAuth sign-in is deliberately NOT implemented (would require an
    owner-registered OAuth client id per provider; no credentials may be
    invented) — recorded in BACKLOG section 7.

## Test counts

| Configuration | Passed | Failed | Manifest match |
| :--- | ---: | ---: | :--- |
| Default features | 495 | 0 | Exact (495 / 495) |
| `--all-features` | 495 | 0 | Exact (495 / 495) |
| `--no-default-features` | 495 | 0 | Exact (495 / 495) |

`docs/test-manifest.txt` lists all 495 tests. The CI test-count guard strictly asserts exact parity between manifest and active workspace tests. Counts re-verified this session (2026-11-07): default, all-features, and no-default-features all exit 0 with 495 passing.

## Recent Milestones Completed

1. **Runtime & CLI Integration of GNU Octave Execution Bridge (`crates/mlapp-runtime`, `crates/mlapp-host`, `crates/mlapp-cli`)**:
   - Integrated `mlapp-octave` as a first-class execution fallback for unsupported functions across App Designer runtime callbacks.
   - Added `AppRuntime::configure_bridges(allow_matlab: bool, allow_octave: bool)` with backward-compatible `configure_matlab_bridge`.
   - Added `allow_octave: bool` to `HostConfig` in `mlapp-host` and `--allow-octave` CLI flag in `mlapp-cli`.
   - Enhanced runtime error reporting: unsupported functions clearly suggest both `--allow-matlab` and `--allow-octave`.
   - Added unit and E2E host integration tests (`test_runtime_configure_bridges_octave_delegation`, `test_runtime_unsupported_function_hints_both_bridges`, and E2E UI mutation testing in `host_tests`).
   - Workspace test count grew from 349 to 351 tests (+2 net tests, 0 regressions).

2. **GNU Octave Subprocess Bridge (`crates/mlapp-octave`)**:
   - Clean-room out-of-process bridge communicating strictly over standard OS pipes (`stdin`/`stdout`/`stderr`) with `octave-cli`.
   - Atomic batch evaluation via `octave-cli --eval` with JSON response markers, avoiding pipe deadlocks.
   - Function-level safety allowlist denying hazardous system commands (`system`, `dos`, `delete`, `exit`, `fork`, etc.).
   - Full serialization/deserialization between `rustlab_math::Value` and JSON-lines IPC protocol.
   - Comprehensive test suite (8 tests) covering process lifecycle, mock execution, safety denials, protocol roundtrips, and differential oracle conformance when Octave is present.

2. **PyO3 Python Package (`matrust` / `crates/mlapp-py`)**:
   - Clean-room CPython extension module compiled with `abi3-py39` forward compatibility for Python 3.9+.
   - `Package` context manager for inspecting `.mlapp`/`.mlappinstall` archives, extracting components, and inspecting manifests.
   - `Session` interactive handle for evaluating scripts, dispatching UI events, and getting/setting component properties.
   - Custom typed exception hierarchy (`MlappError`, `PackageError`, `SessionError`).
   - Clean zero-warning error conversion pattern satisfying strict `-D warnings` without any `#[allow]`.
   - Complete Python package layout under `python/matrust` with PEP 484/561 type stubs (`__init__.pyi`), marker (`py.typed`), and unit test suite.
   - Tested and passing in CI across 6 platform/version combinations (Python 3.9 & 3.12 on Ubuntu, macOS, Windows).

3. **Desktop IDE Shell & Simulink Integration (`crates/matrust_studio`, `crates/mlapp-sim`)**:
   - Complete desktop IDE layout with multi-tab editor, workspace inspector, simulation canvas, and offline documentation browser.
   - Zero AI / Zero Copilot policy enforced across all source files and dependencies.
   - Acceptance test `T-ENTROPY` verified.
   - Simulink block diagram engine (`crates/mlapp-sim`) with discrete/continuous solvers and `.slx` inspection CLI (`slx-inspect`).

4. **CI Matrix & Security Hardening**:
   - 14/14 automated jobs on GitHub Actions:
     - Code Formatting (`cargo fmt --check`, cbindgen header validation)
     - MSRV Check (`cargo check` on Rust 1.88.0)
     - Security & License Checks (`cargo-deny`, `cargo-audit`, Zero-AI crate/symbol check)
     - Unsafe Code Audit (`cargo-geiger`)
     - Cargo Fuzz Targets (5 targets running 60s each in smoke run)
     - 6x Python Bindings matrix (Python 3.9 and 3.12 on Ubuntu, macOS, Windows)
     - 3x OS Build & Test matrix (Ubuntu, macOS, Windows) with Clippy, unit/integration tests, benchmarks, docs, and test count guard.

## Honest limitations

- **0 real `.mlapp` apps tested.** `MLAPP_REAL_APPS_DIR` is unset. The container opt-in test prints `SKIPPED: MLAPP_REAL_APPS_DIR unset (0 real apps tested).`
- **MATLAB Oracle:** A MATLAB R2025a tree exists at `C:\Program Files\MATLAB\R2025a`, but no runnable `matlab.exe` was found; `matlab -batch` cannot run without a host MATLAB binary. The fallback GNU Octave bridge serves as an active open-source differential oracle.
- **GUI Automation:** Native eframe window and automated `--autotest` headless engine are verified; physical Win32 mouse click simulation on remote headless runners remains guarded by synthetic event queues.
- **C ABI:** Header generation (`mlapp.h`), integer handles, null checks, and panic isolation are tested; no Miri/ASan fuzzing of raw pointers.

## Recent Milestones Completed

6. **B2.2 rustlab-plot: renderer-independent plot model + deterministic SVG** (`crates/rustlab-plot`, builtins in `rustlab-stdlib`):
   - New core crate `crates/rustlab-plot` with `Figure`, `Axes`, `Series` (Line/Scatter/Bar/Histogram), `Text`, `Legend`, `Scale`, and deterministic SVG renderer (`to_svg`) with fixed 2-decimal formatting and documented "nice ticks" rule.
   - Thread-local `PlotRegistry` mirrors MATLAB's `gcf`/`gca` within one interpreter session.
   - Builtins registered in `rustlab-stdlib`: `figure`, `plot`, `scatter`, `bar`, `hist`, `hold`, `subplot`, `xlabel`, `ylabel`, `title`, `legend`, `xlim`, `ylim`, `grid`, `saveas`.
   - Help entries, `docs/functions/*.md` for each builtin, `docs/builtins.txt` updated (240 entries).
   - Snapshot tests with byte-equal `.svg` reference files.
   - No GUI, no egui, no resvg; works under `--no-default-features`.

6. **GUI Polish S4: input-handling fixes** (`apps/desktop`):
   - Command Window no longer steals focus when buffer is empty
   - Command Window arrow keys scoped to focused input only
   - Completion popup arrow keys scoped to popup focus
   - Editor gutter rows use stable per-row ids
   - Fixed pre-existing B2.2 rustlab-stdlib compile errors (ishhold_fn,
     ok_or_else on Option, nice_step clippy lint)
   - Added 2 new regression tests; all 63 desktop tests pass

## Next steps

1. Keep CI strictly green on every push without loosening `-D warnings` or deleting tests.
2. Provide real `.mlapp` fixtures via `MLAPP_REAL_APPS_DIR` when available to test full third-party App Designer packages.
3. GUI polish S5 (layout fixes: toolstrip overflow, help panel clipping,
   figures empty state) is the next proposed step.
