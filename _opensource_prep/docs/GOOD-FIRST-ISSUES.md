# Good first issues

The following drafts are based on rows marked **Partial** in the current feature matrix and coverage registry. Review each scope and label before opening it.

## 1. Add useful properties to caught `MException` values

- **Labels:** `good first issue`, `compatibility`
- **Context:** The registry says `try ... catch ME` works, but useful `MException` object properties are missing.
- **Where to look:** `crates/rustlab-eval/src/eval.rs`, `crates/rustlab-eval/tests/milestone_a1_tests.rs`
- **Acceptance criteria:** Define the supported properties; add self-authored tests for property reads and unknown properties; document the limits and add the function coverage entry.
- **Difficulty:** Medium

## 2. Implement core `containers.Map` behavior

- **Labels:** `help wanted`, `compatibility`
- **Context:** The feature matrix identifies `containers.Map` support as partial.
- **Where to look:** `crates/rustlab-stdlib/src/containers.rs`, `crates/rustlab-stdlib/src/lib.rs`
- **Acceptance criteria:** Add a small, documented map API for creating, reading, updating, and checking keys; test empty maps, missing keys, and supported key/value types; update the registry.
- **Difficulty:** Medium

## 3. Complete App Designer event callback arguments

- **Labels:** `help wanted`, `compatibility`
- **Context:** Anonymous functions used as App Designer event callbacks, such as `@(src,evt)`, are marked partial.
- **Where to look:** `crates/mlapp-lang/src/lib.rs`, `crates/mlapp-runtime/src/lib.rs`, `crates/mlapp-runtime/tests/runtime_tests.rs`
- **Acceptance criteria:** Pass source and event values consistently to supported callbacks; add an end-to-end synthetic app test and document unsupported event forms.
- **Difficulty:** Medium

## 4. Improve N-D array display in table-like UI views

- **Labels:** `good first issue`, `help wanted`, `compatibility`
- **Context:** The feature matrix says N-D numerical arrays work, while their display in table-like UI views is partial.
- **Where to look:** `crates/mlapp-ui/src/lib.rs`, `crates/mlapp-render/src`, `crates/rustlab-math`
- **Acceptance criteria:** Define a readable summary for N-D values, test representative shapes and empty arrays, and document when the UI truncates or summarizes values.
- **Difficulty:** Medium

## 5. Add adaptive step control to `ode45`

- **Labels:** `help wanted`, `compatibility`
- **Context:** The coverage registry describes the current `ode45` implementation as fixed-step Dormand-Prince (RK45), so adaptive variable-step behavior remains incomplete.
- **Where to look:** `crates/rustlab-stdlib/src/simulation.rs`, `apps/desktop/tests/desktop_tests.rs`
- **Acceptance criteria:** Add tolerance-controlled step acceptance/rejection, validate against a self-authored analytic ODE case, and cover failure and minimum-step behavior.
- **Difficulty:** Advanced

## 6. Complete and test the `ode23` RK23 solver

- **Labels:** `help wanted`, `compatibility`
- **Context:** `ode23` is listed as partial in the Tier F2 registry.
- **Where to look:** `crates/rustlab-stdlib/src/simulation.rs`, `apps/desktop/tests/desktop_tests.rs`
- **Acceptance criteria:** Specify the supported RK23 behavior, add deterministic analytic and edge-case tests, and update the registry with the remaining limits.
- **Difficulty:** Advanced

## 7. Complete the `butter` bilinear-transform path

- **Labels:** `help wanted`, `compatibility`
- **Context:** The registry marks `butter` partial and identifies bilinear-transform IIR behavior as incomplete.
- **Where to look:** `crates/rustlab-stdlib/src/signal_proc.rs`, `crates/rustlab-stdlib/tests/stdlib_tests.rs`
- **Acceptance criteria:** Add documented coefficient cases for supported orders and normalized frequencies; test invalid inputs and numeric stability; state unsupported cases explicitly.
- **Difficulty:** Advanced

## 8. Implement forward-backward `filtfilt`

- **Labels:** `help wanted`, `compatibility`
- **Context:** The registry marks zero-phase forward-backward filtering as partial.
- **Where to look:** `crates/rustlab-stdlib/src/signal_proc.rs`, `crates/rustlab-stdlib/tests/stdlib_tests.rs`
- **Acceptance criteria:** Implement the supported forward/backward pass and boundary handling; test a known signal, short inputs, and invalid coefficients; document edge behavior.
- **Difficulty:** Advanced

## 9. Complete bracketed `fzero` behavior

- **Labels:** `help wanted`, `compatibility`
- **Context:** `fzero` is partial; the registry names Brent-Dekker root finding as the target.
- **Where to look:** `crates/rustlab-stdlib/src/numerical.rs`, `apps/desktop/tests/desktop_tests.rs`
- **Acceptance criteria:** Support a bracketed root case with convergence limits, reject invalid brackets clearly, and add tests for endpoint roots and non-convergence.
- **Difficulty:** Advanced

## 10. Return frequency-grid data from `bode`

- **Labels:** `help wanted`, `compatibility`
- **Context:** `bode` is partial; frequency-grid magnitude and phase data are not complete.
- **Where to look:** `crates/rustlab-stdlib/src/simulation.rs`, `apps/desktop/tests/desktop_tests.rs`
- **Acceptance criteria:** Define and test frequency, magnitude, and phase outputs for a small transfer function; cover zero/invalid frequency inputs and document the output shape.
- **Difficulty:** Medium

## 11. Add numerical Nyquist data support

- **Labels:** `help wanted`, `compatibility`
- **Context:** `nyquist` is partial; complex frequency-response data and polar presentation remain incomplete.
- **Where to look:** `crates/rustlab-stdlib/src/simulation.rs`, `apps/desktop/tests/desktop_tests.rs`
- **Acceptance criteria:** Return deterministic complex response data for a documented frequency grid; add simple transfer-function tests and document plotting versus numerical-data scope.
- **Difficulty:** Medium

## 12. Add a Simulink relational-operator block

- **Labels:** `good first issue`, `simulink`
- **Context:** The current block model enumerates constants, sources, arithmetic, state, saturation, scopes, and ports, but has no relational-operator block.
- **Where to look:** `crates/mlapp-sim/src/model.rs`, `crates/mlapp-sim/src/parser.rs`, `crates/mlapp-sim/src/solver.rs`, `crates/mlapp-sim/tests/sim_tests.rs`
- **Acceptance criteria:** Parse and simulate one clearly scoped comparison operation, test true/false signal cases and unsupported operators, and document that this is a limited clean-room implementation.
- **Difficulty:** Medium
