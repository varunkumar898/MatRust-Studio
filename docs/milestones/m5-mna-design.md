# Milestone M5 Design Document: Modified Nodal Analysis (MNA) & Linear Circuits

## 1. Objective and Scope

Milestone M5 implements linear electrical circuit simulation within RustLab / MatRust Studio (`crates/mlapp-sim`) using Modified Nodal Analysis (MNA) and companion models.

### Scope IN
- **MNA Matrix Assembly**: Formulates $[A] x(t) = z(t)$ using Ho, Ruehli, Brennan (1975) and Vlach & Singhal (1983).
- **Linear Components**:
  - Resistor ($G = 1/R$)
  - Capacitor (Trapezoidal companion model with Backward Euler step 1 initialization)
  - Inductor (Trapezoidal companion model with Backward Euler step 1 initialization)
  - DC Voltage Source (auxiliary branch current, constant voltage $V$)
  - AC Voltage Source ($v(t) = V_{amp} \sin(2\pi f t + \phi) + V_{bias}$)
  - DC Current Source (independent current injection $I_s$)
  - Ground (Node 0 datum, eliminated from matrix rows/cols)
- **Dense Self-Contained LU with Partial Pivoting**:
  - Allocation-free $PA = LU$ factorization with row permutation.
  - Zero heap allocation during forward/backward substitution in the step loop.
- **Constant-Matrix Fast Path**:
  - For linear time-invariant circuits with fixed step $h$, matrix $[A]$ is factorized once at initialization (`lu_be` for step 1, `lu_trap` for subsequent steps).
  - Time-step execution only forms dynamic RHS $z(t)$ and solves in $O(M^2)$ time.
- **Diagnostics**:
  - `E-SINGULAR`: Zero pivot during LU factorization with clear explanation and remediation steps.
  - `E-SOLVER`: Variable-step solver rejection on electrical diagrams.
  - `E-UNSUPPORTED`: Graceful rejection of nonlinear blocks (Diode, Switch) until M6.

### Scope OUT
- Nonlinear and switching devices (Diode, Ideal Switch, Thyristor) -> Milestone M6.
- Sensors and bridge blocks (Voltage Sensor, Current Sensor, Controlled Sources) -> Milestone M7.

---

## 2. Mathematical Formulation

### 2.1 System Equations
For a circuit with $N$ non-ground nodes and $M_{vsrc}$ independent voltage sources:
$$\begin{bmatrix} G & B \\ B^T & D \end{bmatrix} \begin{bmatrix} v(t) \\ i_v(t) \end{bmatrix} = \begin{bmatrix} j(t) \\ e(t) \end{bmatrix}$$
where $M = N + M_{vsrc}$.

### 2.2 Companion Models (Dommel 1969)
For fixed step $h$:
- **Capacitor**:
  - Backward Euler (Step 1): $G_{C} = \frac{C}{h}$, $I_{eq} = -\frac{C}{h} V_0$
  - Trapezoidal (Step $\ge 2$): $G_{C} = \frac{2C}{h}$, $I_{eq}[k] = -\frac{2C}{h} v_C[k-1] - i_C[k-1]$
- **Inductor**:
  - Backward Euler (Step 1): $G_{L} = \frac{h}{L}$, $I_{eq} = I_0$
  - Trapezoidal (Step $\ge 2$): $G_{L} = \frac{h}{2L}$, $I_{eq}[k] = i_L[k-1] + \frac{h}{2L} v_L[k-1]$

---

## 3. Allowed References
- Dommel, H. W. (1969). "Digital Computer Solution of Electromagnetic Transients in Single-and Multiphase Networks." *IEEE Trans. Power App. Syst.*
- Ho, C.-W., Ruehli, A. E., & Brennan, P. A. (1975). "The Modified Nodal Approach to Network Analysis." *IEEE Trans. Circuits Syst.*
- Vlach, J., & Singhal, K. (1983). *Computer Methods for Circuit Analysis and Design*.
