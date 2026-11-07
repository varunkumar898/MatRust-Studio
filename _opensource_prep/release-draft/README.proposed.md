# MatRust Studio

**MATLAB-style numerical computing and App Designer / Simulink tools in Rust.**

[![CI](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml/badge.svg)](https://github.com/varunkumar898/MatRust-Studio/actions/workflows/ci.yml)
[![Status: Alpha](https://img.shields.io/badge/status-alpha-orange.svg)](https://github.com/varunkumar898/MatRust-Studio)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

> **Alpha:** MATLAB compatibility is incomplete; interfaces and behavior may
> change. Check [the compatibility registry](docs/COVERAGE.md) before relying
> on a feature. Its summary and detailed rows need reconciliation before
> publishing coverage percentages.

MatRust Studio is an AI-assisted Rust workspace for MATLAB-style numerical
computing, App Designer package inspection/runtime experiments, and
Simulink-style model inspection and simulation. It is an independent project,
not affiliated with MathWorks.

## Screenshots

![Desktop application after GUI cleanup](docs/screenshots/gui-clean/after/desktop_1280x720.png)

![Sample app runtime test capture](docs/screenshots/sample_app_autotest.png)

![Sample component layout](docs/assets/gui_window_sample.png)

## What works / what does not

**Implemented areas include:**

- A parser, interpreter, numerical values, arrays, and mathematical
  builtins.
- MAT-file I/O and CSV/statistics support.
- App Designer package inspection, component modeling, callback analysis,
  runtime experiments, and an egui desktop application.
- Simulink `.slx` inspection and a simple fixed-step simulation engine.
- Command-line tools, a C ABI, Python bindings, and optional MATLAB/Octave
  bridges.

These are areas implemented in the workspace, not a promise of complete
compatibility. The registry marks `fzero`, `fminsearch`, `butter`, `filtfilt`,
`bode`, `nyquist`, `ode45`, and `ode23` as partial. Advanced MATLAB language
features and broader Simulink behavior are also incomplete. See
[docs/COVERAGE.md](docs/COVERAGE.md) and [docs/BACKLOG.md](docs/BACKLOG.md).

Repository app fixtures are synthetic and currently named
`r2018b_sample.mlapp`, `r2021a_sample.mlapp`, `r2023b_sample.mlapp`, and
`hazardous_sample.mlapp`. Real app validation is opt-in through
`MLAPP_REAL_APPS_DIR`; do not commit proprietary files.

## Quick start

Install Rust 1.88.0 or newer. `cargo run` builds the CLI before showing its
help. These commands use the same Rust CLI on each OS; desktop GUI packaging
and Linux/macOS runtime support still need release verification.

### Windows PowerShell

```powershell
git clone https://github.com/varunkumar898/MatRust-Studio.git
Set-Location MatRust-Studio
cargo run --release --locked -p mlapp-cli -- --help
```

### Linux

```sh
git clone https://github.com/varunkumar898/MatRust-Studio.git
cd MatRust-Studio
cargo run --release --locked -p mlapp-cli -- --help
```

### macOS

```sh
git clone https://github.com/varunkumar898/MatRust-Studio.git
cd MatRust-Studio
cargo run --release --locked -p mlapp-cli -- --help
```

To try the desktop shell after the CLI build:

```sh
cargo run --release --locked -p matrust_studio
```

## Architecture

- **Language and numerical core:** `rustlab-parse`, `rustlab-math`,
  `rustlab-eval`, `rustlab-stdlib`, `rustlab-data`, `rustlab-matio`.
- **App Designer:** `mlapp-container`, `mlapp-ui`, `mlapp-lang`,
  `mlapp-runtime`, `mlapp-host`, `mlapp-render`.
- **Plots and simulation:** `rustlab-plot`, `mlapp-plot`, `mlapp-sim`.
- **Interfaces and applications:** `rustlab-cli`, `mlapp-cli`, `mlapp-ffi`,
  `mlapp-py`, `mlapp-matlab`, `mlapp-octave`, `apps/desktop`.

For the crate-by-crate map, see
[docs/ARCHITECTURE-OVERVIEW.md](docs/ARCHITECTURE-OVERVIEW.md). For the full
feature table, see [docs/COVERAGE.md](docs/COVERAGE.md).

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Please include a focused test
and explain any compatibility assumptions. Read the
[AI-assisted development and clean-room policy](docs/AI-ASSISTED-DEVELOPMENT.md)
before contributing compatibility behavior or fixtures.

## License

MatRust Studio's own code is offered under either the [MIT License](LICENSE-MIT)
or [Apache License 2.0](LICENSE-APACHE), at your option. Third-party
dependencies and assets retain their own licenses.

## Trademark notice

> **NOTICE**: MatRust Studio and `mlapp-rs` are independent clean-room
> implementations developed without proprietary MathWorks code. MATLAB is a
> registered trademark of The MathWorks, Inc. This project is not affiliated
> with, sponsored by, or endorsed by The MathWorks, Inc.

## Similar project name

Another, separate project named `matrust` is maintained at
[leoom-labs/matrust](https://github.com/leoom-labs/matrust) and is licensed
under GPL-3.0. MatRust Studio is a different repository and codebase, with no
affiliation. The overlapping name may confuse users; consider a distinct
public project name before launch.

## AI-assisted development

AI tools have assisted development. Compatibility decisions and generated
changes require human review; see
[docs/AI-ASSISTED-DEVELOPMENT.md](docs/AI-ASSISTED-DEVELOPMENT.md).
