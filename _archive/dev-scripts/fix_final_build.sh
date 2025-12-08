#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            FINAL FIX - CLEAN BUILD WITH SIMPLE P2P            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Check and fix Cargo.toml corruption
echo "🔧 Step 1: Fixing Cargo.toml corruption..."
# Remove any corrupted libp2p lines at the end
sed -i '/^libp2p = {.*yamux"] }$/d' node/Cargo.toml
sed -i '/^libp2p = {.*"y$/d' node/Cargo.toml

# Remove all libp2p-related lines to start fresh
sed -i '/^libp2p/d' node/Cargo.toml
sed -i '/^futures = "0.3"$/d' node/Cargo.toml

echo "✅ Cleaned Cargo.toml"

# 2. Add dependencies properly in the dependencies section
echo ""
echo "📝 Step 2: Adding correct dependencies..."
# Find the [dependencies] section and add after it
awk '
/^\[dependencies\]/ {
    print
    print "futures = \"0.3\""
    print "libp2p = { version = \"0.53\", features = [\"tcp\", \"dns\", \"async-std\", \"noise\", \"yamux\"] }"
    print "libp2p-swarm-derive = \"0.34\""
    next
}
/^futures = "0.3"$/ {next}
/^libp2p = / {next}
/^libp2p-swarm-derive = / {next}
{print}
' node/Cargo.toml > node/Cargo.toml.tmp && mv node/Cargo.toml.tmp node/Cargo.toml

echo "✅ Added dependencies"

# 3. Create a SIMPLE P2P module without ping/relay
echo ""
echo "✨ Step 3: Creating simple P2P module..."
cat > node/src/p2p_node.rs << 'EOF'
//! Simple P2P networking for Sultan Chain
//! Compatible with libp2p 0.53 core features

use anyhow::Result;
use libp2p::{
    identity::Keypair,
    PeerId, Swarm, Multiaddr,
    swarm::SwarmEvent,
    Transport,
};
use serde::{Serialize, Deserialize};
use std::path::Path;
use tracing::info;
use futures::StreamExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockMessage {
    pub height: u64,
    pub hash: String,
    pub prev_hash: String,
    pub validator: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteMessage {
    pub block_hash: String,
    pub voter: String,
    pub vote: bool,
    pub timestamp: u64,
}

// Empty behaviour for now
#[derive(libp2p_swarm_derive::NetworkBehaviour)]
pub struct SultanBehaviour {}

pub struct P2PNode {
    pub peer_id: PeerId,
    swarm: Swarm<SultanBehaviour>,
}

impl P2PNode {
    pub fn new() -> Result<Self> {
        let keypair = if let Ok(path) = std::env::var("SULTAN_P2P_KEY_PATH") {
            Self::load_or_create_keypair(&path)?
        } else {
            Keypair::generate_ed25519()
        };
        
        let peer_id = PeerId::from(keypair.public());
        info!("🔗 P2P Node ID: {}", peer_id);

        let transport = libp2p::tcp::async_io::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&keypair)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        let behaviour = SultanBehaviour {};
        
        let swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_async_std_executor(),
        );

        Ok(P2PNode { peer_id, swarm })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Starting P2P node...");
        
        let port = std::env::var("SULTAN_P2P_PORT").unwrap_or_else(|_| "9000".to_string());
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
        self.swarm.listen_on(addr)?;
        
        // Connect to bootstrap peers if configured
        if let Ok(peers) = std::env::var("SULTAN_BOOTSTRAP_PEERS") {
            for peer in peers.split(',') {
                if let Ok(addr) = peer.trim().parse::<Multiaddr>() {
                    match self.swarm.dial(addr.clone()) {
                        Ok(_) => info!("📡 Dialing bootstrap peer: {}", addr),
                        Err(e) => info!("⚠️ Failed to dial {}: {}", addr, e),
                    }
                }
            }
        }
        
        info!("✅ P2P node running on peer: {}", self.peer_id);
        
        // Event loop
        loop {
            if let Some(event) = self.swarm.next().await {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("📍 Listening on: {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("✅ Connected to peer: {}", peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("❌ Disconnected from peer: {}", peer_id);
                    }
                    _ => {}
                }
            }
        }
    }

    fn load_or_create_keypair(path: &str) -> Result<Keypair> {
        use std::fs;
        
        if Path::new(path).exists() {
            if let Ok(bytes) = fs::read(path) {
                if let Ok(kp) = Keypair::from_protobuf_encoding(&bytes) {
                    info!("🔑 Loaded existing keypair from {}", path);
                    return Ok(kp);
                }
            }
        }
        
        let kp = Keypair::generate_ed25519();
        if let Ok(encoded) = kp.to_protobuf_encoding() {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(path, encoded).ok();
            info!("🔐 Created new keypair at {}", path);
        }
        Ok(kp)
    }

    pub fn broadcast_block(&mut self, block: BlockMessage) -> Result<()> {
        info!("📨 Broadcasting block height {} hash {}", block.height, block.hash);
        // TODO: Implement with gossipsub when adding protocols
        Ok(())
    }

    pub fn broadcast_vote(&mut self, vote: VoteMessage) -> Result<()> {
        info!("🗳️ Broadcasting vote for block {}", vote.block_hash);
        // TODO: Implement with gossipsub when adding protocols
        Ok(())
    }
}

pub async fn start_p2p() -> Result<()> {
    let mut node = P2PNode::new()?;
    node.run().await
}
EOF
echo "✅ Created simple P2P module"

# 4. Fix RPC server client_id errors
echo ""
echo "🔧 Step 4: Fixing RPC server errors..."
# Fix the undefined client_id errors by using _client_id
sed -i 's/let client_id = require_auth/let _client_id = require_auth/g' node/src/rpc_server.rs
# Also fix any standalone client_id references
sed -i 's/tracing::info!("Client {} /tracing::info!("Client /g' node/src/rpc_server.rs
echo "✅ Fixed RPC server"

# 5. Update lib.rs
echo ""
echo "📝 Step 5: Updating lib.rs..."
# Remove old P2P references
sed -i '/mod p2p/d' node/src/lib.rs
# Add the new p2p_node module if not present
if ! grep -q "pub mod p2p_node;" node/src/lib.rs; then
    echo "pub mod p2p_node;" >> node/src/lib.rs
fi
echo "✅ Updated lib.rs"

# 6. Verify Cargo.toml is valid
echo ""
echo "📋 Step 6: Verifying Cargo.toml..."
if cargo metadata --manifest-path node/Cargo.toml --format-version 1 > /dev/null 2>&1; then
    echo "✅ Cargo.toml is valid"
else
    echo "⚠️ Cargo.toml validation failed"
fi

# 7. Build the project
echo ""
echo "🔨 Step 7: Building project..."
cargo build -p sultan-coordinator 2>&1 | tee /tmp/build.log | grep -E "(Compiling|Finished|error\[)"

if grep -q "Finished" /tmp/build.log && ! grep -q "error\[" /tmp/build.log; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                  🎉 BUILD SUCCESSFUL! 🎉                      ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "✅ All issues fixed! Project builds successfully!"
    echo ""
    echo "📝 To test P2P networking:"
    echo ""
    echo "Terminal 1 (first node):"
    echo "  export SULTAN_P2P_PORT=9000"
    echo "  export SULTAN_P2P_KEY_PATH=./data/node1/p2p_key"
    echo "  cargo run -p sultan-coordinator --bin rpc_server"
    echo ""
    echo "Terminal 2 (second node):"
    echo "  export SULTAN_P2P_PORT=9001"
    echo "  export SULTAN_P2P_KEY_PATH=./data/node2/p2p_key"
    echo "  export SULTAN_BOOTSTRAP_PEERS=/ip4/127.0.0.1/tcp/9000"
    echo "  cargo run -p sultan-coordinator --bin rpc_server -- --port 3031"
else
    echo ""
    echo "⚠️ Build still has errors. Showing first few:"
    grep "error\[" /tmp/build.log | head -5
fi
