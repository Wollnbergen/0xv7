//! Shard Expansion Tests - 16 shards at launch (64K TPS)

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator};
use sultan_core::blockchain::Transaction;
use std::time::Instant;

fn create_test_config(max_shards: usize) -> ShardConfig {
    ShardConfig {
        shard_count: 16,
        max_shards,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        auto_expand_threshold: 0.80,
    }
}

#[tokio::test]
async fn test_expansion_basic() {
    let coordinator = ShardingCoordinator::new(create_test_config(64));
    
    let initial = coordinator.shards.read().await.len();
    assert_eq!(initial, 16, "Launch with 16 shards");
    
    coordinator.expand_shards(16).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 32);
    
    coordinator.expand_shards(32).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 64, "Capped at max");
    
    // Idempotent - should not fail
    coordinator.expand_shards(100).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 64);
    
    println!("✅ Expansion: 16 → 32 → 64 (capped)");
}

#[tokio::test]
async fn test_expansion_preserves_data() {
    let coordinator = ShardingCoordinator::new(create_test_config(32));
    
    coordinator.init_account("alice".to_string(), 1_000_000).await.unwrap();
    coordinator.init_account("bob".to_string(), 2_000_000).await.unwrap();
    
    let alice_before = coordinator.get_balance("alice").await;
    let bob_before = coordinator.get_balance("bob").await;
    
    coordinator.expand_shards(16).await.unwrap();
    
    let alice_after = coordinator.get_balance("alice").await;
    let bob_after = coordinator.get_balance("bob").await;
    
    assert_eq!(alice_before, alice_after, "Alice balance preserved");
    assert_eq!(bob_before, bob_after, "Bob balance preserved");
    
    println!("✅ Balances preserved during expansion");
}

#[tokio::test]
async fn test_expansion_capacity_calculation() {
    let coordinator = ShardingCoordinator::new(create_test_config(128));
    
    // 16 shards × 8K tx/block ÷ 2s = 64K TPS
    let cap_16 = coordinator.get_tps_capacity().await;
    assert_eq!(cap_16, 64_000, "16 shards = 64K TPS");
    
    coordinator.expand_shards(16).await.unwrap();
    let cap_32 = coordinator.get_tps_capacity().await;
    assert_eq!(cap_32, 128_000, "32 shards = 128K TPS");
    
    coordinator.expand_shards(32).await.unwrap();
    let cap_64 = coordinator.get_tps_capacity().await;
    assert_eq!(cap_64, 256_000, "64 shards = 256K TPS");
    
    coordinator.expand_shards(64).await.unwrap();
    let cap_128 = coordinator.get_tps_capacity().await;
    assert_eq!(cap_128, 512_000, "128 shards = 512K TPS");
    
    println!("✅ Capacity scales: 64K → 128K → 256K → 512K TPS");
}

#[tokio::test]
async fn test_expansion_concurrent_transactions() {
    let coordinator = ShardingCoordinator::new(create_test_config(32));
    
    for i in 0..100 {
        coordinator.init_account(format!("user_{}", i), 1_000_000).await.unwrap();
    }
    
    let tx_before: Vec<Transaction> = (0..50).map(|i| Transaction {
        from: format!("user_{}", i),
        to: format!("user_{}", (i + 1) % 100),
        amount: 100,
        gas_fee: 0,
        timestamp: i as u64,
        nonce: 1,
        signature: Some("sig".to_string()),
    }).collect();
    
    let (same_before, cross_before) = coordinator.classify_transactions(tx_before).await;
    
    coordinator.expand_shards(16).await.unwrap();
    
    let tx_after: Vec<Transaction> = (50..100).map(|i| Transaction {
        from: format!("user_{}", i),
        to: format!("user_{}", (i + 1) % 100),
        amount: 100,
        gas_fee: 0,
        timestamp: i as u64,
        nonce: 1,
        signature: Some("sig".to_string()),
    }).collect();
    
    let (same_after, cross_after) = coordinator.classify_transactions(tx_after).await;
    
    println!("✅ Before: {} same, {} cross | After: {} same, {} cross",
        same_before.values().map(|v| v.len()).sum::<usize>(), cross_before.len(),
        same_after.values().map(|v| v.len()).sum::<usize>(), cross_after.len());
}

#[tokio::test]
async fn test_expansion_to_production_scale() {
    let coordinator = ShardingCoordinator::new(create_test_config(8_000));
    
    let start = Instant::now();
    
    // Expand to 1024 shards (4M TPS)
    coordinator.expand_shards(1008).await.unwrap(); // 16 + 1008 = 1024
    
    let elapsed = start.elapsed();
    let shards = coordinator.shards.read().await.len();
    let capacity = coordinator.get_tps_capacity().await;
    
    assert_eq!(shards, 1024);
    assert_eq!(capacity, 4_096_000, "1024 shards = 4M TPS");
    
    println!("✅ Scaled to 1024 shards (4M TPS) in {:?}", elapsed);
}

#[tokio::test]
async fn test_expansion_health_monitoring() {
    let coordinator = ShardingCoordinator::new(create_test_config(48));
    
    // Mark some shards unhealthy
    {
        let shards = coordinator.shards.read().await;
        shards[1].mark_unhealthy().await;
        shards[5].mark_unhealthy().await;
    }
    
    let stats_before = coordinator.get_stats().await;
    println!("Before: {}/{} healthy", stats_before.healthy_shards, stats_before.shard_count);
    
    coordinator.expand_shards(16).await.unwrap();
    
    let stats_after = coordinator.get_stats().await;
    println!("After: {}/{} healthy", stats_after.healthy_shards, stats_after.shard_count);
    
    // New shards should be healthy
    assert!(stats_after.healthy_shards >= stats_before.healthy_shards + 16);
    
    println!("✅ New shards initialize healthy");
}

#[tokio::test]
async fn test_expansion_idempotent() {
    let coordinator = ShardingCoordinator::new(create_test_config(32));
    
    coordinator.expand_shards(0).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 16, "Zero expansion = no change");
    
    coordinator.expand_shards(16).await.unwrap();
    assert_eq!(coordinator.shards.read().await.len(), 32);
    
    // At max - should not fail
    for _ in 0..5 {
        coordinator.expand_shards(100).await.unwrap();
        assert_eq!(coordinator.shards.read().await.len(), 32);
    }
    
    println!("✅ Expansion is idempotent");
}
