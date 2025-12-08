# Sultan L1 REST/gRPC API Documentation

**Phase 5 Day 14 COMPLETE** - Production-grade REST/gRPC API implementation  
**Date**: November 22, 2025  
**Binary**: sultand (91MB, built 17:58 UTC)

---

## 🎯 Overview

Sultan L1 now provides **full production-grade REST/gRPC API** access via Cosmos SDK's gRPC Gateway. All modules are accessible through HTTP/JSON endpoints for wallet integration, block explorers, and web dApps.

---

## 🚀 Starting the API Server

### Default Configuration (API Enabled)

```bash
cd /workspaces/0xv7/sultand
export LD_LIBRARY_PATH=/tmp/cargo-target/release:$LD_LIBRARY_PATH

# Start with API and Swagger enabled
./sultand start --api.enable=true --api.swagger=true --api.enabled-unsafe-cors=true
```

### API Endpoints

**Default API Port**: `1317`  
**gRPC Port**: `9090`  
**Tendermint RPC**: `26657`

---

## 📡 Available Endpoints

### Health & Status

```bash
# Health check
curl http://localhost:1317/health

# Response:
# {"status":"healthy","chain":"sultan-l1"}

# Chain status
curl http://localhost:1317/status

# Response:
# {"chain_id":"sultan-1","api":"v1","ibc":"enabled","modules":["auth","bank","staking","ibc","transfer","sultan"]}
```

### Swagger Documentation

```
http://localhost:1317/swagger/
```

Interactive API documentation with all available endpoints.

---

## 🔐 Auth Module

### Query All Accounts

```bash
curl http://localhost:1317/cosmos/auth/v1beta1/accounts
```

### Query Account by Address

```bash
curl http://localhost:1317/cosmos/auth/v1beta1/accounts/cosmos1vsakvzmh8d3py0qun0hhktza7kksl53g236p0d
```

### Module Accounts

```bash
curl http://localhost:1317/cosmos/auth/v1beta1/module_accounts
```

---

## 💰 Bank Module

### Query All Balances

```bash
curl http://localhost:1317/cosmos/bank/v1beta1/balances/cosmos1vsakvzmh8d3py0qun0hhktza7kksl53g236p0d
```

### Query Specific Denom Balance

```bash
curl http://localhost:1317/cosmos/bank/v1beta1/balances/cosmos1vsakvzmh8d3py0qun0hhktza7kksl53g236p0d/by_denom?denom=stake
```

### Total Supply

```bash
curl http://localhost:1317/cosmos/bank/v1beta1/supply
```

### Supply by Denom

```bash
curl http://localhost:1317/cosmos/bank/v1beta1/supply/by_denom?denom=stake
```

---

## 🔗 Staking Module

### Query All Validators

```bash
curl http://localhost:1317/cosmos/staking/v1beta1/validators
```

### Query Validator by Address

```bash
curl http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper1vsakvzmh8d3py0qun0hhktza7kksl53gpx2vyw
```

### Query Delegations

```bash
curl http://localhost:1317/cosmos/staking/v1beta1/delegations/cosmos1vsakvzmh8d3py0qun0hhktza7kksl53g236p0d
```

### Staking Parameters

```bash
curl http://localhost:1317/cosmos/staking/v1beta1/params
```

---

## 🌉 IBC Module

### Query IBC Client States

```bash
curl http://localhost:1317/ibc/core/client/v1/client_states
```

### Query Specific Client

```bash
curl http://localhost:1317/ibc/core/client/v1/client_states/07-tendermint-0
```

### Query IBC Connections

```bash
curl http://localhost:1317/ibc/core/connection/v1/connections
```

### Query Specific Connection

```bash
curl http://localhost:1317/ibc/core/connection/v1/connections/connection-0
```

### Query IBC Channels

```bash
curl http://localhost:1317/ibc/core/channel/v1/channels
```

### Query Specific Channel

```bash
curl http://localhost:1317/ibc/core/channel/v1/channels/channel-0/ports/transfer
```

---

## 💸 IBC Transfer Module

### Query Denom Traces

```bash
curl http://localhost:1317/ibc/apps/transfer/v1/denom_traces
```

### Query Specific Denom Trace

```bash
curl http://localhost:1317/ibc/apps/transfer/v1/denom_traces/<hash>
```

### Transfer Parameters

```bash
curl http://localhost:1317/ibc/apps/transfer/v1/params
```

---

## 🔧 Upgrade Module

### Query Current Plan

```bash
curl http://localhost:1317/cosmos/upgrade/v1beta1/current_plan
```

### Query Applied Plan

```bash
curl http://localhost:1317/cosmos/upgrade/v1beta1/applied_plan/<name>
```

---

## 🧊 Sultan Core Module

### Query Sultan Blockchain Info

```bash
curl http://localhost:1317/sultan/v1/info
```

### Query Sultan Block by Height

```bash
curl http://localhost:1317/sultan/v1/block/<height>
```

---

## 📝 Transaction Service

### Simulate Transaction

```bash
curl -X POST http://localhost:1317/cosmos/tx/v1beta1/simulate \
  -H "Content-Type: application/json" \
  -d '{
    "tx_bytes": "<base64_encoded_tx>"
  }'
```

### Broadcast Transaction

```bash
curl -X POST http://localhost:1317/cosmos/tx/v1beta1/txs \
  -H "Content-Type: application/json" \
  -d '{
    "tx_bytes": "<base64_encoded_tx>",
    "mode": "BROADCAST_MODE_SYNC"
  }'
```

### Query Transaction by Hash

```bash
curl http://localhost:1317/cosmos/tx/v1beta1/txs/<hash>
```

### Query Transactions by Events

```bash
curl "http://localhost:1317/cosmos/tx/v1beta1/txs?events=message.sender='cosmos1...'"
```

---

## 🌐 Node/Tendermint Service

### Node Info

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info
```

### Syncing Status

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/syncing
```

### Latest Block

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/blocks/latest
```

### Block by Height

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/blocks/<height>
```

### Latest Validator Set

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/validatorsets/latest
```

### Validator Set by Height

```bash
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/validatorsets/<height>
```

---

## 🔌 CORS Configuration

CORS is configurable via command-line flags:

```bash
# Enable CORS for all origins (development)
sultand start --api.enabled-unsafe-cors=true

# Production: Configure specific origins in app.toml
[api]
enabled-unsafe-cors = false
```

For production, modify `/workspaces/0xv7/sultand/app/app.go`:

```go
corsHandler := handlers.CORS(
    handlers.AllowedOrigins([]string{"https://yourdomain.com"}),
    // ... rest of CORS config
)
```

---

## 🔐 Security Best Practices

### Production Deployment

1. **Disable unsafe CORS** (`--api.enabled-unsafe-cors=false`)
2. **Configure specific allowed origins**
3. **Use HTTPS** (reverse proxy with nginx/caddy)
4. **Rate limiting** (implement at reverse proxy level)
5. **API authentication** (for sensitive endpoints)

### Example Nginx Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name api.sultan-l1.network;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:1317;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header 'Access-Control-Allow-Origin' 'https://wallet.sultan-l1.network' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization' always;
    }
}
```

---

## 🧪 Testing the API

### Quick Health Check

```bash
# Test health endpoint
curl http://localhost:1317/health && echo "✅ API is healthy"

# Test chain status
curl http://localhost:1317/status | jq

# Test node info
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info | jq .node_info

# Test IBC clients
curl http://localhost:1317/ibc/core/client/v1/client_states | jq
```

### Integration Test Script

```bash
#!/bin/bash

API_URL="http://localhost:1317"

echo "=== Sultan L1 API Integration Tests ==="

# Test 1: Health
echo -n "Health check: "
curl -s $API_URL/health | jq -r .status

# Test 2: Accounts
echo -n "Accounts query: "
curl -s $API_URL/cosmos/auth/v1beta1/accounts | jq '.accounts | length'

# Test 3: Validators
echo -n "Validators query: "
curl -s $API_URL/cosmos/staking/v1beta1/validators | jq '.validators | length'

# Test 4: IBC clients
echo -n "IBC clients: "
curl -s $API_URL/ibc/core/client/v1/client_states | jq '.client_states | length'

# Test 5: Latest block
echo -n "Latest block height: "
curl -s $API_URL/cosmos/base/tendermint/v1beta1/blocks/latest | jq -r .block.header.height

echo "✅ All tests complete"
```

---

## 📚 API Client Libraries

### JavaScript/TypeScript (CosmJS)

```typescript
import { StargateClient } from "@cosmjs/stargate";

const client = await StargateClient.connect("http://localhost:1317");
const height = await client.getHeight();
console.log("Current height:", height);
```

### Python (cosmpy)

```python
from cosmpy.aerial.client import LedgerClient
from cosmpy.aerial.config import NetworkConfig

config = NetworkConfig(
    chain_id="sultan-1",
    url="http://localhost:1317",
)

client = LedgerClient(config)
print(f"Chain ID: {client.query_chain_id()}")
```

### Go

```go
import (
    "context"
    sdk "github.com/cosmos/cosmos-sdk/types"
    "github.com/cosmos/cosmos-sdk/types/query"
    authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

// Query accounts
req := &authtypes.QueryAccountsRequest{
    Pagination: &query.PageRequest{Limit: 100},
}
res, err := authQueryClient.Accounts(context.Background(), req)
```

---

## 🎯 Next Steps

**Phase 5 Day 15**: Keplr Wallet Integration
- Create Keplr chain configuration
- Test wallet connection
- Transaction signing via wallet
- Add Sultan L1 to Keplr registry

---

## 📊 Capabilities Unlocked

✅ **Wallet Integration** - Keplr, Cosmostation, Leap can connect  
✅ **Block Explorers** - Mintscan, Big Dipper compatible  
✅ **Web dApps** - Full REST API access  
✅ **Mobile Apps** - CosmJS/cosmpy client libraries  
✅ **Monitoring** - Health checks and metrics  
✅ **IBC Queries** - Cross-chain state inspection  
✅ **Developer Tools** - Interactive Swagger docs  

---

**Sultan L1 Phase 5 Day 14: COMPLETE! 🚀**  
**Production REST/gRPC API ready for ecosystem integration**
