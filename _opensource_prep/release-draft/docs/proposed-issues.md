# Draft issues for MatRust Studio

These are ready-to-edit issue drafts based on the current coverage registry,
backlog, owed-items register, and workspace layout. Confirm each scope and
file path against the release branch before publishing.

> **Coverage data needs reconciliation.** `docs/COVERAGE.md` summarizes 44
> partial and 23 missing entries (67 not fully implemented), but the detailed
> rows currently list only eight partial functions and one explicitly missing
> symbolic-math row. Issue 16 below is the required inventory repair. Do not
> label an additional function “missing” until the registry contains that row.

## 1. Add “did you mean?” suggestions to unknown names

- **Why it matters:** New users can recover from a misspelled variable or
  builtin without searching the whole project.
- **Files to touch:** `crates/rustlab-parse/src/parser.rs`,
  `crates/rustlab-eval/src/`, `crates/rustlab-stdlib/src/`.
- **Acceptance criteria:** Unknown-name diagnostics can suggest a nearby
  known builtin or variable; suggestions do not change evaluation; add tests
  for a close spelling, an unrelated name, and no available candidates.
- **Difficulty:** Intermediate.

## 2. Show parser diagnostics in the desktop editor

- **Why it matters:** Syntax errors should be visible where a script is being
  edited, not only after execution.
- **Files to touch:** `apps/desktop/src/syntax.rs`,
  `apps/desktop/src/editor.rs`, `crates/rustlab-parse/src/`.
- **Acceptance criteria:** Parse diagnostics retain source spans, appear
  inline or in a diagnostics panel, update after edits, and have tests for
  valid and invalid scripts.
- **Difficulty:** Intermediate.

## 3. Add debugger breakpoints and step controls

- **Why it matters:** A beginner needs to see where execution stops and what
  statement runs next.
- **Files to touch:** `apps/desktop/src/execution.rs`,
  `apps/desktop/src/editor.rs`, `apps/desktop/src/app.rs`,
  `crates/rustlab-eval/src/`.
- **Acceptance criteria:** A breakpoint pauses before its statement; continue
  and step behave deterministically; cancellation remains responsive; tests
  cover pause, resume, and removal.
- **Difficulty:** Advanced.

## 4. Add a live workspace-variable inspector

- **Why it matters:** Seeing variable names, classes, and shapes makes
  numerical scripts easier to debug.
- **Files to touch:** `apps/desktop/src/workspace_panel.rs`,
  `apps/desktop/src/execution.rs`, `crates/rustlab-math/src/`.
- **Acceptance criteria:** Display current workspace names and safe summaries
  of value class and dimensions; refresh after execution; avoid cloning large
  arrays just to render the panel; include UI/state tests.
- **Difficulty:** Intermediate.

## 5. Generate a privacy-safe script-to-issue report

- **Why it matters:** Bug reports are more useful when they include a small
  reproduction, runtime version, and the exact diagnostic.
- **Files to touch:** `crates/rustlab-cli/src/`,
  `apps/desktop/src/`, `docs/`.
- **Acceptance criteria:** A user can export a minimal report containing the
  selected script only after confirmation; local paths and workspace contents
  are omitted or redacted by default; tests verify redaction and output shape.
- **Difficulty:** Intermediate.

## 6. Add a one-command golden regression runner

- **Why it matters:** Small language examples give contributors fast feedback
  without reading a long workspace test log.
- **Files to touch:** `crates/rustlab-cli/src/`, `tests/`,
  `CONTRIBUTING.md` (or its review-only proposal).
- **Acceptance criteria:** The runner discovers documented synthetic golden
  cases, compares output deterministically, exits nonzero on mismatch, and
  reports each case by filename; include pass and failure tests.
- **Difficulty:** Intermediate.

## 7. Preserve source locations in runtime stack traces

- **Why it matters:** An error inside a called function should point to the
  useful source line and call chain.
- **Files to touch:** `crates/rustlab-parse/src/`,
  `crates/rustlab-eval/src/`, `crates/rustlab-cli/src/`.
- **Acceptance criteria:** Runtime errors keep source spans across nested
  calls; the CLI renders the call chain without losing the original error;
  tests cover nested function calls and missing span data.
- **Difficulty:** Advanced.

## 8. Add an RK4 fixed-step solver option

- **Why it matters:** The current Simulink engine is fixed-step Euler; RK4 is
  a useful next solver for smoother continuous-state examples.
- **Files to touch:** `crates/mlapp-sim/src/solver.rs`,
  `crates/mlapp-sim/src/model.rs`, simulation tests.
- **Acceptance criteria:** Solver selection is validated, an analytical
  first-order ODE fixture shows the expected convergence improvement over
  Euler, and invalid solver names return a clear error.
- **Difficulty:** Advanced.

## 9. Implement an error-controlled variable-step ODE solver

- **Why it matters:** Fixed step sizes can be inefficient or inaccurate when
  a model changes quickly in only part of its time range.
- **Files to touch:** `crates/mlapp-sim/src/solver.rs`,
  `crates/mlapp-sim/src/model.rs`, simulation tests.
- **Acceptance criteria:** Document absolute/relative tolerance behavior,
  bound rejected-step loops, test step growth/shrinkage, and compare a simple
  ODE against an analytical solution. Do not claim MATLAB `ode45` parity
  without sourced behavior and tests.
- **Difficulty:** Advanced.

## 10. Simulate nested subsystems

- **Why it matters:** `Subsystem` is recognized in the block model, but the
  current solver's supported block list does not include it.
- **Files to touch:** `crates/mlapp-sim/src/model.rs`,
  `crates/mlapp-sim/src/parser.rs`, `crates/mlapp-sim/src/solver.rs`.
- **Acceptance criteria:** Parse a synthetic nested subsystem, define
  inport/outport mapping, simulate a two-block subsystem, and reject
  malformed or recursive nesting with a useful diagnostic.
- **Difficulty:** Advanced.

## 11. Expand source, sink, and arithmetic block support

- **Why it matters:** The block model currently recognizes a small set of
  block types; useful diagrams need additional common building blocks.
- **Files to touch:** `crates/mlapp-sim/src/model.rs`,
  `crates/mlapp-sim/src/parser.rs`, `crates/mlapp-sim/src/solver.rs`.
- **Acceptance criteria:** Select one documented block (for example a ramp
  source or mux), add parser and execution tests, define port-width behavior,
  and report unsupported blocks rather than silently substituting behavior.
- **Difficulty:** Intermediate.

## 12. Connect Scope outputs to desktop plots

- **Why it matters:** Scope data is currently collected by the simulator;
  users need a clear visual way to inspect signals over time.
- **Files to touch:** `crates/mlapp-sim/src/solver.rs`,
  `apps/desktop/src/figures.rs`, `crates/mlapp-plot/src/`.
- **Acceptance criteria:** A synthetic model's logged signal can be plotted
  against simulation time; empty and multi-signal outputs are handled; add a
  deterministic test of the data passed to the renderer.
- **Difficulty:** Intermediate.

## 13. Report `.slx` inspection coverage and unsupported constructs

- **Why it matters:** Users need to distinguish a fully inspected model from
  one where blocks or properties were ignored.
- **Files to touch:** `crates/mlapp-sim/src/inspect.rs`,
  `crates/mlapp-sim/src/parser.rs`, `crates/mlapp-cli/src/`.
- **Acceptance criteria:** The report counts recognized, unsupported, and
  malformed constructs; includes stable block identifiers; tests use
  synthetic archives only and never real proprietary `.slx` files.
- **Difficulty:** Intermediate.

## 14. Add verified Windows, Linux, and macOS release packages

- **Why it matters:** Contributors should not need a local Rust toolchain just
  to try the application.
- **Files to touch:** `.github/workflows/`, `docs/`, release scripts only
  after maintainer approval.
- **Acceptance criteria:** CI builds each supported OS on its native runner,
  archives the CLI and desktop binary with required notices, smoke-tests
  `--help`, and publishes checksums. Mark an OS unsupported until its job is
  green.
- **Difficulty:** Advanced.

## 15. Package-manager installation and release automation

- **Why it matters:** Homebrew, Scoop, and AppImage users expect versioned,
  reproducible installation instructions.
- **Files to touch:** `.github/workflows/`, packaging manifests, `docs/`.
- **Acceptance criteria:** Start with one platform-specific package; pin
  release checksums, document updates and rollback, and test installation from
  a tagged release rather than a developer build.
- **Difficulty:** Advanced.

## 16. Reconcile the function coverage registry

- **Why it matters:** The coverage summary says 44 partial plus 23 missing
  (67 not fully implemented), while the detailed table shows only eight
  partial functions and one missing symbolic row.
- **Files to touch:** `docs/COVERAGE.md`, the source/catalog used to generate
  it, and its validation tests or script.
- **Acceptance criteria:** Every summary entry has a detailed row; counts are
  reproducible from the rows; distinguish partial from missing; verify every
  referenced test exists; document intentionally out-of-scope rows. Until
  reconciled, mark the 67-item list `[VERIFY]`.
- **Difficulty:** Intermediate.

## 17. Complete `fzero` convergence and failure reporting

- **Why it matters:** The registry marks `fzero` partial and identifies a
  Brent-Dekker root finder.
- **Files to touch:** `crates/rustlab-stdlib/src/`,
  `crates/rustlab-stdlib/tests/`, `docs/COVERAGE.md`.
- **Acceptance criteria:** Add bracketed root, endpoint root, no sign change,
  NaN, and iteration-limit tests; return documented errors on failure; update
  coverage only after these cases are verified.
- **Difficulty:** Advanced.

## 18. Complete `fminsearch` convergence reporting

- **Why it matters:** The registry marks the Nelder-Mead implementation
  partial; users need to know whether it converged or hit a limit.
- **Files to touch:** `crates/rustlab-stdlib/src/`,
  `crates/rustlab-stdlib/tests/`, `docs/COVERAGE.md`.
- **Acceptance criteria:** Test a convex minimum, a flat objective, invalid
  input, and iteration exhaustion; expose deterministic termination
  information; document limits and tolerances.
- **Difficulty:** Advanced.

## 19. Validate `butter` design inputs and coefficients

- **Why it matters:** The coverage registry marks Butterworth filter design
  partial, and invalid cutoff/order inputs can produce confusing results.
- **Files to touch:** `crates/rustlab-stdlib/src/`,
  `crates/rustlab-stdlib/tests/`, `docs/COVERAGE.md`.
- **Acceptance criteria:** Reject invalid order and normalized cutoff values
  with tested errors; test representative low-pass coefficients against a
  cited allowed source or analytical result; document supported filter types.
- **Difficulty:** Advanced.

## 20. Test `filtfilt` boundary and short-signal behavior

- **Why it matters:** The registry marks forward-backward filtering partial;
  edge handling is important for short data.
- **Files to touch:** `crates/rustlab-stdlib/src/`,
  `crates/rustlab-stdlib/tests/`, `docs/COVERAGE.md`.
- **Acceptance criteria:** Cover empty, one-sample, shorter-than-padding,
  constant, NaN, and normal signals; document boundary strategy and return
  errors rather than silently changing the requested operation.
- **Difficulty:** Intermediate.

## 21. Make `ode23` error handling and solver scope explicit

- **Why it matters:** The coverage registry marks `ode23` partial, and the
  backlog tracks advanced and stiff solvers separately.
- **Files to touch:** `crates/rustlab-stdlib/src/`,
  `crates/rustlab-stdlib/tests/`, `docs/BACKLOG.md`,
  `docs/COVERAGE.md`.
- **Acceptance criteria:** Test tolerance handling, zero-length intervals,
  derivative shape mismatch, and step-limit exhaustion; state whether the
  solver is fixed- or variable-step; keep stiff-solver work separately
  tracked.
- **Difficulty:** Advanced.

## 22. Add `containers.Map` or document its current absence

- **Why it matters:** The backlog lists `containers.Map` as a language
  completeness item; users need a keyed collection for many scripts.
- **Files to touch:** `crates/rustlab-math/src/` (value model),
  `crates/rustlab-stdlib/src/`, `crates/rustlab-eval/src/`, tests, docs.
- **Acceptance criteria:** Agree the key/value type model first; test insert,
  lookup, overwrite, missing key, and deterministic display; otherwise add a
  clear unsupported-feature entry to the compatibility registry.
- **Difficulty:** Advanced.

## 23. Specify a small `classdef`/OOP compatibility target

- **Why it matters:** The backlog identifies class definitions and object
  features as incomplete; a narrow documented subset is safer than implying
  general support.
- **Files to touch:** `crates/rustlab-parse/src/`,
  `crates/rustlab-eval/src/`, `crates/mlapp-lang/src/`, `docs/`.
- **Acceptance criteria:** Publish a design note defining supported syntax
  and unsupported features before implementation; add parser tests for the
  agreed subset; do not claim broad OOP compatibility.
- **Difficulty:** Advanced.

## 24. Write beginner tutorials using synthetic examples

- **Why it matters:** New contributors need a short path from a script to
  understanding a parser, value, or simulation test.
- **Files to touch:** `docs/`, `examples/` only after maintainer approval,
  tutorial tests if available.
- **Acceptance criteria:** Add at least one runnable numerical tutorial and
  one synthetic `.slx` or app-inspection walkthrough; record actual command
  output and Rust prerequisites; do not use proprietary files.
- **Difficulty:** Good first issue.

## 25. Add a reproducible benchmark report and accessible desktop checks

- **Why it matters:** Existing backlog records benchmark contract misses, and
  a desktop UI should remain usable with keyboard navigation and readable
  contrast.
- **Files to touch:** benchmark harnesses under the owning crate,
  `apps/desktop/src/`, `docs/`.
- **Acceptance criteria:** Keep benchmark harnesses/profile/thread counts
  separate and report repeated runs honestly; add a documented keyboard-only
  walkthrough and tests or manual checklist for focus order and contrast.
- **Difficulty:** Intermediate.
