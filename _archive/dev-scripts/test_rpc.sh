#!/bin/bash
echo "Testing RPC endpoints..."

# Generate token
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

echo "Testing with token: ${TOKEN:0:20}..."

# Test create_wallet
echo -n "create_wallet: "
curl -sS -X POST http://127.0.0.1:3030 \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"create_wallet","params":["test"],"id":1}' 2>&1 | \
     grep -q "Method not found" && echo "❌ Not found" || echo "✅ Works"

# Test proposal_create  
echo -n "proposal_create: "
curl -sS -X POST http://127.0.0.1:3030 \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"proposal_create","params":["p1","Test","Desc",null],"id":2}' 2>&1 | \
     grep -q "Method not found" && echo "❌ Not found" || echo "✅ Works"
