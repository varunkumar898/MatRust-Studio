$ErrorActionPreference = "Stop"

# Always run from the rustlab workspace root
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== 1. Checking formatting ==="
cargo fmt --all -- --check

Write-Host "=== 2. Running clippy (all features) ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

Write-Host "=== 3. Running clippy (no default features) ==="
cargo clippy --workspace --all-targets --no-default-features -- -D warnings

Write-Host "=== 4. Running tests (default features) ==="
cargo test --workspace

Write-Host "=== 5. Running tests (all features) ==="
cargo test --workspace --all-features

Write-Host "=== 6. Running tests (no default features) ==="
cargo test --workspace --no-default-features

Write-Host "=== All verification steps passed successfully! ==="
