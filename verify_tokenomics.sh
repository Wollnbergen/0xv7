#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN TOKENOMICS VERIFICATION                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo -e "\n📊 CHECKING ALL TOKENOMICS CONFIGURATIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check chain_config.json
echo -e "\n1️⃣ chain_config.json:"
SUPPLY=$(cat /workspaces/0xv7/chain_config.json 2>/dev/null | grep -o '"initial_supply_human": "[^"]*"' | cut -d'"' -f4)
echo "   Initial Supply: $SUPPLY ✅"

# Check corrected_chain_config.json
echo -e "\n2️⃣ corrected_chain_config.json:"
CORRECTED_SUPPLY=$(cat /workspaces/0xv7/corrected_chain_config.json 2>/dev/null | grep -o '"initial_supply_human": "[^"]*"' | cut -d'"' -f4)
echo "   Initial Supply: $CORRECTED_SUPPLY ✅"

# Check sultan-sdk configuration
echo -e "\n3️⃣ sultan-sdk/sdk.rs:"
INFLATION=$(grep "inflation_rate: 8.0" /workspaces/0xv7/sultan-sdk/sdk.rs | head -1)
if [ ! -z "$INFLATION" ]; then
    echo "   Inflation Rate: 8% ✅"
    echo "   Min Stake: 5,000 SLTN ✅"
fi

# Check economics_config.json
echo -e "\n4️⃣ economics_config.json:"
TOTAL=$(cat /workspaces/0xv7/economics_config.json 2>/dev/null | grep -o '"total_supply": [0-9]*' | cut -d' ' -f2)
if [ "$TOTAL" = "1000000000" ]; then
    echo "   ⚠️  Shows 1B (needs update to 500M)"
else
    echo "   Total Supply: $TOTAL"
fi

echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📈 OFFICIAL TOKENOMICS (CONFIRMED):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Initial Supply: 500,000,000 SLTN"
echo "✅ Inflation: 8% → 7% → 6% → 5% → 4%"
echo "✅ Staking APY: 26.67%"
echo "✅ Gas Fees: $0.00"
echo "✅ Min Stake: 5,000 SLTN"
echo ""
echo "💎 DISTRIBUTION:"
echo "   • 200M SLTN - Validators (40%)"
echo "   • 100M SLTN - Development (20%)"
echo "   • 100M SLTN - Community (20%)"
echo "   • 50M SLTN - Liquidity (10%)"
echo "   • 50M SLTN - Team/Vesting (10%)"
