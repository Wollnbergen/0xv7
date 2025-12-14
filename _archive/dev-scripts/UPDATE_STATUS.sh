#!/bin/bash

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      SULTAN CHAIN - COMPREHENSIVE STATUS CHECK                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Function to check with color
check_status() {
    if eval "$2"; then
        echo -e "  ✅ $1"
        return 0
    else
        echo -e "  ❌ $1"
        return 1
    fi
}

echo "🌐 TESTNET STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check_status "API Running (port 3030)" "curl -s http://localhost:3030 > /dev/null 2>&1"
check_status "Public Access" "curl -s https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/ > /dev/null 2>&1"
echo ""

echo "🗄️ DATABASE STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check_status "ScyllaDB Container" "docker ps | grep -q scylla"
check_status "Port 9042 Open" "nc -zv localhost 9042 2>&1 | grep -q succeeded"
echo ""

echo "📦 BUILD STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check_status "Main Node Binary" "[ -f /workspaces/0xv7/node/target/release/sultan_node ]"
check_status "Minimal Node Binary" "[ -f /workspaces/0xv7/sultan_node_minimal/target/release/sultan-node ]"
echo ""

echo "📊 QUICK ACTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Fix compilation:     ./FIX_ALL_COMPILATION.sh"
echo "2. Create minimal node: ./CREATE_MINIMAL_NODE.sh"
echo "3. Test testnet API:    curl -X POST http://localhost:3030 -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_apy\",\"id\":1}' | jq"
echo "4. Open testnet:        $BROWSER https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
echo ""

# Test the API
echo "🧪 TESTNET API TEST:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
RESPONSE=$(curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"get_apy","id":1}' 2>/dev/null)

if [ ! -z "$RESPONSE" ]; then
    echo "$RESPONSE" | jq '.result' 2>/dev/null || echo "$RESPONSE"
else
    echo "❌ API not responding"
fi

