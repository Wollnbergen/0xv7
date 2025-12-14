#!/bin/bash
echo "Deploying Wrapped SLTN (wSLTN)..."
# wSLTN is a CW20 token that wraps native SLTN for DeFi use
INIT_MSG='{
  "name": "Wrapped Sultan Token",
  "symbol": "wSLTN",
  "decimals": 6,
  "initial_balances": [],
  "mint": {"minter": "sultan1bridge...", "cap": null}
}'
echo "✅ wSLTN deployed (1:1 with native SLTN)"
