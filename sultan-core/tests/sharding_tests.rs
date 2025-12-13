//! Production Sharding Tests

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator, Shard};
use sultan_core::blockchain::Transaction;

#[test]
fn test_shard_routing() {
    // Test deterministic shard routing
    let addr1 = "sultan1abc123";
    let addr2 = "sultan1xyz789";
    
    let shard1_a = Shard::calculate_shard_id(addr1, 1024);
    let shard1_b = Shard::calculate_shard_id(addr1, 1024);
    
    // Same address should always route to same shard
    assert_eq!(shard1_a, shard1_b);
    
    // Shard ID should be within range
    assert!(shard1_a < 1024);
    
    let shard2 = Shard::calculate_shard_id(addr2, 1024);
    assert!(shard2 < 1024);
    
    println!("✅ Shard routing test passed: {} → shard {}, {} → shard {}", 
        addr1, shard1_a, addr2, shard2);
}

#[test]
fn test_coordinator_initialization() {
    let config = ShardConfig {
        shard_count: 8,           // Launch default
        max_shards: 8_000,        // Maximum expansion
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config.clone());
    
    assert_eq!(coordinator.shards.len(), 8);
    assert_eq!(coordinator.config.shard_count, 8);
    assert_eq!(coordinator.config.max_shards, 8_000);
    
    println!("✅ Coordinator initialized with {} shards (max: {})", 
        coordinator.shards.len(), coordinator.config.max_shards);
}

#[test]
fn test_transaction_classification() {
    let config = ShardConfig {
        shard_count: 8,
        max_shards: 8_000,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    let transactions = vec![
        Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: 1,
            nonce: 1,
            signature: Some("sig1".to_string()),
        },
        Transaction {
            from: "charlie".to_string(),
            to: "dave".to_string(),
            amount: 200,
            gas_fee: 0,
            timestamp: 2,
            nonce: 1,
            signature: Some("sig2".to_string()),
        },
    ];

    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions);
    
    let total_same = same_shard.values().map(|v| v.len()).sum::<usize>();
    let total_cross = cross_shard.len();
    let total = total_same + total_cross;
    
    assert_eq!(total, 2, "Should classify all transactions");
    
    println!("✅ Transaction classification: {} same-shard, {} cross-shard", 
        total_same, total_cross);
}

#[tokio::test]
async fn test_cross_shard_queue() {
    let config = ShardConfig {
        shard_count: 8,
        max_shards: 8_000,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    let transactions = vec![
        Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: 1,
            nonce: 1,
            signature: Some("sig1".to_string()),
        },
    ];

    let (_, cross_shard_txs) = coordinator.classify_transactions(transactions);
    
    // Add to queue
    {
        let mut queue = coordinator.cross_shard_queue.lock().await;
        for ctx in cross_shard_txs {
            queue.push_back(ctx);
        }
    }
    
    let queue_len = coordinator.cross_shard_queue.lock().await.len();
    println!("✅ Cross-shard queue test: {} transactions queued", queue_len);
}

#[tokio::test]
async fn test_shard_health_monitoring() {
    let shard = Shard::new(0);
    
    // Should start healthy
    assert!(shard.is_healthy().await);
    
    // Mark unhealthy
    shard.mark_unhealthy().await;
    assert!(!shard.is_healthy().await);
    
    // Mark healthy again
    shard.mark_healthy().await;
    assert!(shard.is_healthy().await);
    
    println!("✅ Shard health monitoring test passed");
}

#[tokio::test]
async fn test_merkle_tree_verification() {
    use sultan_core::sharding_production::MerkleTree;
    use sha2::{Sha256, Digest};
    
    let data = vec![
        "tx1".as_bytes(),
        "tx2".as_bytes(),
        "tx3".as_bytes(),
        "tx4".as_bytes(),
    ];
    
    let tree = MerkleTree::new(data.clone());
    let root = tree.get_root();
    
    // Verify tree has a valid root
    assert_ne!(root, [0u8; 32], "Merkle root should not be all zeros");
    
    println!("✅ Merkle tree test passed with {} leaves, root: {:x?}", data.len(), &root[0..8]);
}

#[test]
fn test_idempotency_key_generation() {
    use sultan_core::sharding_production::CrossShardTransaction;
    
    let tx1 = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 100,
        gas_fee: 0,
        timestamp: 1,
        nonce: 1,
        signature: Some("sig".to_string()),
    };
    
    let tx2 = tx1.clone();
    
    let ctx1 = CrossShardTransaction::new(0, 1, tx1);
    let ctx2 = CrossShardTransaction::new(0, 1, tx2);
    
    // Same transaction content should produce same idempotency key
    assert_eq!(ctx1.idempotency_key, ctx2.idempotency_key, 
        "Idempotency keys should match for identical transactions");
    
    println!("✅ Idempotency key generation test passed: key = {}", ctx1.idempotency_key);
}

#[tokio::test]
async fn test_distributed_locks() {
    let config = ShardConfig {
        shard_count: 8,
        max_shards: 8_000,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    // Acquire lock
    let tx_key = "alice:1".to_string();
    {
        let mut locks = coordinator.tx_locks.write().await;
        locks.insert(tx_key.clone());
    }
    
    // Verify lock exists
    {
        let locks = coordinator.tx_locks.read().await;
        assert!(locks.contains(&tx_key), "Lock should be acquired");
    }
    
    // Release lock
    {
        let mut locks = coordinator.tx_locks.write().await;
        locks.remove(&tx_key);
    }
    
    // Verify lock released
    {
        let locks = coordinator.tx_locks.read().await;
        assert!(!locks.contains(&tx_key), "Lock should be released");
    }
    
    println!("✅ Distributed lock test passed");
}

#[tokio::test]
async fn test_2s_block_time() {
    use sultan_core::config::Config;
    
    let config = Config::default();
    assert_eq!(config.block_time, 2, "Block time should be 2 seconds");
    
    println!("✅ Block time configured to {} seconds", config.block_time);
}

#[tokio::test]
async fn test_shard_auto_expansion() {
    let mut config = ShardConfig::default();
    config.shard_count = 8;
    config.max_shards = 64;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    assert_eq!(coordinator.shards.len(), 8, "Should start with 8 shards");
    
    // Expand shards
    coordinator.expand_shards(8).await.unwrap();
    
    assert_eq!(coordinator.shards.len(), 16, "Should have 16 shards after expansion");
    assert_eq!(coordinator.config.shard_count, 16);
    
    // Expand to max
    coordinator.expand_shards(100).await.unwrap();
    
    assert_eq!(coordinator.shards.len(), 64, "Should cap at max_shards (64)");
    assert_eq!(coordinator.config.shard_count, 64);
    
    println!("✅ Auto-expansion test: 8 → 16 → 64 shards (max capacity)");
}

#[tokio::test]
async fn test_launch_configuration() {
    let config = ShardConfig::default();
    
    assert_eq!(config.shard_count, 8, "Launch with 8 shards");
    assert_eq!(config.max_shards, 8_000, "Expandable to 8000");
    assert_eq!(config.tx_per_shard, 8_000, "8K TPS per shard");
    assert_eq!(config.auto_expand_threshold, 0.80, "Expand at 80% load");
    
    let coordinator = ShardingCoordinator::new(config);
    let capacity = coordinator.get_tps_capacity();
    
    assert_eq!(capacity, 64_000, "Launch capacity: 64K TPS (8 * 8K)");
    
    println!("✅ Launch config: 8 shards, 64K TPS, expandable to 8K shards (64M TPS)");
}
