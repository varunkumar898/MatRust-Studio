# Phase B: Capability Area B2 — `rustlab-data` & `rustlab-plot` Design Note

## 1. Overview & Clean-Room Boundaries

To maintain modularity, fast compilation, and distinct separation of concerns, Phase B2 is partitioned into two specialized crates:
- **`rustlab-data` (Milestone B2.1)**: High-performance tabular data structures (`table`), heterogeneous typed columns, relational operations (`join`, `innerjoin`), and multi-format scientific file I/O (CSV, XLSX via `calamine`, JSON via `serde_json`).
- **`rustlab-plot` (Milestone B2.2)**: Headless, deterministic visualization engine implementing MATLAB Handle Graphics concepts (Figure, Axes, Line, Scatter, Bar, Histogram) with a plot-description model and deterministic W3C SVG vector renderer.

In adherence to clean-room development principles:
- **No GPL / Copyleft code**: Clean-room implementation referencing only public MATLAB documentation, W3C SVG specifications, OpenXML (ECMA-376) standards, and public format behaviors.
- **Zero-unwrap rule**: All functions return structured error types (`DataError`, `PlotError`) with `thiserror`. `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented)]` enforced across all library code.
- **Strict linting**: 100% clean under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

---

## 2. Workspace Architecture

```
crates/
├── rustlab-data/                 # Milestone B2.1: Tabular data & Scientific File I/O
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Crate root and builtin registrations
│       ├── error.rs              # DataError enum via thiserror
│       ├── table/                # Tabular Data Engine
│       │   ├── mod.rs            # Table struct definition & constructors
│       │   ├── column.rs         # Typed column vectors (Double, Int, String, Categorical, Logical)
│       │   ├── indexing.rs       # Row/column subscripting (T(:, {'A','B'}), T(1:5, :))
│       │   ├── summary.rs        # summary(T), head(T), tail(T)
│       │   └── join.rs           # innerjoin, outerjoin, join
│       └── io/                   # Scientific File I/O
│           ├── mod.rs
│           ├── csv.rs            # readtable, writetable, readmatrix, writematrix (via csv crate)
│           ├── spreadsheet.rs    # readtable('file.xlsx', 'Sheet', 1, 'Range', 'A1:C10') via calamine
│           └── json.rs           # jsonencode, jsondecode via serde_json
│
└── rustlab-plot/                 # Milestone B2.2: Retained-mode Plot Description & SVG
    ├── Cargo.toml
    └── src/
        ├── lib.rs                # Plot crate root & registrations
        ├── error.rs              # PlotError enum via thiserror
        ├── scene/                # Retained-Mode Plot-Description Model
        │   ├── figure.rs         # Figure container ('Color', 'Position', 'Visible')
        │   ├── axes.rs           # Coordinate box, limits, tick locators, grid lines
        │   └── series.rs         # Plot series descriptions: Line, Scatter, Bar, Histogram
        ├── properties.rs         # MATLAB Name-Value pair parser ('LineWidth', 'Color', 'Marker')
        ├── render/               # Deterministic Rendering Engine
        │   ├── svg.rs            # Deterministic, byte-stable W3C SVG vector renderer
        │   └── color.rs          # MATLAB standard color order and RGBA conversions
        └── builtins.rs           # plot, scatter, bar, histogram, xlabel, ylabel, title, grid, legend
```

---

## 3. Tabular Data Specification (`rustlab-data` — B2.1)

### Data Structures
1. **`Table`**:
   - `variables: Vec<ColumnData>` (heterogeneous columns: `f64`, `f32`, `i64`, `i32`, `String`, `Categorical`, `Logical`).
   - `variable_names: Vec<String>` (validated identifier strings, non-empty, unique).
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

## 4. File I/O Engine (`rustlab-data` — B2.1)

1. **Delimited Files (`csv.rs`)**:
   - High-throughput streaming CSV parser using the standard Rust `csv` crate (DFA-based fast lexer).
   - Automatic type inference:
     - Pure numeric column $\to$ `f64` / `int64`.
     - Mixed strings / words $\to$ `string` array.
   - Support for MATLAB options: `'Delimiter'`, `'HeaderLines'`, `'VariableNames'`, `'TreatAsMissing'`.
2. **Spreadsheet Files (`spreadsheet.rs`)**:
   - Pure-Rust OpenXML reader using `calamine` (supports `.xlsx`, `.xls`, `.ods`).
   - Options: `'Sheet'`, `'Range'`, `'ReadVariableNames'`.
   - Zero COM/ActiveX or external runtime dependency; 100% cross-platform (Windows, Linux, macOS).
3. **JSON Serialization (`json.rs`)**:
   - `jsonencode(val)`: serializes structs, cells, tables, and numeric arrays to JSON strings via `serde_json`.
   - `jsondecode(str)`: deserializes JSON into MATLAB structs, cells, and numeric matrices.

---

## 5. Graphics & Plotting Specification (`rustlab-plot` — B2.2)

### Retained-Mode Plot-Description Model
The plotting engine separates plot description from rasterization or rendering:
1. **`Figure`**: Canvas descriptor holding figure metadata, sizing, and child axes.
2. **`Axes`**: Coordinate space description with bounding box, affine coordinate transformations ($x, y \to x_{px}, y_{px}$), linear/log scales, auto-ticking algorithms (Wilkinson/Talbot nice numbers), and axis labels.
3. **`SeriesDescriptor`**: Concrete graphical primitives (Line, Scatter, Bar, Histogram) with styling properties (LineWidth, Color, Marker, LineStyle).

### Deterministic SVG Renderer
- **Pure-Rust Vector Output**: Directly emits valid, standalone W3C SVG XML strings.
- **Deterministic & Byte-Stable**: Floating-point coordinates formatted with consistent precision (`{:.2}`) and sorted attribute orders to guarantee identical output across runs for golden regression testing.
- **Zero X11 / Wayland / GPU Dependencies**: Completely headless; generates visual assets suitable for headless CI, web dashboards, documentation, and chat Generative UI.

---

## 6. Verification Plan

1. **`rustlab-data` Goldens**:
   - CSV roundtrips with quotes, commas, escapes, missing values, headers.
   - XLSX workbook reading across multiple sheets and cell ranges.
   - Table relational operations (`innerjoin`, column mutation, logical filtering).
2. **`rustlab-plot` Deterministic Goldens**:
   - Exact byte-matching SVG golden comparisons for 2D line plots, scatter plots, bar charts, and histograms.
   - Verification of coordinate transforms, axis tick locations, and legend rendering.
3. **Clippy and Quality Bar**:
   - Zero unwraps in library code, 100% `clippy -D warnings` clean across all targets, full rustdoc documentation.

---

## 7. Deferred ("Later") Items

- **`timetable` & Time-Series Engine**: Sub-second timestamps, regular/irregular time spacing, retime, synchronize. (Deferred to later time-series milestone).
- **Terminal Braille ANSI Backend**: Rendering Unicode Braille characters in CLI. The primary visualization output for B2 is deterministic SVG.
- **3D Surface Meshes (`surf`, `mesh`)**: 3D lighting, camera view matrices, and z-buffering. (Deferred to later 3D extension).
- **Interactive GUI Event Loops**: Headless SVG rendering satisfies all automated scripting and dashboard requirements; interactive windowing is deferred.
