# Contributing to MatRust Studio

Thanks for helping improve MatRust Studio. The project is in alpha; please discuss larger changes in an issue before investing substantial time.

## Development setup

- Install Rust 1.88.0 or newer, plus the stable toolchain components `rustfmt` and `clippy`.
- Clone the repository and work from its root.
- The default development flow is offline/local. Optional MATLAB and GNU Octave bridges require separately installed tools.

## Workspace map

| Path | Purpose |
|---|---|
| `crates/rustlab-*` | Parser, numerical values, interpreter, standard library, data, MAT I/O, CLI, and plotting |
| `crates/mlapp-*` | App Designer packages, UI, runtime, rendering, simulation, and integration APIs |
| `apps/desktop` | egui desktop application |
| `tests`, `fixtures`, `fuzz` | Tests, synthetic fixtures, and fuzz targets |
| `docs` | Compatibility registry, safety notes, design, and verification evidence |

## Pick an issue

Look for `good first issue` or `help wanted`. Read the acceptance criteria and linked crate before starting. If an issue involves compatibility behavior, include a public source or a small, independently authored test case. Ask before expanding the scope.

## Branch and pull request workflow

1. Fork the repository and create a focused branch, for example `fix/parser-span`.
2. Keep the change small and add or update tests and documentation.
3. Run the checks below and review `git diff` for generated files, secrets, and unrelated changes.
4. Open a pull request against `main`, explain the behavior change, and link the issue.
5. Respond to review feedback and keep the branch focused.

## Verification commands

Run the same workspace checks used by the project:

```sh
cargo test --workspace
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo fmt --all -- --check
```

## Tests and the test manifest

- Add a regression test for each behavior change and cover meaningful edge cases.
- Do not remove a test just to make a check pass. If a test is intentionally removed, explain why in the same pull request.
- `docs/test-manifest.txt` records the test inventory. After changing tests, regenerate it from `cargo test --workspace -- --list` using the extraction rule in `scripts/verify.ps1`, review the diff, and update `docs/test-count.md` when its counts change.
- CI checks the active test count against the manifest. A manifest update is part of the change, not optional bookkeeping.
- Use synthesized fixtures. Never add a real proprietary `.mlapp` or `.slx` file.

## Regenerate the C header

When a C ABI declaration changes, regenerate the committed header and check that it is up to date:

```sh
cbindgen --config cbindgen.toml --crate mlapp-ffi --output crates/mlapp-ffi/include/mlapp.h
git diff --exit-code -- crates/mlapp-ffi/include/mlapp.h
```

## Clean-room policy

Contributors must not paste, copy, adapt, or commit code or prose from MATLAB, MathWorks files or documentation, GNU Octave source or documentation, or other GPL-licensed sources. Do not reverse engineer or decompile proprietary binaries. Do not submit real proprietary `.mlapp`, `.mlappinstall`, or `.slx` files, screenshots containing private data, or copyrighted assets.

Use your own implementation and examples, public file-format specifications, and independently written tests. Cite public references where needed, but do not reproduce their text. Use synthetic fixtures for package and model tests. If the origin or license of a file is uncertain, leave it out and ask the maintainer.

## Reporting issues

Do not post undisclosed vulnerabilities publicly. Use GitHub's private security advisory flow described in `SECURITY.md`.
