#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        STARTING COMPLETE SULTAN CHAIN SYSTEM                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Start the web dashboard
echo "🌐 [1/3] Starting Web Dashboard..."
pkill -f "python3 -m http.server" 2>/dev/null
cd /workspaces/0xv7/public && python3 -m http.server 3000 > /tmp/web.log 2>&1 &
echo "   ✅ Dashboard: http://localhost:3000"

# 2. Check if node binary exists
echo ""
echo "⛓️  [2/3] Checking Blockchain Node..."
if [ -f "/workspaces/0xv7/node/target/release/sultan_node" ]; then
    echo "   ✅ Binary found and ready"
    echo "   Run separately: /workspaces/0xv7/node/target/release/sultan_node"
else
    echo "   ⚠️  Binary not found. Building..."
    cd /workspaces/0xv7/node && cargo build --release --bin sultan_node
fi

# 3. Open in browser
echo ""
echo "🚀 [3/3] Opening Dashboard..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$BROWSER" http://localhost:3000

echo ""
echo "✅ SYSTEM STATUS:"
echo "   Web Dashboard: RUNNING"
echo "   Node Binary: $([ -f '/workspaces/0xv7/node/target/release/sultan_node' ] && echo 'READY' || echo 'NOT BUILT')"
echo ""
echo "To run the blockchain node:"
echo "   /workspaces/0xv7/node/target/release/sultan_node"

