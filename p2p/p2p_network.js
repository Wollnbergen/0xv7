const net = require('net');
const EventEmitter = require('events');
const crypto = require('crypto');

class P2PNode extends EventEmitter {
    constructor(port, peers = []) {
        super();
        this.port = port;
        this.peers = new Map();
        this.connections = new Map();
        this.nodeId = crypto.randomBytes(16).toString('hex');
        
        // Message handlers
        this.handlers = new Map();
        this.setupDefaultHandlers();
        
        // Start server
        this.server = net.createServer((socket) => this.handleConnection(socket));
    }
    
    setupDefaultHandlers() {
        // Ping/Pong for keepalive
        this.on('ping', (peer, data) => {
            this.send(peer, 'pong', { timestamp: Date.now() });
        });
        
        // Peer discovery
        this.on('get_peers', (peer, data) => {
            const peerList = Array.from(this.peers.values());
            this.send(peer, 'peers', { peers: peerList });
        });
        
        // Block propagation
        this.on('new_block', (peer, block) => {
            console.log(`📦 Received block ${block.height} from ${peer}`);
            this.broadcast('new_block', block, peer); // Propagate to others
            this.emit('block_received', block);
        });
        
        // Transaction propagation
        this.on('new_tx', (peer, tx) => {
            console.log(`💸 Received transaction from ${peer}`);
            this.broadcast('new_tx', tx, peer);
            this.emit('tx_received', tx);
        });
        
        // Validator updates
        this.on('validator_update', (peer, validator) => {
            console.log(`👤 Validator update from ${peer}`);
            this.broadcast('validator_update', validator, peer);
            this.emit('validator_joined', validator);
        });
    }
    
    handleConnection(socket) {
        const peerId = `${socket.remoteAddress}:${socket.remotePort}`;
        
        console.log(`🔗 New connection from ${peerId}`);
        this.connections.set(peerId, socket);
        
        socket.on('data', (data) => {
            try {
                const messages = data.toString().split('\n').filter(m => m);
                for (const message of messages) {
                    const msg = JSON.parse(message);
                    this.handleMessage(peerId, msg);
                }
            } catch (err) {
                console.error('Failed to parse message:', err);
            }
        });
        
        socket.on('close', () => {
            console.log(`🔌 Disconnected: ${peerId}`);
            this.connections.delete(peerId);
            this.peers.delete(peerId);
        });
        
        socket.on('error', (err) => {
            console.error(`Socket error from ${peerId}:`, err.message);
        });
        
        // Send handshake
        this.send(peerId, 'handshake', {
            nodeId: this.nodeId,
            version: '1.0.0',
            network: 'sultan-mainnet'
        });
    }
    
    handleMessage(peerId, msg) {
        const { type, data } = msg;
        
        if (type === 'handshake') {
            this.peers.set(peerId, data);
            console.log(`🤝 Handshake complete with ${peerId}`);
        }
        
        // Emit for handlers
        this.emit(type, peerId, data);
    }
    
    send(peerId, type, data) {
        const socket = this.connections.get(peerId);
        if (socket && !socket.destroyed) {
            const message = JSON.stringify({ type, data }) + '\n';
            socket.write(message);
        }
    }
    
    broadcast(type, data, excludePeer = null) {
        for (const [peerId, socket] of this.connections) {
            if (peerId !== excludePeer) {
                this.send(peerId, type, data);
            }
        }
    }
    
    connectToPeer(host, port) {
        const socket = net.createConnection(port, host, () => {
            const peerId = `${host}:${port}`;
            console.log(`✅ Connected to peer ${peerId}`);
            this.connections.set(peerId, socket);
            
            // Setup handlers
            socket.on('data', (data) => {
                try {
                    const messages = data.toString().split('\n').filter(m => m);
                    for (const message of messages) {
                        const msg = JSON.parse(message);
                        this.handleMessage(peerId, msg);
                    }
                } catch (err) {
                    console.error('Failed to parse message:', err);
                }
            });
            
            socket.on('close', () => {
                this.connections.delete(peerId);
                this.peers.delete(peerId);
            });
            
            // Send handshake
            this.send(peerId, 'handshake', {
                nodeId: this.nodeId,
                version: '1.0.0',
                network: 'sultan-mainnet'
            });
        });
        
        socket.on('error', (err) => {
            console.error(`Failed to connect to ${host}:${port}:`, err.message);
        });
    }
    
    start() {
        this.server.listen(this.port, () => {
            console.log(`🌐 P2P node started on port ${this.port}`);
            console.log(`📍 Node ID: ${this.nodeId}`);
        });
    }
    
    getStats() {
        return {
            nodeId: this.nodeId,
            port: this.port,
            peers: this.peers.size,
            connections: this.connections.size
        };
    }
}

// Create 3-node P2P network
const node1 = new P2PNode(5001);
const node2 = new P2PNode(5002);
const node3 = new P2PNode(5003);

// Start nodes
node1.start();
node2.start();
node3.start();

// Connect nodes to each other
setTimeout(() => {
    node2.connectToPeer('localhost', 5001);
    node3.connectToPeer('localhost', 5001);
    node3.connectToPeer('localhost', 5002);
}, 1000);

// Test block propagation
setTimeout(() => {
    console.log('\n📢 Testing block propagation...');
    node1.broadcast('new_block', {
        height: 1,
        hash: 'test_block_001',
        validator: 'validator1',
        transactions: []
    });
}, 2000);

console.log('✅ P2P Network initialized!');
console.log('   • Node 1: localhost:5001');
console.log('   • Node 2: localhost:5002');
console.log('   • Node 3: localhost:5003');
