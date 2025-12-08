# Sultan L1 - Production Deployment Plan

## 📋 Pre-Deployment Checklist

### ✅ Already Completed
- [x] Production sharding code (8→8000 shards)
- [x] BFT consensus implementation
- [x] Zero-fee bridge system (5 chains)
- [x] Quantum-resistant crypto
- [x] Governance system
- [x] Website with RPC integration
- [x] SSL certificates (valid until 2026-03-05)
- [x] Nginx reverse proxy configured

### ⏳ In Progress
- [ ] Binary compilation (sultan-core)
- [ ] Deployment to production server
- [ ] Service configuration update

---

## 🚀 Deployment Steps (Execute in Order)

### Step 1: Verify Build Complete (2 min)
```bash
# Check if binary exists
ls -lh /workspaces/0xv7/sultan-core/target/release/sultan-core

# Verify it's executable
file /workspaces/0xv7/sultan-core/target/release/sultan-core

# Check binary size (should be ~20-30 MB)
du -h /workspaces/0xv7/sultan-core/target/release/sultan-core
```

**Expected Output:**
```
-rwxr-xr-x 1 codespace codespace 25M Dec 6 10:30 sultan-core
sultan-core: ELF 64-bit LSB executable, x86-64
25M     sultan-core
```

---

### Step 2: Stop Current Production Node (1 min)
```bash
# SSH to production server
ssh root@5.161.225.96

# Check current status
systemctl status sultan-node

# Stop the service (currently running wrong binary)
systemctl stop sultan-node

# Verify it's stopped
systemctl status sultan-node
# Should show: "inactive (dead)"
```

---

### Step 3: Backup Current Setup (2 min)
```bash
# On production server (5.161.225.96)

# Backup old binary
cp /root/sultan/target/release/p2p_node /root/sultan/backup/p2p_node.$(date +%Y%m%d)

# Backup data directory (if exists)
if [ -d /var/lib/sultan ]; then
    tar -czf /root/sultan/backup/sultan-data-$(date +%Y%m%d).tar.gz /var/lib/sultan
fi

# Backup systemd service file
cp /etc/systemd/system/sultan-node.service /root/sultan/backup/sultan-node.service.old

echo "✅ Backup complete"
ls -lh /root/sultan/backup/
```

---

### Step 4: Deploy New Binary (3 min)
```bash
# On LOCAL machine (dev container)

# Deploy sultan-core to production
scp /workspaces/0xv7/sultan-core/target/release/sultan-core \
    root@5.161.225.96:/usr/local/bin/sultand

# Verify upload
ssh root@5.161.225.96 'ls -lh /usr/local/bin/sultand'

# Make executable
ssh root@5.161.225.96 'chmod +x /usr/local/bin/sultand'

# Test binary works
ssh root@5.161.225.96 '/usr/local/bin/sultand --version || echo "No version flag, but binary exists"'
```

**Expected:** Binary transferred (~25MB), executable permissions set

---

### Step 5: Update Systemd Service (3 min)
```bash
# On production server

# Create new service configuration
cat > /etc/systemd/system/sultan-node.service << 'EOF'
[Unit]
Description=Sultan L1 Blockchain Node
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=5
User=root
WorkingDirectory=/var/lib/sultan

# Production command with all flags
ExecStart=/usr/local/bin/sultand \
  --validator \
  --enable-sharding \
  --shard-count 8 \
  --max-shards 8000 \
  --rpc-addr 0.0.0.0:8080 \
  --block-time 2 \
  --data-dir /var/lib/sultan

# Resource limits
LimitNOFILE=65536
LimitNPROC=32768

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sultan-node

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

# Show new configuration
systemctl cat sultan-node.service
```

---

### Step 6: Create Data Directory (1 min)
```bash
# On production server

# Create data directory with proper permissions
mkdir -p /var/lib/sultan/blocks
mkdir -p /var/lib/sultan/config
chown -R root:root /var/lib/sultan
chmod 755 /var/lib/sultan

echo "✅ Data directory ready"
ls -la /var/lib/sultan/
```

---

### Step 7: Start Production Node (2 min)
```bash
# On production server

# Start the service
systemctl start sultan-node

# Check status immediately
systemctl status sultan-node

# Should show:
# ● sultan-node.service - Sultan L1 Blockchain Node
#    Loaded: loaded
#    Active: active (running)

# Watch logs in real-time
journalctl -u sultan-node -f
```

**Expected Log Output:**
```
Dec 06 10:35:00 sultan-node[1234]: 🚀 Sultan Core v0.1.0 starting...
Dec 06 10:35:00 sultan-node[1234]: 🔧 Initializing consensus engine
Dec 06 10:35:00 sultan-node[1234]: ✅ Validator added: validator_0 (10000 SLTN)
Dec 06 10:35:00 sultan-node[1234]: ✅ Validator added: validator_1 (10000 SLTN)
...
Dec 06 10:35:01 sultan-node[1234]: 🚀 PRODUCTION SHARDING: 8 shards (expandable to 8000)
Dec 06 10:35:01 sultan-node[1234]: 🌐 RPC server listening on 0.0.0.0:8080
Dec 06 10:35:01 sultan-node[1234]: ⛏️  Block production starting...
Dec 06 10:35:03 sultan-node[1234]: ✅ Block #1 created by validator_0
Dec 06 10:35:05 sultan-node[1234]: ✅ Block #2 created by validator_3
```

---

### Step 8: Verify Block Production (3 min)
```bash
# From ANY machine (local or remote)

# Watch block height increment
watch -n 1 'curl -s https://rpc.sltn.io/status | jq -r "\"Block: \(.height) | Validators: \(.validator_count) | Shards: \(.shard_count) | TPS: \(.tps)\""'

# Expected output (updates every second):
# Block: 1 | Validators: 11 | Shards: 8 | TPS: 0
# Block: 2 | Validators: 11 | Shards: 8 | TPS: 0
# Block: 3 | Validators: 11 | Shards: 8 | TPS: 0
# ... (should increment every 2 seconds)
```

**Success Criteria:**
- ✅ Block height increases: 1, 2, 3, 4...
- ✅ Increases every 2 seconds (block time)
- ✅ All 11 validators active
- ✅ Shards: 8 (initial production config)

---

### Step 9: Verify RPC Endpoints (2 min)
```bash
# Test all endpoints

# 1. Root endpoint
curl https://rpc.sltn.io/
# Expected: "Sultan eternal node ready"

# 2. Status endpoint
curl https://rpc.sltn.io/status | jq

# Expected JSON with:
# - height > 0
# - validator_count: 11
# - shard_count: 8
# - total_supply: 535000000

# 3. Bridges endpoint
curl https://rpc.sltn.io/bridges | jq

# Expected: 5 bridges (BTC, ETH, SOL, TON, IBC)

# 4. Health check
curl https://rpc.sltn.io/health

# Expected: HTTP 200 OK
```

---

### Step 10: Verify Website Integration (1 min)
```bash
# Open website in browser
# https://sultan.network (or your website URL)

# Check that stats update automatically:
# - Block Height: Should increment every 2 seconds
# - Validators: 11
# - Shards: 8 (was 1024, now correctly 8)
# - Bridge indicators: 🟢 Active for BTC/ETH/SOL/TON

# Or check programmatically:
curl -s https://sultan.network | grep -o "blockHeight.*[0-9]\+" | head -1
```

---

## 🔍 Post-Deployment Verification

### Consensus Health Check
```bash
# On production server
journalctl -u sultan-node --since "5 minutes ago" | grep -E "Block|Proposer|validator"

# Should show:
# - Different validators proposing blocks (rotation)
# - Blocks being signed by 2/3+1 validators
# - No consensus errors
```

### Shard Status Check
```bash
# Check initial shard configuration
curl -s https://rpc.sltn.io/status | jq '{shards: .shard_count, max_shards: .max_shards, auto_expand: .auto_expand_enabled}'

# Expected:
# {
#   "shards": 8,
#   "max_shards": 8000,
#   "auto_expand": true
# }
```

### Resource Usage Check
```bash
# On production server
top -bn1 | grep sultand
# Check CPU and memory usage (should be reasonable)

df -h /var/lib/sultan
# Check disk space (blocks directory)

netstat -tlnp | grep 8080
# Verify RPC port listening
```

---

## ⚠️ Troubleshooting

### Problem: Binary won't start
```bash
# Check logs
journalctl -u sultan-node -n 50

# Check binary permissions
ls -la /usr/local/bin/sultand

# Check data directory permissions
ls -la /var/lib/sultan

# Try running manually
/usr/local/bin/sultand --validator --enable-sharding --shard-count 8 --rpc-addr 0.0.0.0:8080
```

### Problem: Blocks not producing (height = 0)
```bash
# Check if --validator flag is set
systemctl cat sultan-node.service | grep validator

# Check consensus logs
journalctl -u sultan-node | grep -i consensus

# Verify validators initialized
curl -s https://rpc.sltn.io/status | jq .validator_count
```

### Problem: RPC endpoint not responding
```bash
# Check if service is running
systemctl status sultan-node

# Check if port 8080 is listening
netstat -tlnp | grep 8080

# Check nginx is forwarding
nginx -t
systemctl status nginx

# Test direct connection (bypass nginx)
curl http://127.0.0.1:8080/status
```

### Problem: High CPU/Memory usage
```bash
# Check resource usage
top -u root

# Reduce shard count temporarily
# Edit service file: --shard-count 4
systemctl daemon-reload
systemctl restart sultan-node
```

---

## 🎯 Success Checklist

After deployment, verify ALL of these:

- [ ] systemctl status sultan-node → **Active: active (running)**
- [ ] curl https://rpc.sltn.io/status → **height > 0**
- [ ] Block height incrementing → **Every 2 seconds**
- [ ] journalctl shows block production → **No errors**
- [ ] Validators rotating → **Different proposers**
- [ ] Shards initialized → **8 shards active**
- [ ] RPC endpoints responding → **All 3 endpoints working**
- [ ] Website showing live data → **Stats updating**
- [ ] No error logs → **journalctl clean**
- [ ] Resource usage normal → **CPU < 50%, RAM reasonable**

---

## 📊 Monitoring Setup (Next Phase)

After successful deployment, set up monitoring:

1. **Prometheus** - Metrics collection
2. **Grafana** - Dashboards
3. **AlertManager** - Alerts
4. **Backup cron job** - Daily snapshots

See `PRODUCTION_READINESS.md` for detailed monitoring setup.

---

## 🔄 Rollback Plan (If Needed)

If deployment fails:

```bash
# On production server

# 1. Stop new service
systemctl stop sultan-node

# 2. Restore old service file
cp /root/sultan/backup/sultan-node.service.old /etc/systemd/system/sultan-node.service
systemctl daemon-reload

# 3. Restore old binary path in service file
# (Edit to point back to /root/sultan/target/release/p2p_node)

# 4. Restart old service
systemctl start sultan-node

# 5. Verify old system working
curl https://rpc.sltn.io/
```

---

## 📅 Timeline Estimate

| Step | Task | Time | Cumulative |
|------|------|------|------------|
| 1 | Verify build | 2 min | 2 min |
| 2 | Stop current node | 1 min | 3 min |
| 3 | Backup current setup | 2 min | 5 min |
| 4 | Deploy new binary | 3 min | 8 min |
| 5 | Update systemd service | 3 min | 11 min |
| 6 | Create data directory | 1 min | 12 min |
| 7 | Start production node | 2 min | 14 min |
| 8 | Verify block production | 3 min | 17 min |
| 9 | Verify RPC endpoints | 2 min | 19 min |
| 10 | Verify website | 1 min | 20 min |

**Total Estimated Time: ~20 minutes**

---

## 🚀 Ready to Deploy?

Once build completes, execute these steps in order. The blockchain will be producing blocks within 20 minutes!

**Next:** Wait for `cargo build --release` to finish, then execute deployment steps.
