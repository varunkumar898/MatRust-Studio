# MATSTUDIO Layout Audit — L1 (Contracts C1–C12)

- **Phase / Task:** Wave 0 Task L1 — Layout audit (read-only, no code changes).
- **Date:** 2026-11-14
- **Auditor:** Antigravity AI Agent
- **Branch:** `task/l1-layout-audit` (Base: `4950614`)
- **Method:** Real-window desktop execution (`matrust_studio.exe`) captured at the three supported resolutions using Win32 `PrintWindow` via `scripts/capture-desktop-s1.ps1`, coupled with static architectural review across `apps/desktop/src/`.

---

## 1. Capture Evidence (Supported Sizes)

Real-window captures produced in this session with clean dock layout reset (`MATSTUDIO_RESET_LAYOUT=1`):

| Relative Path | Resolution | Size (Bytes) | SHA-256 Checksum | Visual Assessment |
| :--- | :---: | :---: | :--- | :--- |
| `docs/screenshots/l1/desktop_900x600.png` | 900×600 | 103,181 B | `AE97E3D11C508D147CD17B3661D930B50437444BBC95F56022ACADC9F4F6F560` | Minimum supported size. Heavy toolstrip overflow past right edge; panels compressed near minimum usability thresholds; global search box cramped. |
| `docs/screenshots/l1/desktop_1280x800.png` | 1280×800 | 151,397 B | `FE1548E6061A159DAC40C631AE731AE455F6840A5215E7A641B108349091788F` | Standard dock preset size. Balanced three-column split; minor toolstrip overflow in HOME ribbon; all primary editor and canvas controls legible. |
| `docs/screenshots/l1/desktop_1920x1080.png` | 1920×1080 | 207,773 B | `3F7C60D4FA23D14EC00BD3AD225B048D0D37A8A8C28FD841DECBFD1A83CDA836` | Full HD desktop size. Ample breathing room; all toolstrips fully visible without overflow; panels comfortably exceed minimum bounds. |

---

## 2. Sources Consulted & Verified

The following source files in `apps/desktop/src/` were reviewed against contracts C1–C12:
- `apps/desktop/src/dock.rs:11-51` — `PanelId` definitions, display titles, and tab bracket icons `[D]`..`[B]`.
- `apps/desktop/src/dock.rs:400-580` — Split-tree recursive node layout and splitter drag interactions.
- `apps/desktop/src/layout_contract.rs:15-33` — `PANEL_MIN` table declaring minimum width/height for 7 core panels.
- `apps/desktop/src/layout_contract.rs:35-137` — `PanelClip`, `LayoutProbe`, and `check_layout` invariant verification functions.
- `apps/desktop/src/app.rs:562-997` — Top menu bar and toolstrips (HOME ribbon overflow behavior).
- `apps/desktop/src/app.rs:589-607` — Global documentation search input box sizing.
- `apps/desktop/src/app.rs:1000-1062` — Address bar and navigation controls.
- `apps/desktop/src/current_folder.rs:55-84` — File tree view, directory navigation, and path handling.
- `apps/desktop/src/editor.rs:940-970` — Script editor content area sizing and scroll handling.
- `apps/desktop/src/figures.rs:115-149` — Figures panel empty state text and canvas allocation.
- `apps/desktop/src/help_panel.rs:358-428` — Help panel layout, heading row, and category filter ribbon.
- `apps/desktop/src/command_window.rs:20-45` — Command Window layout, greeting banner, and REPL input.
- `apps/desktop/src/statusbar.rs:40-99` — Status bar cwd and metadata display.

---

## 3. Contract Rules C1–C12 — Complete Verdict Table

| Rule | Contract Name | Verdict | Concrete Evidence & Source Location |
| :--- | :--- | :---: | :--- |
| **C1** | **Rect ownership** | **PASS** | `dock.rs`: Binary split tree (`SplitOrientation::Horizontal`/`Vertical`) partitions available body rect cleanly among active tab leaves. Each leaf allocates a discrete rect with no child bleed. |
| **C2** | **Minimum sizes** | **UNENFORCED** | Declared in `layout_contract.rs:15-23` (`PANEL_MIN` for 7 panels), but `dock.rs` split-tree rendering does NOT clamp node widths/heights to `panel_min`, allowing splitters to drag panels into collapse. |
| **C3** | **Fill, don't fix** | **PARTIAL** | Editor and Figures/Block canvas allocate `ui.available_size()` dynamically (`editor.rs:944`, `figures.rs:120`); however, ribbon buttons and search box in `app.rs` use static fixed pixel widths. |
| **C4** | **Scroll everything** | **PASS** | Salted `ScrollArea` IDs (`current_folder_scroll`, `command_window_scroll`, `editor_scroll`, `help_scroll`, `sim_library_scroll`) prevent egui id collisions and allow unbounded content scrolling. |
| **C5** | **Toolbars wrap/overflow** | **VIOLATION** | HOME ribbon toolstrip in `app.rs:627-815` overflows past right window edge on 900×600 and 1280×800 without wrapping, scroll affordance, or overflow dropdown. |
| **C6** | **Text fits** | **VIOLATION** | Global search box in `app.rs:589-607` clips at 900×600. Help panel heading text collides with right-aligned subtitle at narrow widths (`help_panel.rs:363-369`). |
| **C7** | **No overlays** | **PASS** | No modal un-dismissable overlays detected in window captures. Egui warning ribbon overlays previously seen in S1 were eliminated by ScrollArea salting. |
| **C8** | **Single source of state** | **VIOLATION** | CWD representation is fragmented: status bar displays folder *name* (`statusbar.rs:94`), while address bar (`app.rs:1000`) and Current Folder display full path strings. |
| **C9** | **Consistent headers** | **VIOLATION** | Tab headers in `dock.rs:40-48` prepend raw bracketed single-letter tags (`[D]`, `[E]`, `[W]`, `[C]`, `[F]`, `[H]`, `[?]`, `[A]`, `[B]`) rather than clean typography. |
| **C10** | **Empty states** | **VIOLATION** | Figures empty state painter text (`figures.rs:132-148`) does not wrap and clips on both horizontal sides when panel width is restricted. |
| **C11** | **Spacing** | **PARTIAL** | Status bar spacing uses ad-hoc margins (`statusbar.rs:71`); dock splitter handle is 4.0 px without a centralized design token system. |
| **C12** | **Responsive** | **VIOLATION** | Cross-resolution evaluation (900×600 vs 1280×800 vs 1920×1080): at 900×600, severe toolbar clipping occurs; at 1280×800, minor toolstrip overflow persists; only 1920×1080 renders cleanly. |

---

## 4. Key Findings Summary

1. **Toolbar Overflow (C5)**: The HOME ribbon in `apps/desktop/src/app.rs` is laid out in a single horizontal row with fixed item widths. On screens narrower than ~1400px, buttons such as Layout presets, Set Path, and Help become unreachable.
2. **Missing Runtime Minimum Size Clamping (C2)**: While `PANEL_MIN` is defined in `layout_contract.rs`, `dock.rs` split calculations use arbitrary fractional weights without enforcing a hard floor, allowing user resize or window shrinkage to crush panels.
3. **Bracket Prefixes (C9)**: Dock tab titles display bracketed indicators (`[D] Current Folder`, `[E] Editor`, etc.) instead of clean icon or text titles.
4. **State Desynchronization (C8)**: Working directory display lacks single-source derivation, showing different representations in status bar versus address bar.
5. **Empty State Painter Clipping (C10)**: Figures panel empty-state text uses raw painter text without wrap bounds, clipping on narrow panel sizes.

---

## 5. Scope Boundary & Handoff to Task L2

- **Task L1 Status:** COMPLETE. All findings documented via probe-free reading and live window captures. Zero production source code was modified in L1.
- **Next Step (Task L2):** Implement the read-only layout probe infrastructure:
  1. Add runtime `LayoutProbe` wiring and `check_layout` entry point.
  2. Implement the C1 container boundary checks and C2 minimum-size table validation.
  3. Wire the L-C size sweep and L-E mutation checks as automated tests.
  4. Findings only; no panel layout fixes in L2 (fixes belong to L3).
