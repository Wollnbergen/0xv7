#!/bin/bash
set -e

# Sultan Testnet Quick Deploy Script
# This script deploys a single-node Sultan testnet

echo "🚀 Sultan Testnet Deployment Script"
echo "===================================="
echo ""

# Configuration
CHAIN_ID="${CHAIN_ID:-sultan-testnet-1}"
MONIKER="${MONIKER:-sultan-validator-1}"
KEYRING_BACKEND="${KEYRING_BACKEND:-file}"
VALIDATOR_STAKE="${VALIDATOR_STAKE:-900000000000stake}"
GENESIS_SUPPLY="${GENESIS_SUPPLY:-1000000000000stake}"

echo "Configuration:"
echo "  Chain ID: $CHAIN_ID"
echo "  Moniker: $MONIKER"
echo "  Keyring: $KEYRING_BACKEND"
echo "  Validator Stake: $VALIDATOR_STAKE"
echo "  Genesis Supply: $GENESIS_SUPPLY"
echo ""

# Check if sultand binary exists
if ! command -v sultand &> /dev/null; then
    echo "❌ sultand binary not found!"
    echo "   Please build and install it first:"
    echo "   cd sultan-cosmos-real && go build -o sultand ./cmd/sultand"
    echo "   sudo cp sultand /usr/local/bin/"
    exit 1
fi

echo "✅ Found sultand binary: $(which sultand)"
echo ""

# Clean existing data (optional)
read -p "⚠️  Remove existing ~/.sultan directory? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  Removing ~/.sultan..."
    rm -rf ~/.sultan
fi

# Initialize node
echo "1️⃣  Initializing node..."
sultand init $MONIKER --chain-id $CHAIN_ID
echo "✅ Node initialized"
echo ""

# Create validator key
echo "2️⃣  Creating validator key..."
if sultand keys show validator --keyring-backend $KEYRING_BACKEND &> /dev/null; then
    echo "⚠️  Validator key already exists"
    read -p "   Use existing key? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        sultand keys delete validator --keyring-backend $KEYRING_BACKEND -y
        sultand keys add validator --keyring-backend $KEYRING_BACKEND
    fi
else
    sultand keys add validator --keyring-backend $KEYRING_BACKEND
fi
echo "✅ Validator key created/verified"
echo ""

# Get validator address
VALIDATOR_ADDR=$(sultand keys show validator -a --keyring-backend $KEYRING_BACKEND)
echo "📍 Validator address: $VALIDATOR_ADDR"
echo ""

# Add genesis account
echo "3️⃣  Adding genesis account..."
sultand add-genesis-account validator $GENESIS_SUPPLY --keyring-backend $KEYRING_BACKEND
echo "✅ Genesis account added"
echo ""

# Create genesis transaction
echo "4️⃣  Creating genesis transaction..."
echo "⚠️  Note: If this fails with 'InterfaceRegistry' error, we'll use manual genesis creation"
if sultand gentx validator $VALIDATOR_STAKE \
    --chain-id $CHAIN_ID \
    --keyring-backend $KEYRING_BACKEND \
    --moniker $MONIKER \
    --commission-rate 0.10 \
    --commission-max-rate 0.20 \
    --commission-max-change-rate 0.01 \
    --min-self-delegation 1 2>/dev/null; then
    
    echo "✅ Genesis transaction created"
    
    # Collect genesis transactions
    echo "5️⃣  Collecting genesis transactions..."
    sultand collect-gentxs
    echo "✅ Genesis transactions collected"
else
    echo "⚠️  gentx command failed (expected with SDK v0.50.5)"
    echo "   Using manual genesis creation method..."
    echo ""
    
    # Get consensus pubkey
    CONSENSUS_PUBKEY=$(cat ~/.sultan/config/priv_validator_key.json | jq -r '.pub_key.value')
    CONSENSUS_ADDR=$(cat ~/.sultan/config/priv_validator_key.json | jq -r '.address')
    
    # Get validator operator address
    VALOPER_ADDR=$(sultand keys show validator --bech val -a --keyring-backend $KEYRING_BACKEND)
    
    # Get bonded pool address
    BONDED_POOL="sultan1fl48vsnmsdzcv85q5d2q4z5ajdha8yu3905xlj"
    
    echo "   Validator operator: $VALOPER_ADDR"
    echo "   Consensus address: $CONSENSUS_ADDR"
    echo "   Consensus pubkey: $CONSENSUS_PUBKEY"
    echo ""
    
    # Create manual genesis using Python
    cat > /tmp/create_genesis.py << 'EOF'
import json
import sys

genesis_file = sys.argv[1]
validator_addr = sys.argv[2]
valoper_addr = sys.argv[3]
consensus_pubkey = sys.argv[4]
consensus_addr = sys.argv[5]
bonded_pool = sys.argv[6]
validator_stake = sys.argv[7]
total_supply = sys.argv[8]

# Load genesis
with open(genesis_file, 'r') as f:
    genesis = json.load(f)

# Calculate balances
stake_amount = validator_stake.replace('stake', '')
total_amount = total_supply.replace('stake', '')
user_balance = str(int(total_amount) - int(stake_amount))

# Update bank balances
genesis['app_state']['bank']['balances'] = [
    {
        'address': validator_addr,
        'coins': [{'denom': 'stake', 'amount': user_balance}]
    },
    {
        'address': bonded_pool,
        'coins': [{'denom': 'stake', 'amount': stake_amount}]
    }
]

# Update supply
genesis['app_state']['bank']['supply'] = [
    {'denom': 'stake', 'amount': total_amount}
]

# Add validator
genesis['app_state']['staking']['validators'] = [{
    'operator_address': valoper_addr,
    'consensus_pubkey': {
        '@type': '/cosmos.crypto.ed25519.PubKey',
        'key': consensus_pubkey
    },
    'jailed': False,
    'status': 'BOND_STATUS_BONDED',
    'tokens': stake_amount,
    'delegator_shares': stake_amount + '.000000000000000000',
    'description': {
        'moniker': 'validator',
        'identity': '',
        'website': '',
        'security_contact': '',
        'details': ''
    },
    'unbonding_height': '0',
    'unbonding_time': '1970-01-01T00:00:00Z',
    'commission': {
        'commission_rates': {
            'rate': '0.100000000000000000',
            'max_rate': '0.200000000000000000',
            'max_change_rate': '0.010000000000000000'
        },
        'update_time': '2025-11-21T00:00:00Z'
    },
    'min_self_delegation': '1',
    'unbonding_on_hold_ref_count': '0',
    'unbonding_ids': []
}]

# Add delegation
genesis['app_state']['staking']['delegations'] = [{
    'delegator_address': validator_addr,
    'validator_address': valoper_addr,
    'shares': stake_amount + '.000000000000000000'
}]

# Add CometBFT validator
genesis['validators'] = [{
    'address': consensus_addr,
    'pub_key': {
        'type': 'tendermint/PubKeyEd25519',
        'value': consensus_pubkey
    },
    'power': str(int(stake_amount) // 1000000),
    'name': 'validator'
}]

# Save genesis
with open(genesis_file, 'w') as f:
    json.dump(genesis, f, indent=2)

print('✅ Genesis manually created')
EOF
    
    python3 /tmp/create_genesis.py \
        ~/.sultan/config/genesis.json \
        "$VALIDATOR_ADDR" \
        "$VALOPER_ADDR" \
        "$CONSENSUS_PUBKEY" \
        "$CONSENSUS_ADDR" \
        "$BONDED_POOL" \
        "$VALIDATOR_STAKE" \
        "$GENESIS_SUPPLY"
    
    rm /tmp/create_genesis.py
fi
echo ""

# Validate genesis
echo "6️⃣  Validating genesis..."
sultand validate-genesis
echo "✅ Genesis validated"
echo ""

# Configure node
echo "7️⃣  Configuring node..."

# Set minimum gas prices to 0
sed -i 's/minimum-gas-prices = ""/minimum-gas-prices = "0stake"/' ~/.sultan/config/app.toml

# Enable API
sed -i 's/enable = false/enable = true/' ~/.sultan/config/app.toml
sed -i 's/swagger = false/swagger = true/' ~/.sultan/config/app.toml

# Set CORS for RPC
sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = ["*"]/' ~/.sultan/config/config.toml

echo "✅ Node configured"
echo ""

# Create systemd service
echo "8️⃣  Creating systemd service..."
read -p "   Create systemd service? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo tee /etc/systemd/system/sultand.service > /dev/null <<EOF
[Unit]
Description=Sultan Cosmos Node
After=network-online.target

[Service]
User=$USER
ExecStart=$(which sultand) start --home $HOME/.sultan
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    sudo systemctl enable sultand
    echo "✅ Systemd service created and enabled"
    echo ""
    
    read -p "   Start node now? (Y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        sudo systemctl start sultand
        echo "✅ Node started"
        echo ""
        sleep 3
        echo "📊 Node status:"
        sudo systemctl status sultand --no-pager
    fi
else
    echo "⏭️  Skipped systemd service creation"
    echo ""
    echo "To start manually, run:"
    echo "  sultand start"
fi

echo ""
echo "🎉 Deployment complete!"
echo ""
echo "📋 Summary:"
echo "  Chain ID: $CHAIN_ID"
echo "  Validator Address: $VALIDATOR_ADDR"
echo "  Home Directory: ~/.sultan"
echo ""
echo "🔍 Check status:"
echo "  curl localhost:26657/status | jq"
echo ""
echo "📖 View logs:"
if systemctl is-active --quiet sultand; then
    echo "  sudo journalctl -u sultand -f"
else
    echo "  (node not running as service)"
fi
echo ""
echo "🌐 Endpoints:"
echo "  RPC:  http://localhost:26657"
echo "  API:  http://localhost:1317"
echo "  gRPC: http://localhost:9090"
echo "  P2P:  tcp://localhost:26656"
echo ""
echo "💡 Next steps:"
echo "  1. Configure firewall: sudo ufw allow 26656,26657,9090,1317/tcp"
echo "  2. Setup domain and SSL (see TESTNET_DEPLOYMENT.md)"
echo "  3. Monitor logs and ensure blocks are being produced"
echo "  4. Share genesis.json and node ID with other validators"
echo ""
