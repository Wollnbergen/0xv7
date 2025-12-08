const express = require('express');
const axios = require('axios');

const app = express();
app.use(express.json());

// Root route: brief help + health
app.get('/', async (req, res) => {
    res.json({
        name: 'Sultan Unified API',
        version: '1.0',
        endpoints: {
            status: '/status',
            rpc_proxy_hint: 'If using TLS proxy, try https://<host>/',
        },
        tips: [
            'Use /status for summarized health',
            'Direct RPC at http://localhost:26657/status (dev only)',
        ],
    });
});

app.get('/status', async (req, res) => {
    try {
        // Try to get Sultan status
        const sultanStatus = await axios.get('http://localhost:3030/status')
            .catch(() => ({ data: { status: 'offline' }}));
        
        // Try to get Cosmos status
        const cosmosStatus = await axios.get('http://localhost:26657/status')
            .catch(() => ({ data: { result: { sync_info: { latest_block_height: 0 }}}}));
        
        res.json({
            chain: 'Sultan Chain (Cosmos-Integrated)',
            sultan: {
                api: 'http://localhost:3030',
                apy: '13.33%',
                status: sultanStatus.data
            },
            cosmos: {
                api: 'http://localhost:26657',
                height: cosmosStatus.data.result?.sync_info?.latest_block_height || 0,
                ibc_enabled: true,
                wasm_enabled: true
            },
            unified_features: {
                zero_gas: true,
                staking_apy: '13.33%',
                ibc_support: true,
                smart_contracts: true,
                quantum_safe: true,
                tps_target: 1230000
            }
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

const PORT = process.env.PORT || 8080;
const HOST = process.env.HOST || '0.0.0.0';
app.listen(PORT, HOST, () => {
    console.log(`✅ Sultan-Cosmos Unified API running on http://${HOST}:${PORT}`);
});
