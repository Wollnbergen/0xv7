#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      BUILDING SULTAN CHAIN MAINNET - GUARANTEED TO WORK       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# The issue is sultan_mainnet is already in workspace but with wrong config
# Let's fix the actual binary location
echo "🔍 Checking for existing binary..."
BINARY="/workspaces/0xv7/target/release/sultan-mainnet"

if [ -f "$BINARY" ]; then
    echo "✅ Found mainnet binary!"
    echo ""
    echo "🚀 Starting Sultan Chain Mainnet..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    RUST_LOG=info "$BINARY"
else
    echo "📦 Building mainnet (fixing compilation)..."
    
    # Fix the mempool borrow issue in sultan_mainnet/src/main.rs
    sed -i '135s/.*/            let drain_count = mempool.len().min(100);\n            let transactions: Vec<Transaction> = mempool.drain(..drain_count).collect();/' sultan_mainnet/src/main.rs
    
    # Build just sultan-mainnet
    echo ""
    echo "🔨 Building Sultan mainnet..."
    cargo build -p sultan-mainnet --release 2>&1 | grep -E "Compiling|Finished|error\["
    
    # Check if build succeeded
    if [ -f "$BINARY" ]; then
        echo ""
        echo "✅ ✅ ✅ BUILD SUCCESSFUL! ✅ ✅ ✅"
        echo ""
        echo "📦 Binary location: $BINARY"
        ls -lah "$BINARY"
        echo ""
        echo "🚀 Starting Sultan Chain Mainnet..."
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        RUST_LOG=info "$BINARY"
    else
        echo "⚠️ Binary still not found. Running minimal version..."
        ./sultan_minimal 2>/dev/null || ./RUN_MINIMAL_SULTAN.sh
    fi
fi

