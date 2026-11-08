# MatRust Studio & Simulation Diagnostics Reference

## 1. Overview and Principles

The MatRust simulation engine adheres to Charter Principle **P4**:
> **Proactive, Localized, and Explainable Diagnostics**:
> *"What happened -> Why it happened -> How to fix it"*

Rather than failing cryptically during numerical integration or matrix decomposition, MatRust Studio runs static and topological diagnostics **before** execution commences. Where safe, automated fixes (`apply_fix`) are provided with single-click resolution and 1-level undo.

---

## 2. Canonical Diagnostics Catalog

| Code | Severity | Title | Explanation | Typical Fix |
|:---|:---|:---|:---|:---|
| **`E-ALG`** | Error | Algebraic Loop | Direct feedthrough closed loop without an intervening state block (`Integrator`, `UnitDelay`, `ZeroOrderHold`, or strictly proper transfer function). | Insert an `Integrator`, `UnitDelay`, or `ZeroOrderHold` to break instantaneous dependency. |
| **`E-STEP`** | Error | Step Size Failure | Variable-step adaptive solver step size shrank below minimum tolerance without achieving error bounds. | Increase `AbsTol` / `RelTol` or check for state discontinuities. |
| **`E-NAN`** | Error | NaN / Infinite State | Numerical integration encountered non-finite (`NaN` or `Inf`) states or derivatives. | Check for division by zero in transfer functions or reduce integration step size. |
| **`E-SAMPLE`** | Error | Missing Sample Time | Discrete block requires an explicit positive `SampleTime` parameter in variable-step solver mode. | Configure parameter `SampleTime > 0` on the discrete component. |
| **`E-PARAM`** | Error | Invalid Parameter | Component parameter is negative, non-positive, or unparseable. | Verify resistance, capacitance, inductance, and fundamental frequency are strictly positive. |
| **`E-GND`** | Error | Missing Ground | Connected electrical circuit island has no Ground reference (`Node 0`). | Connect at least one terminal in the island to a `Ground` block. |
| **`E-FLOAT`** | Error | Floating Net / Island | Electrical net or island has no DC conductance path to Ground. | Add a Ground block to the net or provide a DC discharge path. |
| **`E-VLOOP`** | Error | Voltage Source Loop | Closed loop formed entirely of ideal voltage sources and closed switches without series impedance violates Kirchhoff's Voltage Law (KVL). | Add a small series resistor ($1\text{ m}\Omega$) or internal resistance to break the loop. |
| **`E-ICUT`** | Error | Current Source Cut-Set | Cut-set of ideal current sources violating Kirchhoff's Current Law (KCL) with no parallel path. | Add a large parallel resistor ($1\text{ M}\Omega$) across the current source. |
| **`E-SINGULAR`**| Error | Singular Circuit Matrix| Modified Nodal Analysis (MNA) matrix has a zero pivot during LU decomposition. | Reports offending row/node and attached circuit elements; eliminate floating nodes. |
| **`E-PORT`** | Error | Unconnected Input Port | Signal input port has no incoming wires (strict error in `schema_version >= 4`). | Connect the input port or connect a default `Constant` zero source. |
| **`W-PORT`** | Warning | Unconnected Input Port | Signal input port is unconnected; evaluates to default `0.0` (`schema_version < 4`). | Connect the input port to an explicit driving source. |
| **`E-MULTI`** | Error | Multiple Port Inputs | Multiple lines connect to a single input port without a `Summer` or `Mux` (`schema_version >= 4`). | Insert a `Sum` or `Mux` block to combine signals. |
| **`W-MULTI`** | Warning | Multiple Port Inputs | Multiple lines connect to a single input port (`schema_version < 4`). | Insert a `Sum` or `Mux` block. |
| **`E-SOLVER`** | Error | Incompatible Solver | Electrical domain simulation requires a fixed-step solver (`FixedStepEuler` or `FixedStepRk4`). | Change solver configuration in `SimulationConfig` to fixed-step Euler or RK4. |
| **`E-CHATTER`**| Error | Switching Limit Cycle | Diode or ideal switch limit cycle persisted for over 100 consecutive simulation steps. | Add RC snubber circuits across switches or decrease solver step size. |
| **`W-DELAY`** | Warning | Algebraic Hybrid Loop | Direct feedback loop through sensor and controlled source without state; resolved by inserting 1-step latency. | Insert a `UnitDelay` or `ZeroOrderHold` block. |
| **`W-WINDOW`** | Warning | Non-Integer Window | Measurement window duration $T = 1/f$ is not an exact integer multiple of sample time $T_s$. | Adjust $T_s$ or $f$ so that $1/(f \cdot T_s)$ is an exact integer. |
| **`W-CHATTER`**| Warning | Switch Chattering | Switching device limit cycle detected within simulation step iteration. | Add snubber capacitance or verify threshold hysteresis. |
| **`W-STEP`** | Warning | Excessive Step Size | Solver step size $h$ exceeds $2 \cdot \tau_{min}$ of circuit time constants ($\tau_{min} = \min(RC, L/R)$). | Reduce step size $h \le \tau_{min} / 5$. |
| **`W-WIRE`** | Warning | Redundant Wire | Self-loop wire or duplicate wire between the same terminal pair. | Remove redundant wire connection. |
| **`W-OPEN`** | Warning | Unconnected Terminal | Electrical terminal has no wires attached. | Connect terminal or remove unused component. |
| **`W-THD-LIMIT`**| Warning | Harmonic Clamped | Requested `MaxHarmonic` exceeds Nyquist limit $\lfloor(N_w - 1)/2\rfloor$. | Decrease `MaxHarmonic` or decrease `SampleTime`. |
| **`I-SLOW`** | Info | Fast-Path Fallback | Circuit contains $> 10$ switches or $> 64\text{ MB}$ table; uses runtime LU solver. | Informational notice; bit-exact results maintained. |
| **`I-THD-NOFUND`**| Info | Zero Fundamental | Fundamental harmonic amplitude is zero; THD is mathematically undefined. | Ensure signal has fundamental frequency content. |
| **`E-DOMAIN`** | Error | Domain Mismatch | Direct wire connection between continuous signal port and electrical terminal. | Use a `VoltageSensor`, `CurrentSensor`, `VCVS`, or `VCCS` bridge block. |
| **`E-UNSUPPORTED`**| Error | Unsupported Block | Block type is not supported in the active engine. | Replace with a supported standard library component. |

---

## 3. Automated Fix Engine (`apply_fix`)

The autofix engine enables single-click repair of common topological defects:

```rust
pub fn apply_fix(diagram: &mut BlockDiagram, fix: &FixAction) -> Result<(), SimError>;
```

### Supported Fix Actions:
1. **`AddGroundToNet { net_id, position }`**:
   - Resolves `E-FLOAT` and `E-GND`.
   - Creates a new `Ground` block and routes a wire to the target floating net.
2. **`InsertSeriesResistor { branch_id, value }`**:
   - Resolves `E-VLOOP`.
   - Splices a small damping resistor ($R = 1\text{ m}\Omega$) in series with the target voltage source branch.
3. **`InsertParallelResistor { node_id, value }`**:
   - Resolves `E-ICUT`.
   - Adds a parallel bleeder resistor ($R = 1\text{ M}\Omega$) between the open current source node and Ground.
4. **`ConnectPortToDefaultZero { block, port }`**:
   - Resolves `E-PORT` and `W-PORT`.
   - Adds a `Constant` block with `Value = "0"` and wires it to the unconnected input port.

### Undo Semantics:
Each invocation of `apply_fix` pushes the previous `BlockDiagram` onto an undo stack (`undo_diagram`). The desktop editor provides an **"Undo Fix"** button to restore the previous state immediately.

---

## 4. CLI Validation Tool (`mlapp-cli check`)

The `mlapp-cli check` subcommand performs static analysis on model JSON files without opening the desktop UI.

### Usage:
```bash
# Human-readable colored summary table
mlapp-cli check path/to/model.json

# Machine-readable JSON output for CI/CD pipelines
mlapp-cli check path/to/model.json --json
```

### Exit Codes:
- `0`: Diagram is clean (no errors, no warnings).
- `1`: Diagram contains warnings only (`W-PORT`, `W-STEP`, `W-WINDOW`, etc.).
- `2`: Diagram contains one or more errors (`E-FLOAT`, `E-VLOOP`, `E-ICUT`, `E-PORT`, etc.).

---

## 5. Desktop Problems Panel

In the MatRust Studio desktop editor:
- The top toolbar displays a **`Problems (N)`** toggle button.
- Clicking opens a dockable diagnostics panel listing all active errors, warnings, and informational notices.
- Clicking a diagnostic item automatically **highlights and centers** the offending block on the canvas.
- Clicking the **"Fix"** button next to an autofixable item applies the fix, updates the canvas, re-evaluates diagnostics, and enables **"Undo Fix"**.
