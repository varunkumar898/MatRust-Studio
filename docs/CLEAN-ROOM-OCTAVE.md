# Clean-Room Policy: GNU Octave as an Oracle (docs/CLEAN-ROOM-OCTAVE.md)

**Status:** Active policy. Mirrored as **P14** in `docs/AGENT-PROTOCOL.md`.
**Scope:** Every agent or contributor who uses GNU Octave to check behaviour.

---

## 1. Licence boundary

- GNU Octave is licensed **GPL-3**. This project is **MIT OR Apache-2.0**.
- **NEVER copy, translate or paraphrase Octave source code** (C++ or `.m` files), its built-in test blocks (lines beginning with `%!`), or its documentation and help text.
- Agents **must not open** Octave source files, source repositories or source browsers.

## 2. Allowed inputs

Only these sources may inform behaviour:

1. Octave run as a **separate process** through `crates/mlapp-octave`, as a black-box oracle on inputs **we write**.
2. **Public documentation**, read only to learn behaviour, then described in our own words.
3. **Textbooks and papers**.
4. **Observed MATLAB-compatible behaviour**, never MathWorks text.

## 3. Golden values

- Numbers Octave produces for our inputs may be stored as **small fixtures**.
- Each fixture records the **Octave version**, the **exact command**, and the **date**.
- **Never store** Octave help text, Octave test files, or Octave source excerpts.

## 4. Provenance

- Every new function gets `docs/provenance/<function>.md` containing: behaviour sources, oracle version, test names, and the statement **"no Octave source was consulted"**.

## 5. Error messages

- Use **our own wording**.
- Match **behaviour** (conditions and error identifiers), never the text.

## 6. When it is too intricate

- If a behaviour is too intricate to reproduce clean-room, keep the **bridge fallback** (`--allow-octave`). Do **not** copy code.

## 7. Runtime and distribution

- Octave is an **optional external program**: never bundled, never required.
- **CI never requires Octave.**
- `README.md` and `NOTICE` credit GNU Octave.
- **Installing Octave is a user action.** If it is missing, report it and stop; do not install it silently.
