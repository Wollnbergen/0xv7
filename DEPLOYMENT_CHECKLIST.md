# Production Deployment Checklist
**Date:** December 6, 2025
**Operator:** Sultan Chain Team

---

## ✅ Completed

- [x] SSL certificate configured (Let's Encrypt)
- [x] Nginx reverse proxy setup
- [x] RPC endpoint live at https://rpc.sltn.io
- [x] Bug identified in shard_count reporting
- [x] Fix developed for main.rs line 433
- [x] Fix compiles successfully
- [x] Node producing blocks (5365+ blocks)
- [x] Zero fees confirmed working

---

## 🔄 In Progress

- [ ] **Building release binary** (currently compiling)
  - Command: `cargo build --release -p sultan-core`
  - Binary will be at: `target/release/sultan-node`
  - Status: ~23% complete (86/378 crates)

---

## 📋 Next Steps (After Build)

### Step 1: Deploy Fix to Production
```bash
./deploy_fix.sh
```
**Expected:**
- Upload new binary to server
- Stop sultan-node service
- Backup old binary
- Install new binary
- Restart service
- Verify shard_count shows 8 (not 100)

**Estimated Time:** 2 minutes

---

### Step 2: Verify Website Updates
**Check:**
1. https://rpc.sltn.io/status shows `"shard_count": 8`
2. https://sultan-blockchain.repl.co shows "64K TPS" (not 800K)
3. Explorer shows correct metrics

**If website still shows 800K TPS:**
- Website might cache API response
- Force refresh (Ctrl+F5)
- Or wait for cache timeout (~5 mins)

**Estimated Time:** 5 minutes

---

### Step 3: Deploy 10 Additional Validators

**Requirements per validator:**
- **Server:** Hetzner CX22 (~$10/month)
  - 2 vCPU cores
  - 4GB RAM
  - 80GB SSD
  - Ubuntu 24.04

- **Stake:** 10,000 SLTN per validator
- **Total Cost:** $100/month for 10 validators

**Deployment Process:**

#### 3a. Provision Server
```bash
# On Hetzner Cloud Console
1. Create new server (CX22)
2. Location: Choose different data centers (geographic distribution)
3. OS: Ubuntu 24.04
4. SSH Key: Add your key
5. Note IP address
```

#### 3b. Install Sultan Node
```bash
# SSH to new server
ssh root@<new-validator-ip>

# Install dependencies
apt update && apt install -y build-essential git curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/Wollnbergen/0xv7.git
cd 0xv7/sultan-core
cargo build --release

# Create systemd service
cat > /etc/systemd/system/sultan-node.service << EOF
[Unit]
Description=Sultan Blockchain Node
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/0xv7/sultan-core
ExecStart=/root/0xv7/target/release/sultan-node \\
  --validator \\
  --validator-address validator_1 \\
  --validator-stake 10000 \\
  --enable-sharding \\
  --shard-count 8 \\
  --block-time 2 \\
  --rpc-addr 0.0.0.0:8080 \\
  --p2p-addr /ip4/0.0.0.0/tcp/26656 \\
  --seed-nodes /ip4/5.161.225.96/tcp/26656
Restart=always
RestartSec=3
StandardOutput=append:/root/sultan-node.log
StandardError=append:/root/sultan-node.log

[Install]
WantedBy=multi-user.target
EOF

# Start service
systemctl daemon-reload
systemctl enable sultan-node
systemctl start sultan-node

# Verify
systemctl status sultan-node
tail -f /root/sultan-node.log
```

#### 3c. Repeat for Validators 2-10
**Validator Addresses:**
- validator_1 (done above)
- validator_2 through validator_10

**Geographic Distribution:**
- validator_1, validator_2: Falkenstein, Germany
- validator_3, validator_4: Nuremberg, Germany
- validator_5, validator_6: Helsinki, Finland
- validator_7, validator_8: Ashburn, USA
- validator_9, validator_10: Hillsboro, USA

**Estimated Time:** 2 hours (15 mins per validator)

---

### Step 4: Set Up Monitoring

**Prometheus Configuration:**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sultan-validators'
    static_configs:
      - targets:
        - '5.161.225.96:9310'      # validator_0
        - '<validator-1-ip>:9310'   # validator_1
        - '<validator-2-ip>:9310'   # validator_2
        # ... add all 11 validators
```

**Grafana Dashboard:**
- Import template from `/monitoring/grafana-dashboard.json`
- Metrics to monitor:
  - Block height per validator
  - Block proposal success rate
  - Validator uptime
  - TPS per shard
  - Cross-shard transaction latency
  - Network bandwidth

**Alerts:**
- Validator downtime > 5 minutes
- Block height lag > 10 blocks
- TPS > 80% capacity (auto-scale trigger)

**Estimated Time:** 1 hour

---

### Step 5: Load Testing

**Test Plan:**
```bash
# Install k6
curl https://github.com/grafana/k6/releases/download/v0.49.0/k6-v0.49.0-linux-amd64.tar.gz -L | tar xvz
sudo mv k6 /usr/local/bin

# Run TPS test
k6 run load-test.js

# Test scenarios:
# 1. Baseline: 1K TPS (should handle easily)
# 2. Target: 32K TPS (50% capacity)
# 3. Stress: 51K TPS (80% capacity - should trigger auto-scale)
# 4. Max: 64K TPS (100% capacity)
```

**Success Criteria:**
- ✅ < 3s finality at 32K TPS
- ✅ Auto-scale triggers at 51K TPS
- ✅ Stable block production at 64K TPS
- ✅ No validator failures

**Estimated Time:** 2 hours

---

## 📊 Expected Final State

| Metric | Current | After Deployment |
|--------|---------|-----------------|
| Validators | 1 | 11 ✅ |
| Shard Count (Reported) | 100 ❌ | 8 ✅ |
| TPS Capacity (Reported) | 800K ❌ | 64K ✅ |
| Geographic Locations | 1 | 5 ✅ |
| Monitoring | None ❌ | Prometheus+Grafana ✅ |
| Load Tested | No ❌ | Yes ✅ |
| Byzantine Fault Tolerance | No (single validator) ❌ | Yes (tolerates 3 malicious) ✅ |

---

## 🎯 Success Metrics

**Immediately After Fix Deployment:**
1. `curl https://rpc.sltn.io/status | jq .shard_count` returns `8`
2. Website shows 64,000 TPS capacity
3. No service interruption during deployment

**After Full Validator Deployment:**
1. 11 validators actively producing blocks
2. Block production rotates between validators
3. Consensus reached within 2 seconds
4. Zero downtime during validator additions

**After Load Testing:**
1. Sustained 32K TPS for 10 minutes
2. Auto-scaling triggers at 80% load
3. Sub-3s finality maintained under load

---

## 🚨 Rollback Plan

**If deployment fails:**
```bash
# SSH to production
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96

# Stop new binary
systemctl stop sultan-node

# Restore backup
ls -lh /root/sultan/target/release/sultan-node.backup-*
cp /root/sultan/target/release/sultan-node.backup-YYYYMMDD-HHMMSS \
   /root/sultan/target/release/sultan-node

# Restart
systemctl start sultan-node

# Verify
systemctl status sultan-node
curl localhost:8080/status | jq
```

---

## 📞 Emergency Contacts

- **Primary:** Discord #mainnet-operations
- **Secondary:** Telegram @sultan_devops
- **Email:** emergency@sltn.io

---

## ⏰ Timeline

| Task | Duration | Can Start |
|------|----------|-----------|
| Build binary | ~30 mins | Now ⏳ |
| Deploy fix | 2 mins | After build |
| Verify | 5 mins | After deploy |
| Setup validator 1 | 15 mins | After verify |
| Setup validators 2-10 | 2 hours | Parallel |
| Configure monitoring | 1 hour | Parallel |
| Load testing | 2 hours | After validators |
| **TOTAL** | **~6 hours** | |

---

## 📝 Post-Deployment Report Template

```markdown
# Sultan Mainnet Deployment - [DATE]

## Summary
- Deployed fix for shard_count reporting bug
- Added X validators (total: Y)
- Completed load testing

## Metrics
- Block height before: ____
- Block height after: ____
- Downtime: ____ seconds
- Failed blocks: ____

## Issues Encountered
1. [Issue description]
   - Resolution: [how it was fixed]

## Performance Results
- Max TPS achieved: ____
- Finality time: ____
- Auto-scale triggered: Yes/No

## Next Steps
- [ ] Monitor for 24 hours
- [ ] Community announcement
- [ ] Update documentation
```

---

**Current Status:** Waiting for build to complete (~70% done)
**Next Action:** Run `./deploy_fix.sh` when build finishes
