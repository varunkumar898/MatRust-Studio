# PLAN-F1: Professional Block Editor Architecture & Work Plan (docs/milestones/plan-f1-block-editor.md)

This planning document outlines the technical design, subtask decomposition, touch list, and acceptance testing suite for **Phase F1: Professional Block Editor**.

---

## 1. Phase Objective & Exit Criteria

- **Objective**: Elevate the MatRust Studio block diagram canvas into an industry-grade, responsive graphical modeling environment featuring magnetic grid snapping, orthogonal Manhattan wire routing, multi-block selection, clipboard ergonomics (copy/paste/duplicate), alignment/distribution tools, zoom-to-fit, minimap overview, wire annotations, signal-path tracing, searchable palette, and diagram auto-layout.
- **Entry Criteria**: Baseline editor canvas (M12) with multi-level undo/redo, cancellation, and basic port connectivity.
- **Exit Criteria**:
  1. Every editor capability functions with both mouse and keyboard interactions.
  2. Pure geometric and topological logic (snapping, orthogonal wire routing, alignment, multi-selection, clipboard) has headless automated unit tests.
  3. Interactive user experience checklist is verified.

---

## 2. Work Breakdown & Subtask Grouping

Per Rule 1 (P13), Phase F1 comprises 14 discrete items organized into 5 cohesive work packages:

### Package F1-A: Magnetic Snapping & Block Alignment
- **`F1.01` Snap-to-Grid [M]**:
  - Configurable grid pitch ($16.0\text{ px}$ default, $8.0\text{ px}$ fine).
  - Magnetic snapping during block drag, wire terminal anchoring, and port placement.
  - Helper functions: `snap_point_to_grid(pos: Pos2, pitch: f32) -> Pos2`.
- **`F1.05` Block Alignment and Distribution [S]**:
  - Multi-block alignment commands: Align Left, Align Right, Align Top, Align Bottom, Align Horizontal Centers, Align Vertical Centers.
  - Multi-block distribution commands: Distribute Horizontally, Distribute Vertically (equalizing gaps between bounding boxes).

### Package F1-B: Multi-Selection & Clipboard Ergonomics
- **`F1.03` Multi-Block Selection [M]**:
  - Rubber-band marquee drag selection box (`selection_box: Option<Rect>`).
  - Shift-click / Ctrl-click toggle selection of blocks into `selected_block_sids: HashSet<String>`.
- **`F1.04` Copy, Paste and Duplicate Blocks [M]**:
  - In-memory clipboard struct `EditorClipboard` storing selected blocks, their internal interconnecting wires, and relative offsets.
  - Duplicate (`Ctrl+D`), Copy (`Ctrl+C`), and Paste (`Ctrl+V` with offset $+20\text{ px}, +20\text{ px}$).
  - Unit test `test_block_diagram_duplicate` already verifies diagram duplication; extended with clipboard copy/paste.
- **`F1.12` Keyboard Shortcuts [S]**:
  - Accelerators: `Delete`/`Backspace` (delete selected), `Ctrl+A` (select all), `Ctrl+Z` (undo), `Ctrl+Y` (redo), `Ctrl+C` (copy), `Ctrl+V` (paste), `Ctrl+D` (duplicate), `R` (rotate block 90°), `Space` (quick block add search popup).

### Package F1-C: Orthogonal Wire Routing, Net Labels & Signal Path Highlighting
- **`F1.02` Automatic Wire Routing [L]**:
  - Orthogonal Manhattan A* / Dijkstra pathfinder computing orthogonal waypoint segments `Vec<Pos2>` that avoid block bounding boxes.
  - Fallback to clean 3-segment Z-step routing when straight-line is obstructed.
- **`F1.07` Wire Labels & Annotations [S]**:
  - `wire.label: Option<String>` editable on double-click.
  - Diagram freeform text annotations `DiagramAnnotation { pos: Pos2, text: String, font_size: f32 }`.
- **`F1.08` Signal-Path Highlighting [M]**:
  - Highlighting active upstream signal dependencies and downstream signal propagation paths on mouse hover or block selection.

### Package F1-D: Navigation, Minimap & Library Search
- **`F1.06` Zoom-to-Fit and Minimap [M]**:
  - `zoom_to_fit()` computing bounding box of all blocks and wires, centering viewport with $40\text{ px}$ margin.
  - Minimap floating overlay widget in bottom-right corner displaying miniature canvas view and draggable viewport rectangle.
- **`F1.11` Searchable Block Library [M]**:
  - Palette text filter with fuzzy substring search and keyboard arrow navigation.

### Package F1-E: Visual Metadata, Hierarchy & Auto-Layout
- **`F1.10` Unit and Data-Type Visualization [M]**:
  - Port badge overlays displaying scalar vs vector signal width (`[1]`, `[3]`, `[NxM]`) and physical domain colors (signal: blue, electrical: maroon).
- **`F1.13` Model Hierarchy Navigation [L]**:
  - Subsystem navigation path stack `hierarchy_path: Vec<String>` with interactive breadcrumb bar at the top of the canvas.
- **`F1.14` Diagram Auto-Layout [L]**:
  - Topological layered auto-layout (Sugiyama-style rank assignment) ordering blocks from sources on the left to sinks on the right.

---

## 3. Touch List & Affected Modules

- Primary file: `apps/desktop/src/block_diagram_editor.rs`
- Supporting types in `crates/mlapp-sim/src/model.rs`:
  - `CanvasWire`: Add optional waypoints `pub waypoints: Vec<[f32; 2]>`, `pub label: Option<String>`.
  - `BlockDiagram`: Add annotations `pub annotations: Vec<DiagramAnnotation>`.
- Zero new external dependencies. All geometric and UI calculations use existing `egui` and standard library data structures.

---

## 4. Acceptance Criteria & Automated Tests

1. `test_f1_snap_to_grid_alignment`: Verifies points snap exactly to grid multiples.
2. `test_f1_block_alignment_operations`: Verifies align left/right/top/bottom and center algorithms calculate correct coordinates.
3. `test_f1_block_distribution_operations`: Verifies equal spacing distribution across 3 or more blocks.
4. `test_f1_multi_selection_box`: Verifies marquee rectangle intersects and selects enclosed blocks.
5. `test_f1_clipboard_copy_paste_offsets`: Verifies copied blocks and internal wires paste with fresh unique IDs and offset coordinates.
6. `test_f1_orthogonal_wire_routing`: Verifies routing algorithm generates non-colliding Manhattan segments.
7. `test_f1_zoom_to_fit_bounds`: Verifies bounding box calculation and viewport centering.
8. `test_f1_diagram_auto_layout`: Verifies topological DAG layering places source blocks strictly to the left of sink blocks.

---

## 5. Storage & Performance Budget Impact

- **Executable Size**: Negligible ($< 15\text{ KB}$ increase in compiled code, well within the $35.0\text{ MB}$ budget).
- **Runtime Performance**: Snapping and routing operate in sub-millisecond time ($< 0.1\text{ ms}$ for 100 blocks), maintaining 60 FPS egui rendering.
