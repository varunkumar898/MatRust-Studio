# GNU Octave Subprocess Bridge (`mlapp-octave`)

## 1. Overview & Clean-Room GPL Boundary

The `mlapp-octave` crate provides an out-of-process execution bridge to [GNU Octave](https://octave.org/). It allows MatRust Studio / RustLab to delegate unsupported functions, run differential oracle conformance checks, and execute complex scientific routines via GNU Octave.

### Clean-Room Licensing Boundary
- **GNU Octave License**: GNU General Public License v3 or later (GPLv3+).
- **MatRust Studio License**: MIT OR Apache-2.0.
- **IPC Architecture**: `mlapp-octave` communicates with GNU Octave **strictly through standard operating system process pipes** (`stdin`, `stdout`, `stderr`).
- **No Derivative Works**: No Octave source code or header files are copied into this repository. No Octave libraries (`liboctave`, `liboctinterp`) are linked dynamically or statically.
- **Permissive Distribution**: MatRust Studio does not bundle GNU Octave binaries. Users install GNU Octave independently via their system package manager (`apt`, `brew`, `choco`, `winget`).

---

## 2. Process Lifecycle & Execution Model

The bridge spawns `octave-cli` in headless batch mode:

```bash
octave-cli --no-gui --norc --no-init-file --quiet --no-window-system
```

### Discovery & Fallback
1. Checks the `OCTAVE_EXECUTABLE` environment variable.
2. Checks standard system PATH executables (`octave-cli`, `octave`, `octave-cli.exe`, `octave.exe`).
3. Checks known platform installation directories (e.g., `C:\Program Files\GNU Octave\Octave-*\mingw64\bin\octave-cli.exe` on Windows, `/usr/bin/octave-cli` on Linux, `/opt/homebrew/bin/octave-cli` on macOS).
4. If Octave is not present, `OctaveSession::start` returns `OctaveError::NotInstalled`. The application gracefully falls back to native RustLab runtime execution or mock test execution via `OctaveSession::start_mock()`.

---

## 3. Safety Allow-List & Execution Policy

By default, executing arbitrary code via an external interpreter risks unintended file deletion, system commands, or privilege escalation. `mlapp-octave` enforces a strict function-level security policy:

### Blocked Functions (`BLOCKED_FUNCTIONS`)
The following system, process, reflection, and filesystem functions are blocked by default:
- **System / Shell**: `system`, `dos`, `unix`, `popen`, `popen2`
- **Filesystem Deletion / Modification**: `delete`, `unlink`, `rmdir`, `mkdir`
- **Process Lifecycle**: `exit`, `quit`
- **Uncontrolled Dynamic Evaluation**: `eval`, `evalin`, `assignin`, `feval`, `builtin`, `source`, `run`
- **Foreign Runtimes / Reflection**: `javaMethod`, `javaObject`, `python_invoke`

Calls to any blocked function return `Err(OctaveError::SecurityViolation(...))` unless explicitly permitted via `OctaveConfig { allow_unsafe: true, .. }`.

### Safe Builtins (`SAFE_BUILTINS`)
Over 60 core mathematical, linear algebra, signal processing, and numerical builtins are pre-vetted as recognized safe functions:
- Trigonometric & elementary: `sin`, `cos`, `tan`, `exp`, `log`, `sqrt`, `round`, `floor`, etc.
- Linear algebra: `inv`, `det`, `lu`, `qr`, `chol`, `svd`, `eig`, `pinv`, `rref`, etc.
- Numerical calculus: `diff`, `gradient`, `trapz`, `cumtrapz`, `polyfit`, `roots`, `fzero`, `fsolve`.

---

## 4. Value Marshalling & Serialization

The `OctaveValue` enum mirrors `rustlab_math::Value` for zero-loss bidirectional data exchange:

| MatRust `Value` | `OctaveValue` Representation | Octave Syntax Marshalling |
|---|---|---|
| `Double` | `OctaveValue::Double { dims, data }` | Matrix literal or `reshape([data], [dims])` |
| `Char` | `OctaveValue::Char(String)` | Single-quoted string literal `'...'` |
| `Logical` | `OctaveValue::Logical { dims, data }` | `logical(reshape([...], [...]))` |
| `Cell` | `OctaveValue::Cell { dims, data }` | Cell array syntax `{val1, val2}` |
| `Struct` | `OctaveValue::Struct { fields }` | `struct('field', val, ...)` |

---

## 5. Verification & Test Evidence

Test suite located in `crates/mlapp-octave/tests/octave_tests.rs`:
- `test_octave_detection_functions`: Verifies binary discovery and path detection without crashing.
- `test_octave_value_roundtrip_all_types`: Asserts zero-loss bidirectional roundtrip between `rustlab_math::Value` and `OctaveValue` for scalars, 2D matrices, char vectors, cell arrays, and struct arrays.
- `test_octave_literal_generation`: Verifies valid GNU Octave literal syntax generation for stdin piping.
- `test_octave_safety_allowlist_enforcement`: Asserts that blocked functions (`system`, `delete`, `unlink`, `javaMethod`, etc.) are rejected with security errors by default, and allowed only when `allow_unsafe` is true.
- `test_octave_mock_session_eval_and_call`: Tests session evaluation, function calling, variable assignment, and variable retrieval in mock session mode.
- `test_octave_session_lifecycle_and_shutdown`: Verifies clean subprocess creation, command execution, and orderly shutdown.
- `test_octave_protocol_serialization_roundtrip`: Verifies JSON serialization and deserialization of request/response envelopes.
- `test_real_octave_oracle_conformance_if_installed`: When `octave-cli` is detected on the host machine, executes live numerical calculations (`sin(pi/6)`, matrix inversion) and verifies result equality.
