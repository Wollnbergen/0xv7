#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - QUICK START (AFTER RESTART)            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Start Docker services if needed
echo "🐳 Starting Docker services..."
if ! docker ps | grep -q scylla; then
    docker run --name scylla -d -p 9042:9042 scylladb/scylla 2>/dev/null || echo "   ScyllaDB already exists"
fi
if ! docker ps | grep -q redis; then
    docker run --name redis -d -p 6379:6379 redis:alpine 2>/dev/null || echo "   Redis already exists"
fi

docker start scylla redis 2>/dev/null
echo "✅ Docker services started"

# 2. Start testnet API
echo ""
echo "🌐 Starting Testnet API..."
if [ -d /workspaces/0xv7/api ]; then
    cd /workspaces/0xv7/api
    if [ -f package.json ]; then
        # Install dependencies if needed
        [ -d node_modules ] || npm install --quiet
        
        # Start the API
        npm start > /tmp/sultan-api.log 2>&1 &
        API_PID=$!
        
        # Wait for it to start
        echo -n "   Waiting for API to start"
        for i in {1..10}; do
            if curl -s http://localhost:3030 > /dev/null 2>&1; then
                echo " ✅"
                echo "   API running at: http://localhost:3030"
                echo "   Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
                break
            fi
            echo -n "."
            sleep 1
        done
    else
        echo "   ⚠️ API directory found but no package.json"
    fi
else
    echo "   ⚠️ API directory not found at /workspaces/0xv7/api"
fi

# 3. Build and run mainnet if source exists
echo ""
echo "🔨 Building Sultan Mainnet..."
if [ -f /workspaces/0xv7/sultan_mainnet/Cargo.toml ]; then
    cd /workspaces/0xv7
    cargo build -p sultan-mainnet --release 2>&1 | grep -E "Compiling|Finished|error"
    
    # Find and run the binary
    BINARY=$(find /workspaces/0xv7 -path "*/target/release/sultan-mainnet" -type f 2>/dev/null | head -1)
    if [ -n "$BINARY" ]; then
        echo "✅ Binary built successfully: $BINARY"
        echo ""
        echo "🚀 Starting Sultan Mainnet..."
        RUST_LOG=info "$BINARY"
    else
        echo "⚠️ Binary not found after build"
    fi
elif [ -f /workspaces/0xv7/sultan_minimal ]; then
    echo "🚀 Running existing Sultan minimal node..."
    /workspaces/0xv7/sultan_minimal
else
    echo "⚠️ No Sultan mainnet source found"
fi

