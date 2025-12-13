#!/bin/bash
# ONE-COMMAND DEPLOYMENT - Sultan Production Sharding

# SSH key configuration
SSH_KEY="${HOME}/.ssh/sultan-node-2024"

if [ ! -f "$SSH_KEY" ]; then
    echo "❌ SSH key not found: $SSH_KEY"
    echo "   Please update SSH_KEY variable or ensure the key exists"
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  🚀 Sultan Blockchain - Production Sharding Deployment    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Target: root@5.161.225.96"
echo "SSH Key: $SSH_KEY"
echo "Config: 1024 shards, 8M+ TPS capacity, 2-second blocks"
echo ""
read -p "Deploy production sharding now? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Deployment cancelled."
    exit 0
fi

echo ""
echo "🚀 Starting deployment..."
echo ""

./deploy_production_sharding.sh

if [ $? -eq 0 ]; then
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║  ✅ DEPLOYMENT COMPLETE - Running Verification           ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    sleep 3
    ./verify_production_sharding.sh
else
    echo ""
    echo "❌ Deployment failed - check errors above"
    exit 1
fi
