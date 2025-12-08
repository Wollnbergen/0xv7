#!/bin/bash
# One-command deployment of shard_count fix

echo "🚀 Sultan Production Deployment"
echo "================================"
echo ""

# Check if binary is ready
if [ ! -f "/tmp/cargo-target/release/sultan-node" ]; then
    echo "❌ Binary not found. Building now..."
    echo ""
    cd /workspaces/0xv7
    cargo build --release -p sultan-core
    
    if [ $? -ne 0 ]; then
        echo ""
        echo "❌ Build failed. Check errors above."
        exit 1
    fi
fi

echo "✅ Binary ready"
echo ""

# Run deployment
echo "📦 Running deployment script..."
echo ""
/workspaces/0xv7/deploy_fix.sh

if [ $? -eq 0 ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎉 DEPLOYMENT SUCCESSFUL!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "✅ Status endpoint now reports correct shard count"
    echo "✅ Public API: https://rpc.sltn.io/status"
    echo ""
    echo "Next steps:"
    echo "  1. Check website: https://sultan-blockchain.repl.co"
    echo "  2. Deploy additional validators (see DEPLOYMENT_CHECKLIST.md)"
    echo "  3. Set up monitoring"
    echo ""
else
    echo ""
    echo "❌ Deployment failed. Check logs above."
    echo ""
    echo "To rollback:"
    echo "  ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96"
    echo "  systemctl stop sultan-node"
    echo "  cp /root/sultan/target/release/sultan-node.backup-* \\"
    echo "     /root/sultan/target/release/sultan-node"
    echo "  systemctl start sultan-node"
    exit 1
fi
