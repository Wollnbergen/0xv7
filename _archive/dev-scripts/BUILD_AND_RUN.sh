#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         BUILDING AND RUNNING SULTAN CHAIN NODE                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

echo "🔨 Building Sultan Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build the node
if cargo build --release --bin sultan_node 2>&1 | tee build.log | grep -E "error|warning" | head -10; then
    echo ""
    echo "⚠️  Build completed with warnings"
else
    echo "✅ Build successful!"
fi

# Check if binary exists
if [ -f "target/release/sultan_node" ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🚀 RUNNING SULTAN CHAIN NODE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    ./target/release/sultan_node
else
    echo ""
    echo "❌ Build failed. Checking errors..."
    tail -20 build.log
fi

