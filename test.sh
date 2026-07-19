#!/usr/bin/env bash
set -euo pipefail

cargo_args=()
if [[ $# -gt 0 ]]; then
    cargo_args+=(--release)
fi

echo "=== rustfmt ==="
cargo fmt --check

echo "=== Compile ==="
cargo check "${cargo_args[@]}"

echo "=== Clippy ==="
cargo clippy "${cargo_args[@]}" -- -D warnings

echo "=== Tests ==="
cargo test "${cargo_args[@]}"

echo ""
echo "All checks passed."
