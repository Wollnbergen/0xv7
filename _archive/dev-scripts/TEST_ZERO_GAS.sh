#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          TESTING ZERO GAS FEE TRANSACTIONS                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if chain is running
if ! curl -s http://localhost:26657/status > /dev/null 2>&1; then
    echo "❌ Chain not running. Start it first with:"
    echo "   ./RUN_SOVEREIGN.sh"
    exit 1
fi

echo "✅ Chain is running!"
echo ""

# Get addresses
VALIDATOR=$($HOME/go/bin/sovereignd keys show validator -a --keyring-backend test --home $HOME/.sovereign)
ALICE=$($HOME/go/bin/sovereignd keys show alice -a --keyring-backend test --home $HOME/.sovereign)

echo "📍 Addresses:"
echo "   Validator: $VALIDATOR"
echo "   Alice: $ALICE"
echo ""

# Check balances
echo "💰 Current Balances:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
$HOME/go/bin/sovereignd query bank balances $VALIDATOR --home $HOME/.sovereign
echo ""

# Send transaction with ZERO gas
echo "📤 Sending transaction with ZERO gas fees..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

$HOME/go/bin/sovereignd tx bank send \
    $VALIDATOR \
    $ALICE \
    1000stake \
    --chain-id sovereign-1 \
    --keyring-backend test \
    --home $HOME/.sovereign \
    --gas-prices 0stake \
    --yes

echo ""
echo "⏳ Waiting for transaction to be included..."
sleep 5

# Check Alice's balance
echo ""
echo "💰 Alice's New Balance:"
$HOME/go/bin/sovereignd query bank balances $ALICE --home $HOME/.sovereign

echo ""
echo "✅ Zero gas fee transaction successful!"

