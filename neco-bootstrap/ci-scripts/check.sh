#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Check if --fix option is provided
FIX_MODE=false
if [[ "$1" == "--fix" ]]; then
    FIX_MODE=true
fi

cd "$PROJECT_ROOT/neco-bootstrap"

if $FIX_MODE; then
    echo "=== Starting Rust automatic fixes ==="
    
    # Fix formatting
    echo "Fixing Rust formatting..."
    cargo fmt --all
    
    # Fix clippy warnings (if possible)
    echo "Attempting to fix clippy warnings..."
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged || true
fi

# Check formatting
echo "Checking Rust formatting..."
cargo fmt --all -- --check

# Run clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "Running Rust tests..."
if command -v nvidia-smi >/dev/null 2>&1 && \
   nvidia-smi -L 2>/dev/null | grep -qE '^GPU [0-9]+'; then
    cargo test --workspace --offline --features neco-felis-compile/has-ptx-device
else
    cargo test --workspace --offline
fi
