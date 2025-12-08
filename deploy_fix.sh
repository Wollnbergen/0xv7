#!/bin/bash
# Deploy Fixed Sultan Node to Production
# Fixes shard_count reporting bug (100 → 8)

set -e

SERVER="root@5.161.225.96"
SSH_KEY="$HOME/.ssh/sultan_hetzner"

echo "🔧 Deploying Sultan Node Fix to Production"
echo "==========================================="
echo ""

# Step 1: Build locally
echo "1️⃣  Building sultan-node (release mode)..."
cd /workspaces/0xv7
cargo build --release -p sultan-node
echo "✅ Build complete"
echo ""

# Step 2: Copy to production
echo "2️⃣  Uploading binary to production server..."
scp -i "$SSH_KEY" \
    /tmp/cargo-target/release/sultan-node \
    "$SERVER:/root/sultan/target/release/sultan-node.new"
echo "✅ Upload complete"
echo ""

# Step 3: Stop service
echo "3️⃣  Stopping sultan-node service..."
ssh -i "$SSH_KEY" "$SERVER" 'systemctl stop sultan-node'
echo "✅ Service stopped"
echo ""

# Step 4: Backup old binary
echo "4️⃣  Backing up old binary..."
ssh -i "$SSH_KEY" "$SERVER" 'cp /root/sultan/target/release/sultan-node /root/sultan/target/release/sultan-node.backup-$(date +%Y%m%d-%H%M%S)'
echo "✅ Backup created"
echo ""

# Step 5: Replace binary
echo "5️⃣  Installing new binary..."
ssh -i "$SSH_KEY" "$SERVER" 'mv /root/sultan/target/release/sultan-node.new /root/sultan/target/release/sultan-node'
ssh -i "$SSH_KEY" "$SERVER" 'chmod +x /root/sultan/target/release/sultan-node'
echo "✅ Binary installed"
echo ""

# Step 6: Restart service
echo "6️⃣  Starting sultan-node service..."
ssh -i "$SSH_KEY" "$SERVER" 'systemctl start sultan-node'
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
    echo "❌ WARNING! Shard count still shows $SHARD_COUNT (expected 8)"
    echo "Check logs: ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'journalctl -u sultan-node -f'"
fi

echo ""
echo "🎉 Deployment complete!"
