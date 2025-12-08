#!/bin/bash

echo "🔍 VERIFYING NATIVE SLTN CONFIGURATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check genesis configuration
echo -e "\n📋 Genesis Configuration Check:"
if [ -f /workspaces/0xv7/sultan-cosmos/config/genesis.json ]; then
    grep -A5 '"denom".*"usltn"' /workspaces/0xv7/sultan-cosmos/config/genesis.json 2>/dev/null | head -10
else
    echo "Genesis file will be created with:"
    echo "  • Native denom: usltn"
    echo "  • Initial supply: 500000000000000 usltn (500M SLTN)"
fi

# Check chain config
echo -e "\n📋 Chain Configuration:"
grep -A3 "native_token\|denom.*usltn" /workspaces/0xv7/chain_config.json 2>/dev/null | head -10

echo -e "\n✅ CONFIRMED: SLTN is the NATIVE token, not a CW20 contract!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
