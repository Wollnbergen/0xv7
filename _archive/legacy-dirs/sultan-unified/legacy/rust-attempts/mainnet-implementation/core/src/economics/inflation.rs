# Test 1: Status check
echo "🧪 Test 1: Chain Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
RESPONSE=$(curl -s -X POST http://localhost:3030 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"get_status","id":1}')
if [ $? -ne 0 ]; then
    echo "⚠️  Error: Unable to reach the API."
else
    echo "$RESPONSE" | python3 -m json.tool
fi