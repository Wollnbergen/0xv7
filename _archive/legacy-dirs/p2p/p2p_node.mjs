import net from 'net';
import crypto from 'crypto';

const port = parseInt(process.env.P2P_PORT) || 5001;
const nodeId = process.env.P2P_NODE_ID || `p2p-node-${port}`;
const peers = process.env.P2P_PEERS ? process.env.P2P_PEERS.split(',').map(p => parseInt(p)) : [];

const connections = new Map();
const knownPeers = new Set(peers);

// Create P2P server
const server = net.createServer((socket) => {
    const peerId = crypto.randomBytes(16).toString('hex');
    connections.set(peerId, socket);
    
    console.log(`[${nodeId}] New peer connected: ${peerId}`);
    
    socket.on('data', (data) => {
        const message = data.toString();
        console.log(`[${nodeId}] Received: ${message}`);
        
        // Broadcast to other peers
        connections.forEach((peerSocket, id) => {
            if (id !== peerId) {
                peerSocket.write(`Broadcast from ${nodeId}: ${message}`);
            }
        });
    });
    
    socket.on('end', () => {
        connections.delete(peerId);
        console.log(`[${nodeId}] Peer disconnected: ${peerId}`);
    });
});

server.listen(port, () => {
    console.log(`✅ P2P node ${nodeId} listening on port ${port}`);
    console.log(`   Known peers: [${peers.join(', ')}]`);
    connectToPeers();
});

// Connect to known peers
function connectToPeers() {
    peers.forEach(peerPort => {
        const client = new net.Socket();
        client.connect(peerPort, 'localhost', () => {
            console.log(`[${nodeId}] Connected to peer on port ${peerPort}`);
            client.write(`Hello from ${nodeId}`);
        });
        
        client.on('error', () => {
            console.log(`[${nodeId}] Could not connect to peer on port ${peerPort}`);
        });
    });
}

// Send periodic heartbeat
setInterval(() => {
    connections.forEach((socket) => {
        socket.write(`Heartbeat from ${nodeId} - Zero gas fees active!`);
    });
}, 10000);
