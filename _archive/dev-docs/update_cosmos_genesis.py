#!/usr/bin/env python3
import json

# Load Cosmos genesis
with open('/workspaces/0xv7/sultan-cosmos/genesis.json', 'r') as f:
    genesis = json.load(f)

# Apply Sultan economics (13.33% APY requires ~4% inflation with 30% bonding)
# But we'll set higher inflation to achieve 13.33% APY
genesis['app_state']['mint']['params']['inflation_max'] = "0.800000000000000000"  # 80% max
genesis['app_state']['mint']['params']['inflation_min'] = "0.070000000000000000"  # 7% min
genesis['app_state']['mint']['params']['inflation_rate_change'] = "0.130000000000000000"
genesis['app_state']['mint']['params']['goal_bonded'] = "0.300000000000000000"  # 30% target

# Zero gas fees
genesis['app_state']['globalfee'] = {
    'params': {
        'minimum_gas_prices': [],
        'bypass_min_fee_msg_types': ['*']
    }
}

# Save updated genesis
with open('/workspaces/0xv7/sultan-cosmos/genesis.json', 'w') as f:
    json.dump(genesis, f, indent=2)

print("✅ Cosmos genesis updated with Sultan economics")
