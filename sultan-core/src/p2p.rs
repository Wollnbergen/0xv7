//! Sultan P2P Network Layer
//!
//! Production-grade P2P networking using libp2p with:
//! - Gossipsub for block and transaction propagation
//! - Kademlia DHT for peer discovery
//! - Noise protocol for encryption
//! - Yamux for multiplexing

use anyhow::{Result, Context};
use futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode},
    identity::Keypair,
    kad::{self, store::MemoryStore},
    noise, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, Multiaddr, PeerId, Swarm, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error, debug};

/// Topics for gossipsub messaging
pub const BLOCK_TOPIC: &str = "sultan/blocks/1.0.0";
pub const TX_TOPIC: &str = "sultan/transactions/1.0.0";
pub const VALIDATOR_TOPIC: &str = "sultan/validators/1.0.0";
pub const CONSENSUS_TOPIC: &str = "sultan/consensus/1.0.0";

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// New block proposal
    BlockProposal {
        height: u64,
        proposer: String,
        block_hash: String,
        block_data: Vec<u8>,
    },
    /// Vote on a block
    BlockVote {
        height: u64,
        block_hash: String,
        voter: String,
        approve: bool,
        signature: Vec<u8>,
    },
    /// New transaction
    Transaction {
        tx_hash: String,
        tx_data: Vec<u8>,
    },
    /// Validator announcement
    ValidatorAnnounce {
        address: String,
        stake: u64,
        peer_id: String,
    },
    /// Request block sync
    SyncRequest {
        from_height: u64,
        to_height: u64,
    },
    /// Block sync response
    SyncResponse {
        blocks: Vec<Vec<u8>>,
    },
}

/// Combined network behaviour
#[derive(NetworkBehaviour)]
pub struct SultanBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
}

/// P2P Network implementation for Sultan Chain
pub struct P2PNetwork {
    local_key: Keypair,
    peer_id: PeerId,
    connected_peers: Arc<RwLock<HashSet<PeerId>>>,
    known_validators: Arc<RwLock<HashSet<String>>>,
    message_tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
    message_rx: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    is_running: Arc<RwLock<bool>>,
    listen_addr: Option<Multiaddr>,
    bootstrap_peers: Vec<Multiaddr>,
}

impl P2PNetwork {
    /// Create new P2P network instance
    pub fn new() -> Result<Self> {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        info!("🔐 Node PeerId: {}", peer_id);
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        Ok(P2PNetwork {
            local_key,
            peer_id,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            known_validators: Arc::new(RwLock::new(HashSet::new())),
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            is_running: Arc::new(RwLock::new(false)),
            listen_addr: None,
            bootstrap_peers: Vec::new(),
        })
    }

    /// Create with specific keypair (for persistent identity)
    pub fn with_keypair(keypair: Keypair) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());
        info!("🔐 Node PeerId: {}", peer_id);
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        Ok(P2PNetwork {
            local_key: keypair,
            peer_id,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            known_validators: Arc::new(RwLock::new(HashSet::new())),
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            is_running: Arc::new(RwLock::new(false)),
            listen_addr: None,
            bootstrap_peers: Vec::new(),
        })
    }

    /// Set bootstrap peers for network discovery
    pub fn set_bootstrap_peers(&mut self, peers: Vec<String>) -> Result<()> {
        self.bootstrap_peers = peers
            .into_iter()
            .filter_map(|p| p.parse::<Multiaddr>().ok())
            .collect();
        
        info!("📋 Bootstrap peers configured: {:?}", self.bootstrap_peers);
        Ok(())
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Take the message receiver (can only be called once)
    pub fn take_message_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<NetworkMessage>> {
        self.message_rx.take()
    }

    /// Get message sender for broadcasting
    pub fn message_sender(&self) -> Option<mpsc::UnboundedSender<NetworkMessage>> {
        self.message_tx.clone()
    }

    /// Build the libp2p swarm
    fn build_swarm(&self) -> Result<Swarm<SultanBehaviour>> {
        // Configure gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|msg| {
                // Use hash of data as message id to deduplicate
                let hash = sha2::Sha256::digest(&msg.data);
                gossipsub::MessageId::from(hash.to_vec())
            })
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(self.local_key.clone()),
            gossipsub_config,
        ).map_err(|e| anyhow::anyhow!("Gossipsub error: {}", e))?;

        // Configure Kademlia DHT
        let store = MemoryStore::new(self.peer_id);
        let kademlia = kad::Behaviour::new(self.peer_id, store);

        let behaviour = SultanBehaviour { gossipsub, kademlia };

        let swarm = libp2p::SwarmBuilder::with_existing_identity(self.local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| Ok(behaviour))?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        Ok(swarm)
    }

    /// Start listening and connect to network
    pub async fn start(&mut self, listen_addr: &str) -> Result<()> {
        let addr: Multiaddr = listen_addr.parse()
            .context("Invalid listen address")?;
        
        self.listen_addr = Some(addr.clone());
        *self.is_running.write().await = true;

        let mut swarm = self.build_swarm()?;

        // Subscribe to topics
        let block_topic = IdentTopic::new(BLOCK_TOPIC);
        let tx_topic = IdentTopic::new(TX_TOPIC);
        let validator_topic = IdentTopic::new(VALIDATOR_TOPIC);
        let consensus_topic = IdentTopic::new(CONSENSUS_TOPIC);

        swarm.behaviour_mut().gossipsub.subscribe(&block_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&tx_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&validator_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&consensus_topic)?;

        // Start listening
        swarm.listen_on(addr.clone())?;
        info!("🌐 P2P listening on {}", addr);

        // Connect to bootstrap peers
        for peer_addr in &self.bootstrap_peers {
            info!("🔗 Connecting to bootstrap peer: {}", peer_addr);
            if let Err(e) = swarm.dial(peer_addr.clone()) {
                warn!("Failed to dial bootstrap peer {}: {}", peer_addr, e);
            }
        }

        // Clone Arc references for the event loop
        let connected_peers = self.connected_peers.clone();
        let is_running = self.is_running.clone();
        let message_tx = self.message_tx.clone();

        // Spawn the swarm event loop
        tokio::spawn(async move {
            loop {
                if !*is_running.read().await {
                    info!("🛑 P2P network stopping");
                    break;
                }

                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("📡 Listening on {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("🤝 Connected to peer: {}", peer_id);
                        connected_peers.write().await.insert(peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("👋 Disconnected from peer: {}", peer_id);
                        connected_peers.write().await.remove(&peer_id);
                    }
                    SwarmEvent::Behaviour(event) => {
                        match event {
                            SultanBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                                // Parse and forward message
                                if let Ok(network_msg) = bincode::deserialize::<NetworkMessage>(&message.data) {
                                    debug!("📨 Received message: {:?}", network_msg);
                                    if let Some(tx) = &message_tx {
                                        let _ = tx.send(network_msg);
                                    }
                                }
                            }
                            SultanBehaviourEvent::Kademlia(kad::Event::RoutingUpdated { peer, .. }) => {
                                debug!("📋 Kademlia routing updated for peer: {}", peer);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Stop the P2P network
    pub async fn stop(&mut self) -> Result<()> {
        *self.is_running.write().await = false;
        info!("🛑 P2P network stopped");
        Ok(())
    }

    /// Broadcast a block proposal to the network
    pub async fn broadcast_block(&self, height: u64, proposer: &str, block_hash: &str, block_data: Vec<u8>) -> Result<()> {
        let msg = NetworkMessage::BlockProposal {
            height,
            proposer: proposer.to_string(),
            block_hash: block_hash.to_string(),
            block_data,
        };
        
        self.broadcast_message(BLOCK_TOPIC, msg).await
    }

    /// Broadcast a block vote
    pub async fn broadcast_vote(&self, height: u64, block_hash: &str, voter: &str, approve: bool, signature: Vec<u8>) -> Result<()> {
        let msg = NetworkMessage::BlockVote {
            height,
            block_hash: block_hash.to_string(),
            voter: voter.to_string(),
            approve,
            signature,
        };
        
        self.broadcast_message(CONSENSUS_TOPIC, msg).await
    }

    /// Broadcast a transaction
    pub async fn broadcast_transaction(&self, tx_hash: &str, tx_data: Vec<u8>) -> Result<()> {
        let msg = NetworkMessage::Transaction {
            tx_hash: tx_hash.to_string(),
            tx_data,
        };
        
        self.broadcast_message(TX_TOPIC, msg).await
    }

    /// Announce this validator to the network
    pub async fn announce_validator(&self, address: &str, stake: u64) -> Result<()> {
        let msg = NetworkMessage::ValidatorAnnounce {
            address: address.to_string(),
            stake,
            peer_id: self.peer_id.to_string(),
        };
        
        self.broadcast_message(VALIDATOR_TOPIC, msg).await
    }

    /// Internal: broadcast message to a topic
    async fn broadcast_message(&self, topic: &str, msg: NetworkMessage) -> Result<()> {
        let data = bincode::serialize(&msg)?;
        debug!("📢 Broadcasting to {}: {} bytes", topic, data.len());
        
        // The actual publishing happens through the swarm
        // For now, we send through the channel and let the integration handle it
        if let Some(tx) = &self.message_tx {
            tx.send(msg)?;
        }
        
        Ok(())
    }

    /// Get connected peer count
    pub async fn peer_count(&self) -> usize {
        self.connected_peers.read().await.len()
    }

    /// Get list of connected peers
    pub async fn connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers.read().await.iter().cloned().collect()
    }

    /// Check if running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Register a known validator
    pub async fn register_validator(&self, address: String) {
        self.known_validators.write().await.insert(address);
    }

    /// Get known validators count
    pub async fn known_validator_count(&self) -> usize {
        self.known_validators.read().await.len()
    }
}
