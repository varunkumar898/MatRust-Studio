# Documentation Index

## Getting started

- `README.md` (repository root) — Project introduction, status, build/run commands, architecture summary, limitations, and quick examples.
- `docs/architecture.md` — Workspace design, interpreter approach, value representation (`DenseMatrix`), graphics routing, and quality gates.
- `docs/desktop-ui.md` — Desktop framework (`execution.rs`, `figures.rs`, `app.rs`), session persistence design, verified/unverified behaviors, manual interaction notes.

## Language and runtime

- `docs/rea-compatibility-audit.md` — Sections A–L: audit of the full repository, verified compatibility matrix (`compatibility-matrix.md`), REA evidence register (target blocked), defect register, missing/incomplete feature inventory, dependency-aware roadmap (M1–M8), individual milestone prompts (M2 blocked, M3/M4 complete, M5/M6/M7 complete), native reverse-engineering limitation (`BLOCKED`), stable-release criteria (`PASS` for verified gates; `NOT TESTED LIVE` for desktop visual; `BLOCKED` for M2 independent validation), and release scope definition.
- `docs/progress.md` — Milestone history (stages 1–8, M2 `BLOCKED`, M3 complete, M4 benchmark measurements, M5 session persistence, M6 compatibility tests, M7 release readiness), performance baselines, known limitations preserved, and documentation updates.
- `docs/compatibility-matrix.md` — Feature-by-feature compatibility matrix (`MATCHED AND VERIFIED`, `MOSTLY MATCHED`, `PARTIALLY IMPLEMENTED`, `NOT IMPLEMENTED`, `BLOCKED`, `NOT TESTED LIVE`).
- `docs/function-coverage.md` — Detailed function coverage list with test references and documented partial implementations.

## Numerical algorithms and linear algebra

- `docs/rea-compatibility-audit.md` — Sections 4.3 (mathematical/numerical engine), 4.4 (statistics), 4.5 (graphics/visualization), 4.6 (data import/export/persistence), and 4.8 (specialized products/toolboxes). Numerical limitations documented: complex eigendecomposition (`eigenvalues` only for non-symmetric; eigenvectors missing; ignored regression test preserved); performance baselines measured (M4); no speculative optimization applied.
- `crates/linalg/src/decomposition.rs` — `improved_qr_eig` (non-symmetric eigenvalues only), `jacobi_eigen` (real symmetric), SVD reconstruction verified (`Batch 17`), LU/QR/pseudoinverse validated (`Batch 14`, `Batch 17`).
- `tests/language/tests/numeric_batch14.rs`, `numeric_batch17.rs`, `numeric_stage7_quick.rs` — Numerical regression baselines preserved.

## Desktop and graphics

- `docs/rea-compatibility-audit.md` — Section 4.5 (graphics/visualization) and 9.3 (live GUI verification `NOT TESTED` with reason). Desktop architecture verified (`GraphicsState`, `FigureState`, `PlotItem`); model-level plot propagation verified (`plot_stage4_quick` 8/8); interactive visual interaction not executed in non-interactive session.
- `docs/desktop-ui.md` — Desktop framework description, event paths (`execution.rs` persistent `Session`, cancellation flags, `request_rx`), workspace/variable inspection, figure window architecture, and M3 documentation update.
- `tests/language/tests/plot_stage4_quick.rs`, `desktop_stage9_quick.rs` — Plot/model and desktop workflow tests.

## Testing and performance

- `tests/language/` — Integration test targets (`numeric_batch*`, `user_functions_stage3`, `plot_stage4_quick`, `stats_stage5_quick`, `data_stage6_quick`, `numeric_stage7_quick`, `language_stage8_audit`, `desktop_stage9_quick`, `perf_stage10_quick`, `persist_stage5`, `compat_stage6`, and others).
- Performance baselines (`perf_stage10_quick`): 10×10 mult ~330 µs, 50×50 mult 3.56 ms, 100×100 mult 24.19 ms, 20×20 solve 475 µs, 30×30 svd 9.86 ms, stats 100×100 1.26 ms (host: Xeon E3-1225v5, rustc 1.98.1, dev profile). Measurements are environment-specific observations, not universal guarantees.
- `docs/progress.md` — Milestone results (M2 `BLOCKED`, M3/M4/M5/M6/M7 completed), regression preservation notes, and documentation updates.

## Reverse-engineering evidence and limitations

- `docs/rea-compatibility-audit.md` — Section C (REA status): REA tools discovered; no binary session opened; native MATLAB target (`/home/cherry/Downloads/matlab_R2026b_Linux`) is installer archive; `/usr/local/MATLAB` and `/opt/MATLAB` absent. Section K (native reverse-engineering limitation): what REA inspected (directory listings) and what could not be inspected (internal MATLAB engine, symbol tables, execution traces). Section L (release criteria): supported scope explicitly defined; unsupported scope listed; no tag/release created.
- `docs/rea-compatibility-audit.md` — Section F (defect register): P1 complex eigensolver (`BLOCKED`/`DOCUMENTED`); P2 desktop live rendering (`NOT TESTED`); P3 optional extensions (`NOT IMPLEMENTED`).
- `docs/progress.md` — Documents M2 blocked status explicitly; no fabricated compatibility claims.

## Roadmap and future work

- `docs/rea-compatibility-audit.md` — Section H: dependency-aware roadmap (M1 audit complete, M2 blocked until independently validated, M3 desktop live verification `NOT TESTED`, M4 benchmark suite completed with measurements, M5 persistence partial with regression test, M6 compatibility tests expanded, M7 release readiness final gate satisfied, M8 optional extensions future).
- Section 13.1 (required output): prioritized backlog with user value, prerequisites, source references, complexity estimates, risks, and acceptance criteria.
- Section 13.2 (`docs/rea-compatibility-audit.md` line numbers reference M2, M3, M4, M5, M6, M7, M8 definitions).
