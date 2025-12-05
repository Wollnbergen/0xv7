#!/bin/bash
# Sultan L1 - Clean Shutdown Script
# Safely stops all Sultan L1 processes

echo "════════════════════════════════════════════════════════════"
echo "         🛑 Shutting Down Sultan L1 🛑                      "
echo "════════════════════════════════════════════════════════════"
echo ""

# Stop node
if [ -f /workspaces/0xv7/.sultan-node.pid ]; then
    NODE_PID=$(cat /workspaces/0xv7/.sultan-node.pid)
    if ps -p $NODE_PID > /dev/null 2>&1; then
        echo "🛑 Stopping Sultan node (PID: $NODE_PID)..."
        kill -TERM $NODE_PID
        sleep 2
        
        # Force kill if still running
        if ps -p $NODE_PID > /dev/null 2>&1; then
            echo "   Force stopping..."
            kill -9 $NODE_PID
        fi
        echo "✅ Node stopped"
    else
        echo "ℹ️  Node not running"
    fi
    rm -f /workspaces/0xv7/.sultan-node.pid
else
    echo "ℹ️  No node PID file found"
    pkill -9 sultan-node 2>/dev/null && echo "✅ Killed stray sultan-node processes" || true
fi

# Stop website
if [ -f /workspaces/0xv7/.sultan-website.pid ]; then
    WEBSITE_PID=$(cat /workspaces/0xv7/.sultan-website.pid)
    if ps -p $WEBSITE_PID > /dev/null 2>&1; then
        echo "🛑 Stopping web dashboard (PID: $WEBSITE_PID)..."
        kill -TERM $WEBSITE_PID
        echo "✅ Website stopped"
    else
        echo "ℹ️  Website not running"
    fi
    rm -f /workspaces/0xv7/.sultan-website.pid
else
    echo "ℹ️  No website PID file found"
    pkill -9 -f "python3 -m http.server 8080" 2>/dev/null && echo "✅ Killed stray website processes" || true
fi

# Clean up any orphaned processes
echo ""
echo "🧹 Cleaning up orphaned processes..."
pkill -9 sultan-node 2>/dev/null && echo "   Killed additional sultan-node processes" || echo "   No additional processes found"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "         ✅ Sultan L1 Shutdown Complete ✅                  "
echo "════════════════════════════════════════════════════════════"
echo ""
echo "💾 Data preserved in: sultan-data/"
echo "📝 Logs available in: sultan-node.log, website.log"
echo ""
echo "🚀 To restart: bash start-sultan.sh"
echo ""
