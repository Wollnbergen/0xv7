const http = require('http');

// Chain state
let blockHeight = 12847;
let validators = 4;
let tps = 156;

// Economics configuration
const ECONOMICS = {
    inflation_schedule: "8% → 6% → 4% → 3% → 2%",
    current_inflation: "8% (Year 1)",
    burn_mechanism: "1% on high-volume transactions",
    validator_apy: "26.67% maximum",
    gas_fees: 0.00
};

const server = http.createServer((req, res) => {
    res.setHeader('Content-Type', 'application/json');
    res.setHeader('Access-Control-Allow-Origin', '*');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    if (req.method === 'POST') {
        let body = '';
        req.on('data', chunk => body += chunk.toString());
        req.on('end', () => {
            try {
                const request = JSON.parse(body);
                let response = {
                    jsonrpc: "2.0",
                    id: request.id
                };
                
                switch(request.method) {
                    case 'get_status':
                        blockHeight++;
                        response.result = {
                            chain_id: "sultan-mainnet-1",
                            block_height: blockHeight,
                            latest_block_time: new Date().toISOString(),
                            validators: validators,
                            tps: tps + Math.floor(Math.random() * 20),
                            network: "testnet",
                            version: "1.0.0"
                        };
                        break;
                        
                    case 'get_economics':
                        response.result = ECONOMICS;
                        break;
                        
                    case 'transfer':
                        response.result = {
                            status: "success",
                            tx_hash: "0x" + Math.random().toString(36).substr(2, 64),
                            gas_fee: 0.00,
                            message: "Transfer completed with ZERO fees!"
                        };
                        break;
                        
                    case 'get_apy':
                        response.result = {
                            validator_apy: "26.67%",
                            staking_ratio: "30%",
                            total_staked: "300,000,000 SLTN"
                        };
                        break;
                        
                    default:
                        response.error = {
                            code: -32601,
                            message: "Method not found"
                        };
                }
                
                res.writeHead(200);
                res.end(JSON.stringify(response));
            } catch (e) {
                res.writeHead(400);
                res.end(JSON.stringify({
                    jsonrpc: "2.0",
                    error: {
                        code: -32700,
                        message: "Parse error"
                    }
                }));
            }
        });
    } else if (req.method === 'GET') {
        // Serve a simple web interface
        res.setHeader('Content-Type', 'text/html');
        res.writeHead(200);
        res.end(`
<!DOCTYPE html>
<html>
<head>
    <title>Sultan Chain Testnet</title>
    <style>
        body { font-family: monospace; background: #0a0a0a; color: #0ff; padding: 20px; }
        h1 { text-shadow: 0 0 10px #0ff; }
        .stats { background: #001; padding: 20px; border: 1px solid #0ff; border-radius: 10px; }
        .stat { margin: 10px 0; }
    </style>
</head>
<body>
    <h1>🚀 Sultan Chain Testnet</h1>
    <div class="stats">
        <div class="stat">⛓️ Chain ID: sultan-mainnet-1</div>
        <div class="stat">📦 Block Height: <span id="height">${blockHeight}</span></div>
        <div class="stat">💰 Gas Fees: $0.00 (FOREVER FREE)</div>
        <div class="stat">📈 Validator APY: 26.67%</div>
        <div class="stat">🔥 Status: LIVE</div>
    </div>
    <script>
        setInterval(() => {
            document.getElementById('height').textContent = parseInt(document.getElementById('height').textContent) + 1;
        }, 5000);
    </script>
</body>
</html>
        `);
    }
});

const PORT = 3030;
server.listen(PORT, () => {
    console.log(`Sultan Chain API running on port ${PORT}`);
    console.log(`Access at: http://localhost:${PORT}`);
});
