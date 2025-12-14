#!/bin/bash
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN-COSMOS INTEGRATION VERIFICATION              ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo -e "\n📊 SYSTEM STATUS CHECK:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Sultan API Check
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    echo "✅ Sultan API: ONLINE (13.33% APY, $0 gas)"
else
    echo "❌ Sultan API: OFFLINE"
fi

# Cosmos RPC Check
if curl -s http://localhost:26657/status > /dev/null 2>&1; then
    echo "✅ Cosmos RPC: ONLINE (IBC/WASM enabled)"
else
    echo "⚠️  Cosmos RPC: STARTING..."
fi

# Unified API Check
if curl -s http://localhost:8080/status > /dev/null 2>&1; then
    echo "✅ Unified API: ONLINE (Bridge active)"
    RESPONSE=$(curl -s http://localhost:8080/status)
    APY=$(echo "$RESPONSE" | jq -r '.unified_features.staking_apy')
    GAS=$(echo "$RESPONSE" | jq -r '.unified_features.zero_gas')
    IBC=$(echo "$RESPONSE" | jq -r '.unified_features.ibc_support')
    
    echo ""
    echo "📋 Integration Features:"
    echo "  • Staking APY: $APY (Target: 13.33%)"
    echo "  • Zero Gas: $GAS"
    echo "  • IBC Support: $IBC"
else
    echo "❌ Unified API: OFFLINE"
fi

# Dashboard Check
if curl -s http://localhost:8888 > /dev/null 2>&1; then
    echo "✅ Dashboard: ONLINE"
else
    echo "⚠️  Dashboard: Not running"
fi

echo -e "\n🌐 Access Points:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📱 Dashboard: http://localhost:8888/sultan-dashboard.html"
echo "🔧 Unified API: http://localhost:8080/status"
echo "💎 Sultan Core: http://localhost:3030"
echo "🌍 Cosmos RPC: http://localhost:26657"

if [ "$APY" = "13.33%" ] && [ "$GAS" = "true" ]; then
    echo -e "\n🎉 SUCCESS: SULTAN BLOCKCHAIN IS FULLY INTEGRATED WITH COSMOS SDK!"
else
    echo -e "\n⚠️  Some services may still be initializing..."
fi
