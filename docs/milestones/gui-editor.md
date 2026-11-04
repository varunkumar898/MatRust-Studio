# Milestone GUI-EDITOR — Code editor fills its panel

**Branch:** `gui/polish-1` · **Date:** 2026-11-04 · **Scope:** `apps/desktop` code editor panel only.

## 1. Problem observed in the real window

Before screenshots at 1280x720 and 1920x1080
(`docs/screenshots/gui-editor/before/`) show the document toolbar, then an
approximately 80 px dark text strip, then the gray panel background for the
rest of the Editor panel. The text strip and the gutter were two independent
scroll areas, so they could not be guaranteed to stay vertically aligned.
The panel is otherwise large enough; the content simply did not claim it.

## 2. What this milestone changes

1. A pure layout model (`EditorLayout`, `compute_editor_layout`) computes the
   editor viewport, gutter, text viewport, visible-row range, row tops, content
   size and gutter width from the panel rect, line data and scroll offset.
2. One `ScrollArea::both` owns the text content. The gutter is painted as an
   overlay from the same scroll offset and row model, so row `y` positions come
   from one source and cannot desync. The editor background is painted across
   the whole panel rect (theme `editor_bg`), including rows after end-of-file.
3. Gutter rows exist for every visible row. Rows after the last text line are
   drawn dimmed. The width is derived from the digit count of
   `max(line_count, visible_rows)`, so 99 -> 100 lines grows the gutter.
4. Font is `FontId::monospace`, default 14 px. Ctrl+= / Ctrl+- / Ctrl+0 change
   the editor font only (global egui zoom is disabled). Ctrl+0 resets to 14.
   Line height comes from the same font, so gutter numbers and text rows share
   the same `y`.
5. Tab inserts four spaces when the editor is focused. Selection, copy/paste,
   undo/redo and IME remain owned by egui's `TextEdit`.
6. Clicking below the last text row asks for focus and places the cursor at the
   end of the last line. Typing is handled by the focused `TextEdit` next frame.
7. Word wrap is an explicit, default-off toggle. With wrap off the scroll area
   is wide enough for the longest logical line, so the right edge is reachable.
8. Unique IDs are scoped by `push_id` with a stable per-document file key. A
   small UI control registry (`ui_registry`) records the control spec, the real
   egui id and its rect for every editor control, and reports duplicate ids in
   the same pass.
9. The duplicate syntax badges / Run Section buttons in the old toolbar are
   removed while the toolbar is touched for registry registration.

## 3. Scope

**IN:** `apps/desktop/src/editor.rs`, new `apps/desktop/src/ui_registry.rs`,
`apps/desktop/src/lib.rs`, `apps/desktop/src/app.rs` for the editor font
shortcut plumbing and the idle Ctrl+C copy fix, tests, screenshots, state docs.

**OUT:** other panels, docking layout algorithms, syntax highlighting changes,
new language builtins, word-wrap gutter numbering per visual row (wrap is a
display toggle; gutter numbers stay per logical line), completion behaviour.

## 4. Public API

- `compute_editor_layout(panel_rect, content, font_size, line_height,
  char_width, scroll_offset, word_wrap) -> EditorLayout`
- `EditorLayout { panel_rect, content_rect, gutter_rect, text_rect,
  line_height, visible_rows, first_visible_line, rows, ... }`
- `EditorPanel::show_content(&mut self, ui, panel_rect, theme) -> EditorLayout`
  (the production `show` calls it with the body rect left after the toolbar).
- `EditorPanel::{set_font_size, handle_font_shortcuts, file_key_for_active}`.
- `ui_registry::{begin_frame, register, audit, editor_controls, editor_control}`.

## 5. Error ids

No new user-facing error/warning ids. Registry clips are internal test/audit
data and never raised to the user.

## 6. Risks

- egui's `TextEdit` owns its internal galley. The model's row height is taken
  from the same `FontId`, but a future egui change could add margin; the
  integration test compares the real galley row tops with the model and fails
  if that happens.
- Font shortcuts collide with egui's built-in zoom; global zoom is disabled in
  `MatStudioApp::update` before shortcuts run.
- The gutter is an overlay after the scroll area so horizontally scrolled text
  is covered by the gutter background. This is intentional and keeps the gutter
  fixed horizontally while sharing the vertical offset.

## 7. Test plan (written first)

- `test_editor_content_rect_matches_panel_at_1280_and_1920`: empty and 500-line
  documents; rendered content rect equals the supplied panel rect within 1 px.
- `test_editor_gutter_rows_match_visible_text_rows`: row count equals visible
  rows; every gutter row `y` equals the real galley row `y`.
- `test_editor_click_in_empty_area_focuses_and_puts_cursor_at_end`.
- `test_editor_ids_are_unique_per_file_and_panel`: two different documents in
  one pass have distinct ScrollArea/TextEdit ids and the registry reports no
  clash; a negative control proves the audit detects deliberate duplicates.
- `test_editor_font_shortcuts_and_tab_inserts_spaces`.
- `test_editor_gutter_width_grows_from_99_to_100_lines`.
- `test_editor_controls_are_registered`: every control the editor registers has
  a spec in the central registry, and text/scroll/toolbar controls appear.
