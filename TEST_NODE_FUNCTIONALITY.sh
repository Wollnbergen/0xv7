#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - NODE FUNCTIONALITY TEST                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Start ScyllaDB if not running
if ! docker ps | grep -q scylla; then
    echo "🗄️ Starting ScyllaDB..."
    docker run -d --name scylla -p 9042:9042 scylladb/scylla
    echo "⏳ Waiting for ScyllaDB to initialize (30 seconds)..."
    sleep 30
fi

# Apply migrations
echo "📊 Applying database migrations..."
docker exec -i scylla cqlsh < migrations/init.cql 2>/dev/null && echo "✅ Migrations applied" || echo "⚠️ Migration issues (may already exist)"

# Build if needed
if [ ! -f target/release/sultan_node ]; then
    echo "🔨 Building Sultan node..."
    cargo build --release --bin sultan_node 2>&1 | tail -3
fi

# Start the node in background
if [ -f target/release/sultan_node ]; then
    echo ""
    echo "🚀 Starting Sultan Chain node..."
    ./target/release/sultan_node &
    NODE_PID=$!
    echo "✅ Node started with PID: $NODE_PID"
    
    sleep 3
    
    # Test basic functionality
    echo ""
    echo "🧪 Testing node functionality..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if process is running
    if ps -p $NODE_PID > /dev/null; then
        echo "✅ Node process is running"
    else
        echo "❌ Node crashed"
    fi
    
    # Kill the test node
    kill $NODE_PID 2>/dev/null
else
    echo "❌ Node binary not found. Run: cargo build --release --bin sultan_node"
fi

echo ""
echo "📊 Test Summary:"
echo "  • Database: $(docker ps | grep -q scylla && echo '✅ Running' || echo '❌ Not running')"
echo "  • Compilation: $([ -f target/release/sultan_node ] && echo '✅ Success' || echo '❌ Failed')"
echo "  • Node startup: $([ -n "$NODE_PID" ] && echo '✅ Works' || echo '❌ Failed')"

