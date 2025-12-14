#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        FIXING ALL COMPILATION ERRORS & BUILDING               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Fix 1: Remove duplicate Digest imports
echo "🔧 [1/5] Fixing duplicate Digest imports..."
find src -name "*.rs" -exec sed -i '/use sha2::Digest;/d' {} \;
# Add it back only once where needed
echo "use sha2::Digest;" >> src/multi_consensus.rs

# Fix 2: Remove unused imports
echo "🔧 [2/5] Removing unused imports..."
sed -i '/use crate::blockchain::{Blockchain, Block, Transaction};/s/Block, //' src/consensus.rs
sed -i '/use anyhow::Result;/d' src/rewards.rs
sed -i '/use anyhow::Result;/d' src/types.rs

# Fix 3: Clean up the lib.rs file completely
echo "🔧 [3/5] Fixing lib.rs..."
cat > src/lib.rs << 'RUST'
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod rewards;
pub mod rpc_server;
pub mod scylla_db;
pub mod sdk;
pub mod transaction_validator;
pub mod types;
pub mod persistence;
pub mod p2p;
pub mod multi_consensus;
pub mod state_sync;

// Re-export main types
pub use blockchain::{Blockchain, ChainConfig};
pub use sdk::SultanSDK;
pub use types::SultanToken;
RUST

# Fix 4: Remove the problematic libp2p-swarm-derive line
echo "🔧 [4/5] Cleaning Cargo.toml..."
sed -i '/libp2p-swarm-derive/d' Cargo.toml

# Fix 5: Build step by step
echo "🔧 [5/5] Building Sultan Chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# First, just check if it compiles
cargo check 2>&1 | tail -10

# If check passes, build the binaries
echo ""
echo "Building release binaries..."
cargo build --release 2>&1 | grep -E "Compiling|Finished|error\[" | tail -5

# Check what we have
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 CHECKING BUILT BINARIES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "target/release" ]; then
    BINARIES=$(ls -la target/release/ | grep -E "sultan|rpc|sdk|production" | grep -v "\.d$" | grep -v "\.rlib$" | wc -l)
    echo "Found $BINARIES Sultan Chain binaries:"
    ls -lah target/release/ | grep -E "sultan|rpc|sdk|production" | grep -v "\.d$" | grep -v "\.rlib$"
fi

# Start alternative services while we fix the Rust build
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 STARTING AVAILABLE SERVICES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Kill any existing services on our ports
for port in 3000 3001 3030 8080; do
    lsof -ti:$port | xargs -r kill -9 2>/dev/null
done

# Try to start Node.js API server
if [ -d "/workspaces/0xv7/api" ] && [ -f "/workspaces/0xv7/api/server.js" ]; then
    echo "Starting API server..."
    cd /workspaces/0xv7/api
    npm install --silent 2>/dev/null
    nohup node server.js > /tmp/api.log 2>&1 &
    API_PID=$!
    sleep 2
    ps -p $API_PID > /dev/null && echo "✅ API Server started (PID: $API_PID, Port: 3001)"
fi

# Try to start main server
if [ -d "/workspaces/0xv7/server" ] && [ -f "/workspaces/0xv7/server/server.js" ]; then
    echo "Starting main server..."
    cd /workspaces/0xv7/server
    npm install --silent 2>/dev/null
    nohup node server.js > /tmp/server.log 2>&1 &
    SERVER_PID=$!
    sleep 2
    ps -p $SERVER_PID > /dev/null && echo "✅ Main Server started (PID: $SERVER_PID)"
fi

# Try Python demo servers
if [ -f "/workspaces/0xv7/demo_server.py" ]; then
    echo "Starting Python demo server..."
    cd /workspaces/0xv7
    nohup python demo_server.py > /tmp/demo.log 2>&1 &
    DEMO_PID=$!
    sleep 2
    ps -p $DEMO_PID > /dev/null && echo "✅ Demo Server started (PID: $DEMO_PID, Port: 5000)"
fi

# Create a web interface launcher
cat > /workspaces/0xv7/OPEN_SULTAN_WEB.sh << 'SCRIPT'
#!/bin/bash
echo "🌐 Opening Sultan Chain Web Interface..."
echo ""

# Check which ports are open
for port in 3000 3001 3030 5000 8080; do
    if nc -zv localhost $port 2>/dev/null; then
        echo "✅ Found service on port $port"
        URL="http://localhost:$port"
        echo "Opening: $URL"
        "$BROWSER" "$URL"
        break
    fi
done

echo ""
echo "Alternative access methods:"
echo "  • GitHub Codespace URL: https://${CODESPACE_NAME}-3000.${GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN}"
echo "  • Local tunnel: gh cs ports forward 3000:3000"
SCRIPT
chmod +x /workspaces/0xv7/OPEN_SULTAN_WEB.sh

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 SERVICE STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check what's running
echo "Running services:"
ps aux | grep -E "node|python|sultan|rpc" | grep -v grep | while read line; do
    echo "  • $(echo $line | awk '{print $11, $12, $13}')"
done

echo ""
echo "Open ports:"
for port in 3000 3001 3030 5000 8080; do
    nc -zv localhost $port 2>/dev/null && echo "  ✅ Port $port is open"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 NEXT STEPS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Open web interface:  ./OPEN_SULTAN_WEB.sh"
echo "2. Check API health:    curl http://localhost:3001/api/health"
echo "3. View logs:           tail -f /tmp/*.log"
echo ""

# Try to open the web interface automatically
if [ -n "$BROWSER" ]; then
    for port in 3000 3001 5000; do
        if nc -zv localhost $port 2>/dev/null; then
            echo "🌐 Opening Sultan Chain in browser..."
            "$BROWSER" "http://localhost:$port"
            break
        fi
    done
fi

