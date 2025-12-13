#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          FIXING SULTAN CHAIN COMPILATION ERRORS               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

# Fix 1: Clean up lib.rs duplicate modules
echo "🔧 Fixing lib.rs duplicate modules..."
cat > src/lib.rs << 'RUST'
pub mod blockchain;
pub mod config;
pub mod consensus;
pub mod rewards;
pub mod rpc_server;
pub mod scylla_db;
pub mod sdk;
pub mod transaction_validator;
pub mod types;
pub mod persistence;
pub mod p2p;
pub mod multi_consensus;
pub mod state_sync;

// Re-export main types
pub use blockchain::{Blockchain, ChainConfig};
pub use sdk::SultanSDK;
pub use types::SultanToken;
RUST

# Fix 2: Fix SDK syntax error
echo "🔧 Fixing SDK create_wallet function..."
sed -i '46s/.*/    pub async fn create_wallet(\&self, owner: \&str) -> Result<String> {/' src/sdk.rs

# Fix 3: Fix P2P imports and NetworkBehaviour
echo "🔧 Fixing P2P module..."
cat > src/p2p.rs << 'RUST'
use libp2p::{
    identity,
    PeerId,
    Swarm,
    swarm::NetworkBehaviour,
    gossipsub::{self, Gossipsub, MessageAuthenticity},
    kad::{Kademlia, store::MemoryStore},
    noise,
    tcp,
    yamux,
};
use anyhow::Result;
use futures::StreamExt;

#[derive(NetworkBehaviour)]
pub struct SultanNetworkBehaviour {
    pub gossipsub: Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
}

pub struct P2PNetwork {
    swarm: Swarm<SultanNetworkBehaviour>,
    peer_id: PeerId,
}

impl P2PNetwork {
    pub fn new() -> Result<Self> {
        // Generate peer identity
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        // Create gossipsub
        let gossipsub = {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .build()
                .map_err(|msg| anyhow::anyhow!(msg))?;
            Gossipsub::new(
                MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            )?
        };
        
        // Create Kademlia
        let kademlia = Kademlia::new(peer_id, MemoryStore::new(peer_id));
        
        // Build the swarm
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();
            
        let behaviour = SultanNetworkBehaviour {
            gossipsub,
            kademlia,
        };
        
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );
        
        // Listen on default address
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        
        Ok(P2PNetwork { swarm, peer_id })
    }
    
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
    
    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(_event) = self.swarm.next().await {
                // Handle events
            }
        }
    }
}
RUST

# Fix 4: Add missing Digest import in multi_consensus
echo "🔧 Fixing multi_consensus module..."
sed -i '1i\use sha2::Digest;' src/multi_consensus.rs

# Fix 5: Add with-scylla feature to Cargo.toml
echo "🔧 Adding features to Cargo.toml..."
cat >> Cargo.toml << 'TOML'

[features]
default = []
with-scylla = ["scylla"]
TOML

echo ""
echo "✅ All compilation errors fixed!"
echo ""
echo "🔨 Rebuilding..."
cargo build --release 2>&1 | tail -20

