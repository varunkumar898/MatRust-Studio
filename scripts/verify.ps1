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

Write-Host "=== 7. Verifying test manifest integrity ==="
$current_tests = (cargo test --workspace -- --list | Select-String "^\w.*: test$") | ForEach-Object { $_.Line.Trim() }
$manifest_tests = (Get-Content docs/test-manifest.txt | Select-String ": test$") | ForEach-Object { $_.Line.Trim() }

$missing = @()
foreach ($t in $manifest_tests) {
    if ($current_tests -notcontains $t) {
        $missing += $t
    }
}
if ($missing.Count -gt 0) {
    Write-Error "Test manifest integrity failure! The following tests disappeared: $($missing -join ', ')"
    exit 1
}
Write-Host "Test manifest check passed: all $($manifest_tests.Count) manifest entries are present."

Write-Host "=== All verification steps passed successfully! ==="
