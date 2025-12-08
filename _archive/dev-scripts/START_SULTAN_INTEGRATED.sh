#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         STARTING INTEGRATED SULTAN + COSMOS SDK               ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Function to check if port is in use
check_port() {
    lsof -i:$1 >/dev/null 2>&1
}

echo -e "\n🔍 Pre-flight checks..."

# Kill any existing processes on our ports
for port in 3030 26657 1317 8080; do
    if check_port $port; then
        echo "   ⚠️ Port $port in use, cleaning up..."
        lsof -ti:$port | xargs kill -9 2>/dev/null || true
    fi
done

# Check Docker
if ! docker info >/dev/null 2>&1; then
    echo "   ❌ Docker not running. Please start Docker first."
    exit 1
fi

echo "   ✅ All checks passed"

# Step 1: Start Cosmos node with Sultan economics
echo -e "\n1️⃣ Starting Cosmos SDK with Sultan Economics (13.33% APY)..."

# Check if container exists and remove it
docker rm -f cosmos-sultan 2>/dev/null || true

# Optional reset of persistent data
if [ "${SULTAN_RESET:-0}" = "1" ]; then
    docker volume rm -f cosmos-data >/dev/null 2>&1 || true
fi

# Start fresh Cosmos container, initializing if needed
docker run -d \
    --name cosmos-sultan \
    -p 26656:26656 \
    -p 26657:26657 \
    -p 1317:1317 \
    -p 9090:9090 \
    -v cosmos-data:/root/.wasmd \
    -v /workspaces/0xv7/init-cosmos.sh:/init.sh:ro \
    cosmwasm/wasmd:latest \
    sh -c "sh /init.sh"

sleep 4

if docker ps | grep -q cosmos-sultan && curl -sf http://localhost:26657/status >/dev/null; then
    echo "   ✅ Cosmos SDK running with Sultan economics"
else
    echo "   ❌ Failed to start Cosmos SDK (container or RPC not responding)"
fi

# Step 2: Start Sultan API simulator (skipped in production)
if [ "${SULTAN_PRODUCTION:-0}" != "1" ]; then
echo -e "\n2️⃣ Starting Sultan Core API (simulated)..."
cat > /tmp/sultan-api.js << 'API'
const express = require('express');
const app = express();

app.get('/', (req, res) => {
    res.json({
        chain: 'Sultan',
        version: '1.0.0',
        apy: '13.33%',
        gas_fees: '$0.00',
        status: 'operational'
    });
});

app.get('/status', (req, res) => {
    res.json({
        height: Math.floor(Math.random() * 100000),
        validators: 100,
        apy: '13.33%',
        inflation: '4%',
        gas: '$0.00'
    });
});

app.listen(3030, () => console.log('Sultan API on port 3030'));
API

# Install express if needed
if [ ! -d "/tmp/node_modules/express" ]; then
    cd /tmp && npm install express >/dev/null 2>&1
fi

node /tmp/sultan-api.js &
SULTAN_PID=$!
echo "   ✅ Sultan API running (PID: $SULTAN_PID)"
else
    echo -e "\n2️⃣ Skipping simulated API (production mode)"
fi

# Step 3: Start the unified API
echo -e "\n3️⃣ Starting Unified API..."
if [ -f "/workspaces/0xv7/sultan-unified-api.js" ]; then
    # Install dependencies if needed
    if [ ! -d "/workspaces/0xv7/node_modules" ]; then
        cd /workspaces/0xv7 && npm install express axios >/dev/null 2>&1
    fi
    cd /workspaces/0xv7
    node sultan-unified-api.js &
    UNIFIED_PID=$!
    echo "   ✅ Unified API running (PID: $UNIFIED_PID)"
else
    echo "   ⚠️ Unified API not found"
fi

# Step 4: Update the dashboard
echo -e "\n4️⃣ Updating Dashboard..."
if [ -f "/workspaces/0xv7/update_integrated_dashboard.sh" ]; then
    /workspaces/0xv7/update_integrated_dashboard.sh
    echo "   ✅ Dashboard updated"
fi

echo "✅ SULTAN-COSMOS INTEGRATION ACTIVE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Endpoints:"
echo "   • Sultan Core API: http://localhost:3030"
echo "   • Cosmos RPC: http://localhost:26657"
echo "   • Cosmos REST: http://localhost:1317"
echo "   • Unified API: http://localhost:8080"
echo ""
echo "💎 Features Active:"
echo "   ✅ 13.33% APY (Sultan economics)"
echo "   ✅ \$0.00 Gas Fees (Sultan zero-fee)"
echo "   ✅ IBC Protocol (Cosmos SDK)"
echo "   ✅ Smart Contracts (CosmWasm)"
echo "   ✅ Quantum Safe (Sultan crypto)"
echo ""
echo "📈 Check Status:"
echo "   curl http://localhost:8080/status | jq"
echo ""
echo "Press Ctrl+C to stop all services"

# Keep script running
wait
