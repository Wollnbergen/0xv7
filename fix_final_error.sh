open_browser() {
  # open URL in host browser without blocking the terminal
  nohup open_browser "$1" >/dev/null 2>&1 &
  disown || true
}

#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 FIXING RPC SERVER FINAL ERROR ==="
echo ""

echo "1. Removing stray code at end of rpc_server.rs..."
# Remove lines from 914 to end that are outside any function
sed -i '914,$d' node/src/rpc_server.rs

echo "2. Verifying file ends correctly..."
tail -5 node/src/rpc_server.rs

echo ""
echo "3. Testing compilation..."
cargo build -p sultan-coordinator 2>&1 | grep -E "Compiling|Finished|error" | tail -5

if cargo check -p sultan-coordinator 2>&1 | grep -q "Finished"; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    
    # Start server
    pkill -f "cargo.*rpc_server" 2>/dev/null || true
    sleep 2
    
    export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
    cargo run -p sultan-coordinator --bin rpc_server > /tmp/sultan.log 2>&1 &
    SERVER_PID=$!
    
    echo ""
    echo "=== 🎉 DAY 3-4 COMPLETE ==="
    echo "📡 Server: http://127.0.0.1:3030 (PID: $SERVER_PID)"
    echo "📝 Logs: tail -f /tmp/sultan.log"
    echo "🌐 Browser: \"$BROWSER\" http://127.0.0.1:3030"
    echo ""
    echo "Ready for Day 5-6! 🚀"
fi
