# RustLab Backlog (docs/BACKLOG.md)

Items tracked here are out of scope for the current milestone but recorded for future implementation or verification.

---

## 1. Unverified Behaviors Requiring Verification Against Upstream MATLAB / Octave

- **Display Formatting**:
  - Threshold rule switching non-integers between 0.001 and 0.01 to scientific notation (`1.0000e-03` vs `0.0100`).
  - Common scale factor rule boundary: exact matrix dynamic range triggering `1.0e+03 *` vs per-element scientific notation (`[1e-16 1]` vs `[0.001 1000]`).
  - Mixed matrix non-integer formatting: integer-valued elements in a matrix containing any non-integer print with decimals (`[1.5 22]` -> `1.5000  22.0000`) except exact zero (`0`).
- **Octave Environment Verification**:
  - GNU Octave is not installed in the Windows ARM64 environment. Verification was performed against MathWorks online documentation. Octave parity checks will run when Octave is available.
- **Error and Warning Identifiers**:
  - All identifiers marked `unverified` in `docs/spec/error-catalog.md` (e.g. `MATLAB:integerTypeMismatch`, `MATLAB:dimagree`, `MATLAB:innerdim`).

---

## 2. LU / Linear-Solve Teaching Note Correction (G0.1-VERIFY)

The earlier teaching note said the LU trailing update costs about **n^3/2**
multiply-adds. That is wrong. The forward elimination/trailing update for an
n-by-n LU factorization costs

`sum_{k=1..n} (n-k)^2 = 1^2 + 2^2 + ... + (n-1)^2 ~ n^3/3`

multiply-adds. Back substitution adds about `n^2/2` operations. The `n^3/2`
figure is an order-of-magnitude label only, not the leading constant.

The implementation still updates whole rows. `mldivide_fn` in
`crates/rustlab-stdlib/src/linalg.rs` zeroes the pivot column and then updates
every column from `col + 1` through the last augmented column, including
already-zero left-hand-side columns. `compute_lu` builds full L and U matrices
and computes each entry with dot products; it has no trailing-submatrix
microkernel. A future optimization can restrict the update to the trailing
submatrix and cache blocks, but that is out of scope for verification.

---

## 3. Benchmark Contract Misses & Proposed Next Steps (Milestone G1)

From official benchmark runs on 1,000,000 elements against original G0.1 contracts:

1. **`movmean` (1e6 doubles, k=3)**:
   - **Status**: MISSED. Regressed from 13.865 ms (G0.1-VERIFY) to cross-run median **16.739 ms** (runs: 16.739 ms, 18.397 ms, 14.962 ms) vs contract target $\sim 10$ ms.
   - **Root Cause**: The running-sum algorithm introduced in G1 includes periodic exact recomputations every 1024 steps, finite-window boundary checks, and slice-based window management that adds branching and index computation overhead per element in steady state.
   - **Proposed Next Step**: Profile steady-state loop with CPU counters; widen the periodic recomputation interval from 1024 to 65536 (or adopt a compensated Kahan running sum with zero recomputation for pure finite windows); remove slice slicing in favor of direct raw pointer or unchecked sliding indexing in the verified finite inner loop.

2. **Interpreter `A + B` (1e6 doubles)**:
   - **Status**: MISSED. Cross-run median **3.533 ms** (runs: 0.523 ms, 3.533 ms, 3.578 ms) vs contract target $\le 3.0$ ms. Minimum warm runs (0.434 ms) show hardware capacity is adequate, but median across independent invocations exceeds target due to AST evaluation overhead.
   - **Root Cause**: AST tree-walk evaluation incurs dynamic dispatch overhead, environment variable lookups, and allocation checks for statement results even when operands are borrowed.
   - **Proposed Next Step**: Implement expression-level bytecode compilation or threaded code for binary arithmetic expressions (Milestone B7), bypassing environment hash map lookups on repetitive inner loops; or add a specialized tight eval path for simple binary assignment statements `C = A + B`.

3. **`cummax` (1e6 doubles)**:
   - **Status**: UNSTABLE. Cross-run median **11.556 ms** with high inter-run spread of **16.839 ms** (Run 1: 11.556 ms, Run 2: 24.796 ms, Run 3: 7.957 ms) vs contract target $\sim 10$ ms.
   - **Root Cause**: Memory bandwidth variability, page fault latency, and thread scheduling contention across runs on Windows ARM64. While Run 3 achieved 7.957 ms (well under target), Run 2 hit 24.796 ms.
   - **Proposed Next Step**: Investigate thread affinity / priority during benchmark execution; implement cache-line prefetching or unroll loop by 4 or 8 elements to hide memory latency during sequential write passes.

4. **`zscore` (1e6 doubles)**:
   - **Status**: MARGINAL. Cross-run median **10.197 ms** (runs: 9.942 ms, 11.397 ms, 10.197 ms) vs contract target $\sim 10$ ms.
   - **Root Cause**: Two-pass algorithm computes mean and variance with 4 ILP accumulators, but standard deviation division and final array allocation / normalization pass traverses the 8 MB double array twice.
   - **Proposed Next Step**: Further unroll the normalization pass and explore auto-vectorization friendly loops or fused operations where possible.

---

## 4. Backend Expansion Backlog & Open Items (`wip/backend-expansion`)

1. **B2.2 Plotting Alignment (`rustlab-plot`)**:
   - Interactive GUI plotting currently implemented via `egui_plot` in MatRust Studio figures panel.
   - Milestone B2.2 contract requires a standalone, renderer-independent plot-description model with a deterministic headless SVG renderer (`figure`, `plot`, `saveas('x.svg')`).

2. **Hardware and Embedded Systems (B8) — PLANNED / BLOCKED**:
   - Status: PLANNED / BLOCKED.
   - Milestone B8 requires serial communications (`serialport`), ring buffers with timestamps, TCP/UDP sockets, and MISRA-friendly C99 code generation behind hardware traits with mocks (so tests need no physical devices).
   - Hardware interfaces are currently blocked pending completion of foundational milestones (B2.2 plot model, B2.3 tables, B1 math tools, B4 signal, B3 simulation, B5 ML, B6 robotics, B7 language).

3. **Stiff ODE Solvers & Advanced Simulation**:
   - First- and second-order analytical and numerical step responses implemented in `simulation.rs`.
   - Variable-step stiff ODE solvers (e.g. Radau IIA / BDF) and full state-space block diagram simulation engine tracked for milestone B3.

4. **REA ARM64 Decompilation Support**:
   - REA PE import and hygiene checks verified. Deep instruction-level decompilation of hot loops (`Value::add`, GEMM, FFT) awaits ARM64 decompiler backend support.

---

## 5. Planned Roadmapped Milestones

- **G1**: Built-in Self-Debug Tools (`--dump-tokens`, `--dump-ast`, `--trace`, `doctor`, `selftest`, `explain`) and `tests/golden` corpus (I1..I15).
- **B2.2**: `rustlab-plot` plot-description model and deterministic SVG renderer.
- **B2.3**: Tables and file formats (`table`, `timetable`, XLSX via calamine, JSON via serde_json).
- **B1**: Advanced Math Tools (`fzero`, `fsolve`, `roots`, `polyfit`, `interp1`, `spline`, `integral`, `ode45`, `fminsearch`, `linprog`).
- **B4**: Signal and Image Processing (`filter`, `butter`, `pwelch`, `spectrogram`, `imread`/`imwrite`, image filters).
- **B3**: Simulation & Controls (`tf`, `ss`, `step`, `bode`, block diagrams).
- **B5**: Machine Learning (linear/logistic regression, CART decision trees, k-means, PCA, MLP backprop).
- **B6**: Robotics (rotations, SE(3), forward/inverse kinematics, PID, trajectory generation).
- **B7**: Language completeness (loading path, profiler, parfor, containers.Map, classdef design).
- **B8**: Hardware and C99 Code Generation (serial, ring buffers, MISRA C99 codegen).
