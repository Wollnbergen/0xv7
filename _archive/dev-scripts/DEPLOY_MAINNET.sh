#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - MAINNET DEPLOYMENT CENTER              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🎯 Deployment Options:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Local Development:"
echo "   ./sultan-mainnet/sultand"
echo ""
echo "2. Docker Compose (Full Stack):"
echo "   cd sultan-mainnet/configs && docker-compose up -d"
echo ""
echo "3. Kubernetes (Production):"
echo "   kubectl create namespace sultan"
echo "   kubectl apply -f sultan-mainnet/deployments/"
echo ""
echo "4. Cloud Deployment:"
echo "   • AWS: eksctl create cluster --name sultan-mainnet"
echo "   • GCP: gcloud container clusters create sultan-mainnet"
echo "   • Azure: az aks create --name sultan-mainnet"
echo ""

echo "📊 Current Status:"
if curl -s http://localhost:26657/status > /dev/null 2>&1; then
    echo "   ✅ Production node is running"
    curl -s http://localhost:26657/status | python3 -m json.tool | grep -E '"chain_id"|"block_height"|"network"'
else
    echo "   ⚠️  Production node not running"
fi

echo ""
echo "🔗 Useful Links:"
echo "   • RPC: http://localhost:26657"
echo "   • P2P: http://localhost:26656"
echo "   • gRPC: http://localhost:9090"
echo "   • Metrics: http://localhost:9091"
echo "   • Dashboard: http://localhost:3000/production-dashboard.html"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
