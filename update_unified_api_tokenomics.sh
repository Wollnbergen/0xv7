#!/bin/bash

# Find and update the unified API
API_FILE=$(find /workspaces/0xv7 -name "unified_api.py" -o -name "main.py" | grep unified | head -1)

if [ ! -z "$API_FILE" ]; then
    echo "Updating $API_FILE with correct tokenomics..."
    # Add tokenomics endpoint to existing API
    cat >> "$API_FILE" << 'PYTHON'

@app.route('/tokenomics/corrected')
def get_corrected_tokenomics():
    return {
        "initial_supply": "500,000,000 SLTN",
        "inflation": {
            "year_1": "8%",
            "year_2": "7%",
            "year_3": "6%",
            "year_4": "5%",
            "year_5+": "4%"
        },
        "staking_apy": "13.33%",
        "gas_fees": "$0.00",
        "min_stake": "5,000 SLTN"
    }
PYTHON
    echo "✅ API updated"
else
    echo "⚠️ Unified API file not found, creating new endpoint..."
fi
