# Phase B: Capability Area B2 — `rustlab-data` Design Note

## 1. Overview & Clean-Room Boundaries

`rustlab-data` is a dedicated crate within the RustLab workspace providing high-performance tabular data structures, multi-format scientific file I/O (CSV, XLSX, JSON), and an extensible MATLAB-compatible plotting/visualization engine.

In adherence to clean-room development principles:
- **No GPL / Copyleft code**: Clean-room implementation referencing only public MATLAB documentation, W3C SVG specifications, OpenXML (ECMA-376) standards, and SciPy/pandas public behavior.
- **Zero-unwrap rule**: All functions return structured `Result<T, DataError>` with `thiserror`.
- **Clippy & fmt clean**: Strict `-D warnings` and `cargo fmt`.

---

## 2. Architecture & Components

```
crates/rustlab-data/
├── Cargo.toml
└── src/
    ├── lib.rs              # Crate root and builtin registrations
    ├── error.rs            # DataError enum via thiserror
    ├── table/              # Tabular Data Engine
    │   ├── mod.rs          # Table struct definition & constructors
    │   ├── column.rs       # Typed column vectors (Double, Int, String, Categorical)
    │   ├── indexing.rs     # Row/column subscripting (T(:, {'A','B'}), T(1:5, :))
    │   ├── summary.rs      # summary(T), head(T), tail(T)
    │   └── join.rs         # innerjoin, outerjoin, join
    ├── io/                 # Scientific File I/O
    │   ├── mod.rs
    │   ├── csv.rs          # readtable, writetable, readmatrix, writematrix
    │   ├── spreadsheet.rs  # readtable('file.xlsx', 'Sheet', 1, 'Range', 'A1:C10')
    │   └── json.rs         # jsonencode, jsondecode
    └── plot/               # Graphics & Visualization Engine
        ├── mod.rs          # Figure, Axes, Line, Scatter, Bar scene graph
        ├── scene.rs        # Retained mode scene hierarchy (Root -> Figure -> Axes -> Plots)
        ├── properties.rs   # Name-Value property parsing ('LineWidth', 'Color', 'Marker')
        ├── backends/
        │   ├── mod.rs      # RenderBackend trait
        │   ├── svg.rs      # Pure Rust SVG vector backend (default, headless)
        │   ├── raster.rs   # PNG rasterizer via tiny-skia / resvg
        │   └── terminal.rs # Braille / Unicode ANSI terminal plotting for CLI
        └── builtins.rs     # plot, scatter, bar, histogram, title, xlabel, legend, etc.
```

---

## 3. Tabular Data Specification

### Data Structures
1. **`Table`**:
   - `variables: Vec<ColumnData>` (heterogeneous columns: `f64`, `f32`, `i64`, `String`, `Categorical`, `Logical`).
   - `variable_names: Vec<String>` (validated identifier strings).
   - `row_names: Option<Vec<String>>`.
   - `num_rows: usize`, `num_cols: usize`.
2. **Operations**:
   - **Table Subscripting**:
     - `T.VarName` -> extracts column as typed array.
     - `T(:, {'A', 'C'})` -> subtable extraction by column name list.
     - `T(1:10, :)` -> row slicing.
     - `T.VarName = new_data` -> column assignment/replacement or creation.
   - **Relational Filtering**:
     - `T(T.Age > 30, :)` -> boolean mask indexing.
   - **Join Operations**:
     - `innerjoin(T1, T2, 'Keys', 'ID')`, `outerjoin`.

---

## 4. File I/O Engine

1. **Delimited Files (`csv.rs`)**:
   - High-throughput CSV parser using `csv` crate (fast streaming, SIMD-accelerated where supported).
   - Automatic type inference:
     - Pure numeric column $\to$ `f64` / `int64`.
     - Mixed strings / words $\to$ `string` array.
     - Date/time formats $\to$ `datetime` / string.
   - Support for MATLAB options: `'Delimiter'`, `'HeaderLines'`, `'VariableNames'`, `'TreatAsMissing'`.
2. **Spreadsheet Files (`spreadsheet.rs`)**:
   - Native OpenXML reader using pure Rust `calamine` (supports `.xlsx`, `.xls`, `.ods`).
   - Options: `'Sheet'`, `'Range'`, `'ReadVariableNames'`.
   - Zero external COM/ActiveX or LibreOffice dependency (runs 100% cross-platform on Windows, Linux, macOS).
3. **JSON Serialization (`json.rs`)**:
   - `jsonencode(val)`: serializes structs, cells, tables, and numeric arrays to JSON strings via `serde_json`.
   - `jsondecode(str)`: deserializes JSON into MATLAB structs, cells, and numeric matrices.

---

## 5. Graphics & Plotting Layer

### Scene Graph Architecture
MATLAB graphics follow a strict retained-mode object hierarchy (Handle Graphics):
1. **`Figure`**: Top-level canvas container (`'Color'`, `'Position'`, `'Visible'`).
2. **`Axes`**: Coordinate space with ticks, limits, scale (`'XLim'`, `'YLim'`, `'XScale'`, `'Box'`, `'GridLineStyle'`).
3. **`Primitive`**:
   - `Line`: continuous 2D/3D polyline (`'Color'`, `'LineWidth'`, `'LineStyle'`, `'Marker'`).
   - `Scatter`: point cloud with variable size and color mapping.
   - `Bar`: discrete vertical/horizontal bar chart.
   - `Histogram`: automatic binning with normalization modes (`'count'`, `'probability'`, `'pdf'`).
   - `Surface`: 3D quadrilateral mesh (`surf`, `mesh`).

### Rendering Backends
- **Headless Vector (Default)**: Emits clean, scalable SVG (`.svg`). Ideal for CI, documentation, and web Generative UI.
- **Raster Backend**: Renders SVG to pixel buffers (`.png`) using `resvg` / `tiny-skia` (pure Rust, zero C/X11 dependencies).

---

## 6. Verification and Integration Plan

1. **Golden Tests**:
   - CSV roundtrip with missing values, commas in quotes, headers.
   - XLSX workbook reading across sheets and cell ranges.
   - Table manipulation (`innerjoin`, column assignment, mask filtering).
2. **Visual Golden Regression**:
   - SVG output snapshots checked for line coordinates, axis ticks, and bounding boxes.
3. **Clippy and Quality Bar**:
   - 0 unwraps in library code, 100% `clippy -D warnings` clean, full rustdoc.

---

## 7. Scope Boundaries and Deferred ("Later") Items

To prevent scope creep and maintain strict delivery velocity, the following features are explicitly categorized as **Deferred ("Later") Items** (post-B2 milestone):
- **`timetable` & Time-Series Engine**: Sub-second timestamps, regular/irregular time spacing, retime, synchronize. (Deferred to Phase B time-series milestone).
- **Terminal Braille ANSI Backend**: Rendering Unicode Braille characters `U+2800..U+28FF` in the CLI. The primary and default headless visualization engine for B2 will be the W3C SVG vector renderer.
- **3D Surface Meshes (`surf`, `mesh`)**: 3D lighting, camera view matrices, and polygon z-buffering. (Deferred to B2.2 3D visualization extension).
- **COM / ActiveX / LibreOffice Interop**: All spreadsheet operations are strictly pure-Rust OpenXML (`calamine`), avoiding proprietary platform bindings.

