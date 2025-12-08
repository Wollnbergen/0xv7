#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - ONE-CLICK START                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Run the fix script
/workspaces/0xv7/FIX_ALL_SERVICES.sh

# Wait for services to stabilize
sleep 3

# Verify everything is running
/workspaces/0xv7/VERIFY_ALL.sh

# Open dashboard
echo ""
echo "🌐 Opening dashboard in browser..."
"$BROWSER" /workspaces/0xv7/dashboard.html

echo ""
echo "✅ Sultan Chain is ready!"
echo ""
echo "📝 Quick Commands:"
echo "  • Monitor: /workspaces/0xv7/MONITOR_BLOCKCHAIN.sh"
echo "  • Test: /workspaces/0xv7/TEST_ALL_ENDPOINTS.sh"
echo "  • Dashboard: $BROWSER /workspaces/0xv7/dashboard.html"

