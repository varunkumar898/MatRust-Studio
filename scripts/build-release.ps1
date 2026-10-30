$ErrorActionPreference = "Stop"

# G0.1-VERIFY step 2: reproducible release rebuild.
# Release profile (Cargo.toml): codegen-units = 1, strip = true, plus
# opt-level = 3, lto = "fat", panic = "abort" carried over from G0.1.
# --remap-path-prefix is NOT applied here: on this toolchain (Cargo 1.99.1,
# arm64-pc-windows-gnullvm) the profile key is rejected and setting RUSTFLAGS
# breaks Windows-system-library linking. See
# docs/milestones/g0.1-verify.md for the exact errors.
Set-Location (Join-Path $PSScriptRoot "..")
cargo build --release --locked
exit $LASTEXITCODE