use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{Router, Json, response::IntoResponse, routing::get};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    gas_fee: u64,  // Always 0
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    validator: String,
}

struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis = Block {
            index: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: "genesis_hash".to_string(),
            validator: "sultan".to_string(),
        };
        self.chain.push(genesis);
    }

    fn create_block(&mut self) -> Block {
        let previous_block = match self.chain.last() {
            Some(b) => b,
            None => {
                // Should not happen; ensure genesis exists
                self.create_genesis_block();
                self.chain.last().expect("genesis must exist")
            }
        };
        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            transactions: self.pending_transactions.clone(),
            previous_hash: previous_block.hash.clone(),
            hash: String::new(),
            validator: "validator1".to_string(),
        };
        
        block.hash = self.calculate_hash(&block);
        self.pending_transactions.clear();
        self.chain.push(block.clone());
        block
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}",
            block.index, block.timestamp, block.transactions.len(), block.previous_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }
}

// API handlers
async fn status(blockchain: Arc<Mutex<Blockchain>>) -> impl IntoResponse {
    let chain = match blockchain.lock() {
        Ok(c) => c,
        Err(_) => {
            return Json(serde_json::json!({
                "status": "error",
                "error": "lock poisoned"
            }))
        }
    };
    Json(serde_json::json!({
        "status": "running",
        "blocks": chain.chain.len(),
        "pending_tx": chain.pending_transactions.len(),
        "latest_block": chain.chain.last().map(|b| b.index).unwrap_or(0),
        "gas_fee": 0,
        "staking_apy": "13.33%"
    }))
}

#[derive(Serialize)]
struct BlocksResponse {
    blocks: Vec<Block>,
    error: Option<String>,
}

async fn get_blocks(blockchain: Arc<Mutex<Blockchain>>) -> impl IntoResponse {
    match blockchain.lock() {
        Ok(chain) => Json(BlocksResponse { blocks: chain.chain.clone(), error: None }),
        Err(_) => Json(BlocksResponse { blocks: Vec::new(), error: Some("lock poisoned".into()) }),
    }
}

async fn create_transaction(
    blockchain: Arc<Mutex<Blockchain>>,
) -> impl IntoResponse {
    let tx = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 100,
        gas_fee: 0,  // Zero gas fees!
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs(),
    };
    
    if let Ok(mut chain) = blockchain.lock() {
        chain.add_transaction(tx.clone());
    }
    
    Json(serde_json::json!({
        "message": "Transaction added",
        "transaction": tx,
        "gas_fee": 0
    }))
}

async fn mine_block(blockchain: Arc<Mutex<Blockchain>>) -> impl IntoResponse {
    let block = match blockchain.lock() {
        Ok(mut chain) => chain.create_block(),
        Err(_) => {
            return Json(serde_json::json!({
                "message": "error",
                "error": "lock poisoned"
            }))
        }
    };
    Json(serde_json::json!({
        "message": "Block mined",
        "block": block,
        "gas_fees_collected": 0
    }))
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          SULTAN BLOCKCHAIN - ZERO GAS FEES FOREVER            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    
    // Show initial state
    {
        if let Ok(chain) = blockchain.lock() {
            let hash = chain.chain.first().map(|b| b.hash.clone()).unwrap_or_default();
            let prefix: String = hash.chars().take(16).collect();
            println!("✅ Blockchain initialized");
            println!("   Genesis Block: #{}", chain.chain.first().map(|b| b.index).unwrap_or(0));
            println!("   Genesis Hash: {}", prefix);
            println!("   Gas Fees: $0.00 (Zero forever!)");
            println!("   Staking APY: 13.33%");
        } else {
            eprintln!("⚠️  Failed to acquire blockchain lock at startup");
        }
    }

    // Create API router
    let app = Router::new()
        .route("/status", get({
            let bc = blockchain.clone();
            move || status(bc)
        }))
        .route("/blocks", get({
            let bc = blockchain.clone();
            move || get_blocks(bc)
        }))
        .route("/transaction", get({
            let bc = blockchain.clone();
            move || create_transaction(bc)
        }))
        .route("/mine", get({
            let bc = blockchain.clone();
            move || mine_block(bc)
        }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    
    println!();
    println!("🌐 Sultan Blockchain API running on http://0.0.0.0:8080");
    println!();
    println!("📊 API Endpoints:");
    println!("   GET /status      - Blockchain status");
    println!("   GET /blocks      - List all blocks");
    println!("   GET /transaction - Create test transaction");
    println!("   GET /mine        - Mine a new block");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("❌ Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to bind server: {}", e);
        }
    }
}
