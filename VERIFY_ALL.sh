#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - COMPLETE VERIFICATION                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

SERVICES_UP=0
TOTAL_SERVICES=8

echo "🔍 Checking all services..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check each service
check_service() {
    local port=$1
    local name=$2
    local endpoint=${3:-"/"}
    
    printf "%-20s Port %-5s: " "$name" "$port"
    
    if curl -s "http://localhost:$port$endpoint" > /dev/null 2>&1; then
        echo "✅ ONLINE"
        ((SERVICES_UP++))
        return 0
    else
        echo "❌ OFFLINE"
        return 1
    fi
}

check_service 4001 "Consensus Node 1" "/consensus_state"
check_service 4002 "Consensus Node 2" "/consensus_state"
check_service 4003 "Consensus Node 3" "/consensus_state"
check_service 3000 "API Server"
check_service 3030 "RPC Server"
check_service 5001 "P2P Node 1" "/status"
check_service 5002 "P2P Node 2" "/status"
check_service 5003 "P2P Node 3" "/status"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 RESULTS:"
echo "  Services Online: $SERVICES_UP/$TOTAL_SERVICES"
echo ""

if [ $SERVICES_UP -eq $TOTAL_SERVICES ]; then
    echo "✅ SULTAN CHAIN IS FULLY OPERATIONAL!"
    echo ""
    echo "🌐 Access Points:"
    echo "  • Dashboard: $BROWSER /workspaces/0xv7/dashboard.html"
    echo "  • API: http://localhost:3000"
    echo "  • RPC: http://localhost:3030"
    echo "  • Consensus: http://localhost:4001/consensus_state"
elif [ $SERVICES_UP -ge 4 ]; then
    echo "⚠️ SULTAN CHAIN IS PARTIALLY OPERATIONAL"
    echo ""
    echo "Run this to fix: /workspaces/0xv7/FIX_ALL_SERVICES.sh"
else
    echo "❌ SULTAN CHAIN NEEDS RESTART"
    echo ""
    echo "Run: /workspaces/0xv7/FIX_ALL_SERVICES.sh"
fi

