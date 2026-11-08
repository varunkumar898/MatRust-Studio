# Modified Nodal Analysis (MNA) Theory & Stamps

## 1. Mathematical Formulation
Modified Nodal Analysis formulates circuit equations by combining nodal voltages $\mathbf{v} \in \mathbb{R}^N$ with auxiliary branch currents $\mathbf{i}_b \in \mathbb{R}^M$ for components whose branch relations cannot be expressed purely in admittance form (voltage sources, inductors, ammeters).

The linear system is structured as:
$$\mathbf{A} \mathbf{x} = \mathbf{z} \implies \begin{bmatrix} \mathbf{G} & \mathbf{B} \\ \mathbf{C} & \mathbf{D} \end{bmatrix} \begin{bmatrix} \mathbf{v} \\ \mathbf{i}_b \end{bmatrix} = \begin{bmatrix} \mathbf{i}_s \\ \mathbf{v}_s \end{bmatrix}$$

- $N$: Number of non-ground nodes ($1 \le k \le N$). Node 0 represents the ground reference ($v_0 = 0$).
- $M$: Number of auxiliary current variables.

---

## 2. Component Stamps

### 2.1 Resistor ($R$)
Connected between positive node $p$ and negative node $n$. Conductance $G = \frac{1}{R}$.

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & +G & -G & 0 \\
\text{KCL}_n & -G & +G & 0
\end{array}$$

*(If node $p$ or $n$ is ground (0), the corresponding row and column are omitted.)*

---

### 2.2 Independent DC/AC Voltage Source ($V_s$)
Connected between positive node $p$ and negative node $n$, auxiliary branch current $i_v$ entering terminal $p$.
Branch equation: $v_p - v_n = V_s(t)$.

$$\begin{array}{c|ccc|c}
 & v_p & v_n & i_v & \text{RHS} \\ \hline
\text{KCL}_p & 0 & 0 & +1 & 0 \\
\text{KCL}_n & 0 & 0 & -1 & 0 \\
\text{Branch} & +1 & -1 & 0 & V_s(t)
\end{array}$$

---

### 2.3 Independent Current Source ($I_s$)
Connected from node $n$ to node $p$ (injecting current into node $p$, extracting from $n$).

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & 0 & 0 & +I_s(t) \\
\text{KCL}_n & 0 & 0 & -I_s(t)
\end{array}$$

---

### 2.4 Capacitor ($C$) Companion Model
Governing equation: $i_C(t) = C \frac{dv_C}{dt}$.

#### Backward Euler Discretization ($h$ = step size):
$$i_C[k] = C \frac{v_C[k] - v_C[k-1]}{h} = \frac{C}{h} v_C[k] - \frac{C}{h} v_C[k-1]$$
- Equivalent conductance: $G_{eq} = \frac{C}{h}$
- Equivalent history current: $I_{eq}[k] = \frac{C}{h} v_C[k-1]$ (injected into node $p$, extracted from $n$)

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & +G_{eq} & -G_{eq} & +I_{eq}[k] \\
\text{KCL}_n & -G_{eq} & +G_{eq} & -I_{eq}[k]
\end{array}$$

#### Trapezoidal Rule:
$$G_{eq} = \frac{2C}{h}, \quad I_{eq}[k] = \frac{2C}{h} v_C[k-1] + i_C[k-1]$$

---

### 2.5 Inductor ($L$) Companion Model
Governing equation: $v_L(t) = L \frac{di_L}{dt}$.

#### Backward Euler Discretization:
$$v_L[k] = L \frac{i_L[k] - i_L[k-1]}{h} \implies i_L[k] = \frac{h}{L} v_L[k] + i_L[k-1]$$
- Equivalent conductance: $G_{eq} = \frac{h}{L}$
- Equivalent history current: $I_{eq}[k] = i_L[k-1]$ (extracted from node $p$, injected into $n$)

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & +G_{eq} & -G_{eq} & -I_{eq}[k] \\
\text{KCL}_n & -G_{eq} & +G_{eq} & +I_{eq}[k]
\end{array}$$

#### Trapezoidal Rule:
$$G_{eq} = \frac{h}{2L}, \quad I_{eq}[k] = i_L[k-1] + \frac{h}{2L} v_L[k-1]$$

---

### 2.6 Ideal Current Sensor (`CurrentSensor`)
Modeled as an ideal zero-voltage source ($V = 0$) with auxiliary current variable $i_{sens}$:
$$v_p - v_n = 0$$

$$\begin{array}{c|ccc|c}
 & v_p & v_n & i_{sens} & \text{RHS} \\ \hline
\text{KCL}_p & 0 & 0 & +1 & 0 \\
\text{KCL}_n & 0 & 0 & -1 & 0 \\
\text{Branch} & +1 & -1 & 0 & 0.0
\end{array}$$

The measured current is read directly as $i_{sens}$ in the solution vector without inserting any series resistance.

---

### 2.7 Voltage Sensor (`VoltageSensor`)
Measures $v_p - v_n$. High impedance: injects zero current, adds no matrix rows.

---

### 2.8 Voltage-Controlled Voltage Source (`VCVS`)
Branch equation driven by control signal $u(t)$ with gain $K$:
$$v_p - v_n = K \cdot u(t)$$

$$\begin{array}{c|ccc|c}
 & v_p & v_n & i_{vcvs} & \text{RHS} \\ \hline
\text{KCL}_p & 0 & 0 & +1 & 0 \\
\text{KCL}_n & 0 & 0 & -1 & 0 \\
\text{Branch} & +1 & -1 & 0 & K \cdot u(t)
\end{array}$$

---

### 2.9 Voltage-Controlled Current Source (`VCCS`)
Injects branch current $i = G_m \cdot u(t)$ driven by control signal $u(t)$ with transconductance $G_m$:

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & 0 & 0 & +G_m \cdot u(t) \\
\text{KCL}_n & 0 & 0 & -G_m \cdot u(t)
\end{array}$$

---

### 2.10 Diode & Ideal Switch Companion Stamps
Modeled as a piecewise-linear two-state conductor:

$$\text{State} = \begin{cases} \text{ON}, & G = G_{on} = \frac{1}{R_{on}} \\ \text{OFF}, & G = G_{off} = \frac{1}{R_{off}} \end{cases}$$

For diodes, when ON, an offset current source $I_{offset} = \frac{V_f}{R_{on}}$ is added to the RHS to enforce the forward voltage drop $V_f$:

$$\begin{array}{c|cc|c}
 & v_p & v_n & \text{RHS} \\ \hline
\text{KCL}_p & +G & -G & -I_{offset} \\
\text{KCL}_n & -G & +G & +I_{offset}
\end{array}$$
*(When OFF or for IdealSwitch, $I_{offset} = 0$.)*

---

## 3. LU Factorization with Partial Pivoting
To solve $\mathbf{A} \mathbf{x} = \mathbf{z}$:
1. A permutation vector $\mathbf{P} = [0, 1, \dots, K-1]$ tracks row swaps.
2. In each column $j$, the pivot is chosen as:
   $$r = \arg\max_{i \ge j} |A_{i, j}|$$
3. If $|A_{r, j}| < \epsilon$ (where $\epsilon \approx 10^{-12}$), the matrix is singular, triggering `E-SINGULAR` and identifying the offending nodes.
4. Rows $j$ and $r$ are swapped in $\mathbf{A}$ and $\mathbf{P}$.
5. Multipliers $L_{i, j} = \frac{A_{i, j}}{A_{j, j}}$ are computed, and row operations are performed in place.
6. Permuted forward substitution $\mathbf{L} \mathbf{y} = \mathbf{P} \mathbf{z}$ followed by backward substitution $\mathbf{U} \mathbf{x} = \mathbf{y}$ computes $\mathbf{x}$.

---

## 4. Singularity and Degeneracy Conditions
- **`E-FLOAT`**: An isolated node or subnet has no DC conductive path to Ground (node 0), producing an all-zero row in $\mathbf{G}$.
- **`E-VLOOP`**: A loop of ideal voltage sources or closed ideal switches violates Kirchhoff's Voltage Law, producing linearly dependent rows in $\mathbf{C}$.
- **`E-ICUT`**: A cut-set of ideal current sources violating Kirchhoff's Current Law, producing inconsistent RHS currents.
- **`E-SINGULAR`**: General matrix rank deficiency detected during LU pivoting.
