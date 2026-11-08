# Simulation Engine Architecture and Design

## 1. Overview
The `mlapp-sim` simulation engine is a unified multi-domain simulation runtime supporting:
1. Continuous and discrete signal-flow block diagrams (Simulink-compatible semantics).
2. Physical electrical circuits via Modified Nodal Analysis (MNA).
3. Hybrid coupling across signal and physical electrical domains via ideal sensors and controlled sources.
4. Non-smooth switching networks (diodes and ideal switches) with deterministic within-step iteration and limit-cycle chatter protection.
5. High-performance zero-allocation execution paths using pre-factored switch topology tables.

---

## 2. Signal-Flow Execution Pipeline
The block diagram signal flow is modeled as a directed graph where nodes are blocks and edges are connection lines.

### 2.1 Graph Compilation & Topological Sorting
1. **Feedthrough Analysis**: Every block defines whether each output port directly depends on current input values ($y = f(u, t)$). Blocks with state (e.g., `Integrator`, `UnitDelay`, `DiscreteTransferFunction`, `ZOH`) do not have direct feedthrough ($y = f(x)$).
2. **Topological Order**: Tarjan's or Kahn's algorithm computes a strict topological ordering of blocks.
3. **Algebraic Loop Detection**: A directed cycle of blocks with direct feedthrough triggers diagnostic `E-ALG` (or `W-DELAY` in hybrid feedback) rather than hanging or diverging.
4. **State Sorting**: Continuous states $x$ and discrete states $x_d$ are indexed into contiguous preallocated arrays for cache-friendly execution.

---

## 3. Physical Electrical Simulation: Modified Nodal Analysis (MNA)

### 3.1 Netlist Extraction
- Electrical lines are converted into undirected graph edges.
- Connected component analysis partitions terminals into discrete electrical nets:
  - Net 0 is strictly reserved for `Ground`.
  - Non-ground nets $1, \dots, N$ form the node voltage vector $\mathbf{v} \in \mathbb{R}^N$.

### 3.2 System Equations
The linear circuit equations are expressed in standard MNA matrix form:
$$\begin{bmatrix} \mathbf{G} & \mathbf{B} \\ \mathbf{C} & \mathbf{D} \end{bmatrix} \begin{bmatrix} \mathbf{v} \\ \mathbf{i}_b \end{bmatrix} = \begin{bmatrix} \mathbf{i}_s \\ \mathbf{v}_s \end{bmatrix}$$
where:
- $\mathbf{G} \in \mathbb{R}^{N \times N}$ represents conductance between non-ground nodes.
- $\mathbf{B} \in \mathbb{R}^{N \times M}$ and $\mathbf{C} \in \mathbb{R}^{M \times N}$ couple branch currents of independent voltage sources, controlled voltage sources, ideal switches, and current sensors.
- $\mathbf{D} \in \mathbb{R}^{M \times M}$ contains zero or constitutive terms.
- $\mathbf{i}_b \in \mathbb{R}^M$ contains unknown auxiliary branch currents.
- The total system size is $K = N + M$.

### 3.3 Dynamic Companion Models
Reactive components ($C, L$) are discretized using numerical integration rules into equivalent resistive companion models:
- **Capacitor ($C$)**:
  - Backward Euler: Conductance $G_{eq} = \frac{C}{h}$, history current $I_{eq}[k] = \frac{C}{h} v_C[k-1]$.
  - Trapezoidal: Conductance $G_{eq} = \frac{2C}{h}$, history current $I_{eq}[k] = \frac{2C}{h} v_C[k-1] + i_C[k-1]$.
- **Inductor ($L$)**:
  - Backward Euler: Conductance $G_{eq} = \frac{h}{L}$, history current $I_{eq}[k] = i_L[k-1]$.
  - Trapezoidal: Conductance $G_{eq} = \frac{h}{2L}$, history current $I_{eq}[k] = i_L[k-1] + \frac{h}{2L} v_L[k-1]$.

---

## 4. Hybrid Cross-Domain Coupling

### 4.1 Sensors (Electrical $\to$ Signal)
- `VoltageSensor`: Measures differential voltage between terminals $p$ and $n$ ($V = v_p - v_n$). Injects zero current into the circuit (infinite input impedance).
- `CurrentSensor`: Measures branch current flowing through its terminals. Modeled in MNA as an ideal 0V voltage source:
  $$v_p - v_n = 0$$
  The auxiliary branch current $i_b$ is read directly without affecting circuit voltages or currents.

### 4.2 Controlled Sources (Signal $\to$ Electrical)
- `VoltageControlledVoltageSource` (VCVS): Sets branch voltage $v_p - v_n = \text{Gain} \cdot u(t)$ driven by incoming control signal $u(t)$.
- `VoltageControlledCurrentSource` (VCCS): Injects branch current $i = \text{Transconductance} \cdot u(t)$ into terminal $p$ and extracts from $n$.

### 4.3 Feedback & Algebraic Delay (`W-DELAY`)
When a sensor directly drives a controlled source without an intervening state block (`Integrator`, `UnitDelay`, etc.), a direct cross-domain algebraic loop is formed. The engine detects this condition, emits diagnostic `W-DELAY`, and inserts a deterministic 1-step latency ($u[k] = u[k-1]$) allowing smooth execution without divergence.

---

## 5. Non-Smooth Switching & Within-Step Iteration

### 5.1 Switch & Diode Models
- **Diode**:
  - State: ON if $v_D > V_f$; OFF if $i_D < 0$ (strict inequality).
  - Stamp: Conductance $G = \frac{1}{R_{on}}$ with offset current source $I_{offset} = \frac{V_f}{R_{on}}$ when ON; conductance $G = \frac{1}{R_{off}}$ when OFF ($R_{off} < \infty$).
- **Ideal Switch**:
  - State: ON if control signal $u(t) \ge \text{Threshold}$; OFF if $u(t) < \text{Threshold}$.
  - Stamp: Conductance $G = \frac{1}{R_{on}}$ when ON; conductance $G = \frac{1}{R_{off}}$ when OFF.

### 5.2 Within-Step Iteration & Chatter Policy
1. At each time step $t_k$, switch states are evaluated.
2. If any diode or switch changes state, the MNA matrix is updated and solved again at the same time step.
3. **Convergence**: If the state sequence repeats identically ($A \to A$), the step is converged.
4. **Chatter Detection (`W-CHATTER`)**: If states cycle with length $\ge 2$ ($A \to B \to A$), a limit cycle is detected. The engine terminates iteration, retains the previous step's switch state, emits warning `W-CHATTER` with switch SIDs and timestamp, and proceeds.
5. **Chatter Escalation (`E-CHATTER`)**: If `W-CHATTER` persists for more than 100 consecutive steps, the simulation halts with error `E-CHATTER`.

---

## 6. Pre-Factorization Fast-Path (Zero Heap Allocations)
For circuits with $N_s \le 10$ switching devices:
1. Total combinations $2^{N_s} \le 1024$.
2. During initialization, the engine precomputes LU factorizations $\mathbf{P} \mathbf{A}_s = \mathbf{L}_s \mathbf{U}_s$ for all $2^{N_s}$ configurations into a fixed cache.
3. During time stepping, solving the network requires only forward and backward triangular substitution ($\mathcal{O}(K^2)$ arithmetic), with **zero heap allocations** (`Vec::new`, `Box`, etc.).
