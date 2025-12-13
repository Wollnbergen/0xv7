#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    SULTAN CHAIN TEST SUITE                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Test RPC Server
echo "1. Starting RPC Server (will run for 5 seconds)..."
timeout 5s cargo run -p sultan-coordinator --bin rpc_server 2>&1 | head -10 &
RPC_PID=$!
sleep 2

# 2. Test RPC endpoints
echo ""
echo "2. Testing RPC endpoints..."
curl -s -X POST http://localhost:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' 2>&1 | grep -q "result" && echo "✅ RPC health check works" || echo "❌ RPC not responding"

# 3. Stop RPC server
kill $RPC_PID 2>/dev/null
wait $RPC_PID 2>/dev/null

# 4. Test Wallet CLI
echo ""
echo "3. Testing Wallet CLI..."
echo -e "help\nexit" | cargo run -p sultan-coordinator --bin wallet_cli 2>&1 | grep -q "Available commands" && echo "✅ Wallet CLI works" || echo "❌ Wallet CLI has issues"

echo ""
echo "Test complete!"
