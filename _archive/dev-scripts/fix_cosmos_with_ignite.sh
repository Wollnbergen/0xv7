#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          FIXING COSMOS BUILD WITH IGNITE CLI                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sultan

# Use Ignite to fix the build
echo "🔧 Using Ignite to fix build issues..."
ignite chain build --clear-cache 2>&1 | grep -E "✓|✗|error" | head -20

# If that doesn't work, try serving (which includes auto-fix)
echo ""
echo "🚀 Starting Sultan chain with Ignite..."
timeout 30 ignite chain serve --reset-once 2>&1 | grep -E "✓|✗|Blockchain is running|error" | head -20

echo ""
echo "📊 Build Status:"
if [ -f "build/sultand" ]; then
    echo "✅ Sultan binary built successfully!"
    ls -lh build/sultand
else
    echo "⚠️ Build incomplete - checking for alternative binaries..."
    find . -name "sultand" -type f 2>/dev/null | head -5
fi
