//! Production Sharding Tests - 16 shards at launch (64K TPS)

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator, Shard, MerkleTree, CrossShardTransaction};
use sultan_core::blockchain::Transaction;

#[test]
fn test_shard_routing() {
    let addr1 = "sultan1abc123";
    let addr2 = "sultan1xyz789";
    
    let shard1_a = Shard::calculate_shard_id(addr1, 1024);
    let shard1_b = Shard::calculate_shard_id(addr1, 1024);
    
    assert_eq!(shard1_a, shard1_b);
    assert!(shard1_a < 1024);
    
    let shard2 = Shard::calculate_shard_id(addr2, 1024);
    assert!(shard2 < 1024);
    
    println!("✅ Shard routing: {} → {}, {} → {}", addr1, shard1_a, addr2, shard2);
}

#[tokio::test]
async fn test_coordinator_initialization() {
    let config = ShardConfig::default();
    let coordinator = ShardingCoordinator::new(config);
    
    let shards = coordinator.shards.read().await;
    let cfg = coordinator.config.read().await;
    
    assert_eq!(shards.len(), 16, "Must launch with 16 shards");
    assert_eq!(cfg.shard_count, 16);
    assert_eq!(cfg.max_shards, 8_000);
    assert_eq!(cfg.tx_per_shard, 8_000);
    
    println!("✅ Coordinator: {} shards (max: {})", shards.len(), cfg.max_shards);
}

#[tokio::test]
async fn test_transaction_classification() {
    let coordinator = ShardingCoordinator::new(ShardConfig::default());
    
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

    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions).await;
    let total = same_shard.values().map(|v| v.len()).sum::<usize>() + cross_shard.len();
    
    assert_eq!(total, 2, "Should classify all transactions");
    println!("✅ Classification: {} same-shard, {} cross-shard", 
        same_shard.values().map(|v| v.len()).sum::<usize>(), cross_shard.len());
}

#[tokio::test]
async fn test_shard_health_monitoring() {
    let shard = Shard::new(0);
    
    assert!(shard.is_healthy().await);
    shard.mark_unhealthy().await;
    assert!(!shard.is_healthy().await);
    shard.mark_healthy().await;
    assert!(shard.is_healthy().await);
    
    println!("✅ Shard health monitoring works");
}

#[tokio::test]
async fn test_merkle_tree_verification() {
    let data = vec!["tx1".as_bytes(), "tx2".as_bytes(), "tx3".as_bytes(), "tx4".as_bytes()];
    let tree = MerkleTree::new(data.clone());
    let root = tree.get_root();
    
    assert_ne!(root, [0u8; 32], "Merkle root should not be zeros");
    println!("✅ Merkle tree: {} leaves, root: {:x?}", data.len(), &root[0..8]);
}

#[test]
fn test_idempotency_key_generation() {
    let tx1 = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 100,
        gas_fee: 0,
        timestamp: 1,
        nonce: 1,
        signature: Some("sig".to_string()),
    };
    
    let ctx1 = CrossShardTransaction::new(0, 1, tx1.clone());
    let ctx2 = CrossShardTransaction::new(0, 1, tx1);
    
    assert_eq!(ctx1.idempotency_key, ctx2.idempotency_key);
    println!("✅ Idempotency key: {}", ctx1.idempotency_key);
}

#[tokio::test]
async fn test_distributed_locks() {
    let coordinator = ShardingCoordinator::new(ShardConfig::default());
    
    let tx_key = "alice:1".to_string();
    coordinator.tx_locks.write().await.insert(tx_key.clone());
    assert!(coordinator.tx_locks.read().await.contains(&tx_key));
    coordinator.tx_locks.write().await.remove(&tx_key);
    assert!(!coordinator.tx_locks.read().await.contains(&tx_key));
    
    println!("✅ Distributed locks work");
}

#[tokio::test]
async fn test_2s_block_time() {
    use sultan_core::config::Config;
    let config = Config::default();
    assert_eq!(config.block_time, 2, "Block time must be 2 seconds");
    println!("✅ Block time: {} seconds", config.block_time);
}

#[tokio::test]
async fn test_shard_auto_expansion() {
    let config = ShardConfig {
        shard_count: 16,
        max_shards: 64,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    };
    
    let coordinator = ShardingCoordinator::new(config);
    
    assert_eq!(coordinator.shards.read().await.len(), 16, "Start with 16");
    
    coordinator.expand_shards(16).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 32, "Expand to 32");
    
    coordinator.expand_shards(100).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 64, "Cap at 64");
    
    println!("✅ Expansion: 16 → 32 → 64 (capped)");
}

#[tokio::test]
async fn test_launch_configuration() {
    let config = ShardConfig::default();
    
    assert_eq!(config.shard_count, 16, "Launch with 16 shards");
    assert_eq!(config.max_shards, 8_000, "Expandable to 8000");
    assert_eq!(config.tx_per_shard, 8_000, "8K tx/shard/block");
    assert_eq!(config.auto_expand_threshold, 0.80, "Expand at 80%");
    
    let coordinator = ShardingCoordinator::new(config);
    let capacity = coordinator.get_tps_capacity().await;
    
    // 16 shards × 8K tx/block ÷ 2s = 64K TPS
    assert_eq!(capacity, 64_000, "Launch: 64K TPS (16 × 8K ÷ 2)");
    
    println!("✅ Launch: 16 shards, 64K TPS, expandable to 8K shards");
}
