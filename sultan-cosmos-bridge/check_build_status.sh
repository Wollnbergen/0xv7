#!/bin/bash
# Build status checker for sultan-cosmos-bridge

echo "=== Sultan Cosmos Bridge - Build Status ==="
echo ""

# Check if cargo build is running
if ps aux | grep "cargo build" | grep -v grep > /dev/null; then
    echo "📦 Status: Building (in progress)"
    
    # Find which crate is compiling
    CURRENT_CRATE=$(ps aux | grep "c++" | grep rocksdb | head -1 | awk '{print $NF}' | xargs basename 2>/dev/null)
    if [ -n "$CURRENT_CRATE" ]; then
        echo "🔨 Current: Compiling RocksDB - $CURRENT_CRATE"
    fi
    
    echo ""
else
    echo "📦 Status: Not building"
    echo ""
fi

# Check for output files
echo "=== Expected Output Files ==="
echo ""

LIBRARY_SO="/tmp/cargo-target/release/libsultan_cosmos_bridge.so"
LIBRARY_A="/tmp/cargo-target/release/libsultan_cosmos_bridge.a"
HEADER="/workspaces/0xv7/sultan-cosmos-bridge/include/sultan_bridge.h"

if [ -f "$LIBRARY_SO" ]; then
    SIZE=$(ls -lh "$LIBRARY_SO" | awk '{print $5}')
    echo "✅ Shared library: $LIBRARY_SO ($SIZE)"
else
    echo "❌ Shared library: Not found"
fi

if [ -f "$LIBRARY_A" ]; then
    SIZE=$(ls -lh "$LIBRARY_A" | awk '{print $5}')
    echo "✅ Static library: $LIBRARY_A ($SIZE)"
else
    echo "❌ Static library: Not found"
fi

if [ -f "$HEADER" ]; then
    LINES=$(wc -l < "$HEADER")
    echo "✅ C header: $HEADER ($LINES lines)"
else
    echo "❌ C header: Not found"
fi

echo ""

# If library exists, check symbols
if [ -f "$LIBRARY_SO" ]; then
    echo "=== Exported Symbols (sample) ==="
    echo ""
    nm -D "$LIBRARY_SO" | grep sultan_ | head -10
    SYMBOL_COUNT=$(nm -D "$LIBRARY_SO" | grep sultan_ | wc -l)
    echo ""
    echo "Total sultan_* symbols: $SYMBOL_COUNT (expected: ~25+)"
    echo ""
fi

# Summary
echo "=== Summary ==="
echo ""
if [ -f "$LIBRARY_SO" ] && [ -f "$HEADER" ]; then
    echo "✅ BUILD COMPLETE - Ready for testing!"
    echo ""
    echo "Next steps:"
    echo "  1. cd /workspaces/0xv7/sultan-cosmos-bridge/go"
    echo "  2. export LD_LIBRARY_PATH=/tmp/cargo-target/release"
    echo "  3. go test -v ./bridge"
else
    echo "⏳ Build in progress - please wait"
    echo ""
    echo "Monitor with: watch -n 5 ./check_build_status.sh"
fi
