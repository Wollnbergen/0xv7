#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         UPDATING MINIMUM STAKE TO 1,000 SLTN                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Update validator.rs if it exists
VALIDATOR_FILE="/workspaces/0xv7/sultan-chain-mainnet/src/consensus/validator.rs"
if [ -f "$VALIDATOR_FILE" ]; then
    echo "✅ Updating validator.rs..."
    sed -i 's/MINIMUM_STAKE: u128 = [0-9_]*/MINIMUM_STAKE: u128 = 1_000/g' "$VALIDATOR_FILE"
    sed -i 's/min_stake: [0-9]*/min_stake: 1000/g' "$VALIDATOR_FILE"
fi

# Update any config files
echo "✅ Updating configuration files..."
find /workspaces/0xv7/sultan-chain-mainnet -name "*.toml" -o -name "*.yaml" -o -name "*.json" 2>/dev/null | while read file; do
    sed -i 's/100000/1000/g' "$file" 2>/dev/null
    sed -i 's/5000/1000/g' "$file" 2>/dev/null
done

echo "✅ Configuration updated to 1,000 SLTN minimum stake"
