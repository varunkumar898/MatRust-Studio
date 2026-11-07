# Milestone S4 — GUI Polish: input-handling fixes

**Branch:** `gui/polish-1` (same as S3) · **Scope:** `apps/desktop` input handling only.

## 1. Problem observed (from docs/gui-audit.md section 5)

1. **Command Window steals focus** (`command_window.rs:190-192`): `request_focus()` fires every frame whenever the buffer is empty and the panel is not focused — suspected focus-fight with the Editor.
2. **Command Window reads global arrow keys** (`command_window.rs:128-150`): ArrowUp/ArrowDown fire every frame regardless of focus, rewriting the command-line buffer while working in the Editor.
3. **Global Ctrl+C intercept** (`app.rs:368-375`): `command+C` unconditionally calls `stop()`/`debug_stop()` even when the engine is idle — hijacks copy-to-clipboard everywhere.
4. **Completion popup reads global keys** (`editor.rs:611-623`): ArrowUp/Down/Enter handled globally in one frame alongside other subsystems.
5. **Gutter rows lack stable keys** (`editor.rs:472-485`): fold/breakpoint gutter row ids come from a formatted label string — no stable per-row key, causes gutter/text desync.
6. **Editor duplicated Run-Section blocks** (audit R3): toolbar renders 3× "Run Section", 2× "Run Selection", duplicated badges — literal render of duplicated code blocks.
7. **Emoji labels** in menu/toolbar labels (audit section 8): e.g. "New Script (Ctrl+N)" with emoji prefix.
8. **HOME toolstrip overflow** (audit R7): groups flow past the right window edge with no scroll/wrap/ellipsis at 1280×720.

## 2. Scope

**IN:**
- `apps/desktop/src/command_window.rs` — fix focus steal + scope arrow keys to focused buffer
- `apps/desktop/src/app.rs` — fix global Ctrl+C (only stop when engine running)
- `apps/desktop/src/editor.rs` — scope completion popup keys, stable gutter row keys, remove duplicated Run-Section blocks
- `apps/desktop/src/app.rs` — remove emoji from menu/toolbar labels
- `apps/desktop/src/app.rs` — HOME toolstrip wrap/scroll so no group goes off-screen

**OUT:** docking layout algorithms, docking id salt (done S3), status bar cwd (R12, separate), block diagram window ids (separate), figures empty state (layout pass), workspace table (layout pass).

## 3. Error ids

No new user-facing error ids. All changes are internal focus/input behavior.

## 4. Test plan (written first)

- `test_command_window_does_not_steal_focus_when_buffer_empty`: request_focus not called when buffer empty
- `test_command_window_arrow_keys_scoped_to_focus`: ArrowUp only affects command line when panel focused
- `test_ctrl_c_only_stops_when_engine_running`: idle engine + Ctrl+C does nothing; running engine stops
- `test_completion_popup_keys_scoped`: ArrowUp/Down/Enter only handled when popup open
- `test_gutter_rows_have_stable_keys`: repeated renders produce identical row ids
- `test_editor_toolbar_single_run_control_set`: exactly one "Run Section", one "Run Selection"
- `test_menu_labels_no_emoji`: no emoji glyphs in menu/toolbar labels
- `test_toolstrip_controls_fit_at_1280`: all ribbon controls within panel bounds at 1280×720

## 5. Risks

- Scoping arrow keys may break command-history navigation when panel is focused — test both focused and unfocused.
- Ctrl+C copy-to-clipboard fallback: when engine idle and no selection, clipboard should get "C" — need to verify egui provides clipboard access.
- Stable gutter keys must survive tab switches and document reloads.
