// Add this to your main() or rpc_server.rs startup:

// Load P2P configuration
let bootstrap_peers = std::env::var("SULTAN_BOOTSTRAP_PEERS")
    .unwrap_or_default()
    .split(',')
    .filter(|s| !s.is_empty())
    .filter_map(|s| s.parse().ok())
    .collect();

let p2p_key_path = std::env::var("SULTAN_P2P_KEY_PATH")
    .unwrap_or_else(|_| "./data/p2p_key".to_string());

// Start P2P node
let mut p2p_node = p2p::P2PNode::new(&p2p_key_path, bootstrap_peers).await?;
info!("P2P node started with peer ID: {}", p2p_node.peer_id);

// Subscribe to topics
for topic in ["sultan/blocks/v1", "sultan/txs/v1", "sultan/votes/v1"] {
    p2p_node.swarm.behaviour_mut().subscribe(topic)?;
}

// Spawn P2P event handler
let blockchain = Arc::clone(&blockchain);
tokio::spawn(async move {
    while let Some(event) = p2p_node.next_event().await {
        match event {
            p2p::P2PEvent::MessageReceived { message, .. } => {
                match message {
                    p2p::P2PMessage::Block { height, hash, .. } => {
                        info!("Received block {} at height {}", hash, height);
                        // TODO: Call blockchain.validate_block()
                    }
                    _ => {}
                }
            }
            p2p::P2PEvent::PeerConnected(peer) => {
                info!("New peer connected: {}", peer);
            }
            _ => {}
        }
    }
});
