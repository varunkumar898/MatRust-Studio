# Julia Subprocess Bridge & Solver Cross-Check Design (docs/design/julia-bridge.md)

**Phase:** PLAN-POLY / F16 (Extends F16.01..F16.05)  
**Scope:** Subprocess bridge architecture, zero-external-dependency Julia server script, binary IPC protocol, in-app plotting API (`mlapp`), cross-check harness against Julia differential equation solvers, and headless CI stubbing.

---

## 1. Overview & Clean-Room Governance

The Julia bridge (`crates/mlapp-julia`) connects MatRust Studio to an external Julia runtime. In accordance with Architecture Decision Record F16.01 and Charter Principle P3:
- **Subprocess Isolation**: Julia executes as an isolated out-of-process subprocess. Embedding `libjulia` is rejected due to ABI instability, heavy binary bloat (> 500 MB), complex thread runtime interactions, and memory management conflicts.
- **Licensing Compliance**:
  - Julia's core language runtime is licensed under the permissive **MIT License**.
  - Julia standard libraries and third-party packages carry independent licenses (e.g., BSD, MIT, GPL, Apache).
  - MatRust Studio **never bundles** Julia or any Julia packages. Installation and package management are user actions.
- **Base Installer Discipline**: The MatRust Studio base installer contains zero Julia binaries. The bridge is completely inert unless the user explicitly enables `--allow-julia` and provides or auto-detects an existing system Julia executable.

---

## 2. Platform Detection & Windows ARM64 Status

### 2.1 Detection Mechanism
On startup or configuration request, the bridge searches for a valid Julia installation:
1. User-configured override path in Workspace Settings.
2. Standard system `PATH` resolution (`Get-Command julia` / `which julia`).
3. Standard platform discovery directories:
   - Windows: `%LOCALAPPDATA%\Programs\Julia*`, `%ProgramFiles%\Julia*`, Scoop (`%USERPROFILE%\scoop\apps\julia\current\bin\julia.exe`), WinGet (`%LOCALAPPDATA%\Microsoft\WinGet\Links\julia.exe`).
   - Linux: `/usr/bin/julia`, `/usr/local/bin/julia`, `~/.juliaup/bin/julia`.
   - macOS: `/Applications/Julia-*/Contents/Resources/julia/bin/julia`, `~/.juliaup/bin/julia`.

Upon detection, the bridge executes:
```bash
julia --startup-file=no --version
```
Capturing semver, architecture, and platform target.

### 2.2 Windows ARM64 Status Matrix

| Platform / Architecture | Support Tier | Bridge Execution Mode | Notes |
| :--- | :---: | :--- | :--- |
| **Windows x86_64** | Tier 1 | Native 64-bit Subprocess | Fully tested and supported upstream by Julia. |
| **Windows ARM64** | Tier 3 (Experimental) | 1. Native ARM64 (Nightly/Unofficial)<br>2. x86_64 under Windows 11 Prism Emulation | Official Julia native Windows ARM64 builds are experimental. However, standard x86_64 Julia runs stably under Windows 11 ARM64 dynamic translation (Prism). |
| **Linux x86_64** | Tier 1 | Native 64-bit Subprocess | Full support. |
| **Linux aarch64** | Tier 1 | Native 64-bit Subprocess | Full support on Linux ARM64 (Raspberry Pi 4/5, AWS Graviton). |
| **macOS Apple Silicon (aarch64)** | Tier 1 | Native 64-bit Subprocess | Full support. |

---

## 3. Bridge Server Architecture (Zero External Julia Dependencies)

### 3.1 Motivation
Standard Julia installations do not include a JSON parser in their standard library (the standard JSON package `JSON.jl` requires an external `Pkg.add("JSON")`). To guarantee that the bridge functions immediately on any vanilla Julia installation with zero setup:
- The bridge server relies **exclusively on Julia's Standard Library** (`Base`, `Sockets`, `Serialization`, `Mmap`).
- No external packages (`JSON.jl`, `JSON3.jl`, `HTTP.jl`, `Plots.jl`) are imported or required by the bridge core.

### 3.2 In-Tree Server Script (`server.jl`)
The server script is embedded as a compile-time string literal in `crates/mlapp-julia`:
```rust
pub const JULIA_SERVER_SCRIPT: &str = include_str!("server.jl");
```
On bridge initialization:
1. MatRust Studio creates an isolated temporary directory in `<temp>/matrust_julia_<pid>/`.
2. Writes `server.jl` into this directory.
3. Spawns Julia with clean-room isolation flags:
   ```bash
   julia --startup-file=no --history-file=no --project=@temp_env <temp>/matrust_julia_<pid>/server.jl --port=<random_port> --token=<auth_token>
   ```

### 3.3 Wire Framing & Binary Message Protocol
Communication proceeds over a local loopback TCP connection (`127.0.0.1:[port]`) using length-prefixed binary frames:

```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                  Magic: 0x4A554C49 ("JULI")                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Message Type         |             Flags             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                      Payload Length (MSB)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                      Payload Length (LSB)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Data...                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

#### Message Types:
- `0x0001` **HANDSHAKE**: Client sends 256-bit bearer token; server validates.
- `0x0002` **EVAL_REQUEST**: Source code string + cell identifier.
- `0x0003` **EVAL_RESPONSE**: Execution status, stdout, stderr, execution duration.
- `0x0004` **GET_VAR_REQUEST**: Variable name string.
- `0x0005` **GET_VAR_RESPONSE**: Typed variable payload (binary header + raw column-major data).
- `0x0006` **SET_VAR_REQUEST**: Variable name + typed payload.
- `0x0007` **SET_VAR_RESPONSE**: Success / error acknowledgment.
- `0x0008` **LIST_VARS_REQUEST**: Enumerate workspace.
- `0x0009` **LIST_VARS_RESPONSE**: Array of variable metadata (name, type, dims, bytes).
- `0x000A` **PLOT_EVENT**: Intercepted in-app plot geometry sent to MatRust figure registry.
- `0x000B` **INTERRUPT**: Signal active evaluation cancellation.
- `0x000C` **SHUTDOWN**: Orderly termination.

---

## 4. In-App Plotting API (`mlapp` Julia Module)

To allow Julia code running inside MatRust Studio to generate plots without forcing users to install massive plotting ecosystems (`Plots.jl` takes several minutes to precompile and download hundreds of megabytes of dependencies), the bridge server automatically injects a lightweight `mlapp` module:

```julia
module mlapp

using Sockets

# Global channel handle initialized by server.jl
const _STUDIO_SOCK = Ref{TCPSocket}()

"""
    mlapp.plot(x, y; kwargs...)
Plot 2D line or scatter series directly into MatRust Studio Figures panel.
"""
function plot(x::AbstractVector{<:Real}, y::AbstractVector{<:Real}; 
              label::String="", color::String="", style::String="-")
    payload = _serialize_plot_series(Float64.(x), Float64.(y), label, color, style)
    _send_frame(_STUDIO_SOCK[], 0x000A, payload)
end

function title(text::String)
    _send_frame(_STUDIO_SOCK[], 0x000A, _serialize_plot_attr("title", text))
end

function xlabel(text::String)
    _send_frame(_STUDIO_SOCK[], 0x000A, _serialize_plot_attr("xlabel", text))
end

function ylabel(text::String)
    _send_frame(_STUDIO_SOCK[], 0x000A, _serialize_plot_attr("ylabel", text))
end

function figure(id::Integer)
    _send_frame(_STUDIO_SOCK[], 0x000A, _serialize_plot_attr("figure_id", Int32(id)))
end

function display(obj)
    # Formats object via standard show and streams to studio stdout
    show(stdout, MIME("text/plain"), obj)
    println()
end

end # module mlapp
```

This guarantees zero-overhead, sub-millisecond plotting that routes directly into the desktop figure panel tabs (`crates/rustlab-plot`).

---

## 5. Bidirectional Type Mapping Table

| Julia Type | Wire Type ID | RustLab `Value` Equivalent | Transformation & Tolerance Rules |
| :--- | :---: | :--- | :--- |
| `Float64` | `0x10` | `Value::Double(f64)` | Exact binary IEEE 754 match. |
| `Float32` | `0x11` | `Value::Single(f32)` | Exact binary IEEE 754 match. |
| `Int64` / `Int32` | `0x20` / `0x21` | `Value::Int64(i64)` / `Value::Int32(i32)` | Two's complement integer match. |
| `UInt64` / `UInt32` | `0x28` / `0x29` | `Value::UInt64(u64)` / `Value::UInt32(u32)` | Unsigned binary match. |
| `Complex{Float64}` | `0x30` | `Value::ComplexDouble(Complex<f64>)` | Contiguous pair [real, imag]. |
| `Bool` | `0x01` | `Value::Logical(bool)` | 1 byte: `0x01` (true), `0x00` (false). |
| `String`, `SubString` | `0x40` | `Value::String(String)` | Validated UTF-8 byte stream. |
| `Array{Float64, N}` | `0x50` | `Value::DenseArray<f64>` | Column-major raw binary buffer. Zero-copy. |
| `Array{T, N}` | `0x51` | `Value::DenseArray<T>` | Generic column-major buffer for primitive types. |
| `Dict{String, Any}` | `0x60` | `Value::Struct(HashMap<String, Value>)` | Key-value dictionary. Recursive mapping. |
| `Vector{Any}`, `Tuple`| `0x70` | `Value::CellArray(Vec<Value>)` | Heterogeneous element container. |
| *Other / Unsupported* | `0xFF` | Returns `EngineError::UnsupportedJuliaType` | Descriptive error naming Julia type. |

---

## 6. Precompilation, Latency & Process Lifecycle

### 6.1 First-Run Latency ("Time-to-First-Plot")
- When Julia packages are loaded (e.g. `using DifferentialEquations`), Julia compiles native code into memory, which can take 10–30 seconds on initial run.
- **UI Progress Telemetry**: The bridge monitors subprocess stderr and streams precompilation messages to the UI status indicator: `"Julia: Precompiling packages (DifferentialEquations.jl)..."`.
- **Background Warm-Up Option**: In workspace preferences, users can toggle `Warm-up Julia in background on startup`, allowing JIT compilation to complete before the user executes their first cell.

### 6.2 Cancellation & Interrupt Behavior
- If a user cancels an execution or an operation times out:
  - On Unix platforms: `SIGINT` is sent to the Julia child process.
  - On Windows: The bridge triggers a `CTRL_BREAK_EVENT` via `GenerateConsoleCtrlEvent` on the child process group.
  - If the engine fails to halt within 2.0 seconds, the bridge forcefully terminates the process and relaunches a fresh server session.

### 6.3 Cleanup Guarantee
- The bridge process registers an exit hook. If MatRust Studio terminates (normally or abnormally):
  - Julia server script reads standard input heartbeats. If stdin reaches EOF (parent process died), the server script self-terminates immediately (`exit(0)`).
  - Temporary directories and socket handles are purged.

---

## 7. Model Export & Solver Cross-Check Harness

### 7.1 Continuous Model Exporter
The bridge provides an automated translation path from MatRust Studio continuous ODE models (`crates/mlapp-sim`) to Julia scripts:
- Continuous states $\dot{x} = f(x, u, t)$ and mass matrices $M \dot{x} = f(x, u, t)$ are exported into a standalone `.jl` script targeting `DifferentialEquations.jl`.
- Allows users to run verification studies against independent Julia solvers (`Tsit5`, `Vern7`, `RadauIIA5`).

### 7.2 Differential Test Harness
- A dedicated test suite (`tests/solver_cross_check.rs`) executes identical benchmark models in both MatRust's native solver engine and Julia's solver suite:
  1. Harmonic Oscillator (non-stiff).
  2. Van der Pol oscillator ($\mu = 1000$, stiff).
  3. Robertson chemical reaction (stiff DAE).
- Assertion: Solution trajectories match within relative tolerance $10^{-5}$.
- **Clean Execution Rule**: If Julia or `DifferentialEquations.jl` is not installed on the system, this test suite skips cleanly without failing (`SKIPPED: Julia differential solver not found`).

---

## 8. CI Stubbing & Test Strategy

To guarantee that CI pipelines run in seconds and require zero external Julia installations:

### 8.1 Rust Mock Julia Stub (`julia-stub`)
- A small standalone Rust binary is compiled in `crates/mlapp-julia/tests/julia_stub/`:
  - Implements the exact same binary protocol as `server.jl`.
  - Simulates handshake, evaluation, variable get/set, and plotting frames.
- Standard cargo tests (`cargo test -p mlapp-julia`) invoke this stub binary to verify 100% of the bridge protocol, timeout handlers, buffer allocations, and error paths.

### 8.2 Live Integration Tests
- A single integration test (`tests/real_julia_integration.rs`) runs against the real host Julia executable.
- Marked `#[ignore]` by default; runs only when `MATRUST_TEST_REAL_JULIA=1` is explicitly set in the environment.
