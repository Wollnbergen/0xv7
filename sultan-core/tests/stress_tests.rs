//! Sultan Production Stress Tests - 16 shards at launch (64K TPS)

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator};
use sultan_core::blockchain::Transaction;
use std::time::Instant;

fn create_test_config() -> ShardConfig {
    ShardConfig {
        shard_count: 16,          // Launch with 16 shards
        max_shards: 8_000,        // Expandable to 8K
        tx_per_shard: 8_000,      // 8K tx/shard/block
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    }
}

#[tokio::test]
async fn stress_test_10_percent_capacity() {
    let coordinator = ShardingCoordinator::new(create_test_config());
    
    // 10% of 64K = 6,400 transactions
    let mut transactions = Vec::with_capacity(6_400);
    for i in 0..6_400 {
        transactions.push(Transaction {
            from: format!("user_{}", i % 1000),
            to: format!("user_{}", (i + 1) % 1000),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 1000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions).await;
    let elapsed = start.elapsed();

    println!("✅ 10% Capacity: {} same-shard, {} cross-shard in {:?}", 
        same_shard.values().map(|v| v.len()).sum::<usize>(),
        cross_shard.len(), elapsed);
    
    assert!(same_shard.len() > 0 || cross_shard.len() > 0);
}

#[tokio::test]
async fn stress_test_50_percent_capacity() {
    let coordinator = ShardingCoordinator::new(create_test_config());
    
    // 50% of 64K = 32,000 transactions
    let mut transactions = Vec::with_capacity(32_000);
    for i in 0..32_000 {
        transactions.push(Transaction {
            from: format!("user_{}", i % 5000),
            to: format!("user_{}", (i + 1) % 5000),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 5000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions).await;
    let elapsed = start.elapsed();

    println!("✅ 50% Capacity: {} same-shard, {} cross-shard in {:?}", 
        same_shard.values().map(|v| v.len()).sum::<usize>(),
        cross_shard.len(), elapsed);
}

#[tokio::test]
async fn stress_test_full_capacity() {
    let coordinator = ShardingCoordinator::new(create_test_config());
    
    // 100% of 64K = 64,000 transactions
    let mut transactions = Vec::with_capacity(64_000);
    for i in 0..64_000 {
        transactions.push(Transaction {
            from: format!("user_{}", i % 10000),
            to: format!("user_{}", (i + 1) % 10000),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 10000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions).await;
    let elapsed = start.elapsed();

    let total = same_shard.values().map(|v| v.len()).sum::<usize>() + cross_shard.len();
    assert_eq!(total, 64_000, "Must process all 64K transactions");
    
    println!("✅ 100% Capacity: 64K tx classified in {:?}", elapsed);
}

#[tokio::test]
async fn stress_test_shard_distribution() {
    let coordinator = ShardingCoordinator::new(create_test_config());
    
    let mut transactions = Vec::with_capacity(16_000);
    for i in 0..16_000 {
        transactions.push(Transaction {
            from: format!("user_{}", i),
            to: format!("user_{}", i),  // Same user = same shard
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let (same_shard, _) = coordinator.classify_transactions(transactions).await;
    
    // With 16 shards and 16K same-user tx, each shard should get ~1000 tx
    for (shard_id, txs) in &same_shard {
        let count = txs.len();
        println!("   Shard {}: {} tx", shard_id, count);
        // Allow 50% variance for hash distribution
        assert!(count > 500 && count < 1500, "Shard {} has {} tx (expected ~1000)", shard_id, count);
    }
    
    println!("✅ Distribution: 16K tx spread across {} shards", same_shard.len());
}

#[tokio::test]
async fn stress_test_expansion_under_load() {
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
    
    // Process load before expansion
    let mut transactions = Vec::with_capacity(32_000);
    for i in 0..32_000 {
        transactions.push(Transaction {
            from: format!("user_{}", i % 5000),
            to: format!("user_{}", (i + 1) % 5000),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 5000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }
    
    let before = coordinator.shards.read().await.len();
    let _ = coordinator.classify_transactions(transactions).await;
    
    // Expand under load
    coordinator.expand_shards(16).await.unwrap();
    
    let after = coordinator.shards.read().await.len();
    assert_eq!(after, before + 16, "Expanded from {} to {}", before, after);
    
    println!("✅ Expansion under load: {} → {} shards", before, after);
}
