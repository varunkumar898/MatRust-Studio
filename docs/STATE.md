# MatRust Studio / RustLab Project State

**Current branch:** `hardening` (PR #11 into `main`)
**Repository:** `https://github.com/varunkumar898/MatRust-Studio.git`
**Current code commit:** `be5d46fb2f209129bd9b8366f9fe848252a937c8`
**Date:** 2026-11-01
**Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`)
**Local stable toolchain:** Rust 1.99.1
**Declared MSRV:** Rust 1.88.0 (raised from the false 1.80.0 claim)
**Verification status:** Local default, all-features, and no-default-features test
runs pass at 315/315/315. Local clippy (all-features and no-default-features)
and `cargo fmt --check` pass. CI for `be5d46f` is pending push at the time this
file was written; the previous run at `c19db91` passed every job except MSRV.

## Test counts

| Configuration | Passed | Failed | Raw log |
| :--- | ---: | ---: | :--- |
| Default features | 315 | 0 | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-default.log` |
| `--all-features` | 315 | 0 | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-all-features.log` |
| `--no-default-features` | 315 | 0 | `docs/verify-logs/2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-no-default-features.log` |

`docs/test-manifest.txt` has 315 test names. `Compare-Object` between the
default and no-default `--list` outputs found **0** name differences. The CI
guard now requires exact equality between the active default test count and
the manifest, not a lower-bound check.

## What changed in this session

- `ValueChanged` callbacks receive an event struct with `Source`,
  `EventName`, and `Value`; the new value is available as `event.Value`.
  The E2E test is
  `test_value_changed_callback_receives_event_value_and_app_field`.
- `set_component_property` and callback failures in the host worker are
  surfaced as `HostNotification::Error` instead of being discarded.
- `mlapp run --gui` now calls `mlapp_render::egui_backend::run_gui_window`
  and opens a native eframe window. Previously it printed a message and
  continued headlessly.
- A Windows real-window run opened `FixtureApp_R2021a`, was captured with
  Win32 `PrintWindow` to `docs/assets/gui_window_live.png` (SHA-256
  `06CB900F5C02FD8028DA5D3B1B6FA7339657A999B94E9C1CB2E95A4E8D3F8A4B`),
  and exited 0 via `MLAPP_GUI_AUTOCLOSE_MS`. Button-click dispatch is still
  manual.
- MSRV raised honestly to 1.88.0. The locked dependency graph contains
  crates requiring edition 2024 and Rust up to 1.88. The 1.80.0 CI failure
  is preserved in
  `docs/verify-logs/2026-11-01-c19db918d8a293c8a2c8bc56147a3c7ac03c10a4-msrv-1.80-github-job-124807996217.log`.
  A clean 1.88.0 `cargo check --workspace --all-targets --all-features`
  (x86_64 Windows GNU target) passed in 4m06s.
- Added `cbindgen.toml`; `crates/mlapp-ffi/include/mlapp.h` is regenerated
  from source and checked in CI. Added
  `test_matmul_huge_output_is_rejected_without_allocating`, and a
  `// SAFETY:` comment for the audited `matrixmultiply::dgemm` block.
- Added `docs/SAFETY-AUDIT.md` with marker counts, unsafe inventory, panic
  policy, mutex-poison handling, and residual risks.
- Corrected README claims: test count, MSRV, unsafe-code wording, and GUI
  status.

## Evidence logs at `be5d46f`

- Default tests:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-default.log`
- All-features tests:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-all-features.log`
- No-default tests:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-no-default-features.log`
- Default/no-default `--list`:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-cargo-test-default-list.log`
  and `...-no-default-list.log`
- Clean MSRV check:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-msrv-1.88.0-clean-cargo-check-all-features.log`
- GUI capture:
  `2026-11-01-be5d46fb2f209129bd9b8366f9fe848252a937c8-gui-window-capture.log`
- Safety markers/unsafe sites/panic policy:
  `...-safety-marker-counts.log`, `...-unsafe-sites.log`,
  `...-panic-unwind-mutex.log`
- cbindgen header check:
  `...-cbindgen-header-check.log`

## Honest limitations

- **0 real `.mlapp` apps tested.** `MLAPP_REAL_APPS_DIR` is unset. The
  container opt-in test prints
  `SKIPPED: MLAPP_REAL_APPS_DIR unset (0 real apps tested).`
- **No MATLAB oracle.** A MATLAB R2025a tree exists at
  `C:\Program Files\MATLAB\R2025a`, but no runnable `matlab.exe` was found;
  `matlab -batch` cannot run. MATLAB-vs-runtime comparisons, the MATLAB
  bridge, and Simulink work are blocked.
- **GUI:** native window launch and screenshot are proven on Windows; an
  automated mouse click that mutates app state is not. CI is headless and
  runs only the offscreen screenshot test.
- **C ABI:** header generation, integer handles, null checks, and panic
  isolation are tested; no Miri/ASan/fuzzing of FFI pointers.
- **Real-app failure semantics:** an empty/unset `MLAPP_REAL_APPS_DIR` is
  reported as skipped, not as coverage of real MathWorks apps.

## Next steps

1. Push `be5d46f`, wait for the Linux and Windows test/clippy/fmt/all-features
   and MSRV jobs, and record the run URL and conclusion.
2. Add an automated GUI click test on a desktop runner, or keep GUI marked
   Partial.
3. Supply real `.mlapp` fixtures via `MLAPP_REAL_APPS_DIR` to replace the
   "0 real apps tested" result.
4. Provide a runnable licensed MATLAB installation for oracle and bridge work.

Historical RustLab milestone notes and old binary hashes remain available in
git history; they are not the current MatRust Studio build and must not be
quoted as current evidence.
