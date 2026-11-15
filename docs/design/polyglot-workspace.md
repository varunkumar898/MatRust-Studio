# Polyglot Workspace Architecture & Engine Integration (docs/design/polyglot-workspace.md)

**Phase:** PLAN-POLY / F18  
**Scope:** Universal multi-engine interface, high-throughput binary variable exchange, sandboxed subprocess security, and interactive notebook cell execution across native RustLab, Python, Julia, GNU Octave, and MATLAB.

---

## 1. Architectural Overview

The MatRust Studio polyglot architecture provides unified code execution and data sharing across multiple scientific language runtimes while enforcing strict process isolation and security boundaries.

```
+-----------------------------------------------------------------------+
|                         MatRust Desktop UI                            |
|  [Notebook Cell Editor]   [Figures Panel]   [Engine Status Indicator] |
+-----------------------------------------------------------------------+
                                   |
                                   v
+-----------------------------------------------------------------------+
|                     Workspace Engine Manager                          |
|             (Cell Dispatcher, Security & Lifecycle)                   |
+-----------------------------------------------------------------------+
                                   |
         +-------------------------+-------------------------+
         |                         |                         |
         v                         v                         v
+------------------+      +------------------+      +------------------+
|  Native Engine   |      |  Octave Bridge   |      |   Julia Bridge   |
| (rustlab-eval /  |      |  (mlapp-octave)  |      |  (mlapp-julia)   |
|   stdlib / sim)  |      |                  |      |                  |
+------------------+      +------------------+      +------------------+
         |                         |                         |
         +-------------------------+-------------------------+
                                   |
                                   v
+-----------------------------------------------------------------------+
|               Binary Column-Major Variable Exchange Bus               |
|      (Control IPC < 64 KiB | Shared Memory / Mmap Raw Buffers)        |
+-----------------------------------------------------------------------+
```

---

## 2. Universal Engine Interface (`crates/mlapp-engine`)

The engine abstraction resides in a dedicated headless crate (`crates/mlapp-engine` or `crates/mlapp-core`) with zero GUI dependencies, ensuring that CLI runners, headless test harnesses, and GUI desktop instances share an identical execution interface.

### 2.1 Rust Trait Definition

```rust
use std::collections::HashMap;
use std::time::Duration;

pub type CellId = usize;
pub type FigureId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineId {
    Native,
    Octave,
    Matlab,
    Python,
    Julia,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EngineCapabilities: u32 {
        const EVAL_EXPRESSION      = 1 << 0;
        const EVAL_CELL            = 1 << 1;
        const GET_SET_SCALARS      = 1 << 2;
        const GET_SET_RAW_BUFFER   = 1 << 3;
        const CAPTURED_FIGURES     = 1 << 4;
        const INTERRUPT_SIGNAL     = 1 << 5;
        const LINE_ERROR_MAPPING   = 1 << 6;
        const ASYNC_OUTPUT_STREAM  = 1 << 7;
    }
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub executable_path: Option<std::path::PathBuf>,
    pub working_directory: std::path::PathBuf,
    pub execution_timeout: Duration,
    pub max_memory_mb: usize,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct VariableMeta {
    pub name: String,
    pub type_name: String,
    pub dimensions: Vec<usize>,
    pub size_in_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct EvalOutput {
    pub stdout: String,
    pub stderr: String,
    pub error: Option<ExecutionError>,
    pub execution_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub message: String,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub backtrace: Vec<String>,
}

pub trait ExecutionEngine: Send {
    /// Return engine identifier.
    fn id(&self) -> EngineId;

    /// Return capability bitmask.
    fn capabilities(&self) -> EngineCapabilities;

    /// Start or spawn the engine process.
    fn start(&mut self, config: &EngineConfig) -> Result<(), EngineError>;

    /// Terminate the engine process gracefully or forcefully.
    fn stop(&mut self) -> Result<(), EngineError>;

    /// Check if engine is running and responsive.
    fn is_running(&self) -> bool;

    /// Evaluate a block of code, optionally tagged with a cell identifier.
    fn eval(&mut self, code: &str, cell_id: Option<CellId>) -> Result<EvalOutput, EngineError>;

    /// Retrieve a variable from the engine environment.
    fn get_var(&mut self, name: &str) -> Result<Option<Value>, EngineError>;

    /// Push a variable into the engine environment.
    fn set_var(&mut self, name: &str, value: &Value) -> Result<(), EngineError>;

    /// Enumerate all active workspace variables.
    fn list_vars(&mut self) -> Result<Vec<VariableMeta>, EngineError>;

    /// Send an asynchronous interrupt signal (e.g., SIGINT / Ctrl+C) to cancel active evaluation.
    fn interrupt(&mut self) -> Result<(), EngineError>;

    /// Retrieve and drain figures captured during evaluation.
    fn captured_figures(&mut self) -> Result<Vec<CapturedFigure>, EngineError>;
}
```

### 2.2 Capability Checking

When a caller invokes a method not supported by an engine (for example, attempting `captured_figures` on a backend lacking plot interception), the engine must immediately return:
```rust
EngineError::UnsupportedCapability {
    engine: EngineId,
    required: EngineCapabilities,
}
```
This guarantees deterministic error handling without runtime panics.

---

## 3. Variable Exchange Protocol

### 3.1 Memory Layout: Column-Major Preservation
MATLAB, GNU Octave, Julia, and RustLab's numerical engine all represent multidimensional arrays in **column-major (Fortran order)** layout. To achieve maximum throughput:
- All array buffers transferred across the polyglot bus are transmitted strictly in column-major order.
- Transposition passes ($O(N)$ memory and time overhead) are completely eliminated.
- Python arrays (NumPy) are ingested or emitted using Fortran-contiguous buffers (`np.asfortranarray`).

### 3.2 Supported Data Types

| RustLab `Value` Variant | Rust Representation | C / Binary Equivalent | Engine Mappings (MATLAB / Octave / Julia / Python) |
| :--- | :--- | :--- | :--- |
| `Double` | `f64` / `Vec<f64>` | `double` (IEEE 754) | `double` / `Float64` / `np.float64` |
| `Single` | `f32` / `Vec<f32>` | `float` (IEEE 754) | `single` / `Float32` / `np.float32` |
| `Int8` .. `Int64` | `i8` .. `i64` | Signed two's complement | `int8`..`int64` / `Int8`..`Int64` / `np.int8`..`int64` |
| `UInt8` .. `UInt64` | `u8` .. `u64` | Unsigned binary | `uint8`..`uint64` / `UInt8`..`UInt64` / `np.uint8`..`uint64` |
| `ComplexDouble` | `Complex<f64>` | Pair of `f64` [real, imag] | `complex double` / `ComplexF64` / `np.complex128` |
| `ComplexSingle` | `Complex<f32>` | Pair of `f32` [real, imag] | `complex single` / `ComplexF32` / `np.complex64` |
| `Logical` | `bool` / `Vec<u8>` | Byte array (0 = false, 1 = true) | `logical` / `Bool` / `np.bool_` |
| `String` | `String` | UTF-8 byte stream | `string` / `char` / `String` / `str` |
| `DenseArray<T>` | Strided N-D array | Strided column-major blob | N-D Array / `Array{T, N}` / `ndarray(order='F')` |
| `Struct` | `HashMap<String, Value>` | Key-value record map | `struct` / `Dict{String, Any}` / `dict` |
| `CellArray` | `Vec<Value>` + dims | Heterogeneous pointer array | `cell` / `Vector{Any}` / `list` |
| `Table` | Deferred to F14.04 | Schema + column vectors | Deferred to F14.04 |

### 3.3 Dual-Tier Transport Mechanism

```
Payload Size < 64 KiB:
+-------------------------------------------------------+
| Length-Prefixed Binary Frame on Control Pipe          |
| [Magic: 4B][Type: 2B][Dims: 2B * rank][Payload: ... ]  |
+-------------------------------------------------------+

Payload Size >= 64 KiB:
+-------------------------------------------------------+
| Control Pipe: Descriptor Message                      |
| [Magic: 4B][BufferId: 16B][TotalBytes: 8B][Path/Handle] |
+-------------------------------------------------------+
                            |
                            v
+-------------------------------------------------------+
| Raw Little-Endian Binary Buffer (Mmap / Temp File)    |
| Zero-copy memory mapping directly into engine array   |
+-------------------------------------------------------+
```

1. **Small Values (< 64 KiB)**:
   - Serialized directly over the standard bidirectional IPC pipe (or loopback socket).
   - Minimal framing overhead (< 32 bytes).
2. **Large Arrays ($\ge 64$ KiB)**:
   - Transmitted as raw little-endian binary buffers through a memory-mapped temporary file or anonymous shared memory pipe.
   - Guarded by a strict configurable memory ceiling:
     - Default maximum single array transfer: **512 MiB**.
     - Workspace maximum total buffer allocation: **2 GiB**.
   - Violations reject the transfer with `EngineError::TransferLimitExceeded { requested_bytes, limit_bytes }`.

### 3.4 Round-Trip Verification Suite
Every supported data type is validated by an automated bidirectional round-trip test:
$$\text{RustLab } V \longrightarrow \text{External Engine } E \longrightarrow \text{RustLab } V'$$
Acceptance criteria:
- Integers, logicals, and strings: exact bitwise identity ($V' == V$).
- Floating point values: $\le 1$ ULP (unit in the last place) error.
- Dense array shape, dimensions, and strides: strictly identical.

---

## 4. Security Model & Subprocess Isolation

External interpreters (Julia, Python, Octave, MATLAB) execute user-supplied code that can perform arbitrary OS system calls, disk I/O, or network operations. MatRust Studio enforces a defense-in-depth isolation posture:

### 4.1 Opt-In Activation
- External engines are **disabled by default**.
- Opening an `.mlapp`, `.m`, `.jl`, or `.py` file **never automatically launches** an external subprocess.
- Engines must be explicitly enabled per workspace via configuration flags:
  `--allow-julia`, `--allow-octave`, `--allow-python`, `--allow-matlab`.

### 4.2 Call Safety & Command Sanitization
- Reuses and extends the safety verification rules established in `crates/mlapp-octave/src/safety.rs`:
  - **Blocked Operations**: shell execution (`system`, `evalc`, `!`, `dos`, `unix`), arbitrary process spawning, dynamic library injection, direct disk writes outside workspace boundaries.
  - Interactive prompts that block indefinitely are detected and rejected.

### 4.3 Process Supervision & Hard Timeouts
- All engine subprocesses are launched as supervised child processes with standard input/output pipes monitored by heartbeat threads.
- Every evaluation request carries a deadline (default: 30 seconds, configurable in workspace preferences).
- If an evaluation exceeds its deadline:
  1. An asynchronous interrupt (`SIGINT` or `Ctrl+C` console event) is dispatched.
  2. If the engine remains unresponsive for an additional 2.0 seconds, the child process is terminated (`SIGKILL` / `TerminateProcess`) and clean-up handlers are executed.

### 4.4 Loopback-Only Socket Security
- When an engine bridge uses TCP sockets for IPC (e.g., Python or Julia workers):
  - The socket binds strictly to `127.0.0.1` on an ephemeral OS-assigned port.
  - The server generates a cryptographically secure 256-bit random token on startup.
  - The child process must submit the matching bearer token in its initial handshake within 5.0 seconds; otherwise, the connection is instantly severed.

### 4.5 Path Privacy & Secret Sanitization
- Child engine stderr outputs, panic backtraces, and exception traces are filtered through a sanitization filter:
  - Personal user directory paths and system-specific paths are automatically replaced with `<user-home>`.
  - Repository root paths are replaced with `<repo>`.
  - No secret tokens, environment variables, or private paths appear in logs or user dialogs.

---

## 5. Cell Execution Model & Desktop UI Integration

### 5.1 Notebook Cell Delimiters
The editor parser partitions multi-language scripts into executable cells using standard line-based delimiters:
- **MATLAB / Octave style**: Lines starting with `%% [Optional Cell Title]`
- **Julia / Python style**: Lines starting with `# %% [Optional Cell Title]`

### 5.2 Per-Cell Engine Selector
Cells can declare an explicit engine override via metadata tags:
```matlab
%% [engine=native] Data Generation
x = linspace(0, 2*pi, 1000);
y = sin(x);

%% [engine=julia] High-Performance Differential Solve
# %% [engine=julia]
using DifferentialEquations
f(u,p,t) = 1.01*u
u0 = 1/2
tspan = (0.0, 1.0)
prob = ODEProblem(f, u0, tspan)
sol = solve(prob)

# %% [engine=python] Machine Learning Model
import numpy as np
features = np.array(y)
```
If no `[engine=...]` tag is present, the cell executes on the workspace's default engine.

### 5.3 Execution Feedback & Error Mapping
- **Line Number Re-Mapping**: Because cells are executed as isolated snippets, error offsets are translated back to their absolute line and column positions within the parent document.
- **Interleaved Output Panes**: Stdout and stderr streams are captured and displayed in a collapsible output zone immediately below the active cell.

### 5.4 Universal Figure Capture
- When external code invokes plot commands (`mlapp.plot(...)`, `plot(...)`, `plt.show()`), the bridge intercepts the data payload (coordinates, labels, limits, colors).
- The intercepted geometric data is forwarded directly to `crates/rustlab-plot::PlotRegistry` and rendered within native MatRust Studio figure tabs. No external GUI windows (such as Qt or Tkinter windows) are spawned.

### 5.5 UI Status Indicator & Settings Pane
- **Status Indicator Widget**: Located in the application status bar:
  - Displays active engine icon, runtime state (`IDLE`, `RUNNING`, `STARTING`, `HALTED`), PID, and memory footprint.
  - Clicking the widget opens an engine switcher and restart/interrupt dropdown.
- **Preferences / Settings Page**:
  - Configurable executable paths for Julia, Python, Octave, and MATLAB.
  - Auto-discovery buttons that locate installed system binaries.
  - Per-engine timeout ceilings and memory limits.
