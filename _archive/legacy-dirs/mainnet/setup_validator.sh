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
