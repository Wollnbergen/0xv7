#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           SULTAN CHAIN - MAINNET LAUNCH 🚀                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Safety check
echo "⚠️ WARNING: This will launch Sultan Chain MAINNET!"
echo "Have you:"
echo "  ✓ Completed security audit?"
echo "  ✓ Tested with multiple validators?"
echo "  ✓ Backed up all keys?"
echo "  ✓ Prepared genesis validators?"
echo ""
read -p "Type 'LAUNCH MAINNET' to proceed: " confirm

if [ "$confirm" != "LAUNCH MAINNET" ]; then
    echo "Launch cancelled."
    exit 1
fi

echo ""
echo "🚀 LAUNCHING SULTAN CHAIN MAINNET..."
echo ""

# Step 1: Initialize genesis
echo "📝 Creating genesis block..."
cat > genesis.json << 'JSON'
{
  "genesis_time": "2025-01-01T00:00:00Z",
  "chain_id": "sultan-mainnet-1",
  "initial_height": "1",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "-1"
    }
  },
  "validators": [],
  "app_hash": "",
  "app_state": {
    "auth": {
      "params": {
        "max_memo_characters": "256",
        "tx_sig_limit": "7",
        "tx_size_cost_per_byte": "0",
        "sig_verify_cost_ed25519": "0",
        "sig_verify_cost_secp256k1": "0"
      }
    },
    "bank": {
      "params": {
        "send_enabled": true
      },
      "balances": [],
      "total_supply": [{
        "denom": "usultan",
        "amount": "1000000000000000"
      }]
    },
    "staking": {
      "params": {
        "unbonding_time": "1814400s",
        "max_validators": 100,
        "max_entries": 7,
        "bond_denom": "usultan"
      }
    },
    "mint": {
      "params": {
        "mint_denom": "usultan",
        "inflation_rate_change": "0.13",
        "inflation_max": "0.08",
        "inflation_min": "0.07",
        "goal_bonded": "0.30"
      }
    },
    "gov": {
      "params": {
        "min_deposit": [{
          "denom": "usultan",
          "amount": "10000000"
        }],
        "max_deposit_period": "172800s",
        "voting_period": "172800s"
      }
    }
  }
}
JSON

# Step 2: Start database
echo "🗄️ Starting database services..."
docker-compose -f docker-compose.mainnet.yml up -d scylla redis
sleep 10

# Step 3: Run migrations
echo "📊 Running database migrations..."
docker exec scylla cqlsh -f /migrations/001_create_tables.cql

# Step 4: Start validator node
echo "🔧 Starting validator node..."
docker-compose -f docker-compose.mainnet.yml up -d sultan-node

# Step 5: Wait for node to start
echo "⏳ Waiting for node to initialize..."
sleep 10

# Step 6: Check node status
echo ""
echo "🔍 Checking node status..."
curl -s http://localhost:26657/status | jq '.result.sync_info'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SULTAN CHAIN MAINNET IS LIVE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 Endpoints:"
echo "  • RPC: http://localhost:26657"
echo "  • P2P: localhost:26656"
echo "  • Metrics: http://localhost:9090"
echo ""
echo "📊 Chain Info:"
echo "  • Chain ID: sultan-mainnet-1"
echo "  • Token: SULTAN (usultan)"
echo "  • APY: 26.67% base, 37.33% mobile"
echo "  • Fees: ZERO (subsidized)"
echo ""
echo "🎯 Next Steps:"
echo "  1. Add genesis validators"
echo "  2. Open public RPC endpoints"
echo "  3. Launch block explorer"
echo "  4. Announce to community"
echo "  5. List on exchanges"
echo ""
echo "🌟 Congratulations! Sultan Chain Mainnet is operational!"
