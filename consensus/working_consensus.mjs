import express from 'express';
import crypto from 'crypto';

const app = express();
app.use(express.json());

const port = parseInt(process.env.PORT) || 4001;
const nodeId = process.env.NODE_ID || `validator-${port}`;
const peers = process.env.PEERS ? process.env.PEERS.split(',').map(p => parseInt(p)) : [];

// Initialize state
const state = {
    validators: new Map(),
    blockHeight: 13247,
    totalStaked: 365000,
    transactions: [],
    gasFeesCollected: 0,
    consensusRounds: 0,
    lastConsensus: null,
    votes: new Map()
};

// Add initial validators
state.validators.set('sultan1genesis', { stake: 100000, joined: Date.now() });
state.validators.set('sultan1validator1', { stake: 50000, joined: Date.now() });
state.validators.set('sultan1validator2', { stake: 75000, joined: Date.now() });

// Endpoints
app.get('/consensus_state', (req, res) => {
    res.json({
        nodeId: nodeId,
        port: port,
        blockHeight: state.blockHeight,
        validators: state.validators.size,
        totalStaked: state.totalStaked,
        gasFeesCollected: state.gasFeesCollected,
        consensusRounds: state.consensusRounds,
        lastConsensus: state.lastConsensus,
        peers: peers,
        status: 'active'
    });
});

app.get('/health', (req, res) => {
    res.json({ 
        status: 'healthy', 
        nodeId, 
        blockHeight: state.blockHeight,
        uptime: process.uptime() 
    });
});

app.get('/peers', (req, res) => {
    res.json({
        nodeId: nodeId,
        peers: peers,
        knownValidators: Array.from(state.validators.keys())
    });
});

app.post('/propose_block', async (req, res) => {
    const { block, proposer } = req.body;
    
    console.log(`[${nodeId}] Block proposed by ${proposer}`);
    
    // Simulate consensus voting
    let votes = 1;
    let totalNodes = peers.length + 1;
    
    // Get votes from peers
    for (const peerPort of peers) {
        try {
            const response = await fetch(`http://localhost:${peerPort}/vote`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ block, voter: nodeId })
            });
            if (response.ok) {
                votes++;
            }
        } catch (e) {
            console.log(`Peer ${peerPort} did not respond`);
        }
    }
    
    const consensusThreshold = Math.ceil(totalNodes * 2 / 3);
    const consensusReached = votes >= consensusThreshold;
    
    if (consensusReached) {
        state.blockHeight++;
        state.consensusRounds++;
        state.lastConsensus = Date.now();
        
        console.log(`[${nodeId}] Consensus reached! Block ${state.blockHeight}`);
    }
    
    res.json({
        success: consensusReached,
        blockHeight: state.blockHeight,
        votes: votes,
        required: consensusThreshold,
        totalNodes: totalNodes,
        consensus: consensusReached ? 'REACHED' : 'FAILED',
        gasFeesUsed: 0
    });
});

app.post('/vote', (req, res) => {
    const { block, voter } = req.body;
    res.json({ vote: 'yes', nodeId: nodeId });
});

app.post('/register_validator', (req, res) => {
    const { address, stake } = req.body;
    
    if (!stake || stake < 5000) {
        return res.status(400).json({ error: 'Minimum stake is 5,000 SLTN' });
    }
    
    const validatorAddress = address || `sultan1${crypto.randomBytes(20).toString('hex')}`;
    state.validators.set(validatorAddress, { 
        stake, 
        joined: Date.now(),
        apy: 0.1333,
        dailyRewards: stake * 0.1333 / 365
    });
    
    state.totalStaked += stake;
    
    res.json({
        success: true,
        address: validatorAddress,
        validators: state.validators.size,
        networkStake: state.totalStaked,
        apy: '13.33%',
        dailyRewards: (stake * 0.1333 / 365).toFixed(2) + ' SLTN',
        gasFees: '$0.00'
    });
});

app.post('/send_transaction', (req, res) => {
    const { from, to, amount } = req.body;
    
    const tx = {
        hash: crypto.randomBytes(32).toString('hex'),
        from,
        to,
        amount,
        fee: 0,
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

app.listen(port, () => {
    console.log(`✅ Consensus node ${nodeId} running on port ${port}`);
    console.log(`   Peers: [${peers.join(', ')}]`);
    console.log(`   Zero gas fees: ACTIVE`);
    console.log(`   APY: 13.33%`);
});

// Auto-increment blocks
setInterval(() => {
    state.blockHeight++;
    if (state.blockHeight % 10 === 0) {
        console.log(`[${nodeId}] Block ${state.blockHeight} - Gas fees: $0.00`);
    }
}, 3000);
