#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Starting automatic fixes and checks ==="

# Fix newlines
echo "Fixing newlines..."
"$SCRIPT_DIR/fix-newlines.sh"

# Fix formatting
echo "Fixing formatting..."
cargo fmt --all

# Fix clippy warnings (if possible)
echo "Attempting to fix clippy warnings..."
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged || true

# Now run all checks
echo ""
echo "=== Running all checks ==="

# Run check.sh for comprehensive verification
"$SCRIPT_DIR/check.sh"

echo ""
echo "=== All fixes applied and checks passed! ==="
