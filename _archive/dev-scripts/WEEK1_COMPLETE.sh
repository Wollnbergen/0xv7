#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - WEEK 1 TASKS COMPLETION                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📋 Running Week 1 Tasks..."
echo ""

# Task 1: Fix compilation
echo "1️⃣ Fixing compilation..."
./FIX_COMPILATION_NOW.sh
echo ""

# Task 2: Add persistence
echo "2️⃣ Adding database persistence..."
./ADD_DATABASE_PERSISTENCE.sh
echo ""

# Task 3: Create genesis
echo "3️⃣ Creating genesis block..."
./CREATE_GENESIS_BLOCK.sh
echo ""

# Check progress
echo "📊 WEEK 1 COMPLETION STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

COMPLETED=0
[ -f /workspaces/0xv7/node/src/config.rs ] && ((COMPLETED++)) && echo "✅ Config unified"
[ -f /workspaces/0xv7/node/src/persistence.rs ] && ((COMPLETED++)) && echo "✅ Persistence added"
[ -f /workspaces/0xv7/sultan-mainnet/config/genesis.json ] && ((COMPLETED++)) && echo "✅ Genesis created"

echo ""
echo "Progress: $COMPLETED/3 tasks complete"

if [ $COMPLETED -eq 3 ]; then
    echo ""
    echo "🎉 WEEK 1 COMPLETE! Ready for Week 2: P2P Networking"
else
    echo ""
    echo "⚠️ Some tasks remain. Run individual scripts to complete."
fi

