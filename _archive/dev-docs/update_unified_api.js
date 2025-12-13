const express = require('express');
const app = express();

app.get('/tokenomics', (req, res) => {
    res.json({
        native_token: {
            name: "Sultan Token",
            symbol: "SLTN",
            denom: "usltn",
            type: "NATIVE",
            total_supply: "1000000000000000",
            circulating_supply: "100000000000000",
            features: {
                gas_token: true,
                staking_token: true,
                governance_token: true,
                ibc_enabled: true,
                zero_gas_fees: true
            }
        },
        wrapped_token: {
            name: "Wrapped Sultan Token",
            symbol: "wSLTN",
            type: "CW20",
            contract: "sultan1...",
            use_cases: ["DEX", "Liquidity Pools", "DeFi"]
        },
        economics: {
            staking_apy: "13.33%",
            inflation: "8%",
            gas_fees: "$0.00",
            validator_rewards: "13.33% APY"
        }
    });
});

// Add this to the existing unified API
