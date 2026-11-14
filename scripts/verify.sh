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

echo "=== 7. Checking documentation, privacy, and link integrity ==="
if command -v python3 >/dev/null 2>&1; then
    python3 scripts/check_docs.py
elif command -v python >/dev/null 2>&1; then
    python scripts/check_docs.py
else
    echo "Warning: Python not found, skipping check_docs.py" >&2
fi

echo "=== 8. Checking layout regression gate (Wave 0 Task L4) ==="
if command -v python3 >/dev/null 2>&1; then
    python3 scripts/check_layout_gate.py
elif command -v python >/dev/null 2>&1; then
    python scripts/check_layout_gate.py
else
    echo "Warning: Python not found, skipping check_layout_gate.py" >&2
fi

echo "=== All verification steps passed successfully! ==="
