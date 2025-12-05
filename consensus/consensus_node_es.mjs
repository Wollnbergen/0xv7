import express from 'express';
import crypto from 'crypto';

const app = express();
app.use(express.json());

const port = parseInt(process.env.PORT) || 4001;
const nodeId = process.env.NODE_ID || `validator-${port}`;

const state = {
    validators: new Map(),
    blockHeight: 13247,
    totalStaked: 365000,
    transactions: [],
    gasFeesCollected: 0  // Always 0!
};

// Consensus state endpoint
app.get('/consensus_state', (req, res) => {
    res.json({
        nodeId: nodeId,
        port: port,
        blockHeight: state.blockHeight,
        validators: state.validators.size,
        totalStaked: state.totalStaked,
        gasFeesCollected: state.gasFeesCollected,
        status: 'active'
    });
});

// Register validator endpoint
app.post('/register_validator', (req, res) => {
    const { address, stake } = req.body;
    
    if (stake < 5000) {
        return res.status(400).json({ error: 'Minimum stake is 5,000 SLTN' });
    }
    
    state.validators.set(address || `sultan1${crypto.randomBytes(20).toString('hex')}`, { 
        stake, 
        joined: Date.now(),
        apy: 0.2667,
        dailyRewards: stake * 0.2667 / 365
    });
    
    state.totalStaked += stake;
    
    res.json({
        success: true,
        validators: state.validators.size,
        networkStake: state.totalStaked,
        apy: '26.67%',
        dailyRewards: (stake * 0.2667 / 365).toFixed(2) + ' SLTN'
    });
});

// Send transaction with ZERO fees
app.post('/send_transaction', (req, res) => {
    const { from, to, amount } = req.body;
    
    const tx = {
        hash: crypto.randomBytes(32).toString('hex'),
        from,
        to,
        amount,
        fee: 0,  // ALWAYS ZERO!
        timestamp: Date.now(),
        block: state.blockHeight
    };
    
    state.transactions.push(tx);
    
    res.json({
        success: true,
        txHash: tx.hash,
        fee: 0,
        message: 'Transaction sent with $0.00 gas fees!'
    });
});

// Health check
app.get('/health', (req, res) => {
    res.json({ status: 'healthy', nodeId, blockHeight: state.blockHeight });
});

app.listen(port, () => {
    console.log(`✅ Consensus node ${nodeId} running on port ${port}`);
    console.log(`   Zero gas fees: ACTIVE`);
    console.log(`   APY: 26.67%`);
});

// Increment block height every 3 seconds
setInterval(() => {
    state.blockHeight++;
    if (state.blockHeight % 10 === 0) {
        console.log(`Block ${state.blockHeight} produced - Gas fees: $0.00`);
    }
}, 3000);
