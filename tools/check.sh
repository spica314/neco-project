#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Check newlines
echo "Checking newlines..."
"$SCRIPT_DIR/check-newlines.sh"

# Check file sizes
echo "Checking file sizes..."
"$SCRIPT_DIR/check-file-size.sh"

# Check formatting
echo "Checking formatting..."
cargo fmt --all -- --check

# Run clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "Running tests..."
cargo test --workspace --offline

echo "All checks passed!"
