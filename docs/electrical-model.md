# Electrical Data Model & Editor Integration (Milestone M4)

## 1. Overview and Architecture

Milestone M4 introduces the multi-domain electrical circuit data model and editor integration to RustLab / MatRust Studio (`crates/mlapp-sim` and `apps/desktop`).

The electrical data model provides:
1. **Multi-Domain Ports**: Block ports are explicitly categorized by domain into `PortKind::Signal` and `PortKind::Electrical`, with roles `PortRole::Input`, `PortRole::Output`, and `PortRole::Terminal`.
2. **Rotatable Blocks**: `BlockRotation` (`R0`, `R90`, `R180`, `R270`) allows blocks to be rotated clockwise in 90-degree increments, dynamically updating terminal and pin coordinates via `pin_geometry`.
3. **Physical Electrical Components**: 9 fundamental circuit blocks (Resistor, Capacitor, Inductor, DC Voltage Source, AC Voltage Source, DC Current Source, Ground, Diode, Ideal Switch) with textbook MNA parameter specifications and validation.
4. **Disjoint-Set Netlist Resolution**: Compile-time union-find algorithm resolving electrical terminals into deterministic, undirected voltage nodes (nets).
5. **Pre-M5 Simulation Guard**: Complete rejection of electrical simulation prior to solver support in Milestone M5 via typed diagnostic `E-UNSUPPORTED`.
6. **No Heap Allocation at Runtime**: Pre-run union-find compilation ensures the per-step simulation path remains 100% allocation-free.

---

## 2. Port and Rotation Specification

### 2.1 Port Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortKind {
    Signal,
    Electrical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortRole {
    Input,
    Output,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDesc {
    pub kind: PortKind,
    pub role: PortRole,
    pub label: String,
    pub index: usize,
}
```

- **Signal Ports**: Directed connections (`Input` or `Output`). Signal inputs require exactly one driver. In the editor, signal ports are rendered as circular or triangular glyphs.
- **Electrical Terminals**: Undirected physical pins (`Terminal`). Electrical terminals support multiple connected wires (parallel branches). In the editor, physical terminals are rendered as open squares.

### 2.2 Rotation

`BlockRotation` represents clockwise rotation in 90-degree increments:
- `R0`: 0° (default)
- `R90`: 90° clockwise
- `R180`: 180° clockwise
- `R270`: 270° clockwise

Pin geometry `(u, v) ∈ [0, 1]²` rotates around the block unit center `(0.5, 0.5)` according to:
$$\begin{aligned}
R_0 &: (u, v) \mapsto (u, v) \\
R_{90} &: (u, v) \mapsto (1 - v, u) \\
R_{180} &: (u, v) \mapsto (1 - u, 1 - v) \\
R_{270} &: (u, v) \mapsto (v, 1 - u)
\end{aligned}$$

---

## 3. Supported Electrical Components & Parameters

All electrical blocks are specified in `crates/mlapp-sim/src/spec.rs` under category `"Electrical"` and validated according to physical constraints (`E-PARAM`):

| Block Type | Symbol | Terminals & Ports | Parameters | Validation Rules |
| :--- | :--- | :--- | :--- | :--- |
| **Resistor** | `-[R]-` | Term 0 (`+`), Term 1 (`-`) | `Resistance` (default `1000.0` $\Omega$) | $R > 0$, finite |
| **Capacitor** | `-||-` | Term 0 (`+`), Term 1 (`-`) | `Capacitance` (`1e-6` F), `InitialVoltage` (`0.0` V) | $C > 0$, $V_0$ finite |
| **Inductor** | `-mmm-` | Term 0 (`+`), Term 1 (`-`) | `Inductance` (`1e-3` H), `InitialCurrent` (`0.0` A) | $L > 0$, $I_0$ finite |
| **DC Voltage Source** | `(V)` | Term 0 (`+`), Term 1 (`-`) | `Voltage` (`10.0` V) | $V$ finite |
| **AC Voltage Source** | `(~V)` | Term 0 (`+`), Term 1 (`-`) | `Amplitude` (`120.0` V), `Frequency` (`60.0` Hz), `Phase` (`0.0` rad), `Bias` (`0.0` V) | Amp $\ge 0$, Freq $> 0$, Phase & Bias finite |
| **DC Current Source** | `(I)` | Term 0 (`+`), Term 1 (`-`) | `Current` (`1.0` A) | $I$ finite |
| **Ground** | `[GND]` | Term 0 (`gnd`) | None | Constant 0V reference node |
| **Diode** | `->|-` | Term 0 (`+`), Term 1 (`-`) | `ForwardVoltage` (`0.7` V), `Ron` (`0.01` $\Omega$), `Roff` (`1e6` $\Omega$) | $V_f \ge 0$, $R_{on} > 0$, $R_{off} > R_{on}$ |
| **Ideal Switch** | `-/ -` | Term 0 (`p`), Term 1 (`n`), Input 0 (`ctrl`) | `Threshold` (`0.5`), `Ron` (`0.001` $\Omega$), `Roff` (`1e6` $\Omega$) | $R_{on} > 0$, $R_{off} > R_{on}$, Threshold finite |

---

## 4. Netlist Resolution Algorithm

The netlist compiler (`crates/mlapp-sim/src/netlist.rs`) executes in $O(T \cdot \alpha(T))$ time at compile time, resolving electrical lines into connected nodes:

1. **Parameter Validation**: Validate all parameters against physical bounds via `validate_block_params`.
2. **Terminal Inventory**: Map each electrical terminal `(block_sid, term_idx)` to a contiguous index.
3. **Disjoint-Set Union-Find on Terminals**:
   - Each electrical wire unites `terminal_uf.union(src_term, dst_term)`.
   - Direction is ignored (undirected connection).
   - Duplicate wires and self-loop connections emit warnings (`W-WIRE`) and do not abort compilation.
   - Cross-domain direct connections between `Signal` and `Electrical` emit error `E-DOMAIN`.
4. **Ground Merging (Reference Node 0)**:
   - All Ground block terminals are united into a single global reference node.
   - The equivalence class containing Ground is assigned **Net ID 0**.
5. **Deterministic Net Numbering**:
   - Non-ground nets are assigned sequential positive integers ($1, 2, 3, \dots$) in order of canonical appearance (iterating blocks in diagram block order, then terminals in terminal order).
   - Net assignment is completely invariant to the order in which lines appear in the file.
6. **Island Checking (`E-GND`)**:
   - Disjoint-set union-find on electrical blocks groups connected components of blocks into subcircuits (islands).
   - Any connected electrical island lacking a Ground block emits `E-GND` naming the disconnected block SIDs.
   - Unconnected single blocks emit `W-OPEN` warnings without blocking execution.

---

## 5. Diagnostics Catalog (M4)

All diagnostics follow the standard message format:
`what happened -> why -> how to fix`

| Code | Severity | Description | Fix Action |
| :--- | :--- | :--- | :--- |
| **`E-DOMAIN`** | Error | Direct wire connection between `PortKind::Signal` and `PortKind::Electrical`. | Remove direct wire; interface domains via sensor or controlled source. |
| **`E-GND`** | Error | Electrical island containing components has no Ground reference. | Connect at least one terminal in the island to a Ground block. |
| **`E-PARAM`** | Error | Out-of-bounds or invalid component parameter value (e.g. $R \le 0$). | Set parameter within valid physical bounds. |
| **`E-UNSUPPORTED`** | Error | Simulation attempted on diagram containing electrical components before M5. | Remove electrical blocks or wait for Milestone M5 solver. |
| **`W-OPEN`** | Warning | Unconnected electrical terminal has 0 attached wires. | Connect terminal or remove unused block. |
| **`W-WIRE`** | Warning | Duplicate wire connecting the same terminal pair, or self-loop on a terminal. | Remove redundant wire. |

---

## 6. Island Semantics & MNA Matrix Partitioning

Subcircuits or topological components that are physically disconnected except through one or more Ground blocks share the global reference potential Node 0 (0 V). Because Node 0 is eliminated from the Modified Nodal Analysis (MNA) equations as the datum node, nodes from distinct subcircuits have zero mutual conductances, capacitances, or inductances ($G_{ij} = 0, C_{ij} = 0$ for node $i$ in island $A$ and node $j$ in island $B$). Consequently, in Milestone M5, subcircuits that connect only through Ground form independent diagonal blocks in the overall MNA system matrix $[G + \frac{C}{dt}]$. The solver engine can either simulate the composite system as a single uncoupled block-diagonal sparse matrix or partition the diagram into independent MNA linear systems solved in parallel, guaranteeing zero numerical cross-talk between isolated subcircuits.

---

## 7. Known Limitation: No Galvanic Isolation

All Ground blocks in a diagram are merged into a single reference node (Net 0). The simulator does not currently support separate reference nodes or galvanic isolation (e.g. isolated transformer primaries and secondaries). All subcircuits must share the common global ground.

---

## 8. Zero-Allocation Guarantee

The disjoint-set netlist resolution and validation algorithm runs strictly once prior to simulation. The per-step simulation path (`eval` and `update_discrete`) performs zero heap allocations.

---

## 9. Known Limitations
Switching instants are resolved to the step size (O(h) error); the thyristor and event localization are not implemented. (Deferred to Milestone M6b).
