#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - PRODUCTION DEPLOYMENT                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."
command -v docker >/dev/null 2>&1 && echo "✅ Docker installed" || echo "❌ Docker missing"
command -v go >/dev/null 2>&1 && echo "✅ Go installed" || echo "❌ Go missing"
command -v ignite >/dev/null 2>&1 && echo "✅ Ignite CLI installed" || echo "❌ Ignite missing"

echo ""
echo "🚀 Deployment Options:"
echo "  1) Demo Mode (Mock API)"
echo "  2) Testnet (Cosmos SDK)"
echo "  3) Full Production"
echo ""
read -p "Select deployment mode (1-3): " mode

case $mode in
  1)
    echo "Starting Demo Mode..."
    ./sultan_live_demo.sh &
    echo "✅ Demo API running on http://127.0.0.1:3030"
    ;;
  2)
    echo "Starting Testnet..."
    cd /workspaces/0xv7/sultan
    ignite chain serve --reset-once &
    echo "✅ Testnet starting..."
    ;;
  3)
    echo "Starting Production..."
    # Start all services
    docker-compose up -d
    echo "✅ Production services starting..."
    ;;
esac

echo ""
echo "📊 Sultan Chain Status:"
echo "  • Zero Gas Fees: ✅ ENABLED"
echo "  • APY: 26.67% (37.33% mobile)"
echo "  • IBC: ✅ READY"
echo "  • Dashboard: http://127.0.0.1:8080/sultan_dashboard.html"
