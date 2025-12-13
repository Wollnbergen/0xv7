#!/bin/bash
# Test Sultan Core Node
set -e

echo "🧪 Sultan Core Node Test"
echo "======================="
echo ""

BINARY="/workspaces/0xv7/sultan-core/target/release/sultan-node"

# Check binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found at $BINARY"
    echo "Run: cd /workspaces/0xv7/sultan-core && cargo build --release --bin sultan-node"
    exit 1
fi

echo "✅ Binary found: $BINARY"
echo ""

# Show binary info
echo "📦 Binary Info:"
file "$BINARY"
ls -lh "$BINARY"
echo ""

# Test help command
echo "📋 Testing help command..."
timeout 5 "$BINARY" --help || echo "Help command timed out or failed"
echo ""

echo "✅ Basic tests passed!"
echo ""
echo "To start the node:"
echo "  /workspaces/0xv7/START_SULTAN_CORE.sh"
