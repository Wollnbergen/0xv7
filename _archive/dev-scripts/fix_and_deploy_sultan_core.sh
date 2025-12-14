#!/bin/bash
# Fix and Deploy Sultan-Core to Production
# This script will fix the compilation error and deploy the correct binary

set -e

echo "=== Sultan-Core Production Deployment Script ===" 
echo "Date: $(date)"
echo

# Step 1: Upload clean main.rs to server
echo "Step 1: Uploading clean main.rs to production server..."
scp -i ~/.ssh/sultan_hetzner \
    /workspaces/0xv7/sultan-core/src/main.rs \
    root@5.161.225.96:/root/sultan/sultan-core/src/main.rs

echo "✅ File uploaded"
echo

# Step 2: Build sultan-node on server
echo "Step 2: Building sultan-node on production server..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 << 'ENDSSH'
cd /root/sultan
echo "Building sultan-node binary..."
cargo build --release --bin sultan-node 2>&1 | tee /root/sultan-core-build.log
if [ -f "/root/sultan/target/release/sultan-node" ]; then
    echo "✅ Build successful!"
    ls -lh /root/sultan/target/release/sultan-node
else
    echo "❌ Build failed!"
    tail -50 /root/sultan-core-build.log
    exit 1
fi
ENDSSH

echo "✅ Binary built successfully"
echo

# Step 3: Stop current node
echo "Step 3: Stopping current node..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'systemctl stop sultan-node'
echo "✅ Node stopped"
echo

# Step 4: Update systemd service
echo "Step 4: Updating systemd service..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 << 'ENDSSH'
cat > /etc/systemd/system/sultan-node.service << 'ENDSERVICE'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/sultan
ExecStart=/root/sultan/target/release/sultan-node --validator --enable-sharding --shard-count 8 --block-time 2 --rpc-addr 127.0.0.1:8080 --p2p-addr 0.0.0.0:8081
Restart=always
RestartSec=10
StandardOutput=append:/root/sultan/sultan-node.log
StandardError=append:/root/sultan/sultan-node.log

[Install]
WantedBy=multi-user.target
ENDSERVICE

systemctl daemon-reload
ENDSSH

echo "✅ Service updated"
echo

# Step 5: Start new node
echo "Step 5: Starting new sultan-node..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 'systemctl start sultan-node && sleep 3 && systemctl status sultan-node'
echo "✅ Node started"
echo

# Step 6: Verify it's running
echo "Step 6: Verifying node is operational..."
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 << 'ENDSSH'
echo "Checking logs..."
tail -30 /root/sultan/sultan-node.log
echo
echo "Testing RPC endpoint..."
curl -s http://localhost:8080/ || echo "RPC not responding yet"
echo
echo "Checking process..."
ps aux | grep sultan-node | grep -v grep
ENDSSH

echo
echo "=== Deployment Complete ==="
echo "✅ sultan-node is now running on production!"
echo
echo "Next steps:"
echo "1. Verify 2-second blocks in logs"
echo "2. Confirm 8 shards active"
echo "3. Test RPC endpoints"
echo "4. Create technical whitepaper"
