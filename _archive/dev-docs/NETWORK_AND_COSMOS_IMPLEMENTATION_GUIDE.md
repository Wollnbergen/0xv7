# Sultan L1 - Network & Cosmos SDK Implementation Guide

## 🟢 COMPLETED: RPC Public Access

### ✅ What Was Fixed (Dec 6, 2025)

**Problem:** RPC server was bound to `127.0.0.1:8080` (localhost only)

**Solution:** Updated to `0.0.0.0:8080` (all interfaces)

**Verification:**
```bash
curl http://5.161.225.96:8080/status
# Returns: {"height":2,"shard_count":100,"sharding_enabled":true,...}
```

**Website Access:**
- Temporary: `http://5.161.225.96:8080` (direct IP)
- Planned: `https://rpc.sltn.io` (requires domain + SSL setup)

---

## 🔴 TO-DO: Domain & SSL Configuration

### Step 1: Configure DNS

**Required:**
1. Purchase/configure `sltn.io` domain
2. Add A record: `rpc.sltn.io → 5.161.225.96`
3. Wait for DNS propagation (5-60 minutes)

**Verification:**
```bash
dig rpc.sltn.io +short
# Should return: 5.161.225.96
```

### Step 2: Install & Configure Nginx Reverse Proxy

```bash
# On production server (5.161.225.96)
apt update && apt install -y nginx certbot python3-certbot-nginx

# Create nginx config
cat > /etc/nginx/sites-available/sultan-rpc << 'EOF'
server {
    listen 80;
    server_name rpc.sltn.io;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        
        # CORS headers for website access
        add_header 'Access-Control-Allow-Origin' '*';
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
        add_header 'Access-Control-Allow-Headers' 'Content-Type';
    }
}
EOF

# Enable site
ln -s /etc/nginx/sites-available/sultan-rpc /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx

# Get SSL certificate (after DNS is configured)
certbot --nginx -d rpc.sltn.io --non-interactive --agree-tos -m admin@sltn.io
```

### Step 3: Update Website to Use Domain

Once SSL is configured, update `index.html`:
```javascript
// Change from:
let rpcEndpoint = 'https://rpc.sltn.io';

// No changes needed! Already configured correctly
```

**Timeline:** 1-2 hours (DNS + SSL setup)

---

## 🔴 TO-DO: Cosmos SDK Integration

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Sultan Ecosystem                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  Sultan Core     │◄───────►│ Sultan Cosmos    │          │
│  │  (Rust)          │   FFI   │ Bridge (Rust)    │          │
│  │                  │         │                  │          │
│  │ - Blockchain     │         │ - ABCI Adapter   │          │
│  │ - Sharding       │         │ - FFI Functions  │          │
│  │ - Consensus      │         │ - State Bridge   │          │
│  └──────────────────┘         └─────────┬────────┘          │
│                                          │                    │
│                                          │ C FFI              │
│                                          │                    │
│                                 ┌────────▼────────┐           │
│                                 │  CGo Wrapper    │           │
│                                 │  (Go)           │           │
│                                 └────────┬────────┘           │
│                                          │                    │
│                                          │                    │
│                                 ┌────────▼────────┐           │
│                                 │  Cosmos SDK     │           │
│                                 │  (Go)           │           │
│                                 │                 │           │
│                                 │ - CometBFT      │           │
│                                 │ - IBC Protocol  │           │
│                                 │ - Keplr Wallet  │           │
│                                 └─────────────────┘           │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Current Status

**✅ Completed (sultan-cosmos-bridge):**
- FFI layer (Rust ↔ C interface)
- ABCI protocol adapter
- Error handling & memory safety
- Thread-safe state management

**🔄 In Progress:**
- None (waiting for implementation)

**❌ Not Started:**
- Go wrapper code (CGo integration)
- Cosmos SDK module
- CometBFT consensus integration
- IBC relayer setup
- Keplr wallet configuration

---

## 📋 Implementation Steps

### Phase 1: Go Wrapper Development (1-2 weeks)

**Files to Create:**

#### 1. `sultan-cosmos-sdk/go.mod`
```go
module github.com/Wollnbergen/sultan-cosmos-sdk

go 1.21

require (
    github.com/cosmos/cosmos-sdk v0.50.1
    github.com/cometbft/cometbft v0.38.0
    github.com/cosmos/ibc-go/v8 v8.0.0
)
```

#### 2. `sultan-cosmos-sdk/bridge/bridge.go` (CGo Wrapper)
```go
package bridge

/*
#cgo LDFLAGS: -L../sultan-cosmos-bridge/target/release -lsultan_cosmos_bridge
#include "../sultan-cosmos-bridge/include/bridge.h"
*/
import "C"
import (
    "unsafe"
    "encoding/json"
)

type SultanBridge struct {
    handle C.uintptr_t
}

func NewSultanBridge() (*SultanBridge, error) {
    C.sultan_bridge_init()
    
    handle := C.sultan_blockchain_new()
    if handle == 0 {
        return nil, errors.New("failed to create blockchain")
    }
    
    return &SultanBridge{handle: handle}, nil
}

func (b *SultanBridge) ProcessABCI(request []byte) ([]byte, error) {
    var err C.BridgeError
    
    reqArray := C.CByteArray{
        data: (*C.uint8_t)(unsafe.Pointer(&request[0])),
        len:  C.size_t(len(request)),
    }
    
    result := C.sultan_abci_process(b.handle, reqArray, &err)
    if err.code != 0 {
        return nil, errors.New(C.GoString(err.message))
    }
    
    response := C.GoBytes(unsafe.Pointer(result.data), C.int(result.len))
    C.free(unsafe.Pointer(result.data))
    
    return response, nil
}

func (b *SultanBridge) Close() {
    C.sultan_blockchain_destroy(b.handle)
    C.sultan_bridge_shutdown()
}
```

#### 3. `sultan-cosmos-sdk/app/app.go` (Cosmos SDK App)
```go
package app

import (
    "github.com/cosmos/cosmos-sdk/baseapp"
    "github.com/cosmos/cosmos-sdk/types/module"
    "github.com/cometbft/cometbft/abci/types"
    "github.com/Wollnbergen/sultan-cosmos-sdk/bridge"
)

type SultanApp struct {
    *baseapp.BaseApp
    bridge *bridge.SultanBridge
}

func NewSultanApp() *SultanApp {
    bridge, err := bridge.NewSultanBridge()
    if err != nil {
        panic(err)
    }
    
    app := &SultanApp{
        BaseApp: baseapp.NewBaseApp("sultan", ...),
        bridge:  bridge,
    }
    
    return app
}

func (app *SultanApp) BeginBlock(req types.RequestBeginBlock) types.ResponseBeginBlock {
    request := map[string]interface{}{
        "type":     "BeginBlock",
        "height":   req.Header.Height,
        "proposer": req.Header.ProposerAddress.String(),
    }
    
    reqBytes, _ := json.Marshal(request)
    respBytes, err := app.bridge.ProcessABCI(reqBytes)
    if err != nil {
        panic(err)
    }
    
    return types.ResponseBeginBlock{}
}

func (app *SultanApp) DeliverTx(req types.RequestDeliverTx) types.ResponseDeliverTx {
    request := map[string]interface{}{
        "type":    "DeliverTx",
        "tx_data": req.Tx,
    }
    
    reqBytes, _ := json.Marshal(request)
    respBytes, err := app.bridge.ProcessABCI(reqBytes)
    if err != nil {
        return types.ResponseDeliverTx{Code: 1, Log: err.Error()}
    }
    
    var response struct {
        Code uint32 `json:"code"`
        Log  string `json:"log"`
    }
    json.Unmarshal(respBytes, &response)
    
    return types.ResponseDeliverTx{Code: response.Code, Log: response.Log}
}
```

#### 4. `sultan-cosmos-sdk/cmd/sultand/main.go` (Node Binary)
```go
package main

import (
    "github.com/cosmos/cosmos-sdk/server"
    "github.com/Wollnbergen/sultan-cosmos-sdk/app"
    "github.com/cometbft/cometbft/node"
)

func main() {
    app := app.NewSultanApp()
    
    cfg := server.DefaultConfig()
    cfg.BaseConfig.ChainID = "sultan-1"
    
    node, err := node.NewNode(cfg, ...)
    if err != nil {
        panic(err)
    }
    
    node.Start()
    defer node.Stop()
    
    // Keep running
    select {}
}
```

### Phase 2: Build & Test (3-5 days)

```bash
# Build Rust bridge library
cd sultan-cosmos-bridge
cargo build --release

# Build Go wrapper
cd ../sultan-cosmos-sdk
go mod download
go build -o sultand cmd/sultand/main.go

# Test FFI integration
go test ./bridge -v

# Run integrated node
./sultand start
```

### Phase 3: IBC Relayer Setup (1 week)

```bash
# Install Hermes relayer
wget https://github.com/informalsystems/hermes/releases/download/v1.7.0/hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf hermes-v1.7.0-x86_64-unknown-linux-gnu.tar.gz
mv hermes /usr/local/bin/

# Configure relayer
cat > ~/.hermes/config.toml << EOF
[global]
log_level = 'info'

[[chains]]
id = 'sultan-1'
rpc_addr = 'http://127.0.0.1:26657'
grpc_addr = 'http://127.0.0.1:9090'
websocket_addr = 'ws://127.0.0.1:26657/websocket'
rpc_timeout = '10s'
account_prefix = 'sultan'
key_name = 'relayer'
store_prefix = 'ibc'
gas_price = { price = 0.001, denom = 'sltn' }

[[chains]]
id = 'cosmoshub-4'
rpc_addr = 'https://rpc.cosmos.network'
grpc_addr = 'https://grpc.cosmos.network'
...
EOF

# Start relayer
hermes start
```

### Phase 4: Keplr Wallet Integration (2-3 days)

**Create chain configuration:**

```javascript
// keplr-chain-config.js
const sultanChainConfig = {
    chainId: "sultan-1",
    chainName: "Sultan L1",
    rpc: "https://rpc.sltn.io",
    rest: "https://api.sltn.io",
    bip44: {
        coinType: 118, // Cosmos standard
    },
    bech32Config: {
        bech32PrefixAccAddr: "sultan",
        bech32PrefixAccPub: "sultanpub",
        bech32PrefixValAddr: "sultanvaloper",
        bech32PrefixValPub: "sultanvaloperpub",
        bech32PrefixConsAddr: "sultanvalcons",
        bech32PrefixConsPub: "sultanvalconspub",
    },
    currencies: [{
        coinDenom: "SLTN",
        coinMinimalDenom: "usltn",
        coinDecimals: 6,
        coinGeckoId: "sultan",
    }],
    feeCurrencies: [{
        coinDenom: "SLTN",
        coinMinimalDenom: "usltn",
        coinDecimals: 6,
        coinGeckoId: "sultan",
        gasPriceStep: {
            low: 0.001,
            average: 0.0025,
            high: 0.004,
        },
    }],
    stakeCurrency: {
        coinDenom: "SLTN",
        coinMinimalDenom: "usltn",
        coinDecimals: 6,
        coinGeckoId: "sultan",
    },
    features: ["ibc-transfer", "ibc-go"],
};

// Add to website
await window.keplr.experimentalSuggestChain(sultanChainConfig);
const offlineSigner = window.getOfflineSigner("sultan-1");
const accounts = await offlineSigner.getAccounts();
```

---

## 📊 Timeline Summary

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| ✅ | RPC Public Access | 1 hour | **COMPLETE** |
| 🔄 | DNS + SSL Setup | 1-2 hours | **PENDING** |
| ❌ | Go Wrapper Development | 1-2 weeks | **NOT STARTED** |
| ❌ | Build & Test Integration | 3-5 days | **NOT STARTED** |
| ❌ | IBC Relayer Setup | 1 week | **NOT STARTED** |
| ❌ | Keplr Integration | 2-3 days | **NOT STARTED** |

**Total Estimated Time:** 3-4 weeks full-time development

---

## 🚀 Quick Wins (This Week)

### Priority 1: Fix Website Stats (1 hour)

**Update website to use direct IP temporarily:**

```javascript
// In index.html, change:
let rpcEndpoint = window.location.hostname === 'localhost' 
    ? 'http://localhost:8080'
    : 'http://5.161.225.96:8080';  // Use direct IP until domain is configured
```

### Priority 2: Domain Configuration (2 hours)

1. Configure DNS: `rpc.sltn.io → 5.161.225.96`
2. Install nginx reverse proxy
3. Get Let's Encrypt SSL certificate
4. Enable CORS headers

### Priority 3: Start Cosmos SDK Development (Ongoing)

1. Set up Go development environment
2. Create `sultan-cosmos-sdk` repository
3. Implement CGo wrapper for FFI bridge
4. Test ABCI integration

---

## 📚 Resources

**Cosmos SDK Documentation:**
- https://docs.cosmos.network/
- https://tutorials.cosmos.network/

**IBC Protocol:**
- https://ibc.cosmos.network/
- https://github.com/cosmos/ibc-go

**Keplr Wallet:**
- https://docs.keplr.app/
- https://github.com/chainapsis/keplr-wallet

**CometBFT (Tendermint):**
- https://docs.cometbft.com/
- https://github.com/cometbft/cometbft

---

## ✅ Next Steps

1. **Immediate (Today):**
   - ✅ RPC public access enabled
   - Update website to use `http://5.161.225.96:8080`
   - Test network stats display

2. **Short-term (This Week):**
   - Configure `rpc.sltn.io` domain + SSL
   - Set up nginx reverse proxy with CORS
   - Deploy updated website

3. **Medium-term (Next 2 Weeks):**
   - Begin Go wrapper development
   - Implement CGo bridge to Rust FFI
   - Create Cosmos SDK app structure

4. **Long-term (Next Month):**
   - Complete IBC integration
   - Deploy Keplr wallet support
   - Launch testnet with Cosmos interoperability

---

**Last Updated:** December 6, 2025  
**Status:** RPC public access ✅ | Domain pending | Cosmos SDK in planning
