# Technical Decisions & Trade-Offs: `mlapp-rs`

This document records the design evaluations, architectural decisions, and justifications for the `mlapp-rs` runtime.

---

## Decision 1: Hand-Written Recursive Descent Parser vs. `tree-sitter-matlab`

### Context
MATLAB's syntax contains multiple challenging edge cases:
1. **Transpose vs. Quote**: The character `'` acts as a string delimiter or a transpose operator depending on preceding whitespace and token context (e.g., `A'` vs `[ 'str' ]`).
2. **Whitespace-Sensitive Matrix Literals**: `[1 -2]` is a 1x2 array; `[1 - 2]` is a 1x1 array containing `-1`.
3. **App Designer XML Wrapper**: Code in `.mlapp` is wrapped inside XML CDATA blocks inside OPC ZIP containers, requiring classdef extraction.

### Options Considered
- **`tree-sitter-matlab`**:
  - *Pros*: Incremental parsing, battle-tested for syntax highlighting.
  - *Cons*: Requires an external C compiler toolchain (`cc`), pulls in C FFI dependencies, produces large binary tables, cannot be configured with `#![forbid(unsafe_code)]`, and has rigid grammar rules that struggle with non-standard whitespace sensitivity in matrix expressions.
- **Hand-Written Recursive Descent & Pratt Parser (`rustlab-parse` + `mlapp-lang`)**:
  - *Pros*: 100% pure, safe Rust; exact source span attribution; seamless handling of context-dependent quotes and whitespace in matrix literals; direct synthesis of `AppClassDef` AST nodes (`properties`, `methods`, `startupFcn`, callbacks).
  - *Cons*: Must maintain tokenizer and grammar rules manually.

### Verdict
**Selected Hand-Written Recursive Descent & Pratt Parser**. It eliminates C build dependencies, keeps the toolchain cross-platform (including Windows ARM64), supports `#![forbid(unsafe_code)]`, and provides fine-grained error diagnostic spans.

---

## Decision 2: Coordinate Geometry Mapping

### Context
MATLAB App Designer layouts specify component positions using a rectangular vector `[x, y, width, height]` with origin `(0, 0)` at the **Bottom-Left** of the parent container.
Modern GUI and web rendering backends (egui, Web Canvas, HTML DOM, Tauri) place origin `(0, 0)` at the **Top-Left**.

### Implementation
All coordinate transformations are centralized in `mlapp_ui::matlab_to_top_left_geometry` and its inverse `mlapp_ui::top_left_to_matlab_geometry`:
$$\text{top\_y} = \text{parent\_height} - (\text{matlab\_y} + \text{height})$$
$$\text{top\_x} = \text{matlab\_x}$$

Both transformations are covered by bidirectional round-trip unit tests.

---

## Decision 3: App Handle Object Semantics (`app`)

### Context
In MATLAB App Designer, `app` is passed to every method (`startupFcn(app)`, `ButtonPushed(app, event)`). Mutating `app.EditField.Value` or user properties (`app.Data = rand(10)`) modifies the shared application state, not an isolated local copy.

### Implementation
- `AppHandle` wraps `Arc<Mutex<AppRuntimeState>>`.
- When an app callback is invoked, the interpreter binds `app` as a nested struct populated with current properties and component state.
- Upon completion of callback statements, changes to `app` are inspected, persisted into `AppRuntimeState`, and drained as strongly typed `UiMutation` events sent to the UI/rendering thread.

---

## Decision 4: Compatibility Scoring Formula & Unanalyzable Dynamic Call Diagnostics

### Context
Static analysis of MATLAB App Designer code cannot fully evaluate runtime-constructed calls (e.g., `eval(str)`, `feval(var, ...)`, `assignin('base', ...)`). To provide honest, actionable assessments for the user before launching an app, `mlapp check-compat` analyzes the AST of all method bodies and computes a deterministic compatibility score.

### Scoring Formula
$$\text{Score} = \max\left(0, 100 - (25 \times N_{\text{hazardous}} + 15 \times N_{\text{unanalyzable}} + 5 \times N_{\text{unsupported}})\right)$$

1. **Hazardous Calls ($N_{\text{hazardous}}$)**:
   - Penalty: **-25 points** per call.
   - Identified symbols: `system`, `dos`, `unix`, `delete`, `rmdir`, `exit`, `quit`, `loadlibrary`, `calllib`.
   - Result: App is immediately classified as `Unsupported`.
2. **Unanalyzable Dynamic Calls ($N_{\text{unanalyzable}}$)**:
   - Penalty: **-15 points** per call.
   - Identified symbols: `eval`, `evalc`, `evalin`, `assignin`, `feval`, `builtin`, `str2func`.
   - Result: App cannot achieve `Runs` status; capped at `Partial` even if score >= 90. Diagnostic emitted: `"Method '<name>' contains unanalyzable dynamic call '<symbol>'"`
3. **Unsupported Builtins ($N_{\text{unsupported}}$)**:
   - Penalty: **-5 points** per builtin not implemented in the current runtime library.
4. **Overall Status**:
   - `Runs`: Score $\ge 90$, $N_{\text{hazardous}} = 0$, $N_{\text{unanalyzable}} = 0$.
   - `Partial`: Score $\ge 50$ and $N_{\text{hazardous}} = 0$.
   - `Unsupported`: Score $< 50$ or $N_{\text{hazardous}} > 0$.

---

## Decision 5: Clean-Room Isolation, Real-App Validation & UNVERIFIED Format Items

### Context
To maintain strict clean-room legal integrity, proprietary MathWorks binaries, sample applications, and assets are never checked into version control.

### Current Status
- **0 Real Proprietary Apps Checked In**: Repository test fixtures are clean-room synthesized packages conforming strictly to ISO/IEC 29500-2:2021 (Open Packaging Conventions) and MathWorks public App Designer XML specifications.
- **Opt-In External Validation Harness**: An opt-in test harness (`test_opt_in_real_apps_validation`) inspects, parses, and validates real `.mlapp` packages supplied externally through the `MLAPP_REAL_APPS_DIR` environment variable.

### UNVERIFIED Format Assumptions
The following format aspects remain **UNVERIFIED** pending direct black-box testing against real `.mlapp` files from varying releases:
1. `UNVERIFIED_MLAPP_OPC_NAMESPACES`: Exact XML namespace URI strings across older pre-R2018a releases for custom App Designer packaging.
2. `UNVERIFIED_BINARY_COMPONENT_DATA`: Internal payload representation of legacy or custom third-party ActiveX/Java controls embedded in `.mlapp`.
3. `UNVERIFIED_XML_UUID_FORMAT`: Exact UUID formatting scheme and schema versioning attributes in `appdesigner/app.xml` across future releases.
4. `UNVERIFIED_TIMER_CALLBACK_INTERLEAVING`: Precise callback queuing and re-entrancy semantics for multi-timer App Designer components.


---

## Decision 6: MATLAB Subprocess Bridge (`mlapp-matlab`) & Oracle Conformance Testing

### Context
To maintain 100% clean-room isolation while providing an automated semantic oracle, `mlapp-matlab` provides an inter-process bridge to any locally installed real MATLAB version. It also serves as an explicit runtime fallback when users provide `--allow-matlab`.

### Implementation & Protocol
1. **Engine Discovery**:
   - `MATLAB_ROOT` environment variable
   - System `PATH`
   - Windows Registry (`HKLM\SOFTWARE\MathWorks\MATLAB`)
   - Standard macOS `/Applications/MATLAB_*.app`
   - Standard Linux `/usr/local/MATLAB/*` and `/opt/MATLAB/*`
2. **IPC Protocol**:
   - Spawns `matlab -batch "..."` or `-nodesktop -nosplash -nojvm -r "..."` with piped stdin/stdout.
   - Communicates over newline-delimited JSON with typed envelopes (`MatlabRequest::Eval`, `MatlabRequest::Call`).
   - Responses carry typed values (`Double`, `Char`, `Logical`, `Cell`, `Struct`) or structured error details (`identifier`, `message`).
3. **Graceful Oracle Testing**:
   - If MATLAB is not installed on the runner, `crates/mlapp-matlab/tests/oracle_tests.rs` prints `[SKIPPED] MATLAB not installed on host` and passes cleanly without failing CI.
   - Internal mock session enables deterministic unit testing of envelopes, value conversions, and failure semantics in headless CI.
4. **Runtime Integration**:
   - Without `--allow-matlab`: Unknown builtins return `Unsupported MATLAB function '<name>'. Hint: run with --allow-matlab to delegate to real MATLAB`.
   - With `--allow-matlab`: Unknown builtins are transparently forwarded to `MatlabSession::call()`.
