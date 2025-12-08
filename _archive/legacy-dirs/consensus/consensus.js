const express = require('express');
const axios = require('axios').default;
const crypto = require('crypto');

class ConsensusNode {
    constructor(nodeId, port, peers = []) {
        this.nodeId = nodeId;
        this.port = port;
        this.peers = peers;
        this.app = express();
        this.app.use(express.json());
        
        this.state = {
            blockHeight: 0,
            validators: new Map(),
            proposedBlocks: [],
            votes: new Map()
        };
        
        this.setupEndpoints();
    }
    
    setupEndpoints() {
        // Receive block proposals
        this.app.post('/propose_block', (req, res) => {
            const { block, proposer, signature } = req.body;
            
            // Verify proposer is validator
            if (this.state.validators.has(proposer)) {
                this.state.proposedBlocks.push({
                    block,
                    proposer,
                    signature,
                    timestamp: Date.now()
                });
                
                // Broadcast to peers
                this.broadcast('/vote_block', { block, nodeId: this.nodeId });
                
                res.json({ status: 'accepted', height: block.height });
            } else {
                res.status(403).json({ error: 'Not a validator' });
            }
        });
        
        // Vote on blocks
        this.app.post('/vote_block', (req, res) => {
            const { block, nodeId } = req.body;
            const blockHash = crypto.createHash('sha256')
                .update(JSON.stringify(block))
                .digest('hex');
            
            if (!this.state.votes.has(blockHash)) {
                this.state.votes.set(blockHash, new Set());
            }
            
            this.state.votes.get(blockHash).add(nodeId);
            
            // Check if we have 2/3 consensus
            const validatorCount = this.state.validators.size || 1;
            const votesNeeded = Math.floor((validatorCount * 2) / 3) + 1;
            const currentVotes = this.state.votes.get(blockHash).size;
            
            if (currentVotes >= votesNeeded) {
                this.finalizeBlock(block);
                res.json({ status: 'finalized', votes: currentVotes });
            } else {
                res.json({ status: 'pending', votes: currentVotes, needed: votesNeeded });
            }
        });
        
        // Register validator
        this.app.post('/register_validator', (req, res) => {
            const { address, stake } = req.body;
            this.state.validators.set(address, { stake, joined: Date.now() });
            
            // Sync with peers
            this.broadcast('/sync_validator', { address, stake });
            
            res.json({ 
                success: true, 
                validators: this.state.validators.size,
                networkStake: Array.from(this.state.validators.values())
                    .reduce((sum, v) => sum + v.stake, 0)
            });
        });
        
        // Get consensus state
        this.app.get('/consensus_state', (req, res) => {
            res.json({
                nodeId: this.nodeId,
                blockHeight: this.state.blockHeight,
                validators: this.state.validators.size,
                proposedBlocks: this.state.proposedBlocks.length,
                peers: this.peers.length
            });
        });
    }
    
    finalizeBlock(block) {
        this.state.blockHeight = block.height;
        console.log(`✅ Block ${block.height} finalized by ${this.nodeId}`);
        
        // Clear old proposals
        this.state.proposedBlocks = [];
        this.state.votes.clear();
    }
    
    async broadcast(endpoint, data) {
        for (const peer of this.peers) {
            try {
                await axios.post(`http://localhost:${peer}${endpoint}`, data);
            } catch (err) {
                console.log(`Failed to reach peer on port ${peer}`);
            }
        }
    }
    
    start() {
        this.app.listen(this.port, () => {
            console.log(`🚀 Consensus node ${this.nodeId} running on port ${this.port}`);
            console.log(`   Connected peers: ${this.peers.join(', ')}`);
        });
    }
}

// Start 3-node consensus network
const node1 = new ConsensusNode('validator-1', 4001, [4002, 4003]);
const node2 = new ConsensusNode('validator-2', 4002, [4001, 4003]);
const node3 = new ConsensusNode('validator-3', 4003, [4001, 4002]);

node1.start();
node2.start();
node3.start();

console.log('✅ Multi-node consensus network started!');
console.log('   • Node 1: http://localhost:4001');
console.log('   • Node 2: http://localhost:4002');
console.log('   • Node 3: http://localhost:4003');
