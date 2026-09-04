#!/bin/bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <mir-file>"
    exit 1
fi

file="$1"

blocks=$(grep -cE '^bb[0-9]+:' "$file" || true)
statements=$(grep -cE '^[[:space:]]+_?[0-9]+(_[0-9]+)?[[:space:]]*=' "$file" || true)

echo "Basic blocks: $blocks"
echo "Statements:   $statements"
