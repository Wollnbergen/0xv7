const express = require('express');
const app = express();
app.use(express.json());

const port = process.env.PORT || 4001;
const nodeId = process.env.NODE_ID || `node-${port}`;

const state = {
    validators: new Map(),
    blockHeight: 13247,
    totalStaked: 365000
};

// Simple endpoints
app.get('/consensus_state', (req, res) => {
    res.json({
        nodeId: nodeId,
        port: port,
        blockHeight: state.blockHeight,
        validators: state.validators.size,
        totalStaked: state.totalStaked,
        status: 'active'
    });
});

app.post('/register_validator', (req, res) => {
    const { address, stake } = req.body;
    state.validators.set(address, { stake, joined: Date.now() });
    state.totalStaked += stake;
    
    res.json({
        success: true,
        validators: state.validators.size,
        networkStake: state.totalStaked
    });
});

app.listen(port, () => {
    console.log(`✅ Consensus node ${nodeId} running on port ${port}`);
});

// Increment block height every 3 seconds
setInterval(() => {
    state.blockHeight++;
}, 3000);
