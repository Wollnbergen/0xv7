#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      STARTING SULTAN-COSMOS INTEGRATED BLOCKCHAIN             ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Clean any existing processes
for port in 3030 26657 1317 8080; do
    lsof -ti:$port 2>/dev/null | xargs kill -9 2>/dev/null || true
done

# Start Cosmos with Sultan economics (with init if needed)
echo -e "\n1️⃣ Starting Cosmos SDK (13.33% APY)..."
docker rm -f cosmos-sultan 2>/dev/null || true

# Optional reset of persistent data
if [ "${SULTAN_RESET:-0}" = "1" ]; then
    docker volume rm -f cosmos-data >/dev/null 2>&1 || true
fi

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
sleep 3

# Start Sultan API (simulated)
if [ "${SULTAN_PRODUCTION:-0}" != "1" ]; then
echo "2️⃣ Starting Sultan Core API (dev simulation)..."
cat > /tmp/sultan-sim-api.js <<'API'
const http = require('http');
const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
        chain: 'Sultan',
        version: '1.0.0',
        apy: '13.33%',
        gas_fees: '$0.00',
        height: Math.floor(Math.random() * 100000)
    }));
});
server.listen(3030, () => console.log('Sultan API on 3030'));
API
node /tmp/sultan-sim-api.js &
SULTAN_PID=$!
else
    echo "2️⃣ Skipping dev simulated API (production mode)"
fi

# Start Unified API
echo "3️⃣ Starting Unified API..."
cd /workspaces/0xv7
node sultan-unified-api.js &
API_PID=$!

sleep 2

echo -e "\n✅ SULTAN-COSMOS INTEGRATION RUNNING!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Services:"
echo "   • Sultan API: http://localhost:3030"
echo "   • Cosmos RPC: http://localhost:26657"
echo "   • Unified API: http://localhost:8080"
echo ""
echo "💎 Economics:"
echo "   • APY: 13.33% (Sultan rate)"
echo "   • Gas: \$0.00 (Zero fees)"
echo ""
echo "Test with: curl http://localhost:8080/status | jq"
echo "Press Ctrl+C to stop"

wait
