#!/bin/bash

echo "🌐 Setting up Sultan Chain Testnet API..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd /workspaces/0xv7

# Kill any existing process on port 3030
lsof -ti:3030 | xargs -r kill -9 2>/dev/null

# Create the API directory structure
mkdir -p api
cd api

# Create a simple working server without npm dependencies first
cat > simple_server.js << 'JS'
const http = require('http');

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

const HTML = `<!DOCTYPE html>
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
        <h3>💰 FINAL Economics (Resolved from Codebase)</h3>
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
</html>`;

const server = http.createServer((req, res) => {
    // Handle CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    if (req.method === 'GET') {
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(HTML);
        return;
    }
    
    if (req.method === 'POST') {
        let body = '';
        req.on('data', chunk => body += chunk.toString());
        req.on('end', () => {
            try {
                const { method, params, id } = JSON.parse(body);
                let result;
                
                switch(method) {
                    case 'get_economics':
                        result = ECONOMICS;
                        break;
                    case 'get_apy':
                        result = {
                            base_apy: "13.33%",
                            mobile_validator_bonus: "40%",
                            total_possible: "18.66%",
                            calculation: "4% inflation ÷ 30% staked = 13.33% APY"
                        };
                        break;
                    case 'chain_status':
                        result = {
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
                        };
                        break;
                    case 'wallet_create':
                        result = {
                            address: "sultan1" + Math.random().toString(36).substr(2, 39),
                            balance: 1000,
                            gas_fees: "$0.00 forever"
                        };
                        break;
                    case 'token_transfer':
                        result = {
                            tx_hash: "0x" + Math.random().toString(16).substr(2, 64),
                            gas_used: 0,
                            gas_cost: "$0.00",
                            status: "success"
                        };
                        break;
                    default:
                        res.writeHead(200, { 'Content-Type': 'application/json' });
                        res.end(JSON.stringify({
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
                        }));
                        return;
                }
                
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ jsonrpc: "2.0", result, id }));
            } catch (e) {
                res.writeHead(400, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: 'Invalid JSON' }));
            }
        });
    }
});

const PORT = 3030;
server.listen(PORT, '0.0.0.0', () => {
    console.log('✅ Sultan Chain Testnet API running on http://localhost:' + PORT);
    console.log('🌐 Public URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/');
});
JS

# Start the simple server
echo "🚀 Starting API server..."
node simple_server.js

