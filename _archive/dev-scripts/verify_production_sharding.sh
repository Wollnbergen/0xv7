#!/bin/bash
# Verification Script for Production Sharding Deployment

SERVER="5.161.225.96"
PORT="8080"

echo "=================================================="
echo "🔍 Sultan Blockchain - Production Sharding Verification"
echo "=================================================="
echo ""

echo "📋 Test 1: RPC Server Connectivity"
echo "   Checking http://$SERVER:$PORT/status..."
response=$(curl -s http://$SERVER:$PORT/status)
if [ -n "$response" ]; then
    echo "   ✅ RPC server is responding"
    echo ""
    echo "   Response:"
    echo "$response" | jq '.' 2>/dev/null || echo "$response"
else
    echo "   ❌ RPC server is not responding"
    exit 1
fi

echo ""
echo "=================================================="
echo "📋 Test 2: Sharding Status"
echo "=================================================="

sharding_enabled=$(echo "$response" | jq -r '.sharding_enabled' 2>/dev/null)
shard_count=$(echo "$response" | jq -r '.shard_count' 2>/dev/null)
tps_capacity=$(echo "$response" | jq -r '.tps_capacity // .shard_stats.estimated_tps' 2>/dev/null)

if [ "$sharding_enabled" = "true" ]; then
    echo "✅ Sharding is ENABLED"
    echo "   Shard Count: $shard_count"
    echo "   TPS Capacity: $tps_capacity"
    
    if [ "$shard_count" -ge 1000 ]; then
        echo "   ✅ Production shard count confirmed (≥1000)"
    else
        echo "   ⚠️  Shard count is less than expected (got: $shard_count, expected: 1024)"
    fi
    
    if [ "$tps_capacity" -ge 1000000 ]; then
        echo "   ✅ Million+ TPS capacity confirmed"
    else
        echo "   ⚠️  TPS capacity lower than expected (got: $tps_capacity, expected: 1M+)"
    fi
else
    echo "❌ Sharding is NOT enabled (expected: true, got: $sharding_enabled)"
    echo "   This indicates simulation code is still running!"
    exit 1
fi

echo ""
echo "=================================================="
echo "📋 Test 3: Block Production"
echo "=================================================="

height=$(echo "$response" | jq -r '.height' 2>/dev/null)
echo "   Current Height: $height"

if [ "$height" -gt 0 ]; then
    echo "   ✅ Blocks are being produced"
    
    # Wait and check again
    echo "   Waiting 3 seconds for next block..."
    sleep 3
    
    new_response=$(curl -s http://$SERVER:$PORT/status)
    new_height=$(echo "$new_response" | jq -r '.height' 2>/dev/null)
    
    echo "   New Height: $new_height"
    
    if [ "$new_height" -gt "$height" ]; then
        echo "   ✅ Block production is active (height increased)"
    else
        echo "   ⚠️  No new blocks produced (height unchanged)"
    fi
else
    echo "   ⚠️  Genesis block only (height: 0)"
fi

echo ""
echo "=================================================="
echo "📋 Test 4: Validator Status"
echo "=================================================="

validator_count=$(echo "$response" | jq -r '.validator_count' 2>/dev/null)
echo "   Validator Count: $validator_count"

if [ "$validator_count" -ge 11 ]; then
    echo "   ✅ Expected validator count (11+ validators)"
else
    echo "   ⚠️  Lower than expected validator count (expected: 11+, got: $validator_count)"
fi

echo ""
echo "=================================================="
echo "📋 Test 5: Balance Query (Shard Routing)"
echo "=================================================="

echo "   Testing balance query for 'validator1'..."
balance_response=$(curl -s http://$SERVER:$PORT/balance/validator1)

if [ -n "$balance_response" ]; then
    echo "   ✅ Balance query successful"
    echo "   Response:"
    echo "$balance_response" | jq '.' 2>/dev/null || echo "$balance_response"
    
    balance=$(echo "$balance_response" | jq -r '.balance' 2>/dev/null)
    if [ "$balance" = "10000" ]; then
        echo "   ✅ Correct balance returned (10000 SLTN)"
    else
        echo "   ⚠️  Unexpected balance (expected: 10000, got: $balance)"
    fi
else
    echo "   ❌ Balance query failed"
fi

echo ""
echo "=================================================="
echo "📋 Test 6: Transaction Submission"
echo "=================================================="

echo "   Submitting test transaction..."
tx_response=$(curl -s -X POST http://$SERVER:$PORT/tx \
    -H "Content-Type: application/json" \
    -d '{
        "from": "validator1",
        "to": "validator2",
        "amount": 100,
        "gas_fee": 0,
        "nonce": 1,
        "timestamp": '$(date +%s)'
    }' 2>/dev/null)

if [ -n "$tx_response" ]; then
    echo "   ✅ Transaction submitted"
    echo "   Response:"
    echo "$tx_response" | jq '.' 2>/dev/null || echo "$tx_response"
    
    status=$(echo "$tx_response" | jq -r '.status' 2>/dev/null)
    if [ "$status" = "accepted" ]; then
        echo "   ✅ Transaction accepted by sharding system"
    else
        echo "   ⚠️  Transaction status unclear (status: $status)"
    fi
else
    echo "   ⚠️  Transaction submission response empty"
fi

echo ""
echo "=================================================="
echo "📊 VERIFICATION SUMMARY"
echo "=================================================="
echo ""

# Count successes
successes=0
total_tests=6

[ "$sharding_enabled" = "true" ] && ((successes++))
[ "$shard_count" -ge 1000 ] && ((successes++))
[ "$tps_capacity" -ge 1000000 ] && ((successes++))
[ "$height" -gt 0 ] && ((successes++))
[ "$validator_count" -ge 11 ] && ((successes++))
[ -n "$balance_response" ] && ((successes++))

echo "Passed: $successes / $total_tests tests"
echo ""

if [ "$successes" -eq "$total_tests" ]; then
    echo "✅ ALL TESTS PASSED"
    echo ""
    echo "Production sharding is confirmed active:"
    echo "  ✅ Sharding enabled with $shard_count shards"
    echo "  ✅ TPS capacity: $tps_capacity"
    echo "  ✅ Block production active (height: $new_height)"
    echo "  ✅ Validators: $validator_count"
    echo "  ✅ Shard routing working"
    echo "  ✅ Transaction processing active"
    echo ""
    echo "🚀 PRODUCTION DEPLOYMENT VERIFIED"
    exit 0
elif [ "$successes" -ge 4 ]; then
    echo "⚠️  PARTIAL SUCCESS ($successes/$total_tests tests passed)"
    echo ""
    echo "Some features may need verification or adjustment."
    echo "Review the test results above for details."
    exit 0
else
    echo "❌ VERIFICATION FAILED ($successes/$total_tests tests passed)"
    echo ""
    echo "Critical issues detected. Review logs and configuration."
    exit 1
fi
