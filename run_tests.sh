#!/bin/bash
set -e
cd "$(dirname "$0")/nexus-core"
source "$HOME/.cargo/env"

echo "=== Building ==="
cargo build 2>&1

echo ""
echo "=== Running ALL tests ==="
cargo test 2>&1

echo ""
echo "=== Done ==="
