# Contributing

Thanks for helping improve MatRust Studio. The project is in alpha, so
compatibility and APIs can change. Start with a focused issue or describe the
problem you found before investing in a large change.

## Development setup

Install Rust 1.88.0 or newer, clone the repository, then run:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
```

To try the App Designer / Simulink CLI:

```sh
cargo run -p mlapp-cli --locked -- --help
```

The desktop application package is `matrust_studio`:

```sh
cargo run -p matrust_studio --locked
```

Platform-specific GUI prerequisites may apply. Linux and macOS release support
has not been verified by this audit.

## Workspace map

See [Architecture overview](docs/ARCHITECTURE-OVERVIEW.md) for the crate map.
Choose the crate that owns the behavior; avoid implementing numerical
semantics separately in the desktop UI.

## Pick an issue and prepare a pull request

1. Search open issues and the compatibility registry to avoid duplicate work.
2. Comment on an issue before starting a broad or breaking change.
3. Create a focused branch, for example `fix/parser-call-span`.
4. Add or update tests alongside the behavior change.
5. Run the checks below and include their results in the pull request.
6. Explain user-visible changes, compatibility assumptions, and references.
7. Keep unrelated formatting or cleanup out of the pull request.

## Verification commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

These are the project commands to run; a green result must be from the
revision submitted. Do not update `Cargo.lock` as incidental cleanup.

## Tests and manifest

- Add a regression test for every bug fix and test meaningful edge cases.
- Keep synthetic fixtures small, deterministic, and legally safe.
- CI currently compares the active test count with
  `docs/test-manifest.txt`; it does not compare the names. Review the
  name-level diff before changing the manifest.
- Regenerate the listing from the repository root in PowerShell:

  ```powershell
  $current_tests = (cargo test --workspace -- --list | Select-String "^\w.*: test$") |
      ForEach-Object { $_.Line.Trim() }
  $current_tests | Set-Content -Encoding utf8 docs/test-manifest.txt
  ```

- Include a one-line reason for intentional removals. Never delete or rename a
  test just to satisfy the manifest check.

## C header

When a change affects the C API, regenerate the header using the repository's
`cbindgen.toml`:

```sh
cbindgen --config cbindgen.toml --crate mlapp-ffi --output crates/mlapp-ffi/include/mlapp.h
git diff -- crates/mlapp-ffi/include/mlapp.h
```

Review the generated header diff and include it when required. Do not hand-edit
generated declarations.

## Clean-room and AI-assisted development policy

Contributors must not paste, adapt, or prompt an AI system with code or text
from MATLAB, MathWorks binaries or files, GNU Octave, or other GPL sources.
Do not reverse engineer MathWorks binaries, P-code, or license/protection
code. Do not submit real proprietary `.mlapp` or `.slx` files, licensed
assets, or private customer data.

Base compatibility work on public documentation, mathematical definitions,
public format specifications, and permitted black-box results. Record the
source for behavior claims. AI assistance is allowed, but review all output,
disclose substantial generated contributions, and do not send private or
licensed material to an AI service. See
[AI-assisted development](docs/AI-ASSISTED-DEVELOPMENT.md).
