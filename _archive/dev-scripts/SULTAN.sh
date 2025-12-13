#!/bin/bash

# Quick launcher for Sultan Chain

echo "⚡ SULTAN CHAIN - Quick Access"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "[1] 🎛️  Open Manager"
echo "[2] 📊 View Dashboard" 
echo "[3] 📈 Show Statistics"
echo "[4] 🔍 Test Services"
echo ""
echo -n "Select [1-4]: "
read -n 1 choice
echo ""

case $choice in
    1) /workspaces/0xv7/SULTAN_MANAGER.sh ;;
    2) "$BROWSER" /workspaces/0xv7/production_dashboard.html ;;
    3) /workspaces/0xv7/BLOCKCHAIN_STATS.sh ;;
    4) /workspaces/0xv7/TEST_CHAIN.sh ;;
    *) echo "Invalid choice" ;;
esac

