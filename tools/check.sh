#!/bin/bash

set -e

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
