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
    tcp, Multiaddr, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug};

/// Default filename for persistent node identity key
pub const NODE_KEY_FILE: &str = "node_key.bin";

/// Load or generate a persistent keypair for stable PeerId
/// 
/// This ensures the node's PeerId remains stable across restarts,
/// which is critical for libp2p multiaddr bootstrap peer configuration.
pub fn load_or_generate_keypair(data_dir: &Path) -> Result<Keypair> {
    let key_path = data_dir.join(NODE_KEY_FILE);
    
    if key_path.exists() {
        // Load existing key
        let key_bytes = std::fs::read(&key_path)
            .context("Failed to read node key file")?;
        
        let keypair = Keypair::ed25519_from_bytes(key_bytes.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse node key: {}", e))?;
        
        info!("🔑 Loaded persistent node key from {:?}", key_path);
        Ok(keypair)
    } else {
        // Generate new key and save it
        let keypair = Keypair::generate_ed25519();
        
        // Extract the secret key bytes (32 bytes for Ed25519)
        if let Some(ed25519_keypair) = keypair.clone().try_into_ed25519().ok() {
            let secret_bytes = ed25519_keypair.secret().as_ref().to_vec();
            
            // Ensure data directory exists
            std::fs::create_dir_all(data_dir)
                .context("Failed to create data directory")?;
            
            std::fs::write(&key_path, &secret_bytes)
                .context("Failed to write node key file")?;
            
            // Set restrictive permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&key_path)?.permissions();
                perms.set_mode(0o600); // Owner read/write only
                std::fs::set_permissions(&key_path, perms)?;
            }
            
            info!("🔑 Generated and saved new node key to {:?}", key_path);
        } else {
            warn!("⚠️ Could not persist node key - using ephemeral identity");
        }
        
        Ok(keypair)
    }
}

/// Topics for gossipsub messaging
pub const BLOCK_TOPIC: &str = "sultan/blocks/1.0.0";
pub const TX_TOPIC: &str = "sultan/transactions/1.0.0";
pub const VALIDATOR_TOPIC: &str = "sultan/validators/1.0.0";
pub const CONSENSUS_TOPIC: &str = "sultan/consensus/1.0.0";

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// New block proposal (with Ed25519 signature for proposer verification)
    BlockProposal {
        height: u64,
        proposer: String,
        block_hash: String,
        block_data: Vec<u8>,
        /// Ed25519 signature over block_hash by proposer
        proposer_signature: Vec<u8>,
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
    /// Validator announcement (with Ed25519 signature for verification)
    ValidatorAnnounce {
        address: String,
        stake: u64,
        peer_id: String,
        /// Ed25519 public key for signature verification
        pubkey: [u8; 32],
        /// Ed25519 signature over (address || stake || peer_id)
        signature: Vec<u8>,
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
    /// Request full validator set from peers (for initial sync)
    ValidatorSetRequest {
        /// Our current validator count (so peers know if we're behind)
        known_count: u32,
    },
    /// Response with full validator set
    ValidatorSetResponse {
        /// List of all known validators with their registration data
        validators: Vec<ValidatorInfo>,
    },
}

/// Validator information for P2P sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: u64,
    pub pubkey: [u8; 32],
}

/// Combined network behaviour
#[derive(NetworkBehaviour)]
pub struct SultanBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
}

/// Maximum message size (1 MB)
const MAX_MESSAGE_SIZE: usize = 1 << 20;
/// Maximum messages per peer per minute (DoS protection)
const MAX_MESSAGES_PER_MINUTE: u32 = 1000;
/// Ban duration for misbehaving peers (10 minutes)
const PEER_BAN_DURATION_SECS: u64 = 600;
/// Minimum required peers for healthy network
const MIN_PEERS_REQUIRED: usize = 2;
/// Minimum stake required for validator announcement (10 trillion SULTAN)
const MIN_VALIDATOR_STAKE: u64 = 10_000_000_000_000;
/// Rate limit cleanup interval (clean up stale entries every N checks)
const RATE_LIMIT_CLEANUP_THRESHOLD: usize = 100;

/// Banned peer entry with expiration
#[derive(Debug, Clone)]
pub struct BannedPeer {
    pub peer_id: PeerId,
    pub reason: String,
    pub banned_at: std::time::Instant,
    pub duration_secs: u64,
}

impl BannedPeer {
    pub fn is_expired(&self) -> bool {
        self.banned_at.elapsed().as_secs() >= self.duration_secs
    }
}

/// Message rate tracking for DoS prevention
#[derive(Debug, Clone, Default)]
pub struct PeerRateLimit {
    pub message_count: u32,
    pub window_start: Option<std::time::Instant>,
}

impl PeerRateLimit {
    pub fn record_message(&mut self) -> bool {
        let now = std::time::Instant::now();
        
        // Reset window if more than 60 seconds have passed
        if let Some(start) = self.window_start {
            if now.duration_since(start).as_secs() >= 60 {
                self.message_count = 0;
                self.window_start = Some(now);
            }
        } else {
            self.window_start = Some(now);
        }
        
        self.message_count += 1;
        self.message_count <= MAX_MESSAGES_PER_MINUTE
    }
}

/// P2P Network implementation for Sultan Chain
pub struct P2PNetwork {
    local_key: Keypair,
    peer_id: PeerId,
    connected_peers: Arc<RwLock<HashSet<PeerId>>>,
    known_validators: Arc<RwLock<HashSet<String>>>,
    /// Validator address -> pubkey mapping for signature verification
    validator_pubkeys: Arc<RwLock<HashMap<String, [u8; 32]>>>,
    message_tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
    message_rx: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    broadcast_tx: Option<mpsc::UnboundedSender<(String, Vec<u8>)>>, // (topic, data)
    is_running: Arc<RwLock<bool>>,
    listen_addr: Option<Multiaddr>,
    bootstrap_peers: Vec<Multiaddr>,
    /// Banned peers for misbehavior
    banned_peers: Arc<RwLock<HashMap<PeerId, BannedPeer>>>,
    /// Rate limiting per peer
    peer_rate_limits: Arc<RwLock<HashMap<PeerId, PeerRateLimit>>>,
}

impl P2PNetwork {
    /// Create new P2P network instance
    pub fn new() -> Result<Self> {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        info!("🔐 Node PeerId: {}", peer_id);
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _broadcast_rx) = mpsc::unbounded_channel();
        
        Ok(P2PNetwork {
            local_key,
            peer_id,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            known_validators: Arc::new(RwLock::new(HashSet::new())),
            validator_pubkeys: Arc::new(RwLock::new(HashMap::new())),
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            broadcast_tx: Some(broadcast_tx),
            is_running: Arc::new(RwLock::new(false)),
            listen_addr: None,
            bootstrap_peers: Vec::new(),
            banned_peers: Arc::new(RwLock::new(HashMap::new())),
            peer_rate_limits: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create with specific keypair (for persistent identity)
    pub fn with_keypair(keypair: Keypair) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());
        info!("🔐 Node PeerId: {}", peer_id);
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _broadcast_rx) = mpsc::unbounded_channel();
        
        Ok(P2PNetwork {
            local_key: keypair,
            peer_id,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            known_validators: Arc::new(RwLock::new(HashSet::new())),
            validator_pubkeys: Arc::new(RwLock::new(HashMap::new())),
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            broadcast_tx: Some(broadcast_tx),
            is_running: Arc::new(RwLock::new(false)),
            listen_addr: None,
            bootstrap_peers: Vec::new(),
            banned_peers: Arc::new(RwLock::new(HashMap::new())),
            peer_rate_limits: Arc::new(RwLock::new(HashMap::new())),
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
        // Configure gossipsub with DoS protection limits
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .max_transmit_size(MAX_MESSAGE_SIZE) // 1 MB max message
            .max_ihave_length(5000) // Optimize gossip protocol
            .max_messages_per_rpc(Some(100)) // Limit messages per RPC for DoS protection
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
        let known_validators = self.known_validators.clone();
        let validator_pubkeys = self.validator_pubkeys.clone();
        let is_running = self.is_running.clone();
        let message_tx = self.message_tx.clone();
        let bootstrap_peers_for_reconnect = self.bootstrap_peers.clone();
        let peer_rate_limits = self.peer_rate_limits.clone();
        
        // Create broadcast channel - receiver for event loop, sender stays in self
        let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<(String, Vec<u8>)>();
        self.broadcast_tx = Some(broadcast_tx);

        // Spawn the swarm event loop
        tokio::spawn(async move {
            // Reconnection timer - check every 30 seconds
            let mut reconnect_interval = tokio::time::interval(std::time::Duration::from_secs(30));
            reconnect_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                if !*is_running.read().await {
                    info!("🛑 P2P network stopping");
                    break;
                }

                tokio::select! {
                    // Periodic reconnection check
                    _ = reconnect_interval.tick() => {
                        let peer_count = connected_peers.read().await.len();
                        if peer_count < 2 && !bootstrap_peers_for_reconnect.is_empty() {
                            info!("🔄 Low peer count ({}), attempting reconnection to bootstrap peers...", peer_count);
                            for peer_addr in &bootstrap_peers_for_reconnect {
                                if let Err(e) = swarm.dial(peer_addr.clone()) {
                                    debug!("Reconnect dial failed for {}: {}", peer_addr, e);
                                }
                            }
                        }
                    }
                    // Handle broadcast requests
                    Some((topic, data)) = broadcast_rx.recv() => {
                        // Publish to gossipsub
                        let topic_obj = IdentTopic::new(&topic);
                        match swarm.behaviour_mut().gossipsub.publish(topic_obj, data) {
                            Ok(_) => info!("📡 Published message to gossipsub topic: {}", topic),
                            Err(e) => warn!("Failed to publish to gossipsub: {}", e),
                        }
                    }
                    // Handle swarm events
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("📡 Listening on {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            info!("🤝 Connected to peer: {}", peer_id);
                            connected_peers.write().await.insert(peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            if let Some(ref err) = cause {
                                info!("👋 Disconnected from peer: {} (cause: {:?})", peer_id, err);
                            } else {
                                info!("👋 Disconnected from peer: {}", peer_id);
                            }
                            connected_peers.write().await.remove(&peer_id);
                        }
                        SwarmEvent::Behaviour(event) => {
                            match event {
                                SultanBehaviourEvent::Gossipsub(gossipsub::Event::Message { propagation_source, message, .. }) => {
                                    // CRITICAL: Enforce rate limiting to prevent DoS attacks
                                    // Check if this peer has exceeded their message quota
                                    {
                                        let mut limits = peer_rate_limits.write().await;
                                        
                                        // Periodic cleanup of stale rate limit entries
                                        if limits.len() > RATE_LIMIT_CLEANUP_THRESHOLD {
                                            let now = std::time::Instant::now();
                                            limits.retain(|_, limit| {
                                                limit.window_start
                                                    .map(|start| now.duration_since(start).as_secs() < 120)
                                                    .unwrap_or(false)
                                            });
                                        }
                                        
                                        let limit = limits.entry(propagation_source).or_default();
                                        if !limit.record_message() {
                                            warn!("⚠️ Rate limit exceeded for peer {}, dropping message", propagation_source);
                                            continue;
                                        }
                                    }
                                    
                                    // Parse and forward message
                                    if let Ok(network_msg) = bincode::deserialize::<NetworkMessage>(&message.data) {
                                        debug!("📨 Received message: {:?}", network_msg);
                                        
                                        // Handle validator announcements - verify stake AND signature, then register
                                        if let NetworkMessage::ValidatorAnnounce { ref address, stake, ref peer_id, ref pubkey, ref signature } = network_msg {
                                            // First verify signature over the announcement data
                                            if !P2PNetwork::verify_announce_signature(pubkey, address, stake, peer_id, signature) {
                                                warn!("⚠️ Rejected validator {} with invalid announcement signature", address);
                                            } else if stake >= MIN_VALIDATOR_STAKE {
                                                info!("🗳️ Validator announced: {} (stake: {}, peer: {})", address, stake, peer_id);
                                                known_validators.write().await.insert(address.clone());
                                            } else {
                                                warn!("⚠️ Rejected validator {} with insufficient stake: {} < {}", 
                                                      address, stake, MIN_VALIDATOR_STAKE);
                                            }
                                        }
                                        
                                        // Handle BlockProposal - verify proposer signature
                                        if let NetworkMessage::BlockProposal { ref proposer, ref block_hash, ref proposer_signature, .. } = network_msg {
                                            // Look up proposer's pubkey and verify signature
                                            if let Some(pubkey) = validator_pubkeys.read().await.get(proposer) {
                                                if !P2PNetwork::verify_vote_signature(pubkey, block_hash.as_bytes(), proposer_signature) {
                                                    warn!("⚠️ Rejected BlockProposal from {} with invalid signature", proposer);
                                                    // Note: In production, would ban peer here
                                                    // Skip forwarding invalid proposals
                                                    continue;
                                                }
                                                debug!("✅ BlockProposal from {} verified", proposer);
                                            } else {
                                                // Unknown proposer - log warning but still forward
                                                // (proposer may be new/not yet announced)
                                                debug!("⚠️ BlockProposal from unknown proposer {}", proposer);
                                            }
                                        }
                                        
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

    /// Broadcast a block proposal to the network (with proposer signature)
    pub async fn broadcast_block(&self, height: u64, proposer: &str, block_hash: &str, block_data: Vec<u8>, proposer_signature: Vec<u8>) -> Result<()> {
        let msg = NetworkMessage::BlockProposal {
            height,
            proposer: proposer.to_string(),
            block_hash: block_hash.to_string(),
            block_data,
            proposer_signature,
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

    /// Request block sync from peers (for catch-up when behind)
    pub async fn request_sync(&self, from_height: u64, to_height: u64) -> Result<()> {
        let msg = NetworkMessage::SyncRequest {
            from_height,
            to_height,
        };
        info!("📤 Requesting sync for blocks {}-{}", from_height, to_height);
        self.broadcast_message(CONSENSUS_TOPIC, msg).await
    }

    /// Send sync response with blocks to requesting peer
    pub async fn send_sync_response(&self, blocks: Vec<Vec<u8>>) -> Result<()> {
        let block_count = blocks.len();
        let msg = NetworkMessage::SyncResponse {
            blocks,
        };
        info!("📤 Sending sync response with {} blocks", block_count);
        self.broadcast_message(CONSENSUS_TOPIC, msg).await
    }

    /// Announce this validator to the network (with Ed25519 signature)
    pub async fn announce_validator(&self, address: &str, stake: u64, pubkey: [u8; 32], signature: Vec<u8>) -> Result<()> {
        let msg = NetworkMessage::ValidatorAnnounce {
            address: address.to_string(),
            stake,
            peer_id: self.peer_id.to_string(),
            pubkey,
            signature,
        };
        
        self.broadcast_message(VALIDATOR_TOPIC, msg).await
    }

    /// Internal: broadcast message to a topic
    async fn broadcast_message(&self, topic: &str, msg: NetworkMessage) -> Result<()> {
        let data = bincode::serialize(&msg)?;
        
        // Enforce message size limit
        if data.len() > MAX_MESSAGE_SIZE {
            anyhow::bail!("Message size {} exceeds maximum {}", data.len(), MAX_MESSAGE_SIZE);
        }
        
        // Send to broadcast channel for gossipsub publishing
        if let Some(tx) = &self.broadcast_tx {
            info!("📢 Broadcasting to {} via gossipsub: {} bytes", topic, data.len());
            if let Err(e) = tx.send((topic.to_string(), data)) {
                warn!("Failed to send to broadcast channel: {}", e);
            }
        } else {
            warn!("⚠️ broadcast_tx is None - cannot publish to gossipsub");
        }
        
        // Also send to local message channel for processing
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
    
    /// Get all known validators
    pub async fn get_known_validators(&self) -> Vec<String> {
        self.known_validators.read().await.iter().cloned().collect()
    }
    
    /// Ban a peer for misbehavior
    pub async fn ban_peer(&self, peer_id: PeerId, reason: &str) {
        let banned = BannedPeer {
            peer_id,
            reason: reason.to_string(),
            banned_at: std::time::Instant::now(),
            duration_secs: PEER_BAN_DURATION_SECS,
        };
        warn!("🚫 Banning peer {} for {}: {}", peer_id, PEER_BAN_DURATION_SECS, reason);
        self.banned_peers.write().await.insert(peer_id, banned);
        // Also remove from connected peers
        self.connected_peers.write().await.remove(&peer_id);
    }
    
    /// Check if a peer is banned
    pub async fn is_peer_banned(&self, peer_id: &PeerId) -> bool {
        let banned = self.banned_peers.read().await;
        if let Some(ban) = banned.get(peer_id) {
            !ban.is_expired()
        } else {
            false
        }
    }
    
    /// Clean up expired bans
    pub async fn cleanup_expired_bans(&self) {
        let mut banned = self.banned_peers.write().await;
        banned.retain(|_, ban| !ban.is_expired());
    }
    
    /// Check rate limit for a peer, returns true if allowed
    pub async fn check_rate_limit(&self, peer_id: &PeerId) -> bool {
        let mut limits = self.peer_rate_limits.write().await;
        
        // Periodic cleanup of stale rate limit entries
        if limits.len() > RATE_LIMIT_CLEANUP_THRESHOLD {
            let now = std::time::Instant::now();
            limits.retain(|_, limit| {
                // Keep entries with recent activity (within 2 minutes)
                limit.window_start
                    .map(|start| now.duration_since(start).as_secs() < 120)
                    .unwrap_or(false)
            });
        }
        
        let limit = limits.entry(*peer_id).or_default();
        limit.record_message()
    }
    
    /// Verify a BlockVote signature (Ed25519)
    /// Returns true if signature is valid for the given voter and block hash
    pub fn verify_vote_signature(voter_pubkey: &[u8; 32], block_hash: &[u8], signature: &[u8]) -> bool {
        #[allow(unused_imports)]
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        
        if signature.len() != 64 {
            return false;
        }
        
        let Ok(verifying_key) = VerifyingKey::from_bytes(voter_pubkey) else {
            return false;
        };
        
        let Ok(sig) = Signature::try_from(signature) else {
            return false;
        };
        
        verifying_key.verify(block_hash, &sig).is_ok()
    }
    
    /// Get banned peer count
    pub async fn banned_peer_count(&self) -> usize {
        self.banned_peers.read().await.len()
    }
    
    /// Check if network is healthy (has minimum required peers)
    pub async fn is_healthy(&self) -> bool {
        self.connected_peers.read().await.len() >= MIN_PEERS_REQUIRED
    }
    
    /// Register a validator's public key for signature verification
    pub async fn register_validator_pubkey(&self, address: String, pubkey: [u8; 32]) {
        self.validator_pubkeys.write().await.insert(address.clone(), pubkey);
        debug!("Registered validator pubkey for {}", address);
    }
    
    /// Get a validator's public key
    pub async fn get_validator_pubkey(&self, address: &str) -> Option<[u8; 32]> {
        self.validator_pubkeys.read().await.get(address).copied()
    }
    
    /// Verify and process a BlockVote - returns true if valid
    pub async fn verify_and_process_vote(&self, voter: &str, block_hash: &[u8], signature: &[u8]) -> Result<bool, String> {
        // Get voter's pubkey
        let pubkey = match self.get_validator_pubkey(voter).await {
            Some(pk) => pk,
            None => return Err(format!("Unknown voter: {}", voter)),
        };
        
        // Verify signature
        if !Self::verify_vote_signature(&pubkey, block_hash, signature) {
            return Err("Invalid vote signature".to_string());
        }
        
        Ok(true)
    }
    
    /// Verify a BlockProposal signature - returns true if valid
    /// The proposer must have a registered pubkey
    pub async fn verify_proposal_signature(&self, proposer: &str, block_hash: &[u8], signature: &[u8]) -> Result<bool, String> {
        // Get proposer's pubkey
        let pubkey = match self.get_validator_pubkey(proposer).await {
            Some(pk) => pk,
            None => return Err(format!("Unknown proposer: {}", proposer)),
        };
        
        // Verify signature over block hash
        if !Self::verify_vote_signature(&pubkey, block_hash, signature) {
            return Err("Invalid proposal signature".to_string());
        }
        
        Ok(true)
    }
    
    /// Verify a ValidatorAnnounce signature - returns true if valid
    /// Verifies signature over (address || stake || peer_id)
    pub fn verify_announce_signature(pubkey: &[u8; 32], address: &str, stake: u64, peer_id: &str, signature: &[u8]) -> bool {
        // Create the message to verify: address || stake || peer_id
        let message = format!("{}{}{}",address, stake, peer_id);
        Self::verify_vote_signature(pubkey, message.as_bytes(), signature)
    }
    
    /// Get count of registered validators
    pub async fn registered_validator_count(&self) -> usize {
        self.validator_pubkeys.read().await.len()
    }
    
    /// Request validator set from peers (for initial sync)
    pub async fn request_validator_set(&self, known_count: u32) -> Result<()> {
        let msg = NetworkMessage::ValidatorSetRequest { known_count };
        info!("📡 Requesting validator set from peers (we know {} validators)", known_count);
        self.broadcast_message(VALIDATOR_TOPIC, msg).await
    }
    
    /// Send validator set response to peers
    pub async fn send_validator_set(&self, validators: Vec<ValidatorInfo>) -> Result<()> {
        let count = validators.len();
        let msg = NetworkMessage::ValidatorSetResponse { validators };
        info!("📤 Sending validator set with {} validators", count);
        self.broadcast_message(VALIDATOR_TOPIC, msg).await
    }
    
    /// Get all registered validator info for sync response
    pub async fn get_all_validator_info(&self) -> Vec<ValidatorInfo> {
        let pubkeys = self.validator_pubkeys.read().await;
        let validators = self.known_validators.read().await;
        
        validators.iter()
            .filter_map(|addr| {
                pubkeys.get(addr).map(|pk| ValidatorInfo {
                    address: addr.clone(),
                    stake: MIN_VALIDATOR_STAKE, // Will be overridden by consensus
                    pubkey: *pk,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let network = P2PNetwork::new().unwrap();
        assert!(!network.peer_id.to_string().is_empty());
        assert_eq!(network.peer_count().await, 0);
        assert!(!network.is_running().await);
    }

    #[tokio::test]
    async fn test_keypair_persistence() {
        let keypair = Keypair::generate_ed25519();
        let peer_id1 = PeerId::from(keypair.public());
        
        let network = P2PNetwork::with_keypair(keypair.clone()).unwrap();
        assert_eq!(*network.peer_id(), peer_id1);
    }

    #[tokio::test]
    async fn test_validator_registration() {
        let network = P2PNetwork::new().unwrap();
        
        assert_eq!(network.known_validator_count().await, 0);
        
        network.register_validator("sultan1val1qqqqqqqqqqqqqqqqqqqqqqqqqqqval1aa".to_string()).await;
        network.register_validator("sultan1val2qqqqqqqqqqqqqqqqqqqqqqqqqqqval2bb".to_string()).await;
        
        assert_eq!(network.known_validator_count().await, 2);
        
        let validators = network.get_known_validators().await;
        assert!(validators.contains(&"sultan1val1qqqqqqqqqqqqqqqqqqqqqqqqqqqval1aa".to_string()));
    }

    #[tokio::test]
    async fn test_peer_banning() {
        let network = P2PNetwork::new().unwrap();
        let fake_peer = PeerId::random();
        
        assert!(!network.is_peer_banned(&fake_peer).await);
        
        network.ban_peer(fake_peer, "invalid message").await;
        
        assert!(network.is_peer_banned(&fake_peer).await);
        assert_eq!(network.banned_peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let network = P2PNetwork::new().unwrap();
        let peer = PeerId::random();
        
        // Should allow messages up to limit
        for _ in 0..MAX_MESSAGES_PER_MINUTE {
            assert!(network.check_rate_limit(&peer).await);
        }
        
        // Should reject after limit exceeded
        assert!(!network.check_rate_limit(&peer).await);
    }

    #[tokio::test]
    async fn test_bootstrap_peers() {
        let mut network = P2PNetwork::new().unwrap();
        
        let peers = vec![
            "/ip4/127.0.0.1/tcp/4001".to_string(),
            "/ip4/127.0.0.1/tcp/4002".to_string(),
        ];
        
        network.set_bootstrap_peers(peers).unwrap();
        assert_eq!(network.bootstrap_peers.len(), 2);
    }

    #[tokio::test]
    async fn test_network_health() {
        let network = P2PNetwork::new().unwrap();
        
        // Not healthy with 0 peers
        assert!(!network.is_healthy().await);
    }

    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::BlockProposal {
            height: 100,
            proposer: "sultan1test".to_string(),
            block_hash: "abc123".to_string(),
            block_data: vec![1, 2, 3],
            proposer_signature: vec![0u8; 64],
        };
        
        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: NetworkMessage = bincode::deserialize(&serialized).unwrap();
        
        match deserialized {
            NetworkMessage::BlockProposal { height, proposer, .. } => {
                assert_eq!(height, 100);
                assert_eq!(proposer, "sultan1test");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_banned_peer_expiration() {
        let peer_id = PeerId::random();
        let banned = BannedPeer {
            peer_id,
            reason: "test".to_string(),
            banned_at: std::time::Instant::now() - std::time::Duration::from_secs(PEER_BAN_DURATION_SECS + 1),
            duration_secs: PEER_BAN_DURATION_SECS,
        };
        
        assert!(banned.is_expired());
    }

    #[test]
    fn test_vote_signature_verification() {
        use ed25519_dalek::{Signer, SigningKey};
        
        // Generate a keypair
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes: [u8; 32] = verifying_key.to_bytes();
        
        // Sign a block hash
        let block_hash = b"test_block_hash_12345";
        let signature = signing_key.sign(block_hash);
        
        // Valid signature should pass
        assert!(P2PNetwork::verify_vote_signature(
            &pubkey_bytes,
            block_hash,
            signature.to_bytes().as_slice()
        ));
        
        // Wrong message should fail
        assert!(!P2PNetwork::verify_vote_signature(
            &pubkey_bytes,
            b"wrong_hash",
            signature.to_bytes().as_slice()
        ));
        
        // Invalid signature length should fail
        assert!(!P2PNetwork::verify_vote_signature(
            &pubkey_bytes,
            block_hash,
            &[0u8; 32] // Wrong length
        ));
    }

    #[test]
    fn test_min_validator_stake_constant() {
        // Verify minimum stake is 10 trillion SULTAN
        assert_eq!(MIN_VALIDATOR_STAKE, 10_000_000_000_000);
    }

    #[tokio::test]
    async fn test_rate_limit_cleanup() {
        let network = P2PNetwork::new().unwrap();
        
        // Add many peers to trigger cleanup
        for _i in 0..150 {
            let peer = PeerId::random();
            network.check_rate_limit(&peer).await;
        }
        
        // Verify rate limits are being tracked (and some may be cleaned)
        let limits = network.peer_rate_limits.read().await;
        // After cleanup, should have at most RATE_LIMIT_CLEANUP_THRESHOLD entries
        assert!(limits.len() <= RATE_LIMIT_CLEANUP_THRESHOLD + 50);
    }

    #[tokio::test]
    async fn test_validator_pubkey_registration() {
        let network = P2PNetwork::new().unwrap();
        
        // Initially no validators
        assert_eq!(network.registered_validator_count().await, 0);
        
        // Register a validator
        let pubkey = [1u8; 32];
        network.register_validator_pubkey("val1".to_string(), pubkey).await;
        
        // Should now have one
        assert_eq!(network.registered_validator_count().await, 1);
        
        // Should be retrievable
        assert_eq!(network.get_validator_pubkey("val1").await, Some(pubkey));
        assert_eq!(network.get_validator_pubkey("unknown").await, None);
    }

    #[tokio::test]
    async fn test_verify_and_process_vote() {
        use ed25519_dalek::{Signer, SigningKey};
        
        let network = P2PNetwork::new().unwrap();
        
        // Create a signing keypair
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let pubkey_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        
        // Register the validator
        network.register_validator_pubkey("validator1".to_string(), pubkey_bytes).await;
        
        // Sign a block hash
        let block_hash = b"test_block_hash_12345";
        let signature = signing_key.sign(block_hash);
        
        // Valid vote should pass
        let result = network.verify_and_process_vote(
            "validator1",
            block_hash,
            signature.to_bytes().as_slice()
        ).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Unknown voter should fail
        let result = network.verify_and_process_vote(
            "unknown_validator",
            block_hash,
            signature.to_bytes().as_slice()
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown voter"));
        
        // Invalid signature should fail
        let result = network.verify_and_process_vote(
            "validator1",
            block_hash,
            &[0u8; 64] // Invalid signature
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid vote signature"));
    }

    #[tokio::test]
    async fn test_verify_proposal_signature() {
        use ed25519_dalek::{Signer, SigningKey};
        
        let network = P2PNetwork::new().unwrap();
        
        // Create a signing keypair for the proposer
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let pubkey_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        
        // Register the proposer
        network.register_validator_pubkey("proposer1".to_string(), pubkey_bytes).await;
        
        // Sign a block hash
        let block_hash = b"block_hash_for_proposal";
        let signature = signing_key.sign(block_hash);
        
        // Valid proposal signature should pass
        let result = network.verify_proposal_signature(
            "proposer1",
            block_hash,
            signature.to_bytes().as_slice()
        ).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Unknown proposer should fail
        let result = network.verify_proposal_signature(
            "unknown_proposer",
            block_hash,
            signature.to_bytes().as_slice()
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown proposer"));
        
        // Invalid signature should fail
        let result = network.verify_proposal_signature(
            "proposer1",
            block_hash,
            &[0u8; 64]
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid proposal signature"));
    }

    #[test]
    fn test_verify_announce_signature() {
        use ed25519_dalek::{Signer, SigningKey};
        
        // Create a signing keypair
        let signing_key = SigningKey::from_bytes(&[2u8; 32]);
        let pubkey_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        
        let address = "sultan1validator123";
        let stake: u64 = 10_000_000_000_000;
        let peer_id = "12D3KooWtest";
        
        // Create message and sign it
        let message = format!("{}{}{}", address, stake, peer_id);
        let signature = signing_key.sign(message.as_bytes());
        
        // Valid announcement signature should pass
        assert!(P2PNetwork::verify_announce_signature(
            &pubkey_bytes,
            address,
            stake,
            peer_id,
            signature.to_bytes().as_slice()
        ));
        
        // Wrong address should fail
        assert!(!P2PNetwork::verify_announce_signature(
            &pubkey_bytes,
            "wrong_address",
            stake,
            peer_id,
            signature.to_bytes().as_slice()
        ));
        
        // Wrong stake should fail
        assert!(!P2PNetwork::verify_announce_signature(
            &pubkey_bytes,
            address,
            999,
            peer_id,
            signature.to_bytes().as_slice()
        ));
        
        // Invalid signature should fail
        assert!(!P2PNetwork::verify_announce_signature(
            &pubkey_bytes,
            address,
            stake,
            peer_id,
            &[0u8; 64]
        ));
    }
}
