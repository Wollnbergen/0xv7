#!/bin/bash

PORT="${PORT:-8080}"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           🚀 STARTING SULTAN CHAIN BLOCKCHAIN 🚀              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "📍 Using port: $PORT"

cd /workspaces/0xv7/sultan-simple

# Check if port is already in use
if lsof -iTCP:$PORT -sTCP:LISTEN -P -n >/dev/null 2>&1; then
    echo "⚠️ Port $PORT is already in use! Try a different port."
    echo "   Example: PORT=8081 /workspaces/0xv7/RUN_SULTAN_CHAIN.sh"
    exit 1
fi

# Start with the specified port
SULTAN_PORT=$PORT ./sultand start &
PID=$!
echo "✅ Sultan Chain started with PID: $PID on port $PORT"

sleep 2

# Test the API
echo ""
echo "🧪 Testing blockchain API on port $PORT..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Get status
echo "📊 Chain Status:"
curl -s http://localhost:$PORT/status | python3 -m json.tool

# Add a test transaction
echo ""
echo "📝 Adding test transaction..."
curl -s -X POST http://localhost:$PORT/block \
  -H "Content-Type: application/json" \
  -d '{"data":"First Sultan Chain Transaction - Zero Gas Fees!"}' \
  | python3 -m json.tool

# Get all blocks
echo ""
echo "📦 Current Blocks:"
curl -s http://localhost:$PORT/blocks | python3 -m json.tool | head -50

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Sultan Chain is running successfully!"
echo ""
echo "🌐 Access points:"
echo "   • API Status: http://localhost:$PORT/status"
echo "   • View Blocks: http://localhost:$PORT/blocks"
echo "   • Submit Transaction: POST http://localhost:$PORT/block"
echo ""
echo "📝 To stop the blockchain:"
echo "   /workspaces/0xv7/STOP_SULTAN_CHAIN.sh"
echo ""
echo "🚀 To open in browser:"
echo "   \$BROWSER http://localhost:$PORT/status"
