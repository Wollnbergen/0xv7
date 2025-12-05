#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          CHECKING SULTAN CHAIN BUILD STATUS                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Check if the binary was actually built
echo "🔍 Checking for compiled binary..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "target/release/sultan_node" ]; then
    echo "✅ Binary found: target/release/sultan_node"
    echo "   Size: $(ls -lh target/release/sultan_node | awk '{print $5}')"
    echo "   Modified: $(ls -lh target/release/sultan_node | awk '{print $6, $7, $8}')"
else
    echo "❌ No binary found"
fi

echo ""
echo "📝 Checking last build attempt..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "build.log" ]; then
    echo "Last build errors/warnings:"
    grep -E "error\[|warning:" build.log | head -10 || echo "No errors in log"
else
    echo "No build.log found"
fi

echo ""
echo "🔧 Attempting quick compilation check..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo check --quiet 2>&1 | head -20

