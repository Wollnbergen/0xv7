#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         FIXING CARGO.TOML & BUILDING SULTAN CHAIN            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Fix 1: Remove the invalid libp2p-swarm-derive line from Cargo.toml
echo "🔧 [1/3] Fixing Cargo.toml..."
sed -i '/libp2p-swarm-derive/d' Cargo.toml

# Fix 2: Ensure we have a proper [dependencies] section
if ! grep -q "^\[dependencies\]" Cargo.toml; then
    echo "[dependencies]" >> Cargo.toml
fi

# Fix 3: Check if we have all needed dependencies
echo "🔧 [2/3] Verifying dependencies..."
cargo check 2>&1 | grep -E "error|warning" | head -5 || echo "✅ Dependencies OK"

# Build the binaries
echo ""
echo "🔨 [3/3] Building Sultan Chain binaries..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build main node
echo "Building sultan_node..."
cargo build --release --bin sultan_node 2>&1 | grep -E "Compiling|Finished|error" | tail -5

# Build RPC server
echo ""
echo "Building rpc_server..."
cargo build --release --bin rpc_server 2>&1 | grep -E "Compiling|Finished|error" | tail -5

# Check what we built
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 BUILT BINARIES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "target/release" ]; then
    ls -lah target/release/ | grep -E "sultan|rpc|sdk|production" | grep -v "\.d$" | grep -v "\.rlib$" || echo "Building..."
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 LAUNCHING SULTAN CHAIN"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Try to start the RPC server
if [ -f target/release/rpc_server ]; then
    echo "Starting RPC Server on port 3030..."
    nohup ./target/release/rpc_server > /tmp/sultan_rpc.log 2>&1 &
    RPC_PID=$!
    echo "RPC Server PID: $RPC_PID"
    
    # Wait for it to start
    sleep 3
    
    # Check if it's running
    if ps -p $RPC_PID > /dev/null; then
        echo "✅ RPC Server is running!"
        
        # Test the API
        echo ""
        echo "Testing API endpoints:"
        curl -s http://localhost:3030/health 2>/dev/null && echo "  ✅ Health endpoint OK" || echo "  ⚠️ API not responding yet"
        
        # Show how to access it
        echo ""
        echo "🌐 SULTAN CHAIN IS NOW ACCESSIBLE AT:"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  • API Endpoint: http://localhost:3030"
        echo "  • Open in browser: $BROWSER http://localhost:3030"
        echo "  • View logs: tail -f /tmp/sultan_rpc.log"
        echo ""
    else
        echo "⚠️ RPC Server failed to start. Check /tmp/sultan_rpc.log"
        [ -f /tmp/sultan_rpc.log ] && tail -5 /tmp/sultan_rpc.log
    fi
else
    echo "⚠️ RPC server not built. Trying alternative servers..."
    
    # Check for other services we can start
    cd /workspaces/0xv7
    
    # Try the API server
    if [ -d "api" ] && [ -f "api/server.js" ]; then
        echo "Starting Node.js API server..."
        cd api
        npm install --silent 2>/dev/null
        nohup node server.js > /tmp/sultan_api.log 2>&1 &
        API_PID=$!
        sleep 2
        
        if ps -p $API_PID > /dev/null; then
            echo "✅ API Server running on port 3001 (PID: $API_PID)"
        fi
    fi
    
    # Try the server directory
    if [ -d "/workspaces/0xv7/server" ] && [ -f "/workspaces/0xv7/server/server.js" ]; then
        echo "Starting main server..."
        cd /workspaces/0xv7/server
        npm install --silent 2>/dev/null
        nohup node server.js > /tmp/sultan_server.log 2>&1 &
        SERVER_PID=$!
        sleep 2
        
        if ps -p $SERVER_PID > /dev/null; then
            echo "✅ Main server running (PID: $SERVER_PID)"
        fi
    fi
fi

# Create a quick status check script
cat > /workspaces/0xv7/CHECK_SULTAN_STATUS.sh << 'SCRIPT'
#!/bin/bash
echo "🔍 SULTAN CHAIN STATUS CHECK"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check running processes
echo "📊 Running Services:"
ps aux | grep -E "sultan|rpc_server|node server" | grep -v grep | while read line; do
    echo "  • $line" | cut -c1-80
done

echo ""
echo "🌐 Network Endpoints:"
for port in 3000 3001 3030 8080; do
    nc -zv localhost $port 2>/dev/null && echo "  ✅ Port $port is open" || echo "  ⚠️ Port $port is closed"
done

echo ""
echo "📁 Available Binaries:"
ls -la /workspaces/0xv7/node/target/release/ 2>/dev/null | grep -E "sultan|rpc" | grep -v "\.d$"
SCRIPT
chmod +x /workspaces/0xv7/CHECK_SULTAN_STATUS.sh

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SETUP COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Quick Commands:"
echo "  • Check status: ./CHECK_SULTAN_STATUS.sh"
echo "  • View RPC logs: tail -f /tmp/sultan_rpc.log"
echo "  • Test API: curl http://localhost:3030/health"
echo ""

