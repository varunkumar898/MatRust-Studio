# MatRust Studio

A Rust-based MATLAB-like numerical computing environment with a CLI and native desktop application.

MatRust Studio provides a small, independent implementation of common MATLAB language constructs, dense matrix linear algebra, basic statistics, plotting infrastructure, CSV data I/O, and an interactive session. It is intended as a research and development platform—not a complete replacement for commercial MATLAB or any specific release version.

## Status

This is an evolving scientific-computing project. Several core subsystems work independently and pass their targeted regression tests, but the environment does not claim full MATLAB compatibility. Key verified capabilities include:

- Real-valued dense matrix arithmetic, indexing, assignment, and array growth.
- Linear algebra (LU, SVD, pseudoinverse, basic eigenvalue computation for real symmetric matrices; non-symmetric eigenvalues only—eigenvectors remain incomplete).
- Complex arithmetic and basic complex linear algebra (documented limitations exist for non-symmetric full eigenpairs).
- User-defined functions with scope isolation, recursion depth tracking, and multi-output assignment.
- Basic statistics (`mean`, `std`, `cov`, `percentile`, `quantile`).
- 2D line/scatter/bar plots with model-level verification (`GraphicsState`, `FigureState`, `egui_plot` renderer); actual interactive desktop rendering remains unverified in non-interactive environments.
- CSV import/export (`csvread`/`csvwrite`) and basic session persistence (`load`/`save` using structured CSV format—not MATLAB `.mat` files).
- CLI binary (`matlabr`) and desktop binary (`matrust-studio`).

Documented limitations and unresolved work are recorded explicitly in `docs/progress.md`, `docs/compatibility-matrix.md`, `docs/function-coverage.md`, and `docs/rea-compatibility-audit.md`.

Native MATLAB reverse engineering remains blocked: `/home/cherry/Downloads/matlab_R2026b_Linux` is an installer archive, not a numerical engine binary; `/usr/local/MATLAB` and `/opt/MATLAB` do not contain installed engines. No REA binary-analysis evidence for MATLAB internals exists in this repository.

## Building

Requirements:

- Rust toolchain (`rustc` 1.98.1 or compatible; `rustfmt`, `clippy` components recommended).
- Linux with Wayland or X11 display support (for the desktop application).

Commands verified on this repository:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo build -p matrust_studio
```

Note: not all workspace integration tests are discovered by `cargo test --workspace` alone; targeted tests should be run individually by their file names (e.g., `cargo test -p language_tests --test numeric_batch14`).

## Running

CLI binary (`matrust-studio` package is desktop; CLI is typically `matlabr`):

```bash
cargo run -p matlabr -- -e "A=[1 2;3 4]; disp(A);"
```

Desktop application:

```bash
cargo run -p matrust_studio
```

A basic example script (run from CLI or editor):

```matlab
A = [1 2; 3 4];
B = [5 6; 7 8];
C = A * B;
disp(C);
```

## Project layout

```
crates/
  syntax/          Lexer, parser, AST (language syntax and expressions)
  values/          Value representation (Matrix, String, Boolean, complex)
  linalg/          Dense linear algebra kernels (LU, QR, SVD, eigen, solve)
  diagnostics/     Error messages with spans
  runtime/         Interpreter (`interp.rs`), session environment (`env`),
                   built-in functions (`builtins.rs`), graphics state (`graphics.rs`)
apps/
  cli/             CLI binary (`matlabr`)
  desktop/         Desktop GUI (`app.rs`, `figures.rs`, `execution.rs`)
docs/             Audit, progress, architecture, compatibility matrix, coverage
tests/            Integration tests (language batches, quick stage tests)
```

## Compatibility and limitations

Full compatibility status is maintained in `docs/compatibility-matrix.md` and `docs/function-coverage.md`. Important limitations include:

- Sparse arrays, cell arrays, tables, N-D arrays, structures, classes/packages, and symbolic mathematics: not implemented.
- Full non-symmetric eigendecomposition (eigenvectors): not implemented; only eigenvalues via `improved_qr_eig`. The known ignored regression test (`numeric_batch_stage2_eig`) remains preserved.
- 3D/surface/contour graphics, image plots, figure export, animations, interactive plot updates: not implemented or unverified.
- Advanced statistical toolbox parity: partial (descriptive statistics and basic quantiles verified; regression, distributions, hypothesis tests: unsupported).
- Session persistence is basic (`save`/`load` via structured CSV); it does not replicate full MATLAB `.mat` file formats or complete workspace graphics restoration.
- Native MATLAB execution verification: unavailable (blocked).
- Live desktop rendering: not tested interactively (architecture verified, automated model-level tests pass).

## Development

Run focused tests:

```bash
cargo test -p language_tests --test numeric_batch14
cargo test -p language_tests --test plot_stage4_quick
cargo test -p language_tests --test desktop_stage9_quick
cargo test -p language_tests --test perf_stage10_quick
```

Formatting and quality:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
```

When adding regression tests, prefer descriptive names that state the behavior being checked (`benchmark_matrix_mult_10x10`, `compat_stage6_plot_state_propagation`), include boundary conditions where relevant, and document any unsupported behavior explicitly rather than hiding failures.

## Further reading

- `docs/rea-compatibility-audit.md` — Evidence-based audit, milestone roadmap (M1–M8), REA status, compatibility matrix, defect register, and release criteria.
- `docs/progress.md` — Consolidated milestone history (stages 1–8, M2–M7, M9 desktop verification, M4 benchmark results).
- `docs/architecture.md` — Workspace design, interpreter approach, value representation, graphics routing, and quality gates.
- `docs/compatibility-matrix.md` — Feature-by-feature compatibility status versus intended MATLAB-like behavior.
- `docs/function-coverage.md` — Detailed function coverage list with test references.
- `docs/desktop-ui.md` — Desktop framework description, event paths (`execution.rs`, `figures.rs`), session persistence design, and verified/unverified behaviors.

## License

See workspace manifest (`Cargo.toml`) for declared license information.
