#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - WEEK 2: P2P NETWORKING                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/node

echo "🌐 Adding P2P Networking Layer..."

# Add libp2p to dependencies
cat >> Cargo.toml << 'TOML'
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "gossipsub", "identify", "kad"] }
TOML

# Create P2P networking module
cat > src/p2p.rs << 'RUST'
use libp2p::{
    identity,
    PeerId,
    Swarm,
    SwarmBuilder,
    gossipsub::{self, Gossipsub, GossipsubEvent, MessageAuthenticity},
    kad::{Kademlia, KademliaEvent, store::MemoryStore},
    noise,
    tcp,
    yamux,
    NetworkBehaviour,
};
use anyhow::Result;
use std::collections::HashSet;

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
        
        println!("🆔 Node PeerId: {}", peer_id);
        
        // Configure gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");
            
        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;
        
        // Configure Kademlia DHT
        let store = MemoryStore::new(peer_id);
        let kademlia = Kademlia::new(peer_id, store);
        
        // Create behaviour
        let behaviour = SultanNetworkBehaviour {
            gossipsub,
            kademlia,
        };
        
        // Build swarm
        let swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .build();
            
        Ok(P2PNetwork { swarm, peer_id })
    }
    
    pub async fn start(&mut self, port: u16) -> Result<()> {
        // Listen on TCP
        let addr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
        self.swarm.listen_on(addr)?;
        
        println!("🌐 P2P listening on port {}", port);
        Ok(())
    }
    
    pub async fn connect_peer(&mut self, multiaddr: String) -> Result<()> {
        let addr = multiaddr.parse()?;
        self.swarm.dial(addr)?;
        println!("📡 Connecting to peer: {}", multiaddr);
        Ok(())
    }
    
    pub async fn broadcast_block(&mut self, block_data: Vec<u8>) -> Result<()> {
        let topic = gossipsub::IdentTopic::new("sultan-blocks");
        self.swarm.behaviour_mut().gossipsub.publish(topic, block_data)?;
        Ok(())
    }
}
RUST

# Add to lib.rs
echo "pub mod p2p;" >> src/lib.rs

echo "✅ P2P Networking module added!"
echo ""
echo "📋 P2P Features Implemented:"
echo "  • Peer discovery via Kademlia DHT"
echo "  • Block propagation via Gossipsub"
echo "  • Secure connections with Noise protocol"
echo "  • Multiplexing with Yamux"

