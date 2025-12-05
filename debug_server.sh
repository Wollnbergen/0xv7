open_browser() {
  # open URL in host browser without blocking the terminal
  nohup open_browser "$1" >/dev/null 2>&1 &
  disown || true
}

#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 DAY 3-4 SERVER DEBUG & FIX ==="
echo ""

# Kill all existing servers
echo "1. Cleaning up old servers..."
pkill -f "cargo.*rpc_server" 2>/dev/null || true
sleep 2

# Start fresh server
echo "2. Starting fresh server..."
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
export RUST_LOG='info'

cargo run -p sultan-coordinator --bin rpc_server > /tmp/sultan_fresh.log 2>&1 &
SERVER_PID=$!

echo "   Server PID: $SERVER_PID"
echo "   Waiting for startup..."
sleep 5

# Check if it started
echo ""
echo "3. Checking server status..."
if ps -p $SERVER_PID > /dev/null 2>&1; then
    echo "   ✅ Server is running"
    
    # Check logs
    echo "   Last log entries:"
    tail -5 /tmp/sultan_fresh.log
    
    # Test connectivity
    echo ""
    echo "4. Testing API..."
    
    # Generate token
    TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)
    
    # Test wallet creation
    echo "   Creating wallet..."
    curl -sS -X POST http://127.0.0.1:3030 \
         -H "Authorization: Bearer $TOKEN" \
         -H "Content-Type: application/json" \
         -d '{"jsonrpc":"2.0","method":"create_wallet","params":["test123"],"id":1}' | jq '.'
    
    echo ""
    echo "=== ✅ DAY 3-4 RUNNING ==="
    echo "Server: http://127.0.0.1:3030 (PID: $SERVER_PID)"
    echo "Logs: tail -f /tmp/sultan_fresh.log"
    echo "Browser: \"$BROWSER\" http://127.0.0.1:3030"
else
    echo "   ❌ Server failed to start"
    echo "   Check logs: tail -f /tmp/sultan_fresh.log"
    tail -20 /tmp/sultan_fresh.log
fi
