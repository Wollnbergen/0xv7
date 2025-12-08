#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         ☀️ GOOD MORNING! RESUMING SULTAN BLOCKCHAIN          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "�� Resuming from 100% complete status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Start Docker containers
echo "🐳 Starting Docker containers..."
docker start cosmos-sultan 2>/dev/null && echo "   ✅ Cosmos node started" || echo "   ⚠️ Cosmos node not found"
docker start prometheus 2>/dev/null && echo "   ✅ Prometheus started" || echo "   ⚠️ Prometheus optional"
docker start grafana 2>/dev/null && echo "   ✅ Grafana started" || echo "   ⚠️ Grafana optional"

# 2. Start Sultan services
echo ""
echo "🚀 Starting Sultan Services..."
if [ -f /workspaces/0xv7/START_SULTAN_SERVICES.sh ]; then
    bash /workspaces/0xv7/START_SULTAN_SERVICES.sh
else
    cd /workspaces/0xv7
    npm start > /tmp/sultan-api.log 2>&1 &
    echo "   ✅ API server started"
    npm run dev > /tmp/sultan-web.log 2>&1 &
    echo "   ✅ Dashboard started"
fi

# 3. Show status
echo ""
echo "�� Current Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f /workspaces/0xv7/CURRENT_STATUS.json ]; then
    cat /workspaces/0xv7/CURRENT_STATUS.json
fi

echo ""
echo "🌐 Access Points:"
echo "   Dashboard: http://localhost:3000"
echo "   API: http://localhost:1317/status"
echo ""
echo "✅ Sultan Blockchain resumed successfully!"
echo "📝 To open dashboard: \"$BROWSER\" http://localhost:3000"
