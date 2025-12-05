#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - MAINNET READINESS TEST                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Function to check component
check_component() {
    local name="$1"
    local cmd="$2"
    
    if eval "$cmd" > /dev/null 2>&1; then
        echo "✅ $name: READY"
        return 0
    else
        echo "❌ $name: NOT READY"
        return 1
    fi
}

echo "🔍 Checking Core Components..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check compilation
check_component "Rust Compilation" "cd /workspaces/0xv7/node && cargo check"

# Check database
check_component "ScyllaDB Connection" "docker exec scylla cqlsh -e 'SELECT now() FROM system.local'"

# Check Redis
check_component "Redis Connection" "docker exec redis redis-cli ping"

# Check API
check_component "RPC API" "curl -s http://localhost:3030"

echo ""
echo "🔍 Checking Production Features..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test zero fees
echo -n "Testing zero-fee transactions... "
RESULT=$(curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"token_transfer","params":["test","user",100],"id":1}' \
  | jq -r '.result.fee')
  
if [ "$RESULT" = "0" ]; then
    echo "✅ WORKING"
else
    echo "❌ FAILED"
fi

# Test APY calculation
echo -n "Testing APY calculations... "
APY=$(curl -s -X POST http://localhost:3030 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"get_apy","id":1}' \
  | jq -r '.result.base_apy')
  
if [ "$APY" = "26.67%" ]; then
    echo "✅ CORRECT"
else
    echo "❌ INCORRECT"
fi

echo ""
echo "🔍 Checking Security..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check JWT authentication
echo -n "JWT Authentication: "
if grep -q "JWT_SECRET" /workspaces/0xv7/node/src/rpc_server.rs 2>/dev/null; then
    echo "✅ IMPLEMENTED"
else
    echo "❌ NOT IMPLEMENTED"
fi

# Check rate limiting
echo -n "Rate Limiting: "
if grep -q "rate_limit" /workspaces/0xv7/node/src/rpc_server.rs 2>/dev/null; then
    echo "✅ IMPLEMENTED"
else
    echo "⚠️ NOT IMPLEMENTED (recommended)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 MAINNET READINESS SCORE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

READY_COUNT=0
TOTAL_COUNT=10

# Calculate score
[ -f /workspaces/0xv7/node/target/release/sultan-node ] && ((READY_COUNT++))
docker ps | grep -q scylla && ((READY_COUNT++))
docker ps | grep -q redis && ((READY_COUNT++))
curl -s http://localhost:3030 > /dev/null 2>&1 && ((READY_COUNT++))
[ "$RESULT" = "0" ] && ((READY_COUNT++))
[ "$APY" = "26.67%" ] && ((READY_COUNT++))

PERCENTAGE=$((READY_COUNT * 100 / TOTAL_COUNT))

echo ""
echo "Score: $READY_COUNT/$TOTAL_COUNT ($PERCENTAGE%)"
echo ""

if [ $PERCENTAGE -ge 80 ]; then
    echo "🎉 STATUS: READY FOR MAINNET DEPLOYMENT"
    echo "   Next: Run security audit and launch!"
elif [ $PERCENTAGE -ge 60 ]; then
    echo "⚠️ STATUS: ALMOST READY"
    echo "   Need: Fix compilation and test multi-node"
else
    echo "❌ STATUS: NOT READY FOR MAINNET"
    echo "   Need: Significant development work"
fi

echo ""
echo "📋 Recommended Actions:"
echo "  1. Fix any ❌ items above"
echo "  2. Deploy 3+ validator nodes"
echo "  3. Run 24-hour stability test"
echo "  4. Get security audit"
echo "  5. Launch mainnet!"
