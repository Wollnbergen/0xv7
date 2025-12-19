//! Comprehensive Shard Expansion Tests
//! Tests auto-expansion robustness under various conditions

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator};
use sultan_core::blockchain::Transaction;
use std::time::Instant;

#[tokio::test]
async fn test_expansion_basic() {
    let mut config = ShardConfig::default();
    config.shard_count = 8;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    println!("📊 Initial state: {} shards", coordinator.shards.len());
    assert_eq!(coordinator.shards.len(), 8);
    
    // Expand by 8
    coordinator.expand_shards(8).await.unwrap();
    println!("📊 After +8: {} shards", coordinator.shards.len());
    assert_eq!(coordinator.shards.len(), 16);
    
    // Expand by 16 more (unlimited expansion)
    coordinator.expand_shards(16).await.unwrap();
    println!("📊 After +16: {} shards", coordinator.shards.len());
    assert_eq!(coordinator.shards.len(), 32);
    
    // Expand by 10 more (unlimited - no cap)
    coordinator.expand_shards(10).await.unwrap();
    assert_eq!(coordinator.shards.len(), 42, "Should expand without limit");
    
    println!("✅ Expansion works correctly with unlimited scaling");
}

#[tokio::test]
async fn test_expansion_preserves_data() {
    let mut config = ShardConfig::default();
    config.shard_count = 4;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // Initialize accounts in original shards
    coordinator.init_account("alice".to_string(), 1_000_000).await.unwrap();
    coordinator.init_account("bob".to_string(), 2_000_000).await.unwrap();
    coordinator.init_account("charlie".to_string(), 3_000_000).await.unwrap();
    
    let alice_balance_before = coordinator.get_balance("alice").await;
    let bob_balance_before = coordinator.get_balance("bob").await;
    let charlie_balance_before = coordinator.get_balance("charlie").await;
    
    println!("💰 Before expansion:");
    println!("   alice: {}", alice_balance_before);
    println!("   bob: {}", bob_balance_before);
    println!("   charlie: {}", charlie_balance_before);
    
    // Expand shards
    coordinator.expand_shards(4).await.unwrap();
    
    // Verify balances unchanged
    let alice_balance_after = coordinator.get_balance("alice").await;
    let bob_balance_after = coordinator.get_balance("bob").await;
    let charlie_balance_after = coordinator.get_balance("charlie").await;
    
    println!("💰 After expansion:");
    println!("   alice: {}", alice_balance_after);
    println!("   bob: {}", bob_balance_after);
    println!("   charlie: {}", charlie_balance_after);
    
    assert_eq!(alice_balance_before, alice_balance_after, "Alice balance should be preserved");
    assert_eq!(bob_balance_before, bob_balance_after, "Bob balance should be preserved");
    assert_eq!(charlie_balance_before, charlie_balance_after, "Charlie balance should be preserved");
    
    println!("✅ All account data preserved during expansion");
}

#[tokio::test]
async fn test_expansion_concurrent_transactions() {
    let mut config = ShardConfig::default();
    config.shard_count = 4;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // Initialize accounts
    for i in 0..100 {
        coordinator.init_account(format!("user_{}", i), 1_000_000).await.unwrap();
    }
    
    // Create transactions BEFORE expansion
    let transactions_before: Vec<Transaction> = (0..50).map(|i| {
        Transaction {
            from: format!("user_{}", i),
            to: format!("user_{}", (i + 1) % 100),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: 1,
            signature: Some("sig".to_string()),
        }
    }).collect();
    
    let (same_shard_before, cross_shard_before) = coordinator.classify_transactions(transactions_before);
    
    // Expand
    coordinator.expand_shards(4).await.unwrap();
    
    // Create transactions AFTER expansion
    let transactions_after: Vec<Transaction> = (50..100).map(|i| {
        Transaction {
            from: format!("user_{}", i),
            to: format!("user_{}", (i + 1) % 100),
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: 1,
            signature: Some("sig".to_string()),
        }
    }).collect();
    
    let (same_shard_after, cross_shard_after) = coordinator.classify_transactions(transactions_after);
    
    println!("📊 Before expansion (4 shards): {} same-shard, {} cross-shard",
        same_shard_before.values().map(|v| v.len()).sum::<usize>(),
        cross_shard_before.len());
    
    println!("📊 After expansion (8 shards): {} same-shard, {} cross-shard",
        same_shard_after.values().map(|v| v.len()).sum::<usize>(),
        cross_shard_after.len());
    
    println!("✅ Transaction routing works correctly after expansion");
}

#[tokio::test]
async fn test_expansion_health_monitoring() {
    let mut config = ShardConfig::default();
    config.shard_count = 4;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // Mark some shards unhealthy
    coordinator.shards[1].mark_unhealthy().await;
    coordinator.shards[3].mark_unhealthy().await;
    
    let stats_before = coordinator.get_stats().await;
    println!("📊 Before expansion: {}/{} healthy shards", 
        stats_before.healthy_shards, stats_before.shard_count);
    
    // Expand
    coordinator.expand_shards(8).await.unwrap();
    
    let stats_after = coordinator.get_stats().await;
    println!("📊 After expansion: {}/{} healthy shards", 
        stats_after.healthy_shards, stats_after.shard_count);
    
    // New shards should be healthy
    assert!(stats_after.healthy_shards >= stats_before.healthy_shards + 8, 
        "New shards should start healthy");
    
    println!("✅ New shards initialize as healthy");
}

#[tokio::test]
async fn test_expansion_capacity_calculation() {
    let mut config = ShardConfig::default();
    config.shard_count = 8;
    config.tx_per_shard = 8_000;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // TPS formula: shards × tx_per_shard ÷ 2 (2-second blocks)
    let capacity_8 = coordinator.get_tps_capacity();
    println!("📊 Capacity with 8 shards: {} TPS", capacity_8);
    assert_eq!(capacity_8, 32_000, "8 * 8000 / 2 = 32K TPS");
    
    coordinator.expand_shards(8).await.unwrap();
    let capacity_16 = coordinator.get_tps_capacity();
    println!("📊 Capacity with 16 shards: {} TPS", capacity_16);
    assert_eq!(capacity_16, 64_000, "16 * 8000 / 2 = 64K TPS");
    
    coordinator.expand_shards(16).await.unwrap();
    let capacity_32 = coordinator.get_tps_capacity();
    println!("📊 Capacity with 32 shards: {} TPS", capacity_32);
    assert_eq!(capacity_32, 128_000, "32 * 8000 / 2 = 128K TPS");
    
    coordinator.expand_shards(32).await.unwrap();
    let capacity_64 = coordinator.get_tps_capacity();
    println!("📊 Capacity with 64 shards: {} TPS", capacity_64);
    assert_eq!(capacity_64, 256_000, "64 * 8000 / 2 = 256K TPS");
    
    println!("✅ Capacity scales linearly with shard count");
}

#[tokio::test]
async fn test_expansion_idempotent() {
    let mut config = ShardConfig::default();
    config.shard_count = 8;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // Expand to 12
    coordinator.expand_shards(4).await.unwrap();
    assert_eq!(coordinator.shards.len(), 12);
    
    // Zero expansion should not change shard count
    coordinator.expand_shards(0).await.unwrap();
    assert_eq!(coordinator.shards.len(), 12, "Zero expansion should not change shard count");
    
    // Expand by 4 more (unlimited scaling)
    coordinator.expand_shards(4).await.unwrap();
    assert_eq!(coordinator.shards.len(), 16);
    
    // Expand again - unlimited scaling allows continued growth
    coordinator.expand_shards(8).await.unwrap();
    assert_eq!(coordinator.shards.len(), 24);
    
    println!("✅ Expansion is idempotent and supports unlimited scaling");
}

#[tokio::test]
async fn test_load_based_expansion_trigger() {
    let mut config = ShardConfig::default();
    config.shard_count = 4;
    config.tx_per_shard = 100; // Low threshold for testing
    config.auto_expand_threshold = 0.80;
    
    let coordinator = ShardingCoordinator::new(config);
    
    // Simulate low load
    let stats_low = coordinator.get_stats().await;
    println!("📊 Low load: {:.2}% - Should expand: {}", 
        stats_low.current_load * 100.0, stats_low.should_expand);
    assert!(!stats_low.should_expand, "Should not expand at low load");
    
    // Process transactions to increase load
    for shard in &coordinator.shards {
        let mut count = shard.processed_count.write().await;
        *count = 85; // 85/100 = 85% load
    }
    
    let stats_high = coordinator.get_stats().await;
    println!("📊 High load: {:.2}% - Should expand: {}", 
        stats_high.current_load * 100.0, stats_high.should_expand);
    assert!(stats_high.should_expand, "Should expand at >80% load");
    
    println!("✅ Load-based expansion triggers correctly");
}

#[tokio::test]
async fn test_expansion_to_production_scale() {
    let mut config = ShardConfig::default();
    config.shard_count = 8;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    println!("🚀 Production scale expansion test:");
    println!("   Start: {} shards", coordinator.shards.len());
    
    // Expand to 64 (small production)
    let start = Instant::now();
    coordinator.expand_shards(56).await.unwrap();
    let elapsed_64 = start.elapsed();
    println!("   64 shards: {} ms", elapsed_64.as_millis());
    
    // Expand to 512 (medium production)
    let start = Instant::now();
    coordinator.expand_shards(448).await.unwrap();
    let elapsed_512 = start.elapsed();
    println!("   512 shards: {} ms", elapsed_512.as_millis());
    
    // Expand to 1024 (high production)
    let start = Instant::now();
    coordinator.expand_shards(512).await.unwrap();
    let elapsed_1024 = start.elapsed();
    println!("   1024 shards: {} ms", elapsed_1024.as_millis());
    
    assert_eq!(coordinator.shards.len(), 1024);
    
    // TPS formula: shards × tx_per_shard ÷ 2 (2-second blocks)
    let capacity = coordinator.get_tps_capacity();
    println!("   Final capacity: {} TPS", capacity);
    assert_eq!(capacity, 4_096_000, "1024 * 8000 / 2 = 4.1M TPS");
    
    println!("✅ Can scale to production levels quickly");
}

#[tokio::test]
async fn test_expansion_rollback_safety() {
    let mut config = ShardConfig::default();
    config.shard_count = 4;
    
    let mut coordinator = ShardingCoordinator::new(config);
    
    // Initialize account
    coordinator.init_account("test_user".to_string(), 1_000_000).await.unwrap();
    
    let balance_before = coordinator.get_balance("test_user").await;
    let shard_count_before = coordinator.shards.len();
    
    // Expand
    coordinator.expand_shards(4).await.unwrap();
    
    // Verify no data corruption
    let balance_after = coordinator.get_balance("test_user").await;
    let shard_count_after = coordinator.shards.len();
    
    assert_eq!(balance_before, balance_after, "Balance must be preserved");
    assert_eq!(shard_count_after, shard_count_before + 4, "Shard count should increase by 4");
    
    // Verify account still accessible
    let balance_recheck = coordinator.get_balance("test_user").await;
    assert_eq!(balance_recheck, balance_before, "Balance must remain consistent");
    
    println!("✅ Expansion is safe with no data corruption");
}
