# MatRust Studio Backend Architecture

## 1. Executive Summary

MatRust Studio is an independent, clean-room, MATLAB-compatible scientific computing environment and IDE implemented entirely in Rust. It provides a unified computing runtime where the native GUI, command-line interpreter, script editor, data analysis tools, figures, and domain engines all share a single authoritative evaluation engine and persistent workspace state.

## 2. High-Level System Architecture

```text
                           +---------------------------------------+
                           |        MatRust Studio Desktop         |
                           |   (eframe / egui / glow / winit)      |
                           +---------------------------------------+
                                              |
      +------------------------+--------------+--------------+------------------------+
      |                        |                             |                        |
+--------------+       +---------------+             +---------------+       +------------------+
| Command Win  |       | Multi-Tab Ed  |             | Figures Panel |       | Workspace Browser|
+--------------+       +---------------+             +---------------+       +------------------+
      |                        |                             ^                        ^
      +------------------------+                             |                        |
                               \                             |                        |
                              WorkerRequest (mpsc channel)   | GraphicsState          | VariableRow
                                 \                           |                        |
                           +---------------------------------------+                  |
                           |             WorkerClient              |------------------+
                           |   (Background Thread: Session)        |
                           +---------------------------------------+
                                              |
                                              v
                           +---------------------------------------+
                           |          Session Controller           |
                           |     (rustlab-worker thread loop)      |
                           +---------------------------------------+
                                              |
                                              v
                           +---------------------------------------+
                           |        rustlab-eval (Runtime)         |
                           |    Interpreter & Environment Scope    |
                           +---------------------------------------+
                                              |
                      +-----------------------+-----------------------+
                      |                                               |
                      v                                               v
        +----------------------------+                 +----------------------------+
        |   rustlab-math (Engine)    |                 |   rustlab-stdlib (Builtins)|
        | Value Enum, DenseArray,    |                 | Math, Linalg, Decompositions|
        | CSC Sparse, StructArray    |                 | Statistics, File IO        |
        +----------------------------+                 +----------------------------+
                      |                                               |
        +-------------+----------------+               +--------------+---------------+
        |                              |               |                              |
        v                              v               v                              v
+------------------+          +------------------+ +------------------+      +------------------+
|   rustlab-matio  |          |   rustlab-data   | | Simulation / ML  |      | Robotics / Signal|
|  Level 5 MAT IO  |          | CSV, Tables,     | | Transfer Function|      | Rotations, Arm FK|
|  (flate2 stream) |          | Summary Stats    | | Linear Regression|      | Filters, Windows |
+------------------+          +------------------+ +------------------+      +------------------+
```

## 3. Core Architectural Components

### 3.1 Asynchronous Execution Pipeline (`apps/desktop/src/execution.rs`)
- **WorkerClient**: Owns the communication channels (`tx: Sender<WorkerRequest>`, `rx: Receiver<WorkerResponse>`) to a dedicated background thread named `rustlab-worker`.
- **Session**: Envelopes the interpreter instance, breakpoints, graphics state, and workspace inspection.
- **Panic Boundary**: Worker commands are executed inside `std::panic::catch_unwind(AssertUnwindSafe(...))`. If a calculation panics, the worker captures the error, re-initializes the session safely, notifies the UI via `WorkerResponse::WorkerPanic`, and continues execution without crashing the IDE.
- **Breakpoint Stepping**: Managed via `DebugSync` (Mutex + Condvar). When a breakpoint line is hit, the worker pauses, notifies the UI with `WorkerResponse::BreakpointHit`, and blocks until the user sends `DebugCommand::Continue`, `Step`, or `Stop`.

### 3.2 Evaluation Runtime (`crates/rustlab-eval`)
- **Interpreter**: Recursive-descent AST evaluator managing scopes (`Environment`), built-in registration, script resolution, and function calls.
- **Environment**: Lexical hierarchy with variable storage, persistent global/local variable map, and closures capturing lexical environments by value.
- **Function Handles**: First-class function handles representing named builtins (`@sin`), user scripts, or anonymous functions (`@(x) x.^2 + 1`).

### 3.3 Unified Value Representation (`crates/rustlab-math`)
- **`Value`**: Authoritative enum representing all MATLAB runtime datatypes:
  - `Double(DenseArray<f64>)`: Default 64-bit float array.
  - `ComplexDouble(DenseArray<Complex64>)`: Double-precision complex arrays.
  - `Single(DenseArray<f32>)`: 32-bit single-precision floats.
  - `Int8` .. `UInt64`: Saturating integer arrays with MATLAB rounding semantics.
  - `Logical(DenseArray<bool>)`: Boolean masks and logical arrays.
  - `Char(DenseArray<char>)`: Character vectors.
  - `String(DenseArray<String>)`: String arrays.
  - `Cell(DenseArray<Value>)`: Heterogeneous cell arrays.
  - `Struct(StructArray)`: Structure arrays with named field access.
  - `Sparse(CscMatrix<f64>)`: Compressed Sparse Column double matrices.
  - `FunctionHandle(Arc<FunctionHandle>)`: Closures and function references.

### 3.4 Domain Services
- **Numerical & Calculus Engine**: `roots`, `polyval`, `polyfit`, `trapz`, `cumtrapz`, `gradient`, `interp1`.
- **Data & Tables Engine**: `readtable`, `writetable`, `head`, `tail`, `summary`, `readmatrix`, `writematrix`.
- **Engineering Simulation Engine**: `tf`, `step` response, state-space representations.
- **Signal & Image Engine**: `filter`, `filtfilt`, windowing (`hamming`, `hann`, `blackman`), `xcorr`, `rgb2gray`, `imresize`, `imfilter`.
- **Machine Learning Engine**: `fitlm`, `predict`, `kmeans`, `confusionmat`.
- **Robotics Engine**: 3D rotations (`rotx`, `roty`, `rotz`), homogeneous transforms (`trotx`, `troty`, `trotz`, `transl`), forward kinematics (`fkine_2link`), inverse kinematics (`ikine_2link`), quintic trajectories (`jtraj`).
