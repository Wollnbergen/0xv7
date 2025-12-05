#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              STARTING SULTAN CHAIN TESTNET                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Kill any existing processes
pkill -f "node.*simple_server" 2>/dev/null

# Start the API server
echo "🚀 Starting API server..."
cd /workspaces/0xv7/api
node simple_server.js > /tmp/sultan_api.log 2>&1 &
API_PID=$!

sleep 2

# Test if it's running
if kill -0 $API_PID 2>/dev/null; then
    echo "✅ API Server started (PID: $API_PID)"
    echo ""
    echo "📍 Access your testnet at:"
    echo "   Local: http://localhost:3030"
    echo "   Public: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
    echo ""
    echo "📊 Test with:"
    echo "   curl -X POST http://localhost:3030 -H 'Content-Type: application/json' \\"
    echo "        -d '{\"jsonrpc\":\"2.0\",\"method\":\"get_status\",\"id\":1}'"
    echo ""
    echo "🛑 To stop: pkill -f 'node.*simple_server'"
else
    echo "❌ Failed to start API server"
    echo "Check logs: tail -f /tmp/sultan_api.log"
fi

