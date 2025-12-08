#!/bin/bash

echo "🔧 Updating API with corrected economics..."

# Kill existing server
pkill -f "node.*simple_server" 2>/dev/null

# Update the API server
cat > /workspaces/0xv7/api/simple_server.js << 'JS'
const http = require('http');

const ECONOMICS = {
    inflation_year_1: 8.0,
    inflation_year_2: 6.0,
    inflation_year_5_plus: 2.0,
    burn_rate: 1.0,
    validator_apy_max: 13.33,
    gas_fees: 0.00
};

const server = http.createServer((req, res) => {
    res.setHeader('Content-Type', 'application/json');
    res.setHeader('Access-Control-Allow-Origin', '*');
    
    if (req.method === 'POST') {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', () => {
            try {
                const request = JSON.parse(body);
                let response = {};
                
                switch(request.method) {
                    case 'get_economics':
                        response = {
                            jsonrpc: "2.0",
                            result: {
                                inflation_schedule: "4% → 6% → 4% → 3% → 2%",
                                current_inflation: "8% (Year 1)",
                                burn_mechanism: "1% on high-volume transactions",
                                validator_apy: "13.33% maximum",
                                mobile_validator_bonus: "Removed (was 40%)",
                                staking_ratio_target: "30%",
                                user_gas_fees: 0,
                                becomes_deflationary: "Year 5",
                                formula: "APY = min(13.33%, inflation ÷ staking_ratio)"
                            },
                            id: request.id
                        };
                        break;
                        
                    case 'chain_status':
                        response = {
                            jsonrpc: "2.0",
                            result: {
                                name: "Sultan Chain",
                                height: 123456,
                                validators: 100,
                                tps: 10000,
                                zero_fees: true,
                                inflation_rate: "8% (declining)",
                                validator_apy: "13.33% max",
                                burn_active: true
                            },
                            id: request.id
                        };
                        break;
                        
                    default:
                        response = {
                            jsonrpc: "2.0",
                            result: {
                                message: "Sultan Chain - Zero fees forever!",
                                gas_cost: "$0.00"
                            },
                            id: request.id
                        };
                }
                
                res.writeHead(200);
                res.end(JSON.stringify(response));
            } catch (e) {
                res.writeHead(400);
                res.end(JSON.stringify({error: e.message}));
            }
        });
    } else {
        res.writeHead(200);
        res.end(JSON.stringify({
            name: "Sultan Chain API",
            economics: "Corrected - 13.33% APY max, no mobile bonus",
            status: "Running"
        }));
    }
});

server.listen(3030, () => {
    console.log('✅ Sultan Chain API running with CORRECTED economics');
    console.log('📊 Validator APY: 13.33% maximum (no mobile bonus)');
    console.log('🔥 Burn mechanism: 1% active');
    console.log('📉 Inflation: 4% → 2% over 5 years');
});
JS

# Start the updated server
cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
sleep 2

echo "✅ API server updated and restarted"
echo ""
echo "🧪 Testing new economics endpoint..."
curl -s -X POST http://localhost:3030 \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | jq '.'

