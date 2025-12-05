#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        🚀 SULTAN BLOCKCHAIN - MAINNET PREPARATION             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Current Status: TESTNET ✅ → MAINNET 🚀"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Generate Mainnet Genesis
echo ""
echo "1️⃣ Creating Mainnet Genesis Configuration..."
mkdir -p /workspaces/0xv7/mainnet/config

cat > /workspaces/0xv7/mainnet/config/genesis.json << 'GENESIS'
{
  "genesis_time": "2025-01-01T00:00:00.000000Z",
  "chain_id": "sultan-mainnet-1",
  "initial_height": "1",
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
      "supply": [{
        "denom": "usltn",
        "amount": "500000000000000"
      }]
    },
    "staking": {
      "params": {
        "unbonding_time": "1814400s",
        "max_validators": 100,
        "max_entries": 7,
        "bond_denom": "usltn",
        "min_commission_rate": "0.000000000000000000"
      }
    },
    "mint": {
      "params": {
        "mint_denom": "usltn",
        "inflation_rate_change": "0.130000000000000000",
        "inflation_max": "0.080000000000000000",
        "inflation_min": "0.020000000000000000",
        "goal_bonded": "0.670000000000000000"
      }
    },
    "distribution": {
      "params": {
        "community_tax": "0.000000000000000000",
        "base_proposer_reward": "0.267000000000000000",
        "bonus_proposer_reward": "0.040000000000000000"
      }
    },
    "gov": {
      "params": {
        "min_deposit": [{
          "denom": "usltn",
          "amount": "10000000"
        }],
        "voting_period": "172800s"
      }
    },
    "slashing": {
      "params": {
        "signed_blocks_window": "100",
        "min_signed_per_window": "0.500000000000000000",
        "slash_fraction_double_sign": "0.050000000000000000",
        "slash_fraction_downtime": "0.010000000000000000"
      }
    },
    "sultan": {
      "params": {
        "zero_gas_fees": true,
        "staking_apy": "0.266700000000000000",
        "mobile_apy": "0.373300000000000000",
        "min_stake": "5000000000",
        "max_tps": "1230000",
        "hyper_tps": "10000000",
        "quantum_crypto": "dilithium3"
      }
    }
  }
}
GENESIS

echo "   ✅ Genesis created with Sultan Chain parameters"

# 2. Create Validator Setup Scripts
echo ""
echo "2️⃣ Creating Validator Setup Scripts..."

cat > /workspaces/0xv7/mainnet/setup_validator.sh << 'VALIDATOR'
#!/bin/bash

VALIDATOR_NAME=$1
if [ -z "$VALIDATOR_NAME" ]; then
    echo "Usage: ./setup_validator.sh <validator_name>"
    exit 1
fi

echo "Setting up validator: $VALIDATOR_NAME"

# Generate validator keys
sultand keys add $VALIDATOR_NAME --keyring-backend test

# Create validator
sultand tx staking create-validator \
  --amount=5000000000usltn \
  --pubkey=$(sultand tendermint show-validator) \
  --moniker="$VALIDATOR_NAME" \
  --chain-id=sultan-mainnet-1 \
  --commission-rate="0.10" \
  --commission-max-rate="0.20" \
  --commission-max-change-rate="0.01" \
  --min-self-delegation="5000000000" \
  --gas-prices="0usltn" \
  --from=$VALIDATOR_NAME \
  --keyring-backend test \
  -y

echo "✅ Validator $VALIDATOR_NAME created"
VALIDATOR

chmod +x /workspaces/0xv7/mainnet/setup_validator.sh

# 3. Production Configuration
echo ""
echo "3️⃣ Creating Production Configuration..."

cat > /workspaces/0xv7/mainnet/config/app.toml << 'CONFIG'
# Sultan Chain Mainnet Configuration
minimum-gas-prices = "0usltn"
pruning = "custom"
pruning-keep-recent = "100"
pruning-interval = "10"

[api]
enable = true
address = "tcp://0.0.0.0:1317"

[grpc]
enable = true
address = "0.0.0.0:9090"

[telemetry]
enabled = true
prometheus-retention-time = 600

[sultan]
zero-gas-fees = true
target-tps = 1230000
hyper-mode = true
CONFIG

# 4. Launch Script
echo ""
echo "4️⃣ Creating Mainnet Launch Script..."

cat > /workspaces/0xv7/mainnet/launch_mainnet.sh << 'LAUNCH'
#!/bin/bash

echo "🚀 Launching Sultan Chain Mainnet..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Initialize chain
sultand init sultan-mainnet --chain-id sultan-mainnet-1

# Copy genesis
cp /workspaces/0xv7/mainnet/config/genesis.json ~/.sultan/config/genesis.json

# Start node
sultand start \
    --minimum-gas-prices="0usltn" \
    --api.enable=true \
    --api.address="tcp://0.0.0.0:1317" \
    --grpc.enable=true \
    --grpc.address="0.0.0.0:9090" \
    --p2p.persistent_peers=""

echo "✅ Mainnet launched!"
LAUNCH

chmod +x /workspaces/0xv7/mainnet/launch_mainnet.sh

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║               ✅ MAINNET PREPARATION COMPLETE                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📁 Mainnet files created in: /workspaces/0xv7/mainnet/"
echo ""
echo "🎯 Next Steps:"
echo "1. Setup validators: ./mainnet/setup_validator.sh <name>"
echo "2. Launch mainnet: ./mainnet/launch_mainnet.sh"
echo "3. Monitor network: Check http://localhost:1317/status"
echo ""
echo "📅 Target Launch: January 1, 2025"
