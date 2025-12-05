#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 FIXING RPC METHOD REGISTRATION ==="
echo ""

# Check if methods are properly in the match statement
echo "1. Checking if create_wallet is in a match block..."
if ! grep -q 'match.*method.*{.*"create_wallet"' node/src/rpc_server.rs; then
    echo "   ❌ Methods not properly registered in match statement"
    
    # Find where the handle_call function is
    LINE=$(grep -n "async fn handle_call" node/src/rpc_server.rs | cut -d: -f1)
    echo "   Found handle_call at line $LINE"
    
    # Check the structure around it
    echo ""
    echo "2. Examining handle_call function structure..."
    sed -n "${LINE},$((LINE+50))p" node/src/rpc_server.rs | head -30
else
    echo "   ✅ Methods appear to be registered"
fi

echo ""
echo "3. Let's verify the actual RPC handler implementation..."

# Show the main function to understand the server setup
echo "   Checking main function setup..."
grep -A20 "fn main\|async fn main" node/src/rpc_server.rs | head -25

echo ""
echo "4. Testing a direct fix..."

# Kill existing server
pkill -f "cargo.*rpc_server" 2>/dev/null || true
sleep 2

# Start server with trace logging to see what's happening
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
export RUST_LOG='trace'

echo "5. Starting server with trace logging..."
cargo run -p sultan-coordinator --bin rpc_server > /tmp/sultan_trace.log 2>&1 &
SERVER_PID=$!

echo "   Server PID: $SERVER_PID"
sleep 5

# Check what's happening when we make a request
echo ""
echo "6. Making test request and checking logs..."
curl -sS -X POST http://127.0.0.1:3030 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"create_wallet","params":["test"],"id":1}' \
     2>&1

echo ""
echo "   Relevant log entries:"
grep -i "create_wallet\|method.*not.*found\|handling\|request" /tmp/sultan_trace.log | tail -10

echo ""
echo "=== 📋 DIAGNOSTICS COMPLETE ==="
echo "Full trace logs: tail -f /tmp/sultan_trace.log"
echo "Server PID: $SERVER_PID"
