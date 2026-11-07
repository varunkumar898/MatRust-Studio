# MATSTUDIO Layout Audit — L1 (contract C1–C12)

- **Phase:** LAYOUT master prompt v1 — Phase L1 (AUDIT, no code).
- **Repo / branch:** `<repo>\Desktop\MATSTUDIO`, branch `gui/polish-1`.
- **Date:** 2026-11-04.
- **Method:** L1 = probe-free reading + real-window captures already produced this session
  (`scripts/capture-desktop-s1.ps1` vs `target\debug\matrust_studio.exe`;
  `docs/screenshots/before/*.png`, `docs/screenshots/after/*.png`, `docs/screenshots/gui-editor/`,
  `docs/screenshots/gui-clean/`, `docs/screenshots/s3-ids/`, `docs/screenshots/ba7137f/`,
  `docs/screenshots/c4a0ff9/`, `docs/assets/gui_window_live.png`).

## 1. Sources consulted this session (file:line verified by reading)

- `apps/desktop/src/dock.rs:11-48` — `PanelId` + display labels + bracket prefixes `[D]…[B]`.
- `apps/desktop/src/app.rs` — grep-verified regions: menu `:393-557`, toolstrips `:562-997`,
  address bar `:1000-1062`, global search `:589-607`, Set Path `:1226-1242`, About `:1260-1267`.
- `apps/desktop/src/current_folder.rs:55-84` — struct fields (`view_tree`, `expanded`, `totals`
  at `:62-64`), `set_directory` `:71-77`, `open_project` `:80-84`.
- `apps/desktop/src/figures.rs:7-20, :23-111, :115-149` — panel struct, tabs/controls, canvas +
  empty state.
- `apps/desktop/src/help_panel.rs:329-343, :358-428` — panel struct, heading `:363`,
  search `:368-378`, category filter `:383-400`, filter `:408-417`, list label `:422-426`.
- `apps/desktop/src/statusbar.rs:40-99` — cwd source `:46`, dir label `:94-95`, file label `:98`.
- `apps/desktop/src/preferences.rs:14-60` — presets + panel visibility.
- `apps/desktop/src/command_window.rs:20-44` — struct + welcome hint `:28-42`.
- `docs/gui-audit.md` — prior-session findings R1-R13 (grep-read this session).

## 2. Contract rules C1–C12 — current verdicts

| Rule | Verdict | Evidence (this session) |
|---|---|---|
| C1 Rect ownership | HOLD (partial) | Dock splitter/node logic `dock.rs`; not fully re-probed this session. |
| C2 Minimum sizes | **MISSING** | No `min_width`/`min_height` const table found in `apps/desktop/src` this session; suggested floors not encoded. |
| C3 Fill, don't fix | HOLD | Figures/Block canvas use `available_size`; Editor `show_content` `editor.rs:944`. |
| C4 Scroll everything | HOLD (S3 fixed ids) | Salted ScrollAreas; B61C fix landed `af930a2`. |
| C5 Toolbars wrap/overflow | **VIOLATION (carry-forward)** | HOME toolstrip overflow — prior-session R7, source `app.rs:627-815` (grep). |
| C6 Text fits | **VIOLATION (carry-forward)** | Global search clip — R9 `app.rs:589-607`; Help heading overlap — R8 `help_panel.rs:363-369`. |
| C7 No overlays | HOLD (unprobed this session) | — |
| C8 Single source of state | HOLD | Status bar cwd mirrors current dir; display mismatch R12. |
| C9 Consistent headers | **VIOLATION** | Bracket prefixes live: `dock.rs:40-48`, `current_folder.rs:645-655,745-747`. |
| C10 Empty states | **VIOLATION** | Figures empty text clipped — `figures.rs:132-148`; CmdW hint `command_window.rs:28-42`. |
| C11 Spacing | PARTIAL | Status bar item spacing `statusbar.rs:71`. |
| C12 Responsive | PENDING | L-C sweep not run in L1. |

## 3. Required minimum-set findings (L1)

1. **Block Diagram toolbar overlapping Help** — **UNVERIFIED in L1** (not re-captured this session;
   prior-session R5 area, source `block_diagram_editor.rs` toolbar render).
2. **Canvas height** — Block Diagram canvas fills `available_size`
   (`block_diagram_editor.rs` render_canvas allocate) — minimum height not encoded (C2).
3. **Block Library clipping** — library ScrollArea salted `sim_library_scroll`
   (`block_diagram_editor.rs`) — clipping at narrow widths unverified in L1.
4. **Help detail clipping** — **CONFIRMED.** `help_panel.rs:372` `desired_width(available - 80.0)`
   + 8-item category filter → overflow at 1280x720 (prior-session S2 addendum, owner shot).
5. **Duplicated panel titles** — **NEEDS RECHECK** (heading + panel name duplication suspected;
   not re-probed this session).
6. **Bracket prefixes** — **CONFIRMED.** `dock.rs:40-48`, `current_folder.rs:645-655,745-747`.
7. **Functions (38) vs registry count** — **NEEDS RECHECK** (S1 recorded 38/225; not re-measured in L1).
8. **cwd mismatch** — **CONFIRMED (S1 R12).** Status bar shows folder *name*; Current Folder and
   address bar show full path — three different strings (`statusbar.rs:46,94-99`).
9. **Figures empty text** — **CONFIRMED (S1 R6).** Clipped on both sides — `figures.rs:132-148`.

## 4. Working-directory / OneDrive finding

- Working dir source: `std::env::current_dir()` — `statusbar.rs:46`, `app.rs` cwd wiring.
- No persisted workspace dir in `Preferences` (`preferences.rs:14-60`).
- **OneDrive risk:** default cwd comes from the process; if it lands inside a synced folder,
  OneDrive sync can corrupt build output and muddy current-project semantics. Default cwd must not
  be inside a synced folder.

## 5. L1 verdict / L2 to-do

- Confirmed this session: C2 missing, C9 bracket prefixes, C10 figures/cmdW empty states, C8,
  C11 partial, plus carry-forwards from `docs/gui-audit.md` R1–R13.
- L2 must add: LayoutProbe, `check_layout`, min-size table, C1 container, L-C sweep, L-E mutation
  check. No panel fixes in L2.

---

*L1 produced no code. Report per AGENTS.md section 6, then STOP for owner approval before L2.*
