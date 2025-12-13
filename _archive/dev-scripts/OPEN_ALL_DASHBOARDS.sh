#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              SULTAN CHAIN - DASHBOARD LAUNCHER                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Get the GitHub Codespaces URL prefix
CODESPACE_NAME=$(echo $CODESPACE_NAME)
GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN="app.github.dev"

echo "🌐 OPENING DASHBOARDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Main dashboard (port 3030)
DASHBOARD_URL="https://${CODESPACE_NAME}-3030.${GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN}/"
echo "📊 Main Dashboard: $DASHBOARD_URL"
"$BROWSER" "$DASHBOARD_URL" &

# API endpoint (port 3000)
API_URL="https://${CODESPACE_NAME}-3000.${GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN}/"
echo "🔌 API Endpoint: $API_URL"

# RPC endpoint (port 26657)
RPC_URL="https://${CODESPACE_NAME}-26657.${GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN}/"
echo "⚡ RPC Endpoint: $RPC_URL"

echo ""
echo "📱 LOCAL ACCESS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  • Main: http://localhost:3030"
echo "  • API:  http://localhost:3000"
echo "  • RPC:  http://localhost:26657"

echo ""
echo "💡 QUICK COMMANDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  • Test API:     curl http://localhost:3030"
echo "  • View logs:    tail -f /tmp/sultan.log"
echo "  • Check status: ps aux | grep sultan"
echo "  • Stop all:     pkill -f sultan"

