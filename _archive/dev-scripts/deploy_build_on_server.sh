#!/bin/bash
# Deploy by building directly on production server
# Avoids dev container resource limits

set -e

SERVER="root@5.161.225.96"
SSH_KEY="$HOME/.ssh/sultan_hetzner"

echo "🚀 Deploying Sultan Fix (Build on Server)"
echo "=========================================="
echo ""

# Step 1: Copy updated source file
echo "1️⃣  Uploading fixed main.rs..."
scp -i "$SSH_KEY" \
    /workspaces/0xv7/sultan-core/src/main.rs \
    "$SERVER:/root/sultan/sultan-core/src/main.rs"
echo "✅ Upload complete"
echo ""

# Step 2: Build on server
echo "2️⃣  Building on production server (this takes ~15 minutes)..."
ssh -i "$SSH_KEY" "$SERVER" 'bash -l -c "
    cd /root/sultan/sultan-core && \
    cargo build --release --bin sultan-node && \
    ls -lh /root/sultan/target/release/sultan-node
"'
echo "✅ Build complete"
echo ""

# Step 3: Stop service
echo "3️⃣  Stopping sultan-node service..."
ssh -i "$SSH_KEY" "$SERVER" 'systemctl stop sultan-node'
echo "✅ Service stopped"
echo ""

# Step 4: Backup old binary
echo "4️⃣  Backing up old binary..."
ssh -i "$SSH_KEY" "$SERVER" 'cp /root/sultan/target/debug/sultan-node /root/sultan/target/debug/sultan-node.backup-$(date +%Y%m%d-%H%M%S)'
echo "✅ Backup created"
echo ""

# Step 5: Update service to use release binary
echo "5️⃣  Updating service to use release binary..."
ssh -i "$SSH_KEY" "$SERVER" "bash -c 'cat > /etc/systemd/system/sultan-node.service << \"EOF\"
[Unit]
Description=Sultan Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/sultan/sultan-core
ExecStart=/root/sultan/target/release/sultan-node \\\\
  --validator \\\\
  --validator-address validator_0 \\\\
  --validator-stake 10000 \\\\
  --enable-sharding \\\\
  --shard-count 8 \\\\
  --block-time 2 \\\\
  --rpc-addr 0.0.0.0:8080 \\\\
  --p2p-addr /ip4/0.0.0.0/tcp/26656
Restart=always
RestartSec=3
StandardOutput=append:/root/sultan/sultan-node.log
StandardError=append:/root/sultan/sultan-node.log

[Install]
WantedBy=multi-user.target
EOF
'"
echo "✅ Service updated"
echo ""

# Step 6: Reload and start
echo "6️⃣  Reloading systemd and starting service..."
ssh -i "$SSH_KEY" "$SERVER" 'systemctl daemon-reload && systemctl start sultan-node'
sleep 5
echo "✅ Service started"
echo ""

# Step 7: Verify
echo "7️⃣  Verifying deployment..."
echo ""
ssh -i "$SSH_KEY" "$SERVER" 'systemctl status sultan-node --no-pager | head -10'
echo ""
echo "Checking status endpoint..."
STATUS=$(ssh -i "$SSH_KEY" "$SERVER" 'curl -s localhost:8080/status')
SHARD_COUNT=$(echo "$STATUS" | jq -r .shard_count)
HEIGHT=$(echo "$STATUS" | jq -r .height)

echo ""
echo "📊 Status Check:"
echo "  Block Height: $HEIGHT"
echo "  Shard Count:  $SHARD_COUNT (should be 8)"
echo ""

if [ "$SHARD_COUNT" == "8" ]; then
    echo "✅ SUCCESS! Shard count now correctly reports 8"
    echo ""
    echo "Public endpoint: https://rpc.sltn.io/status"
    echo ""
else
    echo "⚠️  Shard count shows $SHARD_COUNT (expected 8)"
    echo "   Check logs: ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'journalctl -u sultan-node -f'"
fi

echo ""
echo "🎉 Deployment complete!"
