# Architecture overview: where to start

MatRust Studio is a Cargo workspace. Start with the small library that owns
the behavior you want to understand; the desktop application calls those
libraries rather than implementing a second numerical engine.

## Workspace map

| Area | Crates / package | Start here for |
|---|---|---|
| Language front end | `rustlab-parse` | Tokens, expression parsing, statements, and source spans |
| Numerical values and execution | `rustlab-math`, `rustlab-eval` | Arrays and values, operators, environments, and AST evaluation |
| Builtins and data | `rustlab-stdlib`, `rustlab-data`, `rustlab-matio` | Numerical functions, statistics/CSV, and MAT-file I/O |
| Numerical CLI | `rustlab-cli` | Running scripts and REPL-facing commands |
| Plot models | `rustlab-plot`, `mlapp-plot` | Numerical SVG plots and App Designer figures |
| App package and language | `mlapp-container`, `mlapp-ui`, `mlapp-lang` | `.mlapp`/`.mlappinstall` containers, component trees, and callback extraction |
| App execution and rendering | `mlapp-runtime`, `mlapp-host`, `mlapp-render` | Callback execution, host lifecycle, and GUI/headless rendering |
| Simulink inspection/simulation | `mlapp-sim` | `.slx` archive parsing, block diagrams, and the current simulation engine |
| Interfaces and bridges | `mlapp-cli`, `mlapp-ffi`, `mlapp-py`, `mlapp-matlab`, `mlapp-octave` | App commands, C/Python interfaces, and optional external runtimes |
| Desktop | `matrust_studio` (`apps/desktop`) | egui workspace, editor, panels, and user interactions |
| Fuzzing | `fuzz/` | Parser, XML, and archive fuzz targets |

The root `Cargo.toml` is the authoritative workspace member list. Crate
descriptions are useful orientation, not a promise that every listed
compatibility behavior is complete.

## Typical paths through the code

1. A script enters `rustlab-parse`; tokens become syntax nodes with source
   locations.
2. `rustlab-eval` evaluates those nodes using values and operations from
   `rustlab-math`, and resolves builtins through `rustlab-stdlib`.
3. App package commands use `mlapp-container` and `mlapp-lang` to inspect
   package contents, then `mlapp-runtime`, `mlapp-host`, and renderers to run
   supported callbacks.
4. `.slx` inspection and simulation are in `mlapp-sim`; the desktop shell is
   in `apps/desktop`.

## Where to add tests

- Unit tests live beside the implementation or in each crate's `tests/`
  directory.
- Workspace behavior is exercised by `cargo test --workspace`.
- The existing `docs/test-manifest.txt` is checked by CI; its current guard
  compares counts rather than exact names. Review new and removed test names
  explicitly before updating it.
- Test oracles should use analytical expected values, public format
  specifications, or clean-room synthetic fixtures. Do not commit proprietary
  applications or models.

## Useful project documents

- [Function coverage registry](../COVERAGE.md)
- [Backlog](../BACKLOG.md)
- [Owed-items register](../OWED.md)
- [Safety audit](../SAFETY-AUDIT.md)
- [AI-assisted development and clean-room policy](AI-ASSISTED-DEVELOPMENT.md)
