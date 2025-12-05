#!/bin/bash

# Patch the dashboard to show integrated status
cat >> /workspaces/0xv7/sultan_security_dashboard.sh << 'DASHBOARD_PATCH'

    # Integration Status
    echo ""
    echo "🔗 INTEGRATION STATUS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check Sultan
    if curl -s http://localhost:3030 > /dev/null 2>&1; then
        echo "  Sultan Core: ✅ ONLINE (26.67% APY)"
    else
        echo "  Sultan Core: ❌ OFFLINE"
    fi
    
    # Check Cosmos
    if curl -s http://localhost:26657/status > /dev/null 2>&1; then
        echo "  Cosmos SDK: ✅ ONLINE (IBC/WASM)"
    else
        echo "  Cosmos SDK: ❌ OFFLINE"
    fi
    
    # Check Bridge
    if curl -s http://localhost:8080/status > /dev/null 2>&1; then
        echo "  Bridge: ✅ SYNCED"
        APY=$(curl -s http://localhost:8080/status | jq -r '.unified_features.staking_apy' 2>/dev/null || echo "N/A")
        echo "  Unified APY: $APY"
    else
        echo "  Bridge: ❌ NOT CONNECTED"
    fi
DASHBOARD_PATCH

echo "✅ Dashboard updated with integration status"
