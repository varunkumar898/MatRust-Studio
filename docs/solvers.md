# Simulink & Control Block Solvers in MatRust Studio

This document specifies the numerical integration methods, parameter defaults, error diagnostics, and discrete-block execution rules implemented in `mlapp-sim` and the MatRust Studio block diagram environment.

---

## 1. Supported Solvers

| Solver Name | Alias / SLX Identifier | Type | Order | Formulation & Butcher Tableau |
|---|---|---|---|---|
| **`FixedStepEuler`** | `ode1`, `Euler` | Fixed-step explicit | 1 | $y_{n+1} = y_n + h \cdot f(t_n, y_n)$ |
| **`FixedStepRk4`** | `ode4`, `RK4` | Fixed-step explicit | 4 | Classical Runge-Kutta 4-stage method: $k_1 = f(t_n, y_n)$, $k_2 = f(t_n + h/2, y_n + \frac{h}{2} k_1)$, $k_3 = f(t_n + h/2, y_n + \frac{h}{2} k_2)$, $k_4 = f(t_n + h, y_n + h k_3)$, $y_{n+1} = y_n + \frac{h}{6}(k_1 + 2k_2 + 2k_3 + k_4)$ |
| **`VariableStepDormandPrince`** | `ode45`, `DormandPrince`, `DormandPrince5` | Adaptive variable-step explicit | 5(4) with FSAL | Dormand-Prince DOPRI5 7-stage embedded Runge-Kutta pair. Propagates 5th-order candidate solution with 4th-order embedded error estimate and FSAL stage reuse. |

---

## 2. Solver Parameter Defaults & Invariants

| Parameter | Type / Format | Default Value | Valid Range / Invariant | Validation Error |
|---|---|---|---|---|
| **`start_time`** | `f64` | `0.0` | Finite number | `E-PARAM` |
| **`stop_time`** | `f64` | `10.0` (in editor: `5.0`) | Finite number, strictly $> \text{start\_time}$ | `E-PARAM` |
| **`step_size`** (`dt`) | `f64` | `0.01` | Finite number, strictly $> 0.0$ | `E-PARAM` |
| **`solver`** | `SolverType` | `FixedStepEuler` | `FixedStepEuler`, `FixedStepRk4`, `VariableStepDormandPrince` | Serde custom error listing valid solvers |
| **`abs_tol`** (`AbsTol`) | `f64` | `1e-6` | Finite number, strictly $> 0.0$ | `E-PARAM` |
| **`rel_tol`** (`RelTol`) | `f64` | `1e-3` | Finite number, $\ge 1\times 10^{-13}$ | `E-PARAM` |
| **`max_step`** (`MaxStep`) | `Option<f64>` | Automatic: `(stop - start) / 50.0` | Strictly $> \text{MinStep}$ | `E-PARAM` |
| **`min_step`** (`MinStep`) | `Option<f64>` | Dynamic: $\max(10^{-12}, 16 \cdot \varepsilon \cdot \|t\|)$ | Strictly $< \text{MaxStep}$ | `E-PARAM` |
| **`initial_step`** (`InitialStep`) | `Option<f64>` | Automatic (Hairer starting-step heuristic) | Inside $[\text{MinStep}, \text{MaxStep}]$ | `E-PARAM` |
| **`max_steps`** (`MaxSteps`) | `usize` | `1_000_000` | Strictly $> 0$ | `E-PARAM` |

---

## 3. Step-Size Control & Adaptation (Dormand-Prince 5(4))

### 3.1 Error Norm
At each step from $t$ to $t+h$, the scaled RMS error norm across all continuous states $N$ is computed:

$$\text{scale}_i = \text{AbsTol} + \text{RelTol} \cdot \max(|y_i|, |y_{new, i}|)$$

$$e_i = h \cdot \sum_{j=1}^7 E_j \cdot k_{j, i}$$

$$\text{err} = \sqrt{\frac{1}{N} \sum_{i=1}^N \left(\frac{e_i}{\text{scale}_i}\right)^2}$$

### 3.2 Acceptance and Adaptation
- **Accept**: If $\text{err} \le 1.0$, the step is accepted:
  - Continuous states advance: $y_{n+1} = y_{new}$.
  - Simulation time advances: $t_{n+1} = t_n + h$.
  - Discrete states are updated if $t_{n+1}$ hits a sample boundary.
  - Step size for the next step is adjusted:
    $$\text{fac} = \text{clamp}(0.9 \cdot \text{err}^{-1/5}, 0.2, 5.0)$$
    If the immediately preceding step was rejected, $\text{fac} \le 1.0$ is enforced.
    $$h_{next} = h \cdot \text{fac}$$
- **Reject**: If $\text{err} > 1.0$, the candidate step is discarded:
  - Continuous states, discrete states, and simulation time are **not** advanced.
  - No points are logged to `tout` or output signals.
  - Step factor is constrained to $\le 1.0$:
    $$\text{fac} = \text{clamp}(0.9 \cdot \text{err}^{-1/5}, 0.2, 1.0)$$
    $$h_{reduced} = h \cdot \text{fac}$$
  - If $h_{reduced} < \text{MinStep}$ (and the step is not the final step landing on `stop_time`), error `E-STEP` is raised.

### 3.3 First Same As Last (FSAL)
- Stage 7 of the DOPRI5 pair evaluates the derivative $f(t+h, y_{new})$. Since DOPRI5 is an FSAL method with $b_i = a_{7i}$, $k_7$ matches the first derivative evaluation $k_1$ of the subsequent step.
- **FSAL Invalidation Rule**: When an accepted step lands on a model breakpoint (e.g. `Step` transition) or a discrete sample hit, the derivative must not be reused across the discontinuity. FSAL is explicitly invalidated, forcing an independent evaluation at the new point with updated discrete states and time.

---

## 4. Breakpoints and Discontinuities

The execution engine provides `next_breakpoint(t) -> Option<f64>`:
- The `Step` block publishes its transition time $t_{step}$.
- The solver checks upcoming breakpoints and clamps candidate step size $h \le t_{breakpoint} - t$.
- The solver lands exactly on the discontinuity ($t = t_{breakpoint}$ in `tout`), preventing solver thrashing and excessive rejections.

---

## 5. Discrete Blocks and SampleTime Rules

Discrete blocks (e.g., `UnitDelay`) adhere to strict multi-rate and sample-time contracts:

1. **Parameter**: `UnitDelay` accepts an optional `SampleTime` parameter in seconds.
2. **Fixed-Step Solvers (`FixedStepEuler`, `FixedStepRk4`)**:
   - Absent: Inherits $dt$ (bit-identical backward compatibility with legacy models).
   - Specified: Must be a positive integer multiple of $dt$ ($\left|\frac{T_s}{dt} - \text{round}\left(\frac{T_s}{dt}\right)\right| / dt \le 10^{-9}$), otherwise error `E-SAMPLE` is raised naming the offending block.
3. **Variable-Step Solvers (`VariableStepDormandPrince`)**:
   - Absent: Returns error `E-SAMPLE` naming the block and instructing the user to configure `SampleTime`. A variable-step solver never silently guesses a discrete sample period.
   - Specified: Sample hits occur at integer multiples:
     $$t_k = t_{start} + k \cdot T_s, \quad k \in \mathbb{Z}_{\ge 0}$$
     Floating-point accumulation drift is avoided by integer indexing.
   - The variable-step solver truncates candidate steps to land exactly on $t_k$.
   - Discrete block outputs are held constant between sample hits.
   - `update_discrete` executes strictly on accepted steps at sample hits.

---

## 6. Zero Continuous States Handling

When simulating a diagram with zero continuous states ($N = 0$) under `VariableStepDormandPrince`:
- If the diagram contains discrete blocks or scheduled breakpoints:
  The engine steps forward by `MaxStep`, landing exactly on sample hits and breakpoints, updating discrete states at hits, and logging outputs.
- If the diagram contains **neither continuous nor discrete states nor breakpoints**:
  The solver logs $t_{start}$, evaluates $t_{stop}$, logs $t_{stop}$, and completes immediately in 2 evaluations without wasteful empty stepping.

---

## 7. Diagnostic Error Catalog (what -> why -> how to fix)

Every diagnostic message follows the canonical three-part structure:

| Code | Title | Example Trigger | Diagnostic Message Pattern |
|---|---|---|---|
| **`E-ALG`** | Algebraic Loop | Feedthrough cycle Gain -> Sum -> Gain | `Algebraic loop detected (E-ALG): Direct-feedthrough dependency cycle detected among blocks: [...]. The outputs of these blocks instantaneously depend on their own inputs. To fix: insert a Unit Delay or Integrator in the loop to break the instantaneous algebraic feedback.` |
| **`E-STEP`** | Step Size Limit | Finite-time blow-up $\dot{x} = x^2$ | `Step size error (E-STEP): Solver reached minimum step size ({min_step}) at t = {t} without meeting error tolerances. Why: The system equations may be stiff, experiencing a discontinuity, or approaching a singularity. How to fix: Relax AbsTol/RelTol, decrease MinStep, or inspect model dynamics for rapid state transitions.` |
| **`E-NAN`** | Non-Finite Value | Division by zero or overflow | `Non-finite value encountered (E-NAN): Non-finite {context} ({val}) encountered at t = {t} in block '{name}' (SID: {sid}). Why: Division by zero, numerical overflow, or unstable feedback occurred. How to fix: Check input signals, parameters, and saturation bounds to prevent arithmetic overflow.` |
| **`E-SAMPLE`** | Sample Time Error | Missing `SampleTime` under DP | `Sample time error (E-SAMPLE): Discrete block '{name}' (SID: {sid}) is missing parameter 'SampleTime' under variable-step solver. Why: Variable-step solvers cannot infer discrete block update rates without an explicit sample period. How to fix: Set parameter 'SampleTime' on block '{name}' (e.g. '0.01').` |
| **`E-PARAM`** | Parameter Validation | `stop_time <= start_time` | `Invalid simulation parameters (E-PARAM): Stop time ({stop}) must be strictly greater than start time ({start}). Why: Simulation time must advance forward. How to fix: Increase stop time or decrease start time.` |
