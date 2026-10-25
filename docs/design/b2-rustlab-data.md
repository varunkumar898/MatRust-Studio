# Phase B2 — `rustlab-data` Design Note (B2.1 scope)

**Status:** approved scope for Milestone B2.1. Tables, XLSX, JSON, and plotting are explicitly out of
scope here and are deferred to the later milestones named at the end.

## 1. Scope of B2.1

B2.1 delivers **two things only**:

1. **Statistics builtins** implemented in Rust in a new `rustlab-data` crate, with no new
   third-party dependencies.
2. **CSV matrix I/O** (`readmatrix`, `writematrix`) using the `csv` crate.

Everything is exposed through the interpreter behind the Cargo feature **`data`** (enabled by
default, disabled by `--no-default-features`). The crate itself is a workspace member; the feature
controls registration of its builtins in `rustlab-stdlib`.

### Explicitly deferred

| Item | Deferred to |
| :--- | :--- |
| `table`, `timetable`, heterogeneous columns, `join`/`innerjoin`/`outerjoin`, `summary`/`head`/`tail` | **B2.3 (later)** |
| XLSX / ODS spreadsheets (`calamine`), `readtable`/`writetable` | **B2.3 (later)** |
| JSON (`jsonencode`/`jsondecode`, `serde_json`) | **B2.3 (later)** |
| `rustlab-plot`: plot description model, deterministic SVG, `plot`/`scatter`/`bar`/`histogram` | **B2.2** |
| Terminal Braille ANSI rendering | later |
| 3D surfaces, interactive GUI event loops | later |

No table, XLSX, JSON, or plotting code is to be added in B2.1.

## 2. Crate layout

```
crates/
└── rustlab-data/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs          # error helper + register_all
    │   ├── stats.rs        # all statistics builtins
    │   └── csv_io.rs       # readmatrix / writematrix
    ├── tests/
    │   ├── stats_tests.rs
    │   ├── csv_tests.rs
    │   └── data/           # CSV fixtures (BOM, CRLF, quotes, ragged, empty fields)
    └── benches/
        └── b2_csv.rs       # criterion: 1,000,000-row read/write
```

Dependencies: `rustlab-math`, `rustlab-eval`, `rustlab-parse` (for `Span`), and `csv = "1.3"`.
`rustlab-data` must not depend on `rustlab-stdlib` (no cycle); `rustlab-stdlib` depends on it
optionally.

Quality attributes: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic,
clippy::todo, clippy::unimplemented)]` in `lib.rs`; all public functions return `EvalError`; no
panics on user input.

## 3. Statistics API

All functions operate on numeric `Value`s, reduce along the first non-singleton dimension by default,
and accept an optional `dim` where MATLAB does. `'omitnan'` is a case-insensitive string flag.

| Builtin | Contract |
| :--- | :--- |
| `median(X [, dim] [, 'omitnan'])` | Middle value; even count averages the two middle values. Default NaN propagates; `'omitnan'` removes NaN before computing. |
| `mode(X [, dim] [, 'omitnan'])` | Most frequent value; **smallest value on ties**. |
| `var(X [, flag] [, dim] [, 'omitnan'])` | Variance; `flag = 0` (default) normalizes by `N-1`, `flag = 1` by `N`. |
| `std(X [, flag] [, dim] [, 'omitnan'])` | Square root of `var` with the same flag/dim rules. |
| `prctile(X, p)` / `quantile(X, p)` | Percentile/quantile with piecewise linear interpolation through `(i - 0.5)/N`, i.e. position `pos = q * N + 0.5`, clamped to ends `[x_1, x_N]`. Optional `'method'` parameter supports `'numpy'` (`h = (N-1)*q + 1`). Citations: MathWorks `prctile` / `quantile`. |
| `cov(X)` / `cov(X, Y)` | Covariance matrix with `N-1` normalization; columns are variables, rows observations. Two-vector form returns the scalar covariance. |
| `corrcoef(X)` | Correlation matrix from `cov`; diagonal is 1; constant columns produce `NaN`. |
| `histcounts(X [, edges])` | Bin counts; supplied edges are used verbatim; without edges a deterministic nice-step binning rule is used (**unverified** vs MATLAB's automatic rule). |
| `movmean(X, k)` | Centered moving mean over `k` samples ($O(N)$ sliding window); endpoints shrink the window. |
| `movmedian(X, k)` | Centered moving median; endpoints shrink the window. |
| `cummax(X [, dim])` / `cummin(X [, dim])` | Cumulative maximum/minimum along `dim`. |
| `zscore(X)` | `(X - mean(X)) / std(X, 0)` along the first non-singleton dimension ($O(N)$ 3-pass fast path). |
| `sortrows(A [, cols])` | Lexicographic row sort; negative column indices mean descending; optional second output is the row permutation. |
| `unique(A)` | Sorted unique values; preserves row vector orientation `1 x N -> 1 x M` for row inputs. With `[C, ia, ic]`, `ia` is the first-occurrence index and `ic` maps original elements to `C`. |
| `'omitnan'` for `sum`, `mean`, `min`, `max`, `median`, `std`, `var` | Data-feature wrappers preserve the existing stdlib default behaviour and only change the NaN path. `min`/`max` retain index outputs. |

Edge cases required by the test plan: empty input, NaN, Inf, N-D with `dim`, integer classes, and
wrong-argument error text. Property tests: percentiles monotone in `p`, `var >= 0`, `corrcoef`
diagonal = 1.

## 4. CSV API

### `readmatrix(filename, Name, Value, ...)`

- Options: `Delimiter` (single character, default comma), `NumHeaderLines` (non-negative integer,
  default 0).
- Empty field -> `NaN`.
- Quoted fields are parsed by the `csv` crate; quoted delimiters/newlines are supported.
- UTF-8 BOM is stripped; CRLF and LF are accepted.
- Ragged rows are an error that names the 1-based **file line number**.
- Numeric parsing accepts `NaN`, `Inf`, `-Inf`; a non-numeric non-empty cell is an error with its
  line/column.
- The result is a dense `double` matrix; a single row is `1 x n`, a single column `n x 1`.

### `writematrix(A, filename, Name, Value, ...)`

- Options: `Delimiter` (default comma).
- Writes the shortest round-trip `f64` representation produced by Rust's standard formatter
  (`format!("{}", x)`); `NaN`/`Inf`/`-Inf` are written as those spellings.
- No header is written.
- The writer is deterministic: same matrix + options -> byte-identical file.

### `csv` crate licence / maintenance

`csv` is dual-licensed **MIT / Apache-2.0** (version `1.4.0`), maintained by BurntSushi, and is the standard streaming
CSV parser in the Rust ecosystem. It is a normal (non-optional) dependency of `rustlab-data` only.
No GPL/copyleft code is introduced. The dependency and its transitive crates are checked with
`cargo deny check licenses` (see `deny.toml`).

## 5. Feature and registration wiring

- Root workspace adds `crates/rustlab-data`.
- `rustlab-stdlib`:
  - optional dependency `rustlab-data`;
  - `[features] default = ["faer", "data"]`, `data = ["dep:rustlab-data"]`;
  - `register_all` calls `rustlab_data::register_all(interp)` when `data` is enabled, after the
    core registrations (data versions intentionally override `unique`/`sum`/`mean`/`min`/`max`/
    `median`/`std`/`var` only when the feature is on).
- `rustlab-cli`: `default = ["faer", "data"]`, `data = ["rustlab-stdlib/data"]`.
- `--no-default-features` builds and runs the core without data builtins; the `rustlab-data`
  integration tests are gated by `#[cfg(feature = "data")]` so the feature matrix is honest.

## 6. Verification plan

1. **Golden tests** (exact values):
   - `median([1 2 3 4]) = 2.5`
   - `std([1 2 3 4]) = 1.2910` (1.2909944487358056, displayed/asserted to tolerance)
   - `mode([1 2 2 3 3]) = 2` (smallest tie)
   - `prctile([1 2 3 4 5], 50) = 3`
   - `movmean(1:5, 3) = [1.5 2 3 4 4.5]`
2. **Edge cases**: empty, NaN, Inf, N-D with `dim`, integer classes, wrong-argument text.
3. **Property tests**: percentile monotonicity, variance non-negative, `corrcoef` diagonal 1.
4. **CSV fixtures** in `tests/data/`: BOM, CRLF, quoted fields, empty field -> NaN, ragged row
   error line, shortest round-trip.
5. **Release benchmark**: 1,000,000-row CSV read + write under criterion; record in
   `docs/bench-history.md`.
6. **Deliverables**: this note, `LEARNING.md` section, `examples/b2/*.m` with captured real output,
   `docs/functions/<name>.md` + `help.rs` entries for every new builtin, updated
   `docs/test-manifest.txt` / `docs/test-count.md`, unedited final verify log, and git tag `b2.1`.

## 7. Error handling

Use the existing interpreter error channel (`EvalError::UserError`) with these new identifiers:

- `MATLAB:data:badInput` — argument is not numeric / not 2-D where required.
- `MATLAB:data:badDim` — dimension argument is not a positive integer scalar.
- `MATLAB:data:badFlag` — normalization flag is not 0 or 1.
- `MATLAB:data:badOption` — unknown name-value option or malformed option value.
- `MATLAB:data:csvRead` — file open/parse/ragged-row error (message includes the line number).
- `MATLAB:data:csvWrite` — file create/write error.

No `unwrap`/`expect`/`panic` in library code; tests may use them.
