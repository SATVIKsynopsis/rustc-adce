#!/bin/bash
set -euo pipefail

RUST_DIR="${RUST_DIR:-$HOME/rust}"
RUSTC="${RUSTC:-$RUST_DIR/build/host/stage1/bin/rustc}"

if [ ! -x "$RUSTC" ]; then
    echo "Stage1 rustc not found at:"
    echo "$RUSTC"
    echo
    echo "Build the required stage1 compiler first, or set RUSTC explicitly."
    exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS="$ROOT/evaluation/corpus"

echo "Using rustc:"
"$RUSTC" --version

echo
echo "ADCE regression corpus:"
echo

for test in "$CORPUS"/*.rs; do
    name="$(basename "$test" .rs)"

    echo "== $name =="

    "$RUSTC" \
        -Zmir-opt-level=0 \
        -Zmir-enable-passes=+AdcePass \
        "$test" \
        -o /tmp/adce-"$name"

    echo "PASS: $name"
    rm -f /tmp/adce-"$name"
done

echo
echo "All ADCE corpus tests completed successfully."
