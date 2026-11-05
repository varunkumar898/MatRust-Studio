# Roadmap

This is a set of project themes, not a delivery schedule. Compatibility and implementation status are tracked in [docs/COVERAGE.md](docs/COVERAGE.md).

## Developer support

- Improve parser and runtime diagnostics with clearer source locations and actionable explanations.
- Add a MATLAB-style static analyzer that flags likely bugs without executing scripts.
- Add typo-based "did you mean?" suggestions and safe suggested fixes.
- Build debugger support for breakpoints, stepping, and variable inspection.
- Explore a language server (LSP) for diagnostics, navigation, and completion.

## Simulink-style simulation

- Add RK4 and variable-step solver options alongside the current simple fixed-step engine.
- Expand the supported block set and test parameter parsing and signal behavior.
- Add subsystem execution and clearer signal source, sink, and scope reporting.
- Publish `.slx` compatibility coverage by block type and report unsupported blocks.

## MATLAB compatibility

- Improve coverage from the registry baseline and turn partial rows into explicit, tested contracts.
- Add useful `MException` properties and improve `containers.Map` support.
- Expand `classdef` and object-oriented language support in small, documented steps.

## Platforms and packaging

- Produce tested release artifacts for Windows, Linux, and macOS.
- Document installation options such as Homebrew, Scoop, and AppImage where maintainers can support them.
- Automate versioned release builds, checksums, and release notes.

## Real-app compatibility testing

- Improve the opt-in `MLAPP_REAL_APPS_DIR` workflow for user-authorized external apps.
- Report tested features and unsupported constructs without committing real app files.
- Add clear privacy and clean-room guidance for privately supplied test inputs.

## Documentation and tutorials

- Add beginner-friendly numerical computing and interpreter tutorials.
- Document workspace architecture, supported compatibility behavior, and known limitations.
- Add examples that run in CI and publish their actual expected output.
- Explain how to create synthetic `.mlapp` and `.slx` fixtures.
