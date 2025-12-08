#!/bin/bash
# Sultan L1 - Production Deployment Script for Hetzner Server
# Deploys sultan-node to production with feature flags enabled

set -e

echo "🚀 Sultan L1 - Production Deployment"
echo "======================================"
echo ""

# Configuration
HETZNER_IP="5.161.225.96"
HETZNER_USER="root"
BINARY_PATH="/tmp/cargo-target/release/sultan-node"
CONFIG_PATH="sultan-core/chain_config.json"
REMOTE_BINARY="/usr/local/bin/sultand"
REMOTE_CONFIG="/var/lib/sultan/chain_config.json"
SERVICE_NAME="sultan-node"

# Step 1: Verify binary exists
echo "📦 Step 1: Verifying binary..."
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ ERROR: Binary not found at $BINARY_PATH"
    echo "   Run: cd /workspaces/0xv7 && cargo build --release -p sultan-core"
    exit 1
fi

BINARY_SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
echo "✅ Binary found: $BINARY_SIZE"

# Step 2: Verify config exists
echo ""
echo "⚙️  Step 2: Verifying chain config..."
if [ ! -f "$CONFIG_PATH" ]; then
    echo "❌ ERROR: Config not found at $CONFIG_PATH"
    exit 1
fi
echo "✅ Config found with feature flags:"
cat "$CONFIG_PATH" | grep -A 10 "features"

# Step 3: Stop current service
echo ""
echo "🛑 Step 3: Stopping current service on Hetzner..."
ssh "$HETZNER_USER@$HETZNER_IP" "systemctl stop $SERVICE_NAME || true"
echo "✅ Service stopped"

# Step 4: Backup current binary
echo ""
echo "💾 Step 4: Backing up current binary..."
ssh "$HETZNER_USER@$HETZNER_IP" "
    if [ -f $REMOTE_BINARY ]; then
        cp $REMOTE_BINARY ${REMOTE_BINARY}.backup.\$(date +%Y%m%d_%H%M%S)
        echo '✅ Backup created'
    else
        echo 'ℹ️  No existing binary to backup'
    fi
"

# Step 5: Deploy new binary
echo ""
echo "📤 Step 5: Deploying new binary to Hetzner..."
scp "$BINARY_PATH" "$HETZNER_USER@$HETZNER_IP:$REMOTE_BINARY"
ssh "$HETZNER_USER@$HETZNER_IP" "chmod +x $REMOTE_BINARY"
echo "✅ Binary deployed"

# Step 6: Deploy config
echo ""
echo "📤 Step 6: Deploying chain config..."
ssh "$HETZNER_USER@$HETZNER_IP" "mkdir -p /var/lib/sultan"
scp "$CONFIG_PATH" "$HETZNER_USER@$HETZNER_IP:$REMOTE_CONFIG"
echo "✅ Config deployed"

# Step 7: Update systemd service
echo ""
echo "⚙️  Step 7: Updating systemd service..."
ssh "$HETZNER_USER@$HETZNER_IP" "cat > /etc/systemd/system/$SERVICE_NAME.service << 'EOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/var/lib/sultan
ExecStart=$REMOTE_BINARY \\
    --validator \\
    --enable-sharding \\
    --shard-count 8 \\
    --max-shards 8000 \\
    --rpc-addr 0.0.0.0:8080 \\
    --block-time 2 \\
    --data-dir /var/lib/sultan/data \\
    --config /var/lib/sultan/chain_config.json
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sultan-node

# Resource limits for production
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF"
echo "✅ Systemd service updated"

# Step 8: Reload and start service
echo ""
echo "🔄 Step 8: Reloading systemd and starting service..."
ssh "$HETZNER_USER@$HETZNER_IP" "systemctl daemon-reload && systemctl start $SERVICE_NAME"
echo "✅ Service started"

# Step 9: Enable service on boot
echo ""
echo "🔧 Step 9: Enabling service on boot..."
ssh "$HETZNER_USER@$HETZNER_IP" "systemctl enable $SERVICE_NAME"
echo "✅ Service enabled"

# Step 10: Verify service is running
echo ""
echo "✅ Step 10: Verifying service status..."
sleep 3
ssh "$HETZNER_USER@$HETZNER_IP" "systemctl status $SERVICE_NAME --no-pager | head -20"

# Step 11: Check blockchain status
echo ""
echo "🔍 Step 11: Checking blockchain status..."
sleep 5
BLOCK_HEIGHT=$(curl -s https://rpc.sltn.io/status | jq -r '.height' 2>/dev/null || echo "0")
VALIDATORS=$(curl -s https://rpc.sltn.io/status | jq -r '.validators' 2>/dev/null || echo "0")
SHARDS=$(curl -s https://rpc.sltn.io/status | jq -r '.shards' 2>/dev/null || echo "0")

echo ""
echo "======================================"
echo "🎉 DEPLOYMENT COMPLETE!"
echo "======================================"
echo ""
echo "📊 Current Status:"
echo "   Block Height: $BLOCK_HEIGHT"
echo "   Validators: $VALIDATORS"
echo "   Shards: $SHARDS"
echo ""
echo "🔗 Endpoints:"
echo "   RPC: https://rpc.sltn.io"
echo "   Status: https://rpc.sltn.io/status"
echo "   Bridges: https://rpc.sltn.io/bridges"
echo ""
echo "📝 Next Steps:"
echo "   1. Monitor blocks: watch -n 2 'curl -s https://rpc.sltn.io/status | jq .height'"
echo "   2. Check logs: ssh $HETZNER_USER@$HETZNER_IP 'journalctl -u $SERVICE_NAME -f'"
echo "   3. Verify validators: curl -s https://rpc.sltn.io/staking/validators | jq"
echo ""
echo "🎯 Feature Flags:"
echo "   ✅ Sharding: ENABLED (8→8000 shards)"
echo "   ✅ Governance: ENABLED"
echo "   ✅ Bridges: ENABLED (5 chains)"
echo "   ⏳ Smart Contracts: DISABLED (activate via governance)"
echo ""
echo "🚀 Sultan L1 is now running on Hetzner!"
echo ""
