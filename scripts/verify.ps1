$ErrorActionPreference = "Stop"

# Always run from the rustlab workspace root
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== 1. Checking formatting ==="
cargo fmt --all -- --check

Write-Host "=== 2. Running clippy (all features) ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

Write-Host "=== 3. Running clippy (workspace no default features) ==="
cargo clippy --workspace --all-targets --no-default-features -- -D warnings

Write-Host "=== 4. Running clippy (per-crate truly without default features) ==="
$feature_crates = @("rustlab-stdlib", "rustlab-math", "rustlab-cli", "matrust_studio")
foreach ($crate in $feature_crates) {
    Write-Host "--- Clippy: $crate (--no-default-features) ---"
    cargo clippy -p $crate --no-default-features --all-targets -- -D warnings
}

Write-Host "=== 5. Running tests (workspace default features) ==="
cargo test --workspace

Write-Host "=== 6. Running tests (workspace all features) ==="
cargo test --workspace --all-features

Write-Host "=== 7. Running tests (workspace no default features) ==="
cargo test --workspace --no-default-features

Write-Host "=== 8. Running tests (per-crate truly without default features) ==="
foreach ($crate in $feature_crates) {
    Write-Host "--- Tests: $crate (--no-default-features) ---"
    cargo test -p $crate --no-default-features
}

Write-Host "=== 9. Verifying test manifest integrity & feature-gated tests ==="
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

# Also verify that the feature-gated tests are present in rustlab-stdlib per-crate listings
$stdlib_def_tests = (cargo test -p rustlab-stdlib -- --list | Select-String ": test$") | ForEach-Object { $_.Line.Trim() }
$stdlib_nodef_tests = (cargo test -p rustlab-stdlib --no-default-features -- --list | Select-String ": test$") | ForEach-Object { $_.Line.Trim() }

$required_faer_tests = @(
    "test_a2_symmetric_eig_routes_to_faer_and_canonicalizes: test",
    "test_a2_golden_eig_rotation_complex_conjugate: test",
    "test_a2_both_native_and_faer_eig_paths_match: test"
)
foreach ($t in $required_faer_tests) {
    if ($stdlib_def_tests -notcontains $t) {
        Write-Error "Feature gate check failure! Expected feature-gated test '$t' in rustlab-stdlib default features, but it was not found!"
        exit 1
    }
    if ($stdlib_nodef_tests -contains $t) {
        Write-Error "Feature gate check failure! Feature-gated test '$t' unexpectedly appeared in rustlab-stdlib --no-default-features!"
        exit 1
    }
}
Write-Host "Per-crate feature gate check passed: all feature-gated tests correctly gated."

Write-Host "=== All verification steps passed successfully! ==="
