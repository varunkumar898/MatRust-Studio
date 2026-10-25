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

## 2. Planned Roadmapped Milestones

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
