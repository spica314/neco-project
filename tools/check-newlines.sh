#!/bin/bash

set -e

# Check for trailing newlines in all text files
echo "Checking for trailing newlines..."

# Find all text files (excluding binary files and build artifacts)
find . -type f \
  -not -path './target/*' \
  -not -path './.git/*' \
  \( -name '*.rs' -o -name '*.toml' -o -name '*.md' -o -name '*.sh' -o -name '*.fe' \) \
  -print0 | while IFS= read -r -d '' file; do

    # Check if file ends with newline
    if [[ -s "$file" ]] && [[ $(tail -c1 "$file" | wc -l) -eq 0 ]]; then
        echo "ERROR: File '$file' does not end with a newline"
        exit 1
    fi
done

echo "All files have proper trailing newlines!"
