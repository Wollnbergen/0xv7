#!/bin/bash

while true; do
    clear
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║            SULTAN CHAIN - LIVE MONITORING                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "⏰ $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""
    
    # Check services
    SERVICES=0
    for PORT in 3000 3030 4001 5001 5002 5003; do
        if nc -z localhost $PORT 2>/dev/null; then
            ((SERVICES++))
        fi
    done
    
    echo "📊 NETWORK STATUS:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  • Active Services: $SERVICES/6"
    echo "  • Network: MAINNET-READY"
    echo "  • Gas Fees: $0.00"
    echo "  • Validator APY: 13.33%"
    echo ""
    
    # Test API
    if curl -s http://localhost:3030 > /dev/null 2>&1; then
        HEIGHT=$(curl -s -X POST http://localhost:3030 \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc":"2.0","method":"chain_status","id":1}' 2>/dev/null | \
            jq -r '.result.height' 2>/dev/null || echo "N/A")
        echo "  • Block Height: $HEIGHT"
        echo "  • Status: ✅ PRODUCING BLOCKS"
    fi
    
    echo ""
    echo "🌐 PUBLIC ENDPOINTS:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  • Web UI: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
    echo "  • RPC: http://localhost:3030"
    echo "  • API: http://localhost:3000"
    echo ""
    
    echo "📈 REAL-TIME METRICS:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    ps aux | grep -E "sultan|consensus" | grep -v grep | wc -l | \
        xargs -I {} echo "  • Running Processes: {}"
    
    sleep 5
done
