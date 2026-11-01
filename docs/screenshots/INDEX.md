# MatRust-Studio Screenshot Index

This document catalogs all verified GUI window and autotest engine screenshots captured during automated test execution.

| Commit Hash | Artifact File | Resolution | SHA-256 | Description | Verification Status |
|---|---|---|---|---|---|
| `c4a0ff9` | `docs/screenshots/c4a0ff9/gui_window_autotest.png` | 640x480 | `5D700F06844D306C32103928094EF33A293AB667A93A134A58EBAC02C6499AE3` | End-to-end autotest execution of `AutotestDemoApp`: Button click executes callback, mutates `StatusLabel.Text` to `'PlotGenerated:50pts'`, plots 50-point sine curve onto `PlotAxes`, validated via `ExpectProperty` and rendered to PNG | Verified |
| Baseline | `docs/screenshots/sample_app_autotest.png` | 800x600 | (Tracked) | Full fixture autotest of `r2021a_sample.mlapp`: `ThresholdField` mutation to 42, `StartBtn` click dispatch | Verified |
| Baseline | `docs/assets/gui_window_sample.png` | 640x480 | (Tracked) | Static layout capture with UIFigure, Panel, Button, and UIAxes | Verified |

## Verification Command
To reproduce and verify the autotest engine execution:
```bash
cargo test -p mlapp-render --test render_tests -- test_autotest_headless_e2e_with_plot_and_label_mutation
cargo run -p mlapp-cli -- run fixtures/sample_apps/r2021a_sample.mlapp --gui --autotest tests/autotest/sample_autotest.json
```
