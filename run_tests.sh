#!/bin/bash
set -e
cd "$(dirname "$0")/nexus-core"
source "$HOME/.cargo/env"

# Context-efficient test runner for human-as-MCP workflow.
# Only shows: build warnings, test failures, and a summary line.
# Pass --verbose or -v to see all test names.

VERBOSE=false
for arg in "$@"; do
    case "$arg" in
        -v|--verbose) VERBOSE=true ;;
    esac
done

echo "=== Building ==="
cargo build 2>&1

echo ""
echo "=== Running ALL tests ==="

if $VERBOSE; then
    cargo test 2>&1
else
    # Capture full output, filter to only useful information
    OUTPUT=$(cargo test 2>&1)
    
    # Show any compiler warnings (lines with "warning:")
    echo "$OUTPUT" | grep -E "^warning" || true
    
    # Show any failures
    FAILURES=$(echo "$OUTPUT" | grep -E "FAILED|panicked|failures:" || true)
    if [ -n "$FAILURES" ]; then
        echo ""
        echo "=== FAILURES ==="
        # Show the detailed failure output
        echo "$OUTPUT" | sed -n '/^---- .* ----$/,/^$/p'
        echo "$OUTPUT" | grep -E "^failures:" || true
        echo ""
    fi
    
    # Show the summary lines (test result:)
    echo "$OUTPUT" | grep "^test result:" || true
fi

echo ""
echo "=== Done ==="
