#!/usr/bin/env bash
set -e

# Always run from the rustlab workspace root
cd "$(dirname "$0")/.."

echo "=== 1. Checking formatting ==="
cargo fmt --all -- --check

echo "=== 2. Running clippy (all features) ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "=== 3. Running clippy (no default features) ==="
cargo clippy --workspace --all-targets --no-default-features -- -D warnings

echo "=== 4. Running tests (default features) ==="
cargo test --workspace

echo "=== 5. Running tests (all features) ==="
cargo test --workspace --all-features

echo "=== 6. Running tests (no default features) ==="
cargo test --workspace --no-default-features

echo "=== All verification steps passed successfully! ==="
