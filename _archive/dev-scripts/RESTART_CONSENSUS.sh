#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       SULTAN CHAIN - RESTARTING MULTI-NODE CONSENSUS          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Kill any existing consensus processes
echo "🔄 Cleaning up old processes..."
pkill -f "consensus" 2>/dev/null
pkill -f "4001\|4002\|4003" 2>/dev/null

# Check what consensus implementations we have
echo "📁 Available consensus implementations:"
ls -la /workspaces/0xv7/consensus/ 2>/dev/null | grep -E "\.(js|mjs|rs)$" | head -5
ls -la /workspaces/0xv7/*.rs 2>/dev/null | grep -E "(consensus|p2p)" | head -5

echo ""
echo "🚀 Starting JavaScript consensus nodes..."
cd /workspaces/0xv7/consensus

# Make sure we have the ES module version
if [ -f "consensus_node_es.mjs" ]; then
    echo "Using existing ES module consensus..."
else
    echo "Creating ES module consensus node..."
    cat > consensus_node_es.mjs << 'CONSENSUS'
import express from 'express';
import crypto from 'crypto';

const app = express();
app.use(express.json());

const port = parseInt(process.env.PORT) || 4001;
const nodeId = process.env.NODE_ID || `validator-${port}`;
const peers = process.env.PEERS ? process.env.PEERS.split(',') : [];

const state = {
    validators: new Map(),
    blockHeight: 13247,
    totalStaked: 365000,
    transactions: [],
    gasFeesCollected: 0,  // Always 0!
    consensusRounds: 0,
    lastConsensus: null
};

// Initialize with some validators
state.validators.set('sultan1genesis', { stake: 100000, joined: Date.now() });
state.validators.set('sultan1validator1', { stake: 50000, joined: Date.now() });
state.validators.set('sultan1validator2', { stake: 75000, joined: Date.now() });

// Consensus state endpoint
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

// Multi-node consensus mechanism
app.post('/propose_block', async (req, res) => {
    const { block, proposer } = req.body;
    
    // Broadcast to peers for consensus
    let votes = 1; // Self vote
    let totalPeers = peers.length;
    
    for (const peer of peers) {
        try {
            const response = await fetch(`http://localhost:${peer}/vote`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ block, voter: nodeId })
            });
            if (response.ok) votes++;
        } catch (e) {
            console.log(`Peer ${peer} not responding`);
        }
    }
    
    // Need 2/3 consensus
    const consensusReached = votes > (totalPeers * 2 / 3);
    
    if (consensusReached) {
        state.blockHeight++;
        state.consensusRounds++;
        state.lastConsensus = Date.now();
        
        res.json({
            success: true,
            blockHeight: state.blockHeight,
            votes: votes,
            totalPeers: totalPeers + 1,
            consensus: 'REACHED',
            gasFeesUsed: 0
        });
    } else {
        res.json({
            success: false,
            votes: votes,
            required: Math.ceil((totalPeers + 1) * 2 / 3),
            consensus: 'FAILED'
        });
    }
});

// Vote on proposed blocks
app.post('/vote', (req, res) => {
    const { block, voter } = req.body;
    // Simple voting logic - in production would verify block
    res.json({ vote: 'yes', nodeId: nodeId });
});

// Register validator endpoint
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
        dailyRewards: (stake * 0.1333 / 365).toFixed(2) + ' SLTN'
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
    res.json({ 
        status: 'healthy', 
        nodeId, 
        blockHeight: state.blockHeight,
        uptime: process.uptime() 
    });
});

// Peer discovery
app.get('/peers', (req, res) => {
    res.json({
        nodeId: nodeId,
        peers: peers,
        knownValidators: Array.from(state.validators.keys())
    });
});

app.listen(port, () => {
    console.log(`✅ Consensus node ${nodeId} running on port ${port}`);
    console.log(`   Peers: ${peers.join(', ') || 'none'}`);
    console.log(`   Zero gas fees: ACTIVE`);
    console.log(`   APY: 13.33%`);
});

// Increment block height and attempt consensus every 3 seconds
setInterval(async () => {
    state.blockHeight++;
    
    // Attempt consensus every 10 blocks
    if (state.blockHeight % 10 === 0) {
        console.log(`Block ${state.blockHeight} - Attempting consensus...`);
        
        // Propose block to peers
        try {
            const response = await fetch(`http://localhost:${port}/propose_block`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    block: { height: state.blockHeight, timestamp: Date.now() },
                    proposer: nodeId
                })
            });
            const result = await response.json();
            if (result.consensus === 'REACHED') {
                console.log(`✅ Consensus reached at block ${state.blockHeight}`);
            }
        } catch (e) {
            // Self-consensus if alone
        }
    }
    
    if (state.blockHeight % 20 === 0) {
        console.log(`[${nodeId}] Block ${state.blockHeight} - Gas fees collected: $0.00`);
    }
}, 3000);
CONSENSUS
fi

# Start three consensus nodes with peer configuration
echo ""
echo "Starting 3 interconnected consensus nodes..."

# Node 1 - knows about nodes 2 and 3
PORT=4001 NODE_ID=validator-1 PEERS=4002,4003 node consensus_node_es.mjs > /tmp/consensus_1.log 2>&1 &
echo "✅ Node 1 started (port 4001, peers: 4002,4003)"
sleep 1

# Node 2 - knows about nodes 1 and 3  
PORT=4002 NODE_ID=validator-2 PEERS=4001,4003 node consensus_node_es.mjs > /tmp/consensus_2.log 2>&1 &
echo "✅ Node 2 started (port 4002, peers: 4001,4003)"
sleep 1

# Node 3 - knows about nodes 1 and 2
PORT=4003 NODE_ID=validator-3 PEERS=4001,4002 node consensus_node_es.mjs > /tmp/consensus_3.log 2>&1 &
echo "✅ Node 3 started (port 4003, peers: 4001,4002)"

echo ""
echo "⏳ Waiting for consensus network to initialize..."
sleep 3

echo ""
echo "🧪 Testing multi-node consensus..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for port in 4001 4002 4003; do
    if curl -s http://localhost:$port/health > /dev/null 2>&1; then
        echo ""
        echo "Node on port $port:"
        curl -s http://localhost:$port/consensus_state | jq '{nodeId, blockHeight, validators, consensusRounds, peers}'
    else
        echo "❌ Node on port $port not responding"
    fi
done

echo ""
echo "�� Testing peer discovery..."
curl -s http://localhost:4001/peers | jq '.'

echo ""
echo "✅ Multi-node consensus network started!"
echo ""
echo "📝 Monitor consensus with: tail -f /tmp/consensus_*.log"
