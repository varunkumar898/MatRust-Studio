# MatRust Studio Agent Protocol (docs/AGENT-PROTOCOL.md)

This protocol governs all development, verification, audit, and submission activities performed by AI coding agents on the MatRust Studio repository.

---

## 1. Core Operating Principles

- **P0 Mission & Authority**: Build MatRust Studio as a clean-room, high-performance numerical computing platform and App Designer runtime in Rust. Precedence is: (1) `docs/AGENT-PROTOCOL.md`, (2) `docs/ENGINEERING-CHARTER.md`, (3) `docs/legacy/AGENTS-legacy.md`. The project owner is the sole authority for roadmap modifications, scope adjustments, and item deferrals.
- **P1 Evidence Before Citation**: Every claim that code compiles, passes tests, or achieves a performance target must be substantiated by command output executed in the active session. Save raw logs to `<repo>/docs/verify-logs/` or `<repo>/bench-results/`.
- **P2 Documentation Integrity**: Never modify or abbreviate historical milestone records. Maintain docstrings and comments.
- **P3 Clean-Room & Legal Compliance**: Implement clean-room Rust code based exclusively on public standards, published numerical literature, and open references (SciPy, netlib, public documentation). Never inspect or reverse engineer proprietary MathWorks binaries or P-code.
- **P4 Strict Scope Lock**: Work strictly within the designated milestone or task scope. Place out-of-scope ideas or issues into `docs/BACKLOG.md`.
- **P5 Safe Command Shapes**: Use prefix-matchable, single-command invocations. Avoid shell command substitutions, nested subshells, or backgrounding outside sanctioned harness tools.
- **P6 Task Scope & Batching (One Task Per Session)**: Execute one task per session; several tasks only with BATCH n. All code must build cleanly with `-D warnings` under default, all-features, and no-default-features configurations.
- **P7 Tests Never Decrease**: The master test manifest (`docs/test-manifest.txt`) is monotonic; tests may only be removed with explicit written justification.
- **P8 Deterministic Execution**: PRNG seeds must be explicitly controlled. Numerical outputs must remain bit-exact or within documented epsilon bounds across platforms.
- **P9 Path Sanitization**: Audit logs, commit messages, and documentation must contain zero personal or system-specific file paths. Use `<repo>` and `<user-home>` placeholders.
- **P10 No Unilateral Pushes**: Never push branches or open PRs without explicit owner command. Never force-push or delete branches.
- **P11 Small Verified Increments**: Implement changes in digestible increments of $\le 150$ lines, run fast verification, and update documentation ledgers.
- **P12 Honesty Over Defensiveness**: Acknowledge mistakes immediately, record root causes in milestone retrospectives, and apply minimal targeted fixes.

---

## 2. Section P13: Rules for the Phase Items (F1 through F16)

1. **One item = one task = one PR**: Every discrete item in `docs/WORKPLAN-PHASES.md` maps to a single focused task and Pull Request. Items sized **XL** (DAE solver, Radau IIA, FMU export/import, HDL generation, GPU acceleration, Model Predictive Control, HIL, AUTOSAR, PLC code generation, ROS bridge, Gazebo integration, state machine editor) must be decomposed by a formal planning task (`PLAN-F<n>`) into subtasks of size **L** or smaller, accompanied by an Architecture Decision Record (ADR), prior to code implementation.
2. **Standardized status values**:
   - `TODO`: Planned item not yet started or implemented.
   - `EXISTS-VERIFIED`: Implemented in the codebase with named automated test verification cited.
   - `EXISTS-UNVERIFIED`: Implemented in the codebase or UI, pending formal automated test or human visual verification.
   - `DEFERRED`: Explicitly deferred by owner decision (tracked in `docs/ROADMAP.md`).
   - `NEEDS-HUMAN-HARDWARE`: Target requiring physical hardware (microcontroller board, robot, external hardware-in-the-loop simulator). The software interface, code generator, and simulated host loopback mock must be built and tested; on-target claims are disallowed without physical verification.
3. **Dependency and storage discipline**: Any new external crate dependency requires an ADR and owner approval; prefer self-contained in-tree numerical algorithms. Heavy optional capabilities (FMU co-simulation, Python C-API bindings, GPU compute shaders, ROS client libraries) must reside behind optional cargo feature flags or standalone modules, never bloating the default base installer. Every phase must audit release executable size and package footprint against [docs/STORAGE-BUDGET.md](STORAGE-BUDGET.md).
4. **Clean-room naming standards**: Use generic, descriptive terminology in all user interface labels, source code identifiers, documentation, and file names (e.g., *state machine editor*, *model diagnostics & checks*, *custom block extension API*). Never copy MathWorks trademarks, proprietary brand names, or copyrighted interface descriptors.
5. **Traceability and auditability**: Every merged item requires:
   - A named automated regression test in the test suite.
   - A dedicated generated documentation page in `docs/`.
   - An active entry in [docs/FEATURE-REGISTRY.md](FEATURE-REGISTRY.md) (with GUI rendering status marked `UNVERIFIED` until human visual inspection).
   - An updated capability mapping line in [docs/CLAIMS.md](CLAIMS.md) for any public-facing capability.
6. **Phase merge gate**: A phase is declared **MERGED** only when all constituent items are `MERGED`, `EXISTS-VERIFIED`, or `DEFERRED`, all stated entry and exit criteria pass, storage and performance budgets are satisfied, and its designated release-train reference example is integrated into the application's Examples catalog.

---

## 3. Section P14: Clean-Room Policy for GNU Octave

1. **Licence boundary**: GNU Octave is GPL-3; this project is MIT OR Apache-2.0. **NEVER copy, translate or paraphrase Octave source code** (C++ or `.m` files), its built-in test blocks (lines beginning with `%!`), or its documentation and help text. Agents **must not open** Octave source files, source repositories or source browsers.
2. **Allowed inputs**: (1) Octave run as a separate process through `crates/mlapp-octave` as a black-box oracle on inputs **we write**; (2) public documentation read only to learn behaviour, then described in our own words; (3) textbooks and papers; (4) observed MATLAB-compatible behaviour, never MathWorks text.
3. **Golden values**: numbers Octave produces for our inputs may be stored as small fixtures. Record the Octave version, the command and the date. **Never store** Octave help text or test files.
4. **Provenance**: write `docs/provenance/<function>.md` for every new function, recording behaviour sources, oracle version, test names, and the statement **"no Octave source was consulted"**.
5. **Error messages**: use our own wording; match behaviour (conditions, error identifiers), not text.
6. **Too intricate to do clean-room**: keep the bridge fallback (`--allow-octave`); do **not** copy code.
7. **Runtime and distribution**: Octave is an optional external program: never bundled, never required; CI never requires it; `README.md` and `NOTICE` credit it.
8. **Installing Octave is a user action**: if it is missing, report it and stop.

The full policy is mirrored in [docs/CLEAN-ROOM-OCTAVE.md](CLEAN-ROOM-OCTAVE.md).

---

## 4. Section P15: Legacy Rules in Force

Legacy rules R1-R5 and R7-R12 of `docs/legacy/AGENTS-legacy.md` remain in force unless this protocol says otherwise. R6 is replaced by P6.

