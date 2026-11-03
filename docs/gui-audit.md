# GUI Audit S1 — MATSTUDIO Desktop

Milestone: `gui/polish-1` S1 · Branch `gui/polish-1` (created from `main` @ `425f54e`) · Date 2026-11-03
Method, in order: (1) full source read of all 16 files in `apps/desktop/src`; (2) real-window
walk of a debug build (`target\debug\matrust_studio.exe`) captured with PrintWindow at
1280x720 and 1920x1080 (scripts `scripts/capture-desktop-s1.ps1`, `-extras.ps1`); (3) greps
for rule-specific strings (emoji, trademarks, watermarks). No source code was changed; the
only repo changes in this step are this audit, the capture scripts, the screenshots and the
verification log.

Baseline verification (start counts): `scripts/verify.ps1` PASS (log line 4277
"All verification steps passed successfully!"), manifest check passed with 351 entries
(log line 4223), summed pass counts from the log: default 351, all-features 351,
no-default 351. Log:
`docs/verify-logs/2026-11-03-gui-s1-baseline-verify.log`
SHA-256 `42D6F3BBB6A42896665919B4E4E7038843A923A2655177E6ABEC2D687B2D5F2A`.

Capture evidence (docs/screenshots/before/, real PrintWindow captures of the live window):

| File | SHA-256 (first 32) | Content |
| :--- | :--- | :--- |
| desktop_1280x720.png | `356117597960F316D4CA92A939DDB30B` | default layout 1280x720 |
| desktop_1920x1080.png | `CA755F9C869E0EAF0D1FE9CCF44C1316` | default layout 1920x1080 |
| desktop_simulink_panel.png | `B03D3B6AE15D060C07344E4A3778E94F` | Simulink panel opened by click |
| desktop_view_menu.png | `69F2685C83BF4B8A16F9B2CAB4D6387A` | VIEW toolstrip (click hit the tab, not the menu) |
| desktop_addons_panel.png | (no-visual-change; stray click) | Add-Ons panel NOT opened (see 6.5) |
| desktop_history_panel.png | (no-visual-change; stray click) | History panel NOT opened (see 6.5) |

---

## 1. Runtime findings from the real window (instrument observations)

| # | Observation | Evidence |
| :--- | :--- | :--- |
| R1 | Duplicate **egui ScrollArea ID** at runtime: red ribbons visible "First use of ScrollArea ID B61C" (Current Folder panel) and "Second use of ScrollArea ID B61C" (Command Window). Two unsalted `ScrollArea::vertical()`s (`current_folder.rs:73`, `command_window.rs:94`) derive the same auto id `B61C` in one frame and collide. | both 720/1080 shots |
| R2 | Warning ribbons are drawn **on top of panel content**, over the file list and over the Command Window banner text; with the banner text "Numerical Computing Environment" partially hidden underneath. | 1080 shot, bottom center |
| R3 | Editor tab bar shows **3 "Run Section" buttons, 2 "Run Selection", 3 duplicated "Clean/Syntax Warning" badges** — literal render of the duplicated code blocks in `editor.rs:306-346, 357-366, 367-386, 388-397, 399-408`. | 720 shot, editor tab bar |
| R4 | Editor gutter line numbers **desync from the text rows at rest** (visible: gutter shows 1,2,3 while the text shows 4 rows; 4th row `y = sin(x);` has no gutter number). Gutter and text live in two independent ScrollAreas (`editor.rs:443`, `editor.rs:509`) with no sync. | 720/1080 shots |
| R5 | Block Diagram canvas (Simulink tab): **grid rendered but none of the 5 default example blocks or wires visible** in the captured frame. Data model has 5 blocks (tested by `test_simulink_editor_canvas_and_solver`), the *visible* rendering is missing/blocked — cause unknown from static reading (candidate: canvas rect allocation vs block offsets in `simulink_editor.rs:484-662`). Needs a debug step in a later S-milestone. | simulink shot |
| R6 | Figures panel empty state text is **clipped on both sides** ("...igure (Run plot(x, y), scatter(x, y), or bar(x, y) to displ...") instead of wrapping: `figures.rs:132-140` painter.text does not wrap. | simulink shot (right column) |
| R7 | HOME toolstrip **overflows past the right window edge with no scroll/wrap/ellipsis** — at 1920 the ENVIRONMENT/RESOURCES groups are partially off-screen; at 1280 everything after "Favorites" is gone (Add-Ons, Layout, Set Path, Help buttons unreachable by any UI affordance). `app.rs:627-815`. | 720 + 1080 shots |
| R8 | Help panel heading row: caption "Offline Clean-Room Reference (Zero AI)" (`help_panel.rs:365-369`) is right-aligned in the heading row and **collides with the heading text**: renders as "Help & Documentat‖ation-Room Reference (Zero AI)". | simulink shot |
| R9 | Global search box clipped at right edge at 1280 ("Search Documenta…"): `app.rs:589-607` fixes width 280 but the row also holds the "Ctrl+Shift+Space" label. | 720 shot |
| R10 | Status bar `Ln 1, Col 1` never changes — no writer sets `statusbar.cursor_line/col` (fields dead; editor tracks cursor in `DocumentTab` but nothing copies it to the status bar). | all shots |
| R11 | Window title is "MatRust Studio - MATLAB-Compatible Scientific Computing IDE" (`main.rs:10`) — contains the trademark word, and is not the target format "MATSTUDIO - <file or folder>". | all shots |
| R12 | Status bar cwd shows only the folder **name** (e.g. "MATSTUDIO") while the address bar and Current Folder show the full path — the three cwd displays do not show the same text. `statusbar.rs:62-66`. | all shots |

## 2. G1 — watermarks, corner labels, badge text (remove/keep list)

| String | Where | G1 decision for S2 |
| :--- | :--- | :--- |
| "Offline Clean-Room Reference (Zero AI)" | `help_panel.rs:366` (overlays heading, R8) | remove (or About-only) |
| "Documentation (Offline, Zero AI)" button | `app.rs:548` | rename to "Documentation" |
| hover "Toggle Help & Documentation panel (Offline, Zero AI)" | `app.rs:793` | drop "Zero AI" |
| group label "BUILT-IN CLEAN-ROOM APPS" | `app.rs:844` | rename to neutral label |
| author string "MatRust Studio Clean-Room Team" (4x) + fake versions/versions of catalog items | `addons_panel.rs:28,38,48,58` + catalog itself | catalog is placeholder content (G2), see 3 |
| about texts incl. "Clean-Room MATLAB-Compatible..." | `app.rs:1260-1267` | KEEP (About dialog, allowed place) but reword to "compatible with the MATLAB language (not affiliated)" |
| comment-only "Clean-Room"/"Zero AI" in doc comments | various | not user-visible; optional cleanup |
| release binaries | not yet checked — REA strings check is S8 | planned S8 |

Related G1/UX bug: egui warning ribbons drawn over content (R2). These are egui debug
overlays produced by real id misuse (R1); removing the duplicate id removes the overlay.

## 3. G2 — placeholder / demo content in normal use

| Content | Where |
| :--- | :--- |
| Command Window starts with 2 hardcoded banner log lines | `command_window.rs:29-37` (W1 wants an empty Command Window) |
| Every new script is pre-filled with a hardcoded sine-wave demo script | `editor.rs:122-137` |
| Block Diagram starts pre-loaded with a 5-block feedback demo and a "Feedback Example" reload button | `simulink_editor.rs:85-93, 328-330` |
| Add-Ons catalog is 5 static fake items (fixed versions 1.2.0/1.1.0/..., "Community Contributor"), with pretend Install/Uninstall buttons that only toggle a Vec | `addons_panel.rs:23-74, 232-243` |
| Toolstrip buttons execute hardcoded demo scripts (plot/bar/histogram demos) | `app.rs:820-834, 867-870, 1069-1074` |
| Set Path dialog displays hardcoded fake path bullets ("<matrust-studio>/crates/...") rather than the real search path | `app.rs:1226-1242` |
| No-op buttons that only exist as grayed-capable-looking controls with hover "Not implemented yet": New Live Script, Find Files, Open, Clean Data, Favorites, Area, Pie, Preferences, Connections, Parallel, Publish to HTML/PDF, Feedback, Format Code | `app.rs:642-650, 681-683, 693-695, 745, 773-781, 811-813, 836-837, 948-950, 961-966, 987-995` |

## 4. G3 — clear/scoped actions and duplicates

Clear-family: 3 controls clear the Command Window (Menu "Clear Console (clc)" `app.rs:434`;
toolstrip "Clear Commands" `app.rs:697`; panel "Clear Console" `command_window.rs:81` — none
labeled "Clear Command Window"). Workspace clear: Menu `app.rs:438`, toolstrip `app.rs:675`,
panel `workspace_panel.rs:29` — none ask for confirmation when variables exist. History
clear is labeled bare "Clear" (`command_window.rs:208`). Figures have a per-tab "✕" close
(`figures.rs:51`) — equivalent of "Close Figure" albeit unlabeled. No confirm dialogs
anywhere. Layout presets appear twice (View menu `app.rs:472-499` + Home toolstrip
`app.rs:728-755`). Run set appears 3 times (Run menu, editor tab bar 6 buttons, editor
toolstrip) in addition to the missing per-section margin buttons.

Every user-visible interaction is a raw `ui.button/checkbox/selectable_label` — there is no
action registry/named Action layer anywhere in the app (grep: no registry/action helper
exists). G4 will need both the registry and a CI check to be built from scratch.

## 5. G5 — IDs, focus, input handling

| Finding | Where |
| :--- | :--- |
| Duplicate ScrollArea id runtime-collides (B61C twice per frame) | `current_folder.rs:73`, `command_window.rs:94` (both unsalted) |
| No panel-level `push_id(panel)`; no row-level stable keys in the Workspace list, Current Folder list, Help list, Add-Ons list (auto ids from labels/child-index only) | all list panels |
| `Window::new(format!("Block Parameters: {}", block.name))` — two blocks with the same name produce the same window id | `simulink_editor.rs:670-673` |
| Command Window reads **global** ArrowUp/ArrowDown input every frame (not scoped to its own focused text box): pressing ArrowUp while working in the Editor rewrites the command-line buffer | `command_window.rs:128-150` |
| Command Window **steals focus** whenever its buffer is empty and it is not focused (`request_focus()` every frame) — suspected focus-fight with the editor; not yet verified interactively in this S-step | `command_window.rs:190-192` |
| Global Ctrl+C intercept: `command+C` unconditionally calls stop()/debug_stop() even when the engine is idle, hijacking copy-to-clipboard everywhere | `app.rs:368-375` |
| Completion popup also reads global ArrowUp/Down/Enter (multi-subsystem key handling in one frame) | `editor.rs:611-623` |
| Editor fold/breakpoint gutter rows get ids from a formatted label string; no stable row key | `editor.rs:472-485` |

Runtime ID warnings: only the ScrollArea B61C pair was visible in the default layout. A
complete EGUI-warning sweep (hover-visited all panels/overlays) was not possible without
code changes (no logger is wired); the static candidate list above is the audit-level
result. The B61C pair is confirmed fixed-evidence from the real window.

## 6. Layout / G6 evidence (also see section 1)

6.1 Toolstrip overflow past window edge, no scroll/wrap (R7). 6.2 Help panel columns clip
mid-word (no wrap/ellipsis/tooltip): heading, captions, function list items
(`help_panel.rs:444-447`). 6.3 Figures empty state clipped (R6). 6.4 Workspace "table" is
not a table: headers are labels with separators; values in `ui.horizontal` rows have no
fixed columns, no truncation, no tooltip (long previews push the row) `workspace_panel.rs:58-126`.
6.5 The extras capture script targeted the View *menu* with a click at (369,88) but the
app receives it as the VIEW *tab* click; the two follow-up clicks therefore missed and the
Add-Ons and History panels were not captured this round — those two screenshots show an
unchanged layout and must be treated as no-ops (honest limitation; both panels are still
fully inventoried from source in section 8).
6.6 Status bar not visible in any captured frame (both resolutions end with panel content;
the egui bottom panel seems to fall outside the captured region or collapses to zero
height). Needs a rect check in S4 (G6) — flagged, not diagnosed.

## 7. G7 — single source of truth

The Worker Client IS the engine session (cwd concept is NOT in the engine; cwd lives in 2
GUI places — Current Folder panel + address bar + status bar name — with the panel "Up"
and directory-enter paths not writing back to the address bar (`current_folder.rs:33-41,
102-105`; app only syncs address bar edits via `set_directory` at `app.rs:1050`). Engine has
no notion of cwd: `save('workspace.mat')` writes relative to the process cwd (started where
the app started), not the GUI cwd. Last-error single home: status bar state + command window
push (`execution.rs` RunResult path) — but the red gutter/`error_line` is only set from
breakpoint hits, never from errors (`app.rs:270-295`).

## 8. Panel-by-panel control inventory (id/panel/label/action)

All controls are raw egui calls; no named action. "Id" column = the egui identity source.

### 8.1 Menu bar (`app.rs:393-557`)
| Panel | Label | Action | Notes |
| :--- | :--- | :--- | :--- |
| File | New Script (Ctrl+N) | editor.new_script | emoji label |
| File | Open File... (Ctrl+O) | none (no-op) | missing implementation + missing Ctrl+O shortcut |
| File | Save (Ctrl+S) | doc.save | |
| File | Save As... | none (no-op) | |
| File | Import Data (CSV)... | import dialog | |
| File | Close Tab (Ctrl+W) | close_active_tab | Ctrl+W shortcut not implemented |
| File | Exit | viewport close | |
| Edit | Undo/Redo (Ctrl+Z/Y) | none (no-op) | |
| Edit | Clear Console (clc) | command_window.clear | G3 label |
| Edit | Clear Workspace (clear) | worker.clear | no confirm (G3) |
| View | 9 panel checkboxes | dock open/remove | emoji labels |
| View > Layout Presets | 5 buttons | set preset | duplicate of toolstrip |
| View | theme switch | Theme::light/dark | prefs.theme_mode only in memory |
| Run | Run Script (F5)/Run Section (Ctrl+Enter)/Run Selection (F9) | execute code | duplicate set; F9 executes whole content not selection |
| Run | Stop Execution (Ctrl+C) | worker.stop | |
| Help | Documentation (Offline, Zero AI) | open Help panel | G1 string |
| Help | About MatRust Studio | about dialog | |

### 8.2 Toolstrip ribbons (`app.rs:562-997`)
HOME: FILE (New Script, New Live Script*, Open*, Find Files*), VARIABLE (Import Data, Save
Workspace, Clear Workspace*—no confirm, Clean Data*), CODE (Favorites*, Clear Commands),
SIMULINK (Simulink→opens panel; trademark), ENVIRONMENT (Layout*dup, Set Path* dialog is
fake, Add-Ons, Preferences*, Connections*, Parallel*), RESOURCES (Help toggle, Learn→opens
log2 help article, Feedback*). PLOTS: Line/Scatter/Bar/Histogram (each runs a hardcoded
demo; **`hist` is not registered by the GUI Session** `execution.rs:212-429` — Histogram
button is guaranteed to error), Area*, Pie*. APPS: 5 buttons bound to `handle_launch_app`.
EDITOR: Run/Run Section/Run Selection/Stop/Save/Format Code* (dup Run set). PUBLISH: HTML*/PDF*.
VIEW: 7 panel checkboxes (Add-On Explorer and Command History missing here but present in View menu).

(*) = no-op button.

### 8.3 Address bar (`app.rs:1000-1062`)
Back/Forward/Up glyph buttons, folder emoji, editable path TextEdit (Enter applies; invalid
input silently reverts). History is GUI-only Vec, not persisted.

### 8.4 Current Folder (`current_folder.rs`)
Heading; "⮤ Up" (dup of address-bar Up; doesn't sync back G7); path label (not a breadcrumb);
file list rows: icon by extension (+ blank 2-space icon for "other"; `.svg` not covered),
**no dotfile/build-dir hiding, no reveal toggle**; single click select; double click
open/enter; read-failure error label. Scroll area unsalted (id B61C source #1).

### 8.5 Workspace (`workspace_panel.rs`)
Heading; "🗑 Clear" (no confirm); "🔄" refresh icon-only button; emoji search field + "✕"
clear-search; Name/Value/Size/Class label-row pseudo-table; row click selects, double-click
opens inspector window ("Variable: <name>"), which shows Class/Size/Bytes + monospace
preview. No sorting, no right-click menu, no column resize/ellipsis, no "show constants"
toggle. Empty states: "No variables in workspace"/"No matching variables" (1 line — good).

### 8.6 Editor (`editor.rs`)
Tab strip (title + dirty "● {} *" (star duplicates the badge), active-only "✕"), "+" new; on
the right: 3x duplicated Clean/Syntax-Warning badge and Run Section and Run Selection/Run
Script buttons (R3; "Run Selection" runs whole doc). Gutter: per-line rows — click toggles
breakpoint, fold-toggle at fold starts (no stable row key); text area: code_editor,
`lock_focus(true)`; Ctrl+Space completion popup (window id from title, arrow navigation
global). Fully hardcoded default demo content (G2). No syntax highlighting: theme fields
`syn_keyword/syn_number/syn_comment/syn_operator` are completely unused (only `syn_builtin`
is used for the Workspace class color and a folder icon color; `theme.rs:31-35`,
`workspace_panel.rs:114`, `current_folder.rs:85`) — the header comment "syntax formatting"
is not true by code.

### 8.7 Command Window (`command_window.rs`)
Heading; "Clear Console" button; log ScrollArea (id B61C source #2) with monospace
input/output/error colored lines; input line: global-up/down history handling, "▶ Run"
button, `clc` special-case, Enter executes; auto `request_focus()` (see 5./G5). History
panel (same file): heading, bare "Clear", reversed list, double-click re-executes. Banner
lines hardcoded (G2/W1).

### 8.8 Command History — see 8.7 (`show_history_panel`).

### 8.9 Figures (`figures.rs`)
Figure tab labels ("● <title>"), "✕" per figure, "+ New", "Reset View", per-figure controls:
grid checkbox, "Series: N" label; canvas: drag = view_offset (zoom state exists but unused
in mapping `figures.rs:9-10, 192-199`); grid lines + fixed 5-tick labels `{:.1}`; axis
labels painted at hardcoded offsets; series drawing Line/Scatter/Bar/Histogram with
matplotlib-style default palette colors; save-as SVG/PNG absent; zoom/pan = drag only, no
wheel. Empty-state clipped (R6). No "Close Figure" scope button (G3) — close is "✕".

### 8.10 Help (`help_panel.rs`)
Heading + colliding G1 caption (R8); search box w/ emoji + "Clear"; category combo
(from_label "Category" renders confusingly after the dropdown); 2-column list/detail; list
rows "name - summary" (clipped mid-word); detail: heading, category, summary, Syntax block
(hardcoded light-blue `Color32::from_rgb(180,220,255)` — near-invisible in light theme),
Description, Example block (hardcoded light-green) — **no "Run example" button (G11)**.
Catalog = 38 hand-written entries (`DOC_CATALOG 21-326`) — panel caption shows "Functions
(38)" — vs **225 builtins in the tracked `docs/builtins.txt`**: coverage ≈ 17%, and it
includes entries that exist only in docs (e.g. `hist`, `readtable`, `sim`—the last as a GUI
builtin is not registered; `sim` is only in mlapp-sim, not the interpreter) → "Run example"
equivalence (W11) cannot be met by the current catalog.

### 8.11 Add-Ons (`addons_panel.rs`)
Heading, "N Installed Packages" counter, search + "Clear", 2-column list/detail; fake static
catalog (G2), pretend Install/Uninstall, "Launch App" maps to 4 real targets in
`app.rs:1064-1077`. List rows use id_salt for scroll ("addons_list"/"addon_detail") — better
than most, but rows still label-based ids.

### 8.12 Block Diagram Editor (Simulink tab) (`simulink_editor.rs`)
Toolbar: heading (trademark+emoji), "▶ Run Simulation" (**runs `simulate_diagram`
synchronously on the UI thread** `simulink_editor.rs:238-254, 296-299` — G14 violation for
larger models), "📈 Open Scope", Stop Time + dt DragValues, "Feedback Example" reload,
"Clear Canvas" (no confirm), status label (collides with clipped "Clear Canvas" in capture).
Library: 4 collapsing categories with "Add <block>" buttons. Canvas: wire-route uses source
block size for destination offset (`simulink_editor.rs:518` — misroutes to Sum's second
input), block interact ids stable ouf-of-sid; in-port click always `dst_port 0` (`:640-645` —
**no way to wire into Sum's second input port**); no block delete, no wire delete, no
rename, no pan/zoom wire-through (pan_offset/zoom fields stored but canvas ignores zoom),
dark hardcoded colors (light theme broken), **default view shows no blocks (R5)**. Param
modal: window id derived from block name; Save/Cancel. Scope modal: window title has emoji +
trademark; scale forces y_min<=-0.2 / y_max>=1.2 (`:746-747`, hard-coded, not data-honest);
"Final: %.4f" real value shown; gold curve hardcoded color.

### 8.13 Status bar (`statusbar.rs`)
"● Ready"/"⏳ Busy (t)"/"⏸ Paused (Line N)"/"✕ Error: <text>" where <text> is the *formatted*
message (already prefixed "Error: " by `rustlab-cli/src/format.rs:82,90`) → **doubled
prefix** visible as "✕ Error: Error: ..." (G8). Truncation `&err[..37]` slices bytes — will
panic on multi-byte UTF-8 error text >40 chars. Then cwd (folder name only), active file
name, right-aligned theme "🌙 Dark"/"☀️ Light" + "Ln X, Col Y" (dead, R10).

## 9. G8/G9/G10/G11 summary vs target

- G8: one "Error:" prefix required — status bar double-prefix + raw block text in command
  window; error location present as text line ("--> Line X, Column Y") but NOT clickable and
  the editor never receives error line from a plain-error run.
- G9: constants hide-list not implemented; no show-constants toggle; no sort; no context
  menu; no 40-char preview clamp (preview clamps at 6 elements/30 chars — no char budget;
  long struct field lists can be huge: `execution.rs:697`).
- G10: no dotfile/build-dir hiding; no refresh; no breadcrumb; no `.svg` icon; "other" has
  blank icon; no context menu. Icons: emoji, not our own vector icons (G10/G13 both).
- G11: 38/225 coverage; no run-example; count displayed is consistent internally but must
  equal the registry by test later.
- G12: Figures empty state is clipped (R6); "plot with undefined variable" shows the real
  error in the Command Window (engine error format confirmed: `format.rs:44-93`).
- G13: emojis everywhere (12+ files — full grep list in docstring of section 1 source); one
  accent color #007ACC; spacing not from a single scale; dark+light themes exist but
  hardcoded dark color constants in simulink canvas + scope + editor analysis badge colors
  (`editor.rs:311-343`) break light mode; window title (R11); status cap fields present but
  wrong labels/order per spec above.
- G14: engine is a real worker thread (good) BUT (a) plain (non-debug) runs have *no*
  cancellation points: `Session.exec` → `eval_script` path checks `cancel` only once before
  parsing (`execution.rs:489-498, 544-546`) — Stop cannot interrupt `while true, end` in a
  normal F5 run; (b) `RunResult.output` is accumulated and delivered once at the end (`:551-560`)
  — no streaming into the Command Window; (c) GUI builtin `disp` also `println!`s to the
  process stdout (`:238`) — leftover noise; (d) sim runs synchronous on the UI thread
  (8.12); (e) no progress indicator for imports (`import_csv` whole-file read `:736`).
- G15: only dock layout persisted — to `%TEMP%\matrust_dock_layout.json`, corrupt JSON
  silently ignored (no notice), everything else (theme, recent files, last folder, open
  files) restarts from defaults (Preferences struct exists but is dead state: `font_size`,
  panel widths/show_* and `add_recent_file` have no callers outside `preferences.rs` itself).

## 10. Additional repo-hygiene finding (test pollution)

Running the workspace tests **modifies tracked files**: `docs/builtins.txt` loses 20 entries
when tests run with different feature sets (doc-generator test writes the file), and
`fixtures/sample_models/simple_feedback.slx` is re-zipped every run (bytes changed 1445→1447).
Both were restored (`git restore`) after the verify run; the pollution will continue until
tests write only to temp paths — to be fixed in an early small step (not S1 scope; noted for
BACKLOG).

## 11. S1 verdict

Audit is complete against the required artifact list: every panel inventoried (8.1-8.13),
watermarks/placeholder strings listed (2, 3), duplicates listed (4), runtime ID warning
observed and its exact code source identified (R1/5.), overlaps/clips recorded with real
screenshots (1, 6), registry absence proven (4), test pollution found (10). Not done in S1
(and scheduled): interactive row-by-row registry CI work, complete per-menu screenshot
walkthrough for Add-Ons/History/Help menus (click-mapping misses documented in 6.5),
EGUI-warning capture instrumentation (needs a logger; belongs to S3), release-binary strings
check (S8).

## 12. S2 addendum — owner-screenshot reconciliation (2026-11-03)

Added by the owner request dated 2026-11-03 ("check docs/gui-audit.md lists every item
visible in the owner's screenshot; add any missing"). Six items, per-item status with
in-session evidence.

| # | Item | Verdict | Evidence |
| :--- | :--- | :--- | :--- |
| 1 | "Clean-Room Reference (Zero AI)" heading overlay in Help | already covered | section 2 row 1 + R8 overlap row; removal is part of the S2 code step |
| 2 | Current Folder lists `.git` / `.github` / `.re-ghidra` | NEW finding (R13) | `current_folder.rs:50-76`: `fs::read_dir` renders every entry with no dot-entry filter; visible in the 1280x720 before-shot. Hiding rule (hide dot-entries vs "show hidden" toggle) is an S3 design decision; tracked in BACKLOG 6.2 |
| 3 | Workspace shows constants pi, i, j, inf, nan, NaN, true, false, on, off | NOT REPRODUCED in the current build | Workspace rows come only from `interp.env.locals()` (`execution.rs:599-632`); the GUI session seeds no constants; the S1 before-shots show "No variables in workspace". Live probe this session: `docs/functions/true.md` + `false.md` exist (true/false are registered builtins); `pi`, `i`, `j`, `inf`, `nan`, `NaN`, `on`, `off` have neither a doc file nor a registration, so they cannot display as workspace rows (or even evaluate). W2 quick-math risk: typing `pi` errors as an unsupported function. Engine-side work: BACKLOG 6.1. The owner's screenshot most likely predates the current tree, or the rows were created by a script (e.g. assignments named i/j) |
| 4 | Status bar says "No active file" while a file is open | CONFIRMED display-condition bug | `statusbar.rs:72-79` switches on `active_file` being a PATH; an open UNTITLED document has no path yet, so the label prints "No active file" although an editor tab exists (owner shot + 8.13). Fix in S5: tie the label to "an editor tab is open", not to the existence of a saved path |
| 5 | Help list clipped at panel width | CONFIRMED layout constraint | `help_panel.rs:374-399`: fixed desired widths (search `desired_width(available - 80)`) plus an 8-item category filter can overflow at 1280x720; the owner shot records the clip at the list right edge. Rework in the S4 layout pass |
| 6 | Figures empty-state hint text cut off | CONFIRMED | `figures.rs:132-141`: a single centered 14px text "Empty Figure (Run plot(x, y), scatter(x, y), or bar(x, y) to display data)" with no wrapping inside `canvas_rect` overflows small canvases (cut off on both edges in the owner shot) |

Section-11 note: the test-pollution finding (10) was FIXED in commit 9ea305c — every test
artifact writer now targets `%TEMP%` and `scripts/verify.ps1` step 10 fails the run on a
dirty tree. Evidence: `docs/verify-logs/2026-11-03-gui-s2-hygiene-verify.log`, SHA-256
`09287F07129071D3FE34ADBA131D984FB4E4273150F1078B2C76FE7E417F571E` (from-clean, ends
"All verification steps passed successfully!", manifest check passed: all 351 entries present).
