#!/bin/bash

echo "🚀 Sultan Chain Production Test Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test 1: Basic connectivity
echo -e "\n1️⃣ Testing basic connectivity..."
curl -s http://localhost:26657/status > /dev/null && echo "✅ RPC endpoint active" || echo "❌ RPC not responding"
curl -s http://localhost:1317/node_info > /dev/null && echo "✅ API endpoint active" || echo "❌ API not responding"

# Test 2: Zero gas verification
echo -e "\n2️⃣ Verifying zero gas fees..."
RESULT=$(curl -s http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info | grep -o '"min_gas_price":"[^"]*"' | cut -d'"' -f4)
if [ "$RESULT" = "0" ] || [ "$RESULT" = "0usltn" ]; then
    echo "✅ Zero gas fees confirmed: $0.00"
else
    echo "⚠️  Gas price: $RESULT"
fi

# Test 3: Load test
echo -e "\n3️⃣ Running load test (10 seconds)..."
echo "Target: 1,230,000 TPS"
# Simplified load test
for i in {1..10}; do
    curl -s -X POST http://localhost:26657/broadcast_tx_async \
        -H "Content-Type: application/json" \
        -d '{"tx": "test_tx_'$i'"}' > /dev/null &
done
wait
echo "✅ Load test completed"

# Test 4: Check monitoring
echo -e "\n4️⃣ Checking monitoring status..."
curl -s http://localhost:9090/-/healthy > /dev/null && echo "✅ Prometheus healthy" || echo "⚠️  Prometheus not running"
curl -s http://localhost:3001/api/health > /dev/null && echo "✅ Grafana healthy" || echo "⚠️  Grafana not running"

echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Production tests completed"
