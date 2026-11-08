# Milestone M6 Design: Piecewise-Linear & Switching Circuits (Diode & Ideal Switch)

## 1. Overview & Clean-Room Architecture
Milestone M6 extends the Modified Nodal Analysis (MNA) linear circuit solver implemented in Milestone M5 to support nonlinear piecewise-linear and switching elements:
- **Diode**: Two-terminal semiconductor element with forward voltage drop $V_f$, on-conductance $G_{on} = 1/R_{on}$, and off-conductance $G_{off} = 1/R_{off}$.
- **Ideal Switch**: Three-terminal element (positive terminal $p$, negative terminal $n$, and control signal input $ctrl$) with state determined by $u_{ctrl} \ge \text{Threshold}$.

Clean-room references:
- Ho, C.-W., Ruehli, A. E., & Brennan, P. A. (1975). *The modified nodal approach to network analysis*. IEEE Transactions on Circuits and Systems.
- Dommel, H. W. (1969). *Digital computer solution of electromagnetic transients in single-and multiphase networks*. IEEE Trans. Power App. Syst.
- Vlach, J., & Singhal, K. (1983). *Computer Methods for Circuit Analysis and Design*. Van Nostrand Reinhold.
- Levis, M., & Pejovic, P. (1995). *Simulation of power electronic circuits using piecewise linear models*. IEEE Trans. Power Electronics.

---

## 2. Companion Models & State Transitions

### 2.1 Diode Piecewise-Linear Companion Model
A diode connected between anode $p$ and cathode $n$ is parameterized by:
- $V_f \ge 0$: Forward voltage drop (V).
- $R_{on} > 0$: On-resistance ($\Omega$). Conductance $G_{on} = 1/R_{on}$.
- $R_{off} > R_{on}$: Off-resistance ($\Omega$). Conductance $G_{off} = 1/R_{off}$.

Companion stamp:
- **ON state** ($s = 1$):
  - Conductance $G_{on}$ stamped between nodes $(p, n)$.
  - Independent current source $I_{eq} = G_{on} V_f$ in parallel with $G_{on}$, directed from $p$ to $n$.
  - RHS contributions: $b[p] \mathrel{-}= I_{eq}$, $b[n] \mathrel{+}= I_{eq}$.
  - Branch relation: $i_d = G_{on}(v_p - v_n - V_f)$.
- **OFF state** ($s = 0$):
  - Conductance $G_{off}$ stamped between nodes $(p, n)$.
  - Independent current source $I_{eq} = 0$.
  - Branch relation: $i_d = G_{off}(v_p - v_n)$.

**State Transition Rules**:
- From $\text{OFF} \to \text{ON}$: occurs if $v_d = v_p - v_n \ge V_f$.
- From $\text{ON} \to \text{OFF}$: occurs if $i_d \le 0$.

### 2.2 Ideal Switch Companion Model
An ideal switch between terminals $p$ and $n$ with control signal input $u_{ctrl}$:
- If $u_{ctrl} \ge \text{Threshold}$: State is $\text{ON}$ ($G = G_{on}$).
- If $u_{ctrl} < \text{Threshold}$: State is $\text{OFF}$ ($G = G_{off}$).

---

## 3. Within-Step Fixed-Point Iteration & Chattering Guard

At each time step $t$:
1. Determine ideal switch states directly from control input signals.
2. Initialize diode states from previous step (or initial condition operating point).
3. **Inner Iteration Loop** (up to $K_{max} = 100$ iterations):
   - Assemble or select MNA matrix $A(s)$ for current switch configuration vector $s$.
   - Assemble RHS vector $b(s)$ including companion history currents ($C, L$), independent sources, and diode $I_{eq}$.
   - Solve $A(s) x = b(s)$ via LU decomposition.
   - Evaluate branch voltage $v_d$ and current $i_d$ for each diode.
   - Determine tentative next state $s_{next}$.
   - If $s_{next} == s$: **Converged!** Break loop.
   - If $s$ was already visited in this time step (limit cycle / chattering detected), or if $k == K_{max}$:
     Terminate with diagnostic error `E-CHATTER` detailing the participating blocks and cycling state sequence.
   - Otherwise, update $s \leftarrow s_{next}$ and continue iteration.
4. Advance dynamic companion states ($C, L$) with the converged solution.

---

## 4. Constant-Matrix Fast Path & Zero Heap Allocations
- Pre-factorization table size formula for $N_s$ switches and system size $N$ across 3 integration operators (Initial DC operating point, Backward Euler, Trapezoidal):
  $$\text{Memory} = 3 \times 2^{N_s} \times (N^2 + N) \times 8 \text{ bytes}$$
- Table memory footprints for typical dimensions:
  - $N_s = 1$: $3 \times 2^1 \times (100 + 10) \times 8 = 5,280 \text{ B } (\approx 5.15 \text{ KB for } N = 10)$
  - $N_s = 4$: $3 \times 2^4 \times (100 + 10) \times 8 = 42,240 \text{ B } (\approx 41.25 \text{ KB for } N = 10)$
  - $N_s = 8$: $3 \times 2^8 \times (100 + 10) \times 8 = 675,840 \text{ B } (\approx 660 \text{ KB for } N = 10)$
  - $N_s = 10$: $3 \times 2^{10} \times (100 + 10) \times 8 = 2,703,360 \text{ B } (\approx 2.58 \text{ MB for } N = 10; \approx 62.7 \text{ MB for } N = 50)$
- **Fallback Rule**: When $N_s > 10$ or table memory exceeds 64 MB ($67,108,864$ bytes), the compiler emits informational diagnostic `I-SLOW` and transitions to an in-place scratch LU decomposition path ($O(N^3)$), allocating zero heap memory during transient stepping.
- Signal logging exposes `{name}.v`, `{name}.i`, and `{name}.state` (1 for ON, 0 for OFF).

---

## 5. Known Limitations
Switching instants are resolved to the step size (O(h) error); the thyristor and event localization are not implemented. (Deferred to Milestone M6b).

---

## 6. Analytical Oracle Accuracy & Discretization Convergence Table

Measurements executed in `test_m6_accuracy_convergence_table`:

| Model / System | Solver / Method | $h$ | $h/2$ | $E(h)$ | $E(h/2)$ | Ratio $\frac{E(h)}{E(h/2)}$ | Order |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Continuous Integrator** ($\dot{x} = -x$) | FixedStepEuler | $1.0\text{ ms}$ | $0.5\text{ ms}$ | $2.379630 \times 10^{-5}$ | $1.189426 \times 10^{-5}$ | **2.001** | $O(h)$ |
| **MNA Companion Model** (RC Discharge) | Trapezoidal (in Euler) | $1.0\text{ ms}$ | $0.5\text{ ms}$ | $2.764979 \times 10^{-4}$ | $6.931008 \times 10^{-5}$ | **3.989** | $O(h^2)$ |
| **Diode Commutation Event** (Rectifier) | Switching event | $100\ \mu\text{s}$ | $50\ \mu\text{s}$ | $1.677449 \times 10^{-2}$ | $1.948952 \times 10^{-2}$ | **0.861** | Discontinuous boundary |

*Note on Switching Events*: During smooth intervals, Trapezoidal companion models exhibit exact quadratic convergence ($E(h)/E(h/2) \approx 4.0$). When a switching event occurs within a step, discrete step boundary resolution introduces an $O(h)$ switching latency error, degrading convergence across the event until event localization (M6b) is enabled.
