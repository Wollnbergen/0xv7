const http = require('http');

const ECONOMICS = {
    inflation_year_1: 8.0,
    inflation_year_2: 6.0,
    inflation_year_5_plus: 2.0,
    burn_rate: 1.0,
    validator_apy_max: 26.67,
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
                                inflation_schedule: "8% → 6% → 4% → 3% → 2%",
                                current_inflation: "8% (Year 1)",
                                burn_mechanism: "1% on high-volume transactions",
                                validator_apy: "26.67% maximum",
                                mobile_validator_bonus: "Removed (was 40%)",
                                staking_ratio_target: "30%",
                                user_gas_fees: 0,
                                becomes_deflationary: "Year 5",
                                formula: "APY = min(26.67%, inflation ÷ staking_ratio)"
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
                                validator_apy: "26.67% max",
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
            economics: "Corrected - 26.67% APY max, no mobile bonus",
            status: "Running"
        }));
    }
});

server.listen(3030, () => {
    console.log('✅ Sultan Chain API running with CORRECTED economics');
    console.log('📊 Validator APY: 26.67% maximum (no mobile bonus)');
    console.log('🔥 Burn mechanism: 1% active');
    console.log('📉 Inflation: 8% → 2% over 5 years');
});
