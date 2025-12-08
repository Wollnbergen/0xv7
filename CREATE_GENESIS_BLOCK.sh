#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         CREATING SULTAN CHAIN GENESIS BLOCK                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

mkdir -p /workspaces/0xv7/sultan-mainnet/config

cat > /workspaces/0xv7/sultan-mainnet/config/genesis.json << 'JSON'
{
  "genesis_time": "2025-11-03T00:00:00Z",
  "chain_id": "sultan-mainnet-1",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "0",
      "time_iota_ms": "5000"
    }
  },
  "initial_validators": [
    {
      "name": "validator-1",
      "power": "100000",
      "stake": "5000000",
      "commission": "0.10"
    }
  ],
  "economics": {
    "total_supply": "1000000000",
    "initial_inflation": "0.08",
    "burn_rate": "0.01",
    "validator_apy_cap": "0.1333"
  },
  "app_state": {
    "accounts": [
      {
        "address": "sultan1foundation",
        "balance": "100000000"
      }
    ]
  }
}
JSON

echo "✅ Genesis block created at sultan-mainnet/config/genesis.json"

