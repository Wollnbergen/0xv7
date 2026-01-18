# Sultan L1 Christmas Day 2025 Launch - DEPLOY NOW

## Quick Deploy (from your Mac)

\`\`\`bash
# Pull latest code
cd /path/to/0xv7  # wherever you cloned the repo
git pull origin feat/become-validator

# Deploy to all 6 validators
cd deploy
./deploy_validators.sh deploy
\`\`\`

## Validators
| Name | IP | Location |
|------|-----|----------|
| sultan-nyc | 134.122.96.36 | New York |
| sultan-sfo | 143.198.205.21 | San Francisco |
| sultan-fra | 142.93.238.33 | Frankfurt |
| sultan-ams | 46.101.122.13 | Amsterdam |
| sultan-sgp | 24.144.94.23 | Singapore |
| sultan-lon | 206.189.224.142 | London |

## Genesis
- **Wallet**: \`sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g\`
- **Balance**: 500,000,000,000,000 (500M with 6 decimals)

## What the deploy script does:
1. Stops existing sultan-node service
2. Uploads new binary (14.5MB)
3. Creates systemd service with:
   - Block time: 2s
   - Shards: 16
   - Genesis wallet configured
   - P2P peers configured
4. Starts service
5. Checks status

## Manual Deploy (if script fails)

For each validator, SSH and run:
\`\`\`bash
# Stop old
systemctl stop sultan-node 2>/dev/null || true

# Upload binary (from your Mac)
scp deploy/sultan-node root@134.122.96.36:/root/

# SSH to server
ssh root@134.122.96.36

# Create service
cat > /etc/systemd/system/sultan-node.service << 'EOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target

[Service]
Type=simple
ExecStart=/root/sultan-node \\
    --name sultan-nyc \\
    --data-dir /var/lib/sultan \\
    --block-time 2 \\
    --validator \\
    --validator-stake 1000000 \\
    --p2p-addr /ip4/0.0.0.0/tcp/26656 \\
    --rpc-addr 0.0.0.0:8545 \\
    --peers /ip4/134.122.96.36/tcp/26656,/ip4/143.198.205.21/tcp/26656,/ip4/142.93.238.33/tcp/26656 \\
    --genesis sultan15g5nwnlemn7zt6rtl7ch46ssvx2ym2v2umm07g:500000000000000 \\
    --enable-sharding \\
    --shard-count 16
Restart=always
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

# Start
mkdir -p /var/lib/sultan
systemctl daemon-reload
systemctl enable sultan-node
systemctl start sultan-node

# Check
systemctl status sultan-node
journalctl -u sultan-node -f
\`\`\`

## After Deployment - Verify Sync

Check all validators show SAME height:
\`\`\`bash
for ip in 134.122.96.36 143.198.205.21 142.93.238.33 46.101.122.13 24.144.94.23 206.189.224.142; do
  echo -n "\$ip: "
  curl -s http://\$ip:8545/status | jq -r '.block_height // .height // "N/A"'
done
\`\`\`

---

## Troubleshooting

### Website Shows "Network Stats Unavailable" (CORS Error)

**Symptoms:**
- sltn.io shows "—" for all network stats
- Browser console shows: `Access-Control-Allow-Origin cannot contain more than one origin`

**Root Cause:**
Both nginx AND sultan-node are adding CORS headers = duplicate headers = browser rejects

**Quick Fix:**
```bash
# Run the automated fix script
./scripts/fix_rpc_cors.sh
```

**Manual Fix:**
1. Remove CORS headers from nginx (`/etc/nginx/sites-available/rpc.sltn.io`)
2. Ensure sultan-node has `--allowed-origins "*"` flag in systemd service
3. Only ONE of them should handle CORS (we use sultan-node)

**Key Config Files on RPC Server (206.189.224.142):**
- `/etc/nginx/sites-available/rpc.sltn.io` - nginx proxy (NO CORS headers!)
- `/etc/systemd/system/sultan-node.service` - must have `--allowed-origins "*"`

---
**Merry Christmas 2025! 🎄 Sultan L1 Launch Day!**
