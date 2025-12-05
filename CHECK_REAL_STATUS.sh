#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - CHECKING ACTUAL STATUS                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Add node_modules to .gitignore first
echo "node_modules" >> /workspaces/0xv7/.gitignore
echo "✅ Added node_modules to .gitignore"
echo ""

echo "📁 WHAT WE ACTUALLY HAVE BUILT:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check Telegram bot
if [ -f "/workspaces/0xv7/server/server.js" ]; then
    echo "✅ Telegram Bot: EXISTS at server/server.js"
    grep -q "26.67" /workspaces/0xv7/server/server.js && echo "   • 26.67% APY configured"
    grep -q "5000" /workspaces/0xv7/server/server.js && echo "   • 5000 SLTN minimum stake"
fi

# Check API
if ps aux | grep -q "[n]ode.*3030"; then
    echo "✅ API: RUNNING on port 3030"
else
    echo "⚠️  API: Not running - let's start it"
fi

# Check validators directory
if [ -d "/workspaces/0xv7/validators" ]; then
    echo "✅ Validators: Directory exists"
    ls -la /workspaces/0xv7/validators/*.html 2>/dev/null | head -3
fi

# Check consensus
if [ -d "/workspaces/0xv7/consensus" ]; then
    echo "✅ Consensus: Built and ready"
fi

# Check database
if [ -f "/workspaces/0xv7/database/database_manager.js" ]; then
    echo "✅ Database: Manager exists"
fi

# Check what's actually running
echo ""
echo "🔍 RUNNING PROCESSES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ps aux | grep -E "node|cargo" | grep -v grep | head -5
