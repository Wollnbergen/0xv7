#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        FINDING AND RUNNING SULTAN CHAIN MAINNET               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Find the sultan-mainnet binary
echo "🔍 Searching for sultan-mainnet binary..."
BINARY=$(find /workspaces/0xv7 -name "sultan-mainnet" -type f -executable 2>/dev/null | head -1)

if [ -n "$BINARY" ]; then
    echo "✅ Found binary at: $BINARY"
    ls -lah "$BINARY"
    echo ""
    echo "🚀 Starting Sultan Chain Mainnet..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    RUST_LOG=info "$BINARY"
else
    echo "❌ Binary not found. Let's build it properly..."
    echo ""
    
    # Build sultan-mainnet directly from workspace
    cd /workspaces/0xv7
    echo "🔨 Building sultan-mainnet from workspace..."
    cargo build -p sultan-mainnet --release 2>&1 | tail -5
    
    # Check again
    BINARY=$(find /workspaces/0xv7 -name "sultan-mainnet" -type f -executable 2>/dev/null | head -1)
    if [ -n "$BINARY" ]; then
        echo ""
        echo "✅ Build successful! Binary at: $BINARY"
        echo ""
        echo "🚀 Starting Sultan Chain Mainnet..."
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        RUST_LOG=info "$BINARY"
    else
        echo "⚠️ Still not found. Creating minimal version..."
        ./CREATE_MINIMAL_SULTAN.sh
    fi
fi

