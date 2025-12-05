# 📋 NEXT SESSION: SULTAN LAYER 3 IMPLEMENTATION

**Current Status:** ✅ L1 Running (Block 2876+) + ✅ L2 Go Bridge Complete  
**Next Milestone:** 🎯 Build Cosmos SDK Integration (Layer 3)  
**Session Date:** TBD (after this session)

---

## 🎯 PRIMARY GOAL: COSMOS SDK MODULE (Layer 3)

Build the final layer that enables full Cosmos ecosystem compatibility:
- REST API server for Keplr wallet
- gRPC services for SDK clients
- IBC protocol for cross-chain transfers
- Native Cosmos SDK module wrapping our Go bridge

---

## ✅ PRE-SESSION VERIFICATION

Before starting new work, verify everything from this session is still working:

### 1. Check Sultan Node Status
```bash
# Verify process running
ps aux | grep sultan-node | grep -v grep

# Expected: PID 60815 or higher, 0.0% CPU, 0.2% MEM
```

### 2. Check Block Height
```bash
# Query RPC endpoint
curl -s http://localhost:26657/status | jq '.height'

# Expected: >2876 (incrementing every 5 seconds)
```

### 3. Verify Go Bridge Tests
```bash
cd /workspaces/0xv7/sultan-cosmos-go
export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH
CGO_ENABLED=1 go test -v

# Expected: 5/5 tests passing
```

### 4. Check Logs for Errors
```bash
tail -50 /workspaces/0xv7/sultan-core/sultan-node.log | grep -i error

# Expected: No critical errors (genesis logs OK)
```

**If any checks fail:** See "TROUBLESHOOTING" section below.

---

## 🚀 LAYER 3 IMPLEMENTATION PLAN

### Phase 1: Cosmos SDK Module (Week 1)

**Goal:** Create `x/sultan` module that wraps the Go bridge

**Steps:**
1. Create new Go package: `sultan-cosmos-sdk/`
   ```bash
   cd /workspaces/0xv7
   mkdir -p sultan-cosmos-sdk/{x/sultan/{keeper,types,client},cmd/sultand}
   cd sultan-cosmos-sdk
   go mod init github.com/Wollnbergen/0xv7/sultan-cosmos-sdk
   ```

2. Implement Keeper (uses sultan-cosmos-go bridge)
   ```go
   // x/sultan/keeper/keeper.go
   type Keeper struct {
       bridge *sultancosmos.SultanBridge
       cdc    codec.BinaryCodec
   }
   
   func (k Keeper) GetBalance(ctx sdk.Context, addr sdk.AccAddress) sdk.Coin
   func (k Keeper) SetBalance(ctx sdk.Context, addr sdk.AccAddress, amount sdk.Coin)
   func (k Keeper) Transfer(ctx sdk.Context, from, to sdk.AccAddress, amount sdk.Coin)
   ```

3. Define Message Types (transactions)
   ```go
   // x/sultan/types/msgs.go
   type MsgSend struct {
       FromAddress string
       ToAddress   string
       Amount      sdk.Coin
   }
   ```

4. Add Query Service (gRPC)
   ```protobuf
   // proto/sultan/v1/query.proto
   service Query {
       rpc Balance(QueryBalanceRequest) returns (QueryBalanceResponse);
       rpc Height(QueryHeightRequest) returns (QueryHeightResponse);
   }
   ```

5. Wire up to Cosmos SDK app
   ```go
   // app/app.go
   sultanKeeper := sultankeeper.NewKeeper(
       appCodec,
       keys[sultantypes.StoreKey],
       bridge,
   )
   ```

**Deliverables:**
- [ ] x/sultan module compiles
- [ ] Keeper implements bank-like interface
- [ ] Messages for send/receive
- [ ] gRPC queries working

---

### Phase 2: REST API Server (Week 1-2)

**Goal:** HTTP API for Keplr wallet and web apps

**Steps:**
1. Add REST endpoints
   ```go
   // x/sultan/client/rest/rest.go
   func RegisterRoutes(r *mux.Router, keeper keeper.Keeper) {
       // Cosmos-standard endpoints
       r.HandleFunc("/cosmos/bank/v1beta1/balances/{address}", 
           queryBalance).Methods("GET")
       r.HandleFunc("/cosmos/tx/v1beta1/txs", 
           broadcastTx).Methods("POST")
   }
   ```

2. Implement Cosmos-compatible responses
   ```json
   // GET /cosmos/bank/v1beta1/balances/sultan1abc...
   {
     "balances": [
       {
         "denom": "usltn",
         "amount": "500000000000000"
       }
     ]
   }
   ```

3. Add CORS support for browser apps
   ```go
   c := cors.New(cors.Options{
       AllowedOrigins: []string{"*"},
       AllowedMethods: []string{"GET", "POST"},
   })
   ```

4. Test with curl/Postman
   ```bash
   # Query balance
   curl http://localhost:1317/cosmos/bank/v1beta1/balances/genesis
   
   # Submit transaction
   curl -X POST http://localhost:1317/cosmos/tx/v1beta1/txs \
     -d '{"tx_bytes":"...","mode":"BROADCAST_MODE_SYNC"}'
   ```

**Deliverables:**
- [ ] REST server running on port 1317
- [ ] Balance queries work
- [ ] Transaction submission works
- [ ] CORS enabled for browsers

---

### Phase 3: Keplr Integration (Week 2)

**Goal:** Connect Sultan to Keplr wallet

**Steps:**
1. Create chain registration file
   ```javascript
   // keplr-config.js
   const sultanChainInfo = {
       chainId: "sultan-1",
       chainName: "Sultan Network",
       rpc: "http://localhost:26657",
       rest: "http://localhost:1317",
       bip44: { coinType: 118 },
       bech32Config: {
           bech32PrefixAccAddr: "sultan",
           bech32PrefixAccPub: "sultanpub",
           bech32PrefixValAddr: "sultanvaloper",
           bech32PrefixValPub: "sultanvaloperpub",
       },
       currencies: [{
           coinDenom: "SLTN",
           coinMinimalDenom: "usltn",
           coinDecimals: 6,
       }],
       feeCurrencies: [{
           coinDenom: "SLTN",
           coinMinimalDenom: "usltn",
           coinDecimals: 6,
       }],
       stakeCurrency: {
           coinDenom: "SLTN",
           coinMinimalDenom: "usltn",
           coinDecimals: 6,
       },
   };
   ```

2. Add Keplr to website (index.html)
   ```javascript
   async function connectKeplr() {
       if (!window.keplr) {
           alert("Please install Keplr extension");
           return;
       }
       
       // Suggest chain to Keplr
       await window.keplr.experimentalSuggestChain(sultanChainInfo);
       
       // Enable Keplr for Sultan
       await window.keplr.enable("sultan-1");
       
       // Get offlineSigner
       const offlineSigner = window.keplr.getOfflineSigner("sultan-1");
       const accounts = await offlineSigner.getAccounts();
       
       console.log("Connected:", accounts[0].address);
   }
   ```

3. Test transaction signing
   ```javascript
   const tx = {
       msgs: [{
           typeUrl: "/cosmos.bank.v1beta1.MsgSend",
           value: {
               fromAddress: accounts[0].address,
               toAddress: "sultan1xyz...",
               amount: [{ denom: "usltn", amount: "1000000" }],
           },
       }],
       fee: { amount: [], gas: "200000" },
   };
   
   const result = await window.keplr.signAndBroadcast("sultan-1", accounts[0].address, [tx.msgs[0]], tx.fee);
   ```

**Deliverables:**
- [ ] Keplr connects to Sultan chain
- [ ] Balance displays in Keplr
- [ ] Can send SLTN via Keplr
- [ ] Website integration working

---

### Phase 4: IBC Protocol (Week 3-4)

**Goal:** Cross-chain transfers with Cosmos ecosystem

**Steps:**
1. Add IBC module
   ```go
   // app/app.go
   import ibctransfer "github.com/cosmos/ibc-go/v7/modules/apps/transfer"
   
   transferKeeper := ibctransferkeeper.NewKeeper(...)
   ibcModule := transfer.NewAppModule(transferKeeper)
   ```

2. Implement IBC channels
   ```bash
   # Create channel between Sultan and Cosmos Hub
   hermes create channel \
     --a-chain sultan-1 \
     --b-chain cosmoshub-4 \
     --a-port transfer \
     --b-port transfer
   ```

3. Test IBC transfer
   ```bash
   # Send 100 SLTN to Cosmos Hub
   sultand tx ibc-transfer transfer \
     transfer channel-0 \
     cosmos1abc... \
     100000000usltn \
     --from alice
   ```

**Deliverables:**
- [ ] IBC module compiled
- [ ] Channel to testnet established
- [ ] Can send SLTN to other chains
- [ ] Can receive tokens from other chains

---

## 🌐 DEPLOYMENT PLAN

### Option A: Make Codespaces Public (Quick Test)

1. **Forward Port 26657**
   - Go to VS Code "Ports" panel
   - Right-click port 26657 → "Port Visibility" → "Public"
   - Copy public URL (e.g., `https://scaling-fortnight-xxxxxx.github.dev`)

2. **Update Website**
   ```javascript
   // index.html
   const rpcEndpoint = 'https://scaling-fortnight-xxxxxx.github.dev';
   const restEndpoint = 'https://scaling-fortnight-xxxxxx.github.dev';
   ```

3. **Test from External Browser**
   ```bash
   # From your local machine (not Codespaces)
   curl https://scaling-fortnight-xxxxxx.github.dev/status
   ```

**Pros:** Fast, free, no server needed  
**Cons:** Temporary URL, Codespaces may sleep after inactivity

---

### Option B: Production Server (Recommended)

1. **Get a Server** (DigitalOcean, AWS, Hetzner, etc.)
   - Specs: 2 CPU, 4GB RAM, 100GB SSD (~$12/month)
   - OS: Ubuntu 22.04 LTS

2. **Deploy Sultan Node**
   ```bash
   # On server
   git clone https://github.com/Wollnbergen/0xv7.git
   cd 0xv7
   cargo build --release --bin sultan-node
   
   # Create systemd service
   sudo nano /etc/systemd/system/sultan.service
   ```
   
   **Service File:**
   ```ini
   [Unit]
   Description=Sultan Blockchain Node
   After=network.target
   
   [Service]
   Type=simple
   User=sultan
   WorkingDirectory=/home/sultan/0xv7/sultan-core
   ExecStart=/home/sultan/0xv7/target/release/sultan-node \
     --name "genesis-validator" \
     --validator \
     --validator-address "genesis" \
     --validator-stake 500000000000000 \
     --genesis "genesis:500000000000000" \
     --data-dir ./sultan-data \
     --rpc-addr "0.0.0.0:26657" \
     --block-time 5
   Restart=always
   
   [Install]
   WantedBy=multi-user.target
   ```
   
   ```bash
   sudo systemctl enable sultan
   sudo systemctl start sultan
   ```

3. **Configure Nginx Reverse Proxy**
   ```nginx
   # /etc/nginx/sites-available/sultan
   server {
       listen 80;
       server_name rpc.sultan.network;
       
       location / {
           proxy_pass http://localhost:26657;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

4. **Enable HTTPS with Let's Encrypt**
   ```bash
   sudo apt install certbot python3-certbot-nginx
   sudo certbot --nginx -d rpc.sultan.network
   ```

5. **Update DNS**
   - Add A record: `rpc.sultan.network` → Server IP
   - Add A record: `api.sultan.network` → Server IP (for REST)

**Deliverables:**
- [ ] Sultan node running as systemd service
- [ ] HTTPS enabled (Let's Encrypt)
- [ ] DNS configured
- [ ] Monitoring setup (optional: Prometheus + Grafana)

---

## 🐛 TROUBLESHOOTING

### Node Not Running
```bash
# Check if process exists
ps aux | grep sultan-node

# If not running, restart:
cd /workspaces/0xv7/sultan-core
/tmp/cargo-target/release/sultan-node \
  --name "genesis-validator" \
  --validator \
  --validator-address "genesis" \
  --validator-stake 500000000000000 \
  --genesis "genesis:500000000000000" \
  --data-dir ./sultan-data \
  --rpc-addr "0.0.0.0:26657" \
  --block-time 5 > sultan-node.log 2>&1 &
```

### Go Tests Failing
```bash
# Rebuild FFI library
cd /workspaces/0xv7
cargo build --release -p sultan-cosmos-bridge

# Verify .so exists
ls -lh /tmp/cargo-target/release/libsultan_cosmos_bridge.so

# Run tests with verbose output
cd sultan-cosmos-go
export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH
CGO_ENABLED=1 go test -v
```

### RPC Not Responding
```bash
# Check if port 26657 is listening
netstat -tlnp | grep 26657

# Check logs for errors
tail -100 /workspaces/0xv7/sultan-core/sultan-node.log

# Test RPC manually
curl http://localhost:26657/status
```

### Cargo Build Errors
```bash
# Clean and rebuild
cd /workspaces/0xv7
rm -rf /tmp/cargo-target
cargo clean
cargo build --release --bin sultan-node
cargo build --release -p sultan-cosmos-bridge
```

---

## 📚 REFERENCE DOCUMENTATION

### Key Files Created This Session
- `/tmp/cargo-target/release/sultan-node` (14MB) - L1 blockchain binary
- `/tmp/cargo-target/release/libsultan_cosmos_bridge.so` (6.4MB) - FFI library
- `/workspaces/0xv7/sultan-cosmos-go/bridge.go` - Go CGo wrapper
- `/workspaces/0xv7/sultan-cosmos-go/bridge_test.go` - Test suite (5/5 passing)
- `/workspaces/0xv7/LAYER2_COMPLETE.md` - Completion report
- `/workspaces/0xv7/PRODUCTION_READY_STATUS.md` - L1 status
- `/workspaces/0xv7/SESSION_RESTART_GUIDE.md` - Session restart instructions

### Quick Commands
```bash
# Check node status
ps aux | grep sultan-node | grep -v grep

# Query block height
curl -s http://localhost:26657/status | jq '.height'

# Query genesis balance
curl -s http://localhost:26657/balance/genesis | jq '.balance'

# View logs (last 50 lines)
tail -50 /workspaces/0xv7/sultan-core/sultan-node.log

# Run Go bridge tests
cd /workspaces/0xv7/sultan-cosmos-go && \
  export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH && \
  CGO_ENABLED=1 go test -v

# Benchmark FFI performance
cd /workspaces/0xv7/sultan-cosmos-go && \
  export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH && \
  CGO_ENABLED=1 go test -bench=. -benchtime=5s
```

---

## ✅ SUCCESS CRITERIA (Next Session)

**Layer 3 Complete When:**
- [ ] Cosmos SDK module (x/sultan) compiles and runs
- [ ] REST API responds to `/cosmos/bank/v1beta1/balances/{addr}`
- [ ] gRPC queries work via grpcurl
- [ ] Keplr wallet connects and displays balance
- [ ] Can send SLTN via Keplr interface
- [ ] IBC transfer to testnet works (optional)

**Production Deployment Complete When:**
- [ ] Sultan node running on public server
- [ ] HTTPS enabled for RPC/API endpoints
- [ ] DNS configured (rpc.sultan.network, api.sultan.network)
- [ ] Website deployed and accessible
- [ ] Keplr wallet works from external browser
- [ ] Monitoring/alerting setup (optional)

---

## 🎯 ESTIMATED TIMELINE

| Phase | Duration | Completion |
|-------|----------|------------|
| Layer 1 (Sultan Core) | 1 day | ✅ DONE (Block 2876+) |
| Layer 2 (Go Bridge) | 1 day | ✅ DONE (5/5 tests passing) |
| Layer 3 Phase 1 (SDK Module) | 2-3 days | ⏳ Next session |
| Layer 3 Phase 2 (REST API) | 1-2 days | ⏳ After Phase 1 |
| Layer 3 Phase 3 (Keplr) | 1 day | ⏳ After Phase 2 |
| Layer 3 Phase 4 (IBC) | 2-3 days | ⏳ Optional |
| Deployment | 1 day | ⏳ After Layer 3 |

**Total Remaining:** ~7-10 days for full production deployment

---

## 🚀 READY TO START?

When you return for the next session:

1. **Verify Current State** (see "PRE-SESSION VERIFICATION" above)
2. **Choose Next Goal:**
   - Option A: Build Cosmos SDK Module (recommended)
   - Option B: Deploy to production server (if want public access first)
3. **Follow Implementation Plan** (see Phase 1 above)
4. **Test Incrementally** (don't wait until end to test)

**Remember the user's directive:** "No stubs or TODOs, just the real deal!" 🔥

---

**Built with ❤️ - November 23, 2025**  
**Next update: TBD**
