#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN MAINNET - INSTANT START                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# The binary WAS built successfully! It's in the workspace target directory
BINARY="/workspaces/0xv7/target/release/sultan-mainnet"

if [ -f "$BINARY" ]; then
    echo "✅ FOUND MAINNET BINARY!"
    echo "   Location: $BINARY"
    echo "   Size: $(stat -c%s "$BINARY" | numfmt --to=iec-i --suffix=B)"
    echo ""
    echo "🚀 Starting Sultan Chain Mainnet..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    RUST_LOG=info "$BINARY"
else
    # Try the sultan_minimal that already exists
    if [ -f sultan_minimal ]; then
        echo "✅ Running existing Sultan minimal node..."
        ./sultan_minimal
    else
        echo "❌ No binary found. Let's check all locations..."
        find . -name "*sultan*" -type f -executable 2>/dev/null | head -5
    fi
fi

