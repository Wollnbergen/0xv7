#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          CHECKING SULTAN COSMOS SDK STRUCTURE                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sultan

# Check if it's a proper Cosmos chain
echo "📂 Sultan Cosmos structure:"
ls -la | head -10

echo ""
echo "📄 Checking go.mod..."
if [ -f "go.mod" ]; then
    head -5 go.mod
else
    echo "❌ go.mod not found"
fi

echo ""
echo "🔍 Checking for x/ modules..."
if [ -d "x" ]; then
    ls -la x/
else
    echo "No x/ modules directory found"
fi

echo ""
echo "🔨 Attempting to build Sultan Cosmos chain..."
if [ -f "Makefile" ]; then
    make build 2>&1 | grep -E "go:|error:|Error:" | head -10
fi

echo ""
echo "📊 Sultan Chain Status:"
echo "  • Cosmos SDK: $([ -f "go.mod" ] && echo "✅ Initialized" || echo "❌ Not initialized")"
echo "  • Custom Modules: $([ -d "x" ] && echo "✅ Present" || echo "⚠️ Need to create")"
echo "  • Build Status: $([ -f "build/sultand" ] && echo "✅ Built" || echo "⚠️ Not built")"
