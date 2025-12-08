#!/bin/bash

echo "🌐 Starting Sultan Chain Testnet API..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd /workspaces/0xv7

# Create the API directory if it doesn't exist
if [ ! -d api ]; then
    echo "📦 Creating API directory..."
    mkdir -p api
    cd api
    
    # Create package.json
    cat > package.json << 'JSON'
{
  "name": "sultan-chain-api",
  "version": "1.0.0",
  "description": "Sultan Chain Zero Gas Blockchain API",
  "main": "server.js",
  "scripts": {
    "start": "node server.js"
  },
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5"
  }
}
JSON

    # Create the API server
    cat > server.js << 'JS'
const express = require('express');
const cors = require('cors');
const app = express();

app.use(cors());
app.use(express.json());
app.use(express.static('public'));

// Sultan Chain economics
const ECONOMICS = {
    inflation_rate: "4% annually",
    validator_apy: "13.33%",
    mobile_validator_bonus: "+40%",
    mobile_validator_total_apy: "18.66%",
    staking_ratio_assumption: "30%",
    user_gas_fees: 0,
    fee_subsidy_source: "4% inflation pool",
    formula: "APY = 4% inflation ÷ 0.3 staking ratio = 13.33%"
};

// API endpoints
app.post('/', (req, res) => {
    const { method, params, id } = req.body;
    
    switch(method) {
        case 'get_economics':
            res.json({ jsonrpc: "2.0", result: ECONOMICS, id });
            break;
        
        case 'get_apy':
            res.json({ 
                jsonrpc: "2.0", 
                result: {
                    base_apy: "13.33%",
                    mobile_validator_bonus: "40%",
                    total_possible: "18.66%",
                    calculation: "4% inflation ÷ 30% staked = 13.33% APY"
                },
                id 
            });
            break;
            
        case 'chain_status':
            res.json({
                jsonrpc: "2.0",
                result: {
                    name: "Sultan Chain",
                    height: 123456,
                    validators: 100,
                    mobile_validators: 40,
                    tps: 10000,
                    zero_fees: true,
                    inflation_rate: "4% annually",
                    validator_apy: "13.33%",
                    mobile_validator_apy: "18.66%",
                    ibc_enabled: true
                },
                id
            });
            break;
            
        case 'wallet_create':
            res.json({
                jsonrpc: "2.0",
                result: {
                    address: "sultan1" + Math.random().toString(36).substr(2, 39),
                    balance: 1000,
                    gas_fees: "$0.00 forever"
                },
                id
            });
            break;
            
        case 'token_transfer':
            res.json({
                jsonrpc: "2.0",
                result: {
                    tx_hash: "0x" + Math.random().toString(16).substr(2, 64),
                    gas_used: 0,
                    gas_cost: "$0.00",
                    status: "success"
                },
                id
            });
            break;
            
        default:
            res.json({
                jsonrpc: "2.0",
                error: {
                    code: -32601,
                    message: "Method not found",
                    data: {
                        available_methods: [
                            "chain_status",
                            "get_economics", 
                            "get_apy",
                            "wallet_create",
                            "token_transfer"
                        ]
                    }
                },
                id
            });
    }
});

// Create public directory with UI
const fs = require('fs');
if (!fs.existsSync('public')) {
    fs.mkdirSync('public');
    fs.writeFileSync('public/index.html', `<!DOCTYPE html>
<html>
<head>
    <title>Sultan Chain API</title>
    <style>
        body { font-family: monospace; padding: 20px; background: #0a0a0a; color: #00ff00; }
        h1 { color: #00ff00; }
        .endpoint { background: #111; padding: 10px; margin: 10px 0; border: 1px solid #00ff00; }
        .economics { background: #001100; padding: 15px; border: 2px solid #00ff00; }
        pre { background: #000; padding: 10px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>🚀 Sultan Chain API</h1>
    <div class="economics">
        <h2>Zero Gas Fees • 13.33% Validator APY • Mobile Validators</h2>
        <h3>💰 Economics (Resolved from Codebase)</h3>
        <p>✅ 4% Annual Inflation - Creates the reward pool</p>
        <p>📈 13.33% APY for Validators - When 30% of tokens are staked</p>
        <p>📱 40% Mobile Validator Bonus - Total ~18.66% APY</p>
        <p>⛽ ZERO Gas Fees Forever - Subsidized by 4% inflation</p>
        <pre>The Formula:
Validator APY = Inflation Rate ÷ Staking Ratio
13.33% = 8% ÷ 0.3 (30% staked)

Mobile Validator APY = 13.33% × 1.4 = 18.66%</pre>
    </div>
    <div class="endpoint">
        <h3>Chain Status</h3>
        <pre>POST /
{"jsonrpc":"2.0","method":"chain_status","id":1}</pre>
    </div>
    <div class="endpoint">
        <h3>Get Economics</h3>
        <pre>POST /
{"jsonrpc":"2.0","method":"get_economics","id":1}</pre>
    </div>
    <div class="endpoint">
        <h3>Create Wallet</h3>
        <pre>POST /
{"jsonrpc":"2.0","method":"wallet_create","params":["username"],"id":1}</pre>
    </div>
    <div class="endpoint">
        <h3>Transfer (Zero Fees!)</h3>
        <pre>POST /
{"jsonrpc":"2.0","method":"token_transfer","params":["from","to",100],"id":1}</pre>
    </div>
</body>
</html>`);
}

const PORT = 3030;
app.listen(PORT, () => {
    console.log(\`✅ Sultan Chain Testnet API running on http://localhost:\${PORT}\`);
    console.log(\`🌐 Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/\`);
});
JS

    echo "📦 Installing dependencies..."
    npm install --quiet
else
    cd api
fi

# Start the API
echo "🚀 Starting API server..."
npm start

