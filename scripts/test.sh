#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/engine"

echo "== cargo fmt =="
cargo fmt --all -- --check

echo "== cargo check =="
cargo check --all-targets

echo "== cargo test =="
cargo test --all-targets

echo "== cargo clippy =="
cargo clippy --all-targets --all-features -- -D warnings
