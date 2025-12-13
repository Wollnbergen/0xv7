# 🏰 Sultan L1 - Session Restart Guide
**Date:** November 23, 2025  
**Status:** Genesis validator running, websites deployed

---

## ✅ CURRENT STATUS

### Blockchain Status - ✅ CORRECT ARCHITECTURE RUNNING!

**Currently Running (CORRECT!):**
- **Chain:** Sultan-1 (Pure Rust - Sultan Core) ✅ Correct architecture
- **Implementation:** `/tmp/cargo-target/release/sultan-node`
- **Data Dir:** `/workspaces/0xv7/sultan-core/sultan-data/`
- **Current Height:** 2024+ blocks (5-second block time)
- **Validator:** genesis (500M SLTN stake)
- **RPC:** http://0.0.0.0:26657
- **Status:** ✅ **SULTAN-FIRST ARCHITECTURE ACHIEVED!**

**Architecture (As Planned):**
- ✅ **Layer 1:** Sultan Core (Rust) - YOUR blockchain ← **RUNNING NOW**
- ⏳ **Layer 2:** Cosmos Bridge (FFI) - Compatibility wrapper ← To be added
- ⏳ **Layer 3:** Cosmos SDK modules - IBC, Keplr, etc. ← To be added

**Genesis Account:**
- Address: `genesis`
- Balance: 500,000,000 SLTN (500000000000000 usltn)
- Nonce: 0

**Old Cosmos SDK Node:**
- **Status:** Stopped (was wrong architecture)
- **Location:** `/workspaces/0xv7/sultan-cosmos-real/sultand`
- **Note:** Can be used later as Layer 3 reference

### Endpoints (Currently Localhost Only)
- **RPC:** http://localhost:26657 (port 26657)
- **REST API:** http://localhost:1317 (port 1317)
- **P2P:** Port 26656
- **CORS:** ✅ Enabled for both RPC and REST

### Configuration Files
- **Genesis:** `~/.sultan/config/genesis.json`
- **App Config:** `~/.sultan/config/app.toml` (API enabled, CORS on)
- **Node Config:** `~/.sultan/config/config.toml` (RPC on 0.0.0.0, CORS on)

### Websites
1. **Full Website:** https://wollnbergen.github.io/SULTAN
   - Status: Deployed to GitHub (manual fix done)
   - File: `index.html` in SULTAN repo
   - GitHub Pages: Should be live (check if working)

2. **Replit Landing Page:** Ready to deploy
   - Location: `/workspaces/0xv7/replit-website/`
   - Files: `index.html`, `.replit`, `README.md`
   - Status: Not deployed yet (user logs in and uploads)

3. **SDK Repository:** https://github.com/Wollnbergen/BUILD
   - Status: ✅ Complete (4/5 files uploaded)
   - Missing: LICENSE file (user needs to upload)

---

## 🚀 HOW TO START THE VALIDATOR WHEN YOU LOG BACK IN

### Quick Start Command
```bash
# Check if node is already running
ps aux | grep sultand | grep -v grep

# If not running, start it:
cd /workspaces/0xv7/sultan-cosmos-real
nohup ./sultand start > sultan.log 2>&1 &

# Wait 5 seconds then verify:
sleep 5
curl -s http://localhost:26657/status | jq '.result.sync_info | {height: .latest_block_height, catching_up}'
```

### Verify All Endpoints
```bash
# RPC endpoint
curl -s http://localhost:26657/status | jq '.result.node_info.network'
# Should return: "sultan-1"

# REST API endpoint
curl -s http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info | jq '.default_node_info.network'
# Should return network info

# Check ports listening
netstat -tlnp 2>/dev/null | grep sultand
# Should show: 26657 (RPC), 1317 (REST API), 26656 (P2P)
```

---

## 📋 IMMEDIATE TODO LIST

### 1. Check GitHub Pages Deployment
```bash
# Verify website is live
curl -s https://wollnbergen.github.io/SULTAN/ | grep -i "sultan" | head -5

# If still showing 404, manually:
# 1. Go to https://github.com/Wollnbergen/SULTAN/settings/pages
# 2. Change source to "Deploy from a branch"
# 3. Select "main" and "/" (root)
# 4. Save and wait 2 minutes
```

### 2. Upload LICENSE to BUILD Repo
File content already prepared. Manually create file named `LICENSE` in BUILD repo with:
```
MIT License

Copyright (c) 2024 Sultan L1 Core Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### 3. Deploy Replit Landing Page (Optional)
1. Go to https://replit.com/new/html
2. Upload files from `/workspaces/0xv7/replit-website/`:
   - `index.html`
   - `.replit`
3. Click "Run"
4. Share URL: `https://sultan-l1.YOUR_USERNAME.repl.co`

---

## 🔧 TROUBLESHOOTING

### If Node Won't Start
```bash
# Check logs
tail -50 /workspaces/0xv7/sultan-cosmos-real/sultan.log

# Common issues:
# 1. Port already in use
pkill sultand
sleep 3
cd /workspaces/0xv7/sultan-cosmos-real && nohup ./sultand start > sultan.log 2>&1 &

# 2. Corrupted state
# (Last resort - loses blockchain data)
# rm -rf ~/.sultan/data
# ./sultand init validator1 --chain-id sultan-1
```

### If RPC Not Responding
```bash
# Check if listening
netstat -tlnp 2>/dev/null | grep 26657

# Check config
grep "laddr.*26657" ~/.sultan/config/config.toml
# Should be: tcp://0.0.0.0:26657

# Check CORS
grep "cors_allowed_origins" ~/.sultan/config/config.toml
# Should be: cors_allowed_origins = ["*"]
```

### If REST API Not Working
```bash
# Check if enabled
grep -A 2 "^\[api\]" ~/.sultan/config/app.toml
# enable should be true

# Check address
grep "address.*1317" ~/.sultan/config/app.toml
# Should be: tcp://0.0.0.0:1317

# Check CORS
grep "enabled-unsafe-cors" ~/.sultan/config/app.toml
# Should be: true
```

---

## 📊 NETWORK INFORMATION

### Chain Configuration
- **Chain ID:** sultan-1
- **Total Supply:** 500,000,000 SLTN
- **Min Validator Stake:** 10,000 SLTN
- **Validator APY:** 13.33%
- **Delegator APY:** 10%
- **Gas Fees:** $0.00 (zero forever)
- **Block Time:** ~6 seconds
- **Token Decimals:** 6 (1 SLTN = 1,000,000 usltn)

### Genesis Account
Check with:
```bash
cd /workspaces/0xv7/sultan-cosmos-real
./sultand keys list
```

### Current Validator
```bash
./sultand comet show-validator
# Shows validator public key
```

---

## 🚨 ARCHITECTURE CORRECTION NEEDED

**Your Plan Says:**
- Layer 1: Sultan Core (Rust) - YOUR blockchain with YOUR rules
- Layer 2: Cosmos Bridge - FFI wrapper for compatibility
- Layer 3: Cosmos modules - IBC, Keplr, etc. (optional)

**What's Running:**
- Full Cosmos SDK (Go) - Not your code, Cosmos rules, wrong!

**Next Session Priority:**
1. Build Sultan Rust node (`sultan-core`)
2. Migrate chain data if needed
3. Stop Cosmos SDK node
4. Start Sultan Rust node
5. Later: Add Cosmos bridge as Layer 2

## 🎯 NEXT STEPS (After Restart)

### Phase 1: Verify Current (Wrong) Setup Works (5 minutes)
1. ✅ Start validator node
2. ✅ Check RPC responding
3. ✅ Check REST API responding
4. ✅ Verify block production
5. ✅ Check GitHub Pages live

### Phase 2: Make Endpoints Public (Production)
**Current:** Endpoints only work on localhost  
**Goal:** Make accessible at rpc.sultan.network and api.sultan.network

**Options:**
- **A) Port Forwarding in Codespaces:**
  ```bash
  # Make ports public in VS Code
  # Ports panel → Right-click 26657 → Port Visibility → Public
  # Ports panel → Right-click 1317 → Port Visibility → Public
  # Get forwarded URLs from ports panel
  ```

- **B) Deploy to Production Server:**
  - Copy sultand binary to server
  - Copy ~/.sultan/ directory
  - Setup systemd service
  - Configure nginx/caddy reverse proxy
  - Setup DNS (rpc.sultan.network → server IP)
  - Enable HTTPS with Let's Encrypt

### Phase 3: Test Keplr Integration
Once endpoints are public:
1. Update website RPC/REST URLs (if needed)
2. Open https://wollnbergen.github.io/SULTAN
3. Click "Connect Wallet"
4. Test balance display
5. Test validator registration

---

## 📁 KEY FILE LOCATIONS

### Workspace Files
- **Main Website:** `/workspaces/0xv7/index.html` (1,757 lines)
- **Replit Site:** `/workspaces/0xv7/replit-website/index.html`
- **SDK Files:** `/workspaces/0xv7/sultan-sdk/` (sdk.rs, Cargo.toml, etc.)
- **This Guide:** `/workspaces/0xv7/SESSION_RESTART_GUIDE.md`

### Blockchain Files
- **Binary:** `/workspaces/0xv7/sultan-cosmos-real/sultand`
- **Config:** `~/.sultan/config/`
- **Data:** `~/.sultan/data/`
- **Keys:** `~/.sultan/keyring-test/`

### Documentation
- **Tomorrow Checklist:** `/workspaces/0xv7/TOMORROW_CHECKLIST.md`
- **Architecture Plan:** Various planning docs in workspace
- **Actual Status:** `/workspaces/0xv7/ACTUAL_TRUE_STATUS.md` (outdated - from Nov 20)

---

## 🔐 SECURITY REMINDERS

### DO NOT COMMIT:
- Private keys in `~/.sultan/config/priv_validator_key.json`
- Node keys in `~/.sultan/config/node_key.json`
- Keyring files
- Any mnemonics or seeds

### .gitignore Already Configured
The updated `.gitignore` protects:
- `priv_validator_key.json`
- `node_key.json`
- `keyring-test/`
- `keyring-file/`
- All `*.key` and `*.pem` files

---

## 🚨 EMERGENCY CONTACTS

If something breaks:

1. **Check logs first:**
   ```bash
   tail -100 /workspaces/0xv7/sultan-cosmos-real/sultan.log
   ```

2. **Check this guide** for troubleshooting section

3. **Check network status:**
   ```bash
   curl -s http://localhost:26657/status | jq .
   ```

4. **Last resort - restart fresh:**
   ```bash
   pkill sultand
   cd /workspaces/0xv7/sultan-cosmos-real
   ./sultand start
   ```

---

## 📝 WHAT WE ACCOMPLISHED TODAY

1. ✅ Updated .gitignore with security protections
2. ✅ Discovered working sultand binary (Cosmos SDK)
3. ✅ Verified blockchain running (2276+ blocks)
4. ✅ Enabled REST API in config
5. ✅ Enabled CORS for RPC and REST
6. ✅ Configured endpoints for external access
7. ✅ Verified chain ID: sultan-1
8. ✅ Created Replit landing page
9. ✅ Fixed GitHub Pages deployment (manual)
10. ✅ Created this restart guide

---

## 💡 QUICK REFERENCE COMMANDS

```bash
# Start node
cd /workspaces/0xv7/sultan-cosmos-real && nohup ./sultand start > sultan.log 2>&1 &

# Check status
curl -s http://localhost:26657/status | jq '.result.sync_info.latest_block_height'

# Stop node
pkill sultand

# View logs
tail -f /workspaces/0xv7/sultan-cosmos-real/sultan.log

# Check processes
ps aux | grep sultand

# List keys
cd /workspaces/0xv7/sultan-cosmos-real && ./sultand keys list

# Check validators
./sultand query staking validators --chain-id sultan-1
```

---

**🎯 TL;DR - First 3 Commands When You Log Back In:**

```bash
# 1. Start the Sultan Rust blockchain
cd /workspaces/0xv7/sultan-core && /tmp/cargo-target/release/sultan-node \
  --name "genesis-validator" --validator \
  --validator-address "genesis" \
  --validator-stake 500000000000000 \
  --genesis "genesis:500000000000000" \
  --data-dir ./sultan-data \
  --rpc-addr "0.0.0.0:26657" \
  --block-time 5 > sultan-node.log 2>&1 &

# 2. Verify it's running (wait 5 seconds)
sleep 5 && curl -s http://localhost:26657/status | jq '.'

# 3. Check balance
curl -s http://localhost:26657/balance/genesis | jq '.'
```

**If those work, you're good to go! 🚀**

---

*Last Updated: November 23, 2025 - Session End*
*Next Session: Start with validator restart, verify websites, make endpoints public*
