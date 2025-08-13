#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Find all text files and add newline if missing
find . \
    -type f \
    \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.sh" -o -name "*.yml" -o -name "*.yaml" \) \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    -exec sh -c 'if [ -s "$1" ] && [ "$(tail -c 1 "$1" | wc -l)" -eq 0 ]; then echo >> "$1"; echo "Fixed: $1"; fi' _ {} \;

echo "Newline fixes completed"
