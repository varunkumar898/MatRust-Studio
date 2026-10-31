# MatRust Studio Data Capabilities

## 1. Overview

MatRust Studio provides full data analysis and file format capabilities supporting delimited files (CSV, TSV), Level 5 MAT-files, and tabular dataset structures.

## 2. Supported Formats & Operations

### 2.1 File Formats
- **CSV / TSV / Delimited Text**:
  - `readmatrix(filename)`: Fast numerical parsing with RFC 4180 compliance, UTF-8 BOM handling, and missing fields mapped to `NaN`.
  - `writematrix(A, filename)`: High-precision round-trip float export via `ryu`.
  - `readtable(filename)`: Imports column-oriented heterogeneous datasets into structured tables with type inference (`Double` or `String`).
  - `writetable(T, filename)`: Exports structured tables to comma-delimited files.
- **Level 5 MAT-Files (`.mat`)**:
  - `load('file.mat')`: Reads compressed and uncompressed Level 5 MAT-files (`flate2`).
  - `save('file.mat', 'v1', 'v2')`: Writes workspace variables to Level 5 MAT-files.

### 2.2 Table Operations
- **Dot Indexing**: Fields and variables can be accessed using standard dot syntax: `data.temperature`.
- **Row Inspection**:
  - `head(T)` / `head(T, k)`: Returns the top $k$ rows (default 8).
  - `tail(T)` / `tail(T, k)`: Returns the bottom $k$ rows (default 8).
- **Statistical Summary**:
  - `summary(T)`: Computes `Min`, `Mean`, and `Max` statistics for each numeric column.

### 2.3 Statistical Functions (`crates/rustlab-data`)
- **Central Tendency & Dispersion**: `mean`, `median`, `mode`, `var`, `std`, `min`, `max`, `sum`, `prod`.
- **Ranking & Percentiles**: `prctile` (MATLAB-compliant interpolation), `quantile`, `sortrows`, `unique`.
- **Association**: `cov` (covariance matrix), `corrcoef` (correlation coefficients).
- **Distributions**: `histcounts` (histogram binning).
- **Windowed & Cumulative**: `movmean`, `movmedian`, `cummax`, `cummin`, `zscore`.
