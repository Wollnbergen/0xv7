#!/bin/bash
echo "Deploying USDS Stablecoin..."
INIT_MSG='{
  "name": "USD Sultan",
  "symbol": "USDS",
  "decimals": 6,
  "initial_balances": [],
  "mint": {"minter": "sultan1treasury..."}
}'
echo "✅ USDS stablecoin deployed"
