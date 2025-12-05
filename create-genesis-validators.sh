#!/bin/bash
# Sultan L1 - Genesis Validator Creator
# Create multiple genesis validators for network launch

set -e

echo "════════════════════════════════════════════════════════════"
echo "       👥 Sultan L1 Genesis Validator Setup 👥             "
echo "════════════════════════════════════════════════════════════"
echo ""

# Configuration
VALIDATOR_COUNT="${1:-5}"
BASE_STAKE="10000000000000"  # 10,000 SLTN per validator
TOTAL_SUPPLY="500000000000000000"  # 500M SLTN

echo "📋 Genesis Configuration:"
echo "   Validators: $VALIDATOR_COUNT"
echo "   Stake per validator: 10,000 SLTN"
echo "   Total staked: $((VALIDATOR_COUNT * 10000)) SLTN"
echo "   Network supply: 500M SLTN"
echo ""

# Create validators directory
VALIDATORS_DIR="/workspaces/0xv7/genesis-validators"
mkdir -p "$VALIDATORS_DIR"

# Generate validators
echo "🔑 Generating validator keys..."
echo ""

for i in $(seq 1 $VALIDATOR_COUNT); do
    VALIDATOR_NAME="genesis-validator-$i"
    
    # Generate unique address (deterministic for testing)
    SEED="sultan-genesis-$i-$(date +%s)"
    HASH=$(echo -n "$SEED" | sha256sum | cut -c1-40)
    VALIDATOR_ADDRESS="sultan1validator${HASH:0:34}"
    
    # Random commission between 5-10%
    COMMISSION=$(awk -v min=0.05 -v max=0.10 'BEGIN{srand(); print min+rand()*(max-min)}')
    
    # Create validator config
    cat > "$VALIDATORS_DIR/validator-$i.json" <<EOF
{
  "validator_id": $i,
  "name": "$VALIDATOR_NAME",
  "address": "$VALIDATOR_ADDRESS",
  "stake": "$BASE_STAKE",
  "commission": $COMMISSION,
  "moniker": "Sultan Genesis Validator #$i",
  "website": "https://sultanchain.io/validators/$i",
  "details": "Genesis validator for Sultan L1 mainnet",
  "created_at": $(date +%s)
}
EOF
    
    printf "✅ Validator #%d: %s (%.1f%% commission)\n" $i "$VALIDATOR_ADDRESS" $(echo "$COMMISSION * 100" | bc)
done

echo ""

# Create genesis file
echo "📜 Creating genesis configuration..."

GENESIS_FILE="$VALIDATORS_DIR/genesis.json"

cat > "$GENESIS_FILE" <<EOF
{
  "genesis_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "chain_id": "sultan-1",
  "initial_height": "1",
  "consensus_params": {
    "block": {
      "max_bytes": "22020096",
      "max_gas": "-1",
      "time_iota_ms": "2000"
    },
    "evidence": {
      "max_age_num_blocks": "100000",
      "max_age_duration": "172800000000000"
    },
    "validator": {
      "pub_key_types": ["ed25519"]
    }
  },
  "validators": [
EOF

# Add validators to genesis
for i in $(seq 1 $VALIDATOR_COUNT); do
    VALIDATOR_DATA=$(cat "$VALIDATORS_DIR/validator-$i.json")
    VALIDATOR_ADDRESS=$(echo "$VALIDATOR_DATA" | jq -r '.address')
    VALIDATOR_STAKE=$(echo "$VALIDATOR_DATA" | jq -r '.stake')
    
    if [ $i -gt 1 ]; then
        echo "    ," >> "$GENESIS_FILE"
    fi
    
    cat >> "$GENESIS_FILE" <<EOF
    {
      "address": "$VALIDATOR_ADDRESS",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "$(openssl rand -base64 32)"
      },
      "power": "$VALIDATOR_STAKE",
      "name": "genesis-validator-$i"
    }
EOF
done

cat >> "$GENESIS_FILE" <<EOF
  ],
  "app_state": {
    "staking": {
      "validators": $(cat $VALIDATORS_DIR/validator-*.json | jq -s '.')
    },
    "bank": {
      "total_supply": "$TOTAL_SUPPLY",
      "balances": [
EOF

# Distribute initial balances
BALANCE_PER_VALIDATOR="50000000000000000"  # 50M SLTN each

for i in $(seq 1 $VALIDATOR_COUNT); do
    VALIDATOR_ADDRESS=$(jq -r '.address' "$VALIDATORS_DIR/validator-$i.json")
    
    if [ $i -gt 1 ]; then
        echo "        ," >> "$GENESIS_FILE"
    fi
    
    cat >> "$GENESIS_FILE" <<EOF
        {
          "address": "$VALIDATOR_ADDRESS",
          "coins": [
            {
              "denom": "usltn",
              "amount": "$BALANCE_PER_VALIDATOR"
            }
          ]
        }
EOF
done

cat >> "$GENESIS_FILE" <<EOF
      ]
    },
    "governance": {
      "starting_proposal_id": "1",
      "deposit_params": {
        "min_deposit": [
          {
            "denom": "usltn",
            "amount": "1000000000000"
          }
        ],
        "max_deposit_period": "604800000000000"
      },
      "voting_params": {
        "voting_period": "100800"
      },
      "tally_params": {
        "quorum": "0.334",
        "threshold": "0.5",
        "veto_threshold": "0.334"
      }
    },
    "economics": {
      "inflation_rate": 0.08,
      "validator_apy": 0.2667,
      "block_time": 2,
      "blocks_per_year": 15768000
    },
    "sharding": {
      "enabled": true,
      "shard_count": 100,
      "tx_per_shard": 10000
    }
  }
}
EOF

echo "✅ Genesis file created"
echo ""

# Create validator startup scripts
echo "📝 Creating startup scripts..."

for i in $(seq 1 $VALIDATOR_COUNT); do
    VALIDATOR_DATA=$(cat "$VALIDATORS_DIR/validator-$i.json")
    VALIDATOR_ADDRESS=$(echo "$VALIDATOR_DATA" | jq -r '.address')
    VALIDATOR_STAKE=$(echo "$VALIDATOR_DATA" | jq -r '.stake')
    
    RPC_PORT=$((26657 + i - 1))
    P2P_PORT=$((26656 + i - 1))
    
    cat > "$VALIDATORS_DIR/start-validator-$i.sh" <<EOF
#!/bin/bash
# Start Sultan L1 Validator #$i

BINARY="/tmp/cargo-target/release/sultan-node"

\$BINARY \\
  --name "genesis-validator-$i" \\
  --validator \\
  --validator-address "$VALIDATOR_ADDRESS" \\
  --validator-stake "$VALIDATOR_STAKE" \\
  --enable-sharding \\
  --shard-count 100 \\
  --tx-per-shard 10000 \\
  --block-time 2 \\
  --data-dir "/workspaces/0xv7/data/validator-$i" \\
  --rpc-addr "0.0.0.0:$RPC_PORT" \\
  --p2p-addr "0.0.0.0:$P2P_PORT"
EOF
    
    chmod +x "$VALIDATORS_DIR/start-validator-$i.sh"
done

echo "✅ Startup scripts created"
echo ""

# Create test script
cat > "$VALIDATORS_DIR/test-consensus.sh" <<'EOF'
#!/bin/bash
# Test multi-validator consensus

echo "Testing consensus across validators..."

for i in {1..5}; do
    PORT=$((26657 + i - 1))
    echo -n "Validator #$i (port $PORT): "
    
    HEIGHT=$(curl -s "http://localhost:$PORT/status" | jq -r '.height // "ERROR"')
    echo "Block $HEIGHT"
    
    sleep 0.5
done

echo ""
echo "If all validators show the same (or very close) block height, consensus is working!"
EOF

chmod +x "$VALIDATORS_DIR/test-consensus.sh"

# Create start-all script
cat > "$VALIDATORS_DIR/start-all-validators.sh" <<EOF
#!/bin/bash
# Start all genesis validators

echo "Starting $VALIDATOR_COUNT validators..."

for i in \$(seq 1 $VALIDATOR_COUNT); do
    echo "Starting validator #\$i..."
    bash "$VALIDATORS_DIR/start-validator-\$i.sh" > "$VALIDATORS_DIR/validator-\$i.log" 2>&1 &
    PID=\$!
    echo "\$PID" > "$VALIDATORS_DIR/validator-\$i.pid"
    echo "  PID: \$PID"
    sleep 1
done

echo ""
echo "All validators started!"
echo "Test consensus: bash $VALIDATORS_DIR/test-consensus.sh"
EOF

chmod +x "$VALIDATORS_DIR/start-all-validators.sh"

# Create stop-all script
cat > "$VALIDATORS_DIR/stop-all-validators.sh" <<EOF
#!/bin/bash
# Stop all genesis validators

echo "Stopping all validators..."

for i in \$(seq 1 $VALIDATOR_COUNT); do
    if [ -f "$VALIDATORS_DIR/validator-\$i.pid" ]; then
        PID=\$(cat "$VALIDATORS_DIR/validator-\$i.pid")
        echo "Stopping validator #\$i (PID: \$PID)..."
        kill -TERM \$PID 2>/dev/null || true
        rm -f "$VALIDATORS_DIR/validator-\$i.pid"
    fi
done

echo "All validators stopped"
EOF

chmod +x "$VALIDATORS_DIR/stop-all-validators.sh"

# Summary
echo "════════════════════════════════════════════════════════════"
echo "         ✅ Genesis Validators Created! ✅                  "
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📁 Location: $VALIDATORS_DIR"
echo ""
echo "📊 Validators Created: $VALIDATOR_COUNT"
echo "   - Each with 10,000 SLTN stake"
echo "   - Each with 50M SLTN balance"
echo "   - Commission: 5-10% (random)"
echo ""
echo "📝 Files Created:"
echo "   • genesis.json - Network genesis state"
echo "   • validator-*.json - Individual validator configs"
echo "   • start-validator-*.sh - Individual startup scripts"
echo "   • start-all-validators.sh - Start all validators"
echo "   • stop-all-validators.sh - Stop all validators"
echo "   • test-consensus.sh - Test consensus working"
echo ""
echo "🚀 Quick Start:"
echo "   # Start all validators:"
echo "   bash $VALIDATORS_DIR/start-all-validators.sh"
echo ""
echo "   # Test consensus (wait 10 seconds after starting):"
echo "   bash $VALIDATORS_DIR/test-consensus.sh"
echo ""
echo "   # Stop all validators:"
echo "   bash $VALIDATORS_DIR/stop-all-validators.sh"
echo ""
echo "📍 RPC Endpoints:"
for i in $(seq 1 $VALIDATOR_COUNT); do
    PORT=$((26657 + i - 1))
    echo "   Validator #$i: http://localhost:$PORT"
done
echo ""
