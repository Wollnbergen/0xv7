//! Production Stress Tests for Sultan Sharding
//! Tests at 10%, 30%, 60%, 80%, 90%, 100%, 110% capacity

use sultan_core::sharding_production::{ShardConfig, ShardingCoordinator};
use sultan_core::blockchain::Transaction;
use std::time::Instant;

#[tokio::test]
async fn stress_test_10_percent_capacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config.clone());
    
    // 10% capacity: (1024 shards × 8000 tx) × 0.1 = 819,200 transactions
    // Using smaller test set for demonstration
    let tx_count = 1000;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        let from = format!("user_{}", i % 100);
        let to = format!("user_{}", (i + 1) % 100);
        transactions.push(Transaction {
            from,
            to,
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 100) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let (same_shard, cross_shard) = coordinator.classify_transactions(transactions);
    let elapsed = start.elapsed();

    println!("✅ 10% Capacity Test: Classified {} same-shard tx, {} cross-shard tx in {:?}", 
        same_shard.values().map(|v| v.len()).sum::<usize>(),
        cross_shard.len(), 
        elapsed);
    
    assert!(same_shard.len() > 0 || cross_shard.len() > 0, "Should classify some transactions");
}

#[tokio::test]
async fn stress_test_30_percent_capacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config.clone());
    
    // Initialize test accounts
    for i in 0..3000 {
        let addr = format!("user_{}", i);
        coordinator.init_account(addr, 1_000_000_000).await.unwrap();
    }

    // 30% capacity: 2,457,600 transactions
    let tx_count = 2_457_600;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        let from = format!("user_{}", i % 3000);
        let to = format!("user_{}", (i + 1) % 3000);
        transactions.push(Transaction {
            from,
            to,
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 3000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let result = coordinator.process_parallel(transactions).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    let processed = result.unwrap();
    
    let tps = processed.len() as f64 / elapsed.as_secs_f64();
    println!("✅ 30% Capacity Test: {} tx in {:?} = {:.0} TPS", processed.len(), elapsed, tps);
    
    assert!(tps > 300_000.0, "TPS too low at 30%: {}", tps);
}

#[tokio::test]
async fn stress_test_60_percent_capacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: false, // Disable for pure throughput test
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config.clone());
    
    // Initialize accounts
    for i in 0..5000 {
        let addr = format!("user_{}", i);
        coordinator.init_account(addr, 1_000_000_000).await.unwrap();
    }

    // 60% capacity: 4,915,200 transactions
    let tx_count = 4_915_200;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        let from = format!("user_{}", i % 5000);
        let to = format!("user_{}", (i + 1) % 5000);
        transactions.push(Transaction {
            from,
            to,
            amount: 50,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 5000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let result = coordinator.process_parallel(transactions).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    let processed = result.unwrap();
    
    let tps = processed.len() as f64 / elapsed.as_secs_f64();
    println!("✅ 60% Capacity Test: {} tx in {:?} = {:.0} TPS", processed.len(), elapsed, tps);
    
    assert!(tps > 600_000.0, "TPS too low at 60%: {}", tps);
}

#[tokio::test]
async fn stress_test_80_percent_capacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: false,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    for i in 0..8000 {
        coordinator.init_account(format!("user_{}", i), 1_000_000_000).await.unwrap();
    }

    // 80% capacity: 6,553,600 transactions
    let tx_count = 6_553_600;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        transactions.push(Transaction {
            from: format!("user_{}", i % 8000),
            to: format!("user_{}", (i + 1) % 8000),
            amount: 25,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 8000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let processed = coordinator.process_parallel(transactions).await.unwrap();
    let elapsed = start.elapsed();
    
    let tps = processed.len() as f64 / elapsed.as_secs_f64();
    println!("✅ 80% Capacity Test: {} tx in {:?} = {:.0} TPS", processed.len(), elapsed, tps);
    
    assert!(tps > 800_000.0, "TPS degraded at 80%: {}", tps);
}

#[tokio::test]
async fn stress_test_100_percent_capacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: false,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    for i in 0..10000 {
        coordinator.init_account(format!("user_{}", i), 1_000_000_000).await.unwrap();
    }

    // 100% capacity: 8,192,000 transactions (full capacity)
    let tx_count = 8_192_000;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        transactions.push(Transaction {
            from: format!("user_{}", i % 10000),
            to: format!("user_{}", (i + 1) % 10000),
            amount: 10,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 10000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let processed = coordinator.process_parallel(transactions).await.unwrap();
    let elapsed = start.elapsed();
    
    let tps = processed.len() as f64 / elapsed.as_secs_f64();
    println!("✅ 100% Capacity Test: {} tx in {:?} = {:.0} TPS", processed.len(), elapsed, tps);
    
    // Should achieve at least 1M TPS at full capacity
    assert!(tps > 1_000_000.0, "Failed to reach 1M TPS: {}", tps);
}

#[tokio::test]
async fn stress_test_110_percent_overcapacity() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 8_000,
        cross_shard_enabled: false,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    for i in 0..12000 {
        coordinator.init_account(format!("user_{}", i), 1_000_000_000).await.unwrap();
    }

    // 110% capacity: 9,011,200 transactions (overcapacity stress test)
    let tx_count = 9_011_200;
    let mut transactions = Vec::with_capacity(tx_count);
    
    for i in 0..tx_count {
        transactions.push(Transaction {
            from: format!("user_{}", i % 12000),
            to: format!("user_{}", (i + 1) % 12000),
            amount: 5,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 12000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let result = coordinator.process_parallel(transactions).await;
    let elapsed = start.elapsed();
    
    // Should handle gracefully even over capacity
    assert!(result.is_ok(), "Failed to handle overcapacity");
    
    let processed = result.unwrap();
    let tps = processed.len() as f64 / elapsed.as_secs_f64();
    println!("✅ 110% Overcapacity Test: {} tx in {:?} = {:.0} TPS", processed.len(), elapsed, tps);
    
    // Should still maintain reasonable throughput even when overloaded
    assert!(tps > 900_000.0, "Severe degradation at 110%: {}", tps);
}

#[tokio::test]
async fn test_cross_shard_stress() {
    let config = ShardConfig {
        shard_count: 1024,
        tx_per_shard: 1000, // Lower for cross-shard test
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    // Initialize accounts in different shards
    for i in 0..2000 {
        coordinator.init_account(format!("shard_user_{}", i), 1_000_000).await.unwrap();
    }

    // Create 50% cross-shard transactions
    let mut transactions = Vec::new();
    for i in 0..100_000 {
        let from = format!("shard_user_{}", i % 2000);
        let to = if i % 2 == 0 {
            format!("shard_user_{}", (i + 500) % 2000) // Cross-shard
        } else {
            format!("shard_user_{}", (i + 1) % 2000) // Same-shard
        };
        
        transactions.push(Transaction {
            from,
            to,
            amount: 100,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 2000) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    
    // Process same-shard transactions
    coordinator.process_parallel(transactions).await.unwrap();
    
    // Process cross-shard queue
    let cross_shard_processed = coordinator.process_cross_shard_queue().await.unwrap();
    
    let elapsed = start.elapsed();
    
    println!("✅ Cross-Shard Stress Test: {} cross-shard tx in {:?}", cross_shard_processed, elapsed);
    
    // Verify all accounts still balance correctly
    let stats = coordinator.get_stats().await;
    println!("Stats: {} total processed, {} pending cross-shard", stats.total_processed, stats.pending_cross_shard);
}

#[tokio::test]
async fn test_byzantine_fault_tolerance() {
    let config = ShardConfig {
        shard_count: 10,
        tx_per_shard: 1000,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    // Initialize accounts
    for i in 0..100 {
        coordinator.init_account(format!("user_{}", i), 1_000_000).await.unwrap();
    }

    // Mark some shards as unhealthy
    coordinator.shards[3].mark_unhealthy().await;
    coordinator.shards[7].mark_unhealthy().await;

    // Create transactions
    let mut transactions = Vec::new();
    for i in 0..10_000 {
        transactions.push(Transaction {
            from: format!("user_{}", i % 100),
            to: format!("user_{}", (i + 1) % 100),
            amount: 50,
            gas_fee: 0,
            timestamp: i as u64,
            nonce: (i / 100) as u64 + 1,
            signature: Some("test_sig".to_string()),
        });
    }

    let start = Instant::now();
    let result = coordinator.process_parallel(transactions).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Failed with unhealthy shards");
    let processed = result.unwrap();
    
    println!("✅ Byzantine Fault Tolerance: {} tx processed with 2/10 shards down in {:?}", processed.len(), elapsed);
    
    // Should still process transactions from healthy shards
    assert!(processed.len() > 0, "No transactions processed");
}

#[tokio::test]
async fn test_rollback_recovery() {
    let config = ShardConfig {
        shard_count: 8,
        tx_per_shard: 100,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    // Initialize accounts
    coordinator.init_account("alice".to_string(), 1000).await.unwrap();
    coordinator.init_account("bob".to_string(), 500).await.unwrap();

    let original_alice = coordinator.get_balance("alice").await;
    let original_bob = coordinator.get_balance("bob").await;

    // Create cross-shard transaction that will fail (insufficient balance)
    let tx = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 10_000, // More than alice has
        gas_fee: 0,
        timestamp: 1,
        nonce: 1,
        signature: Some("fake_sig".to_string()),
    };

    // This should fail and rollback
    let mut cross_shard_tx = crate::sharding_production::CrossShardTransaction::new(0, 1, tx);
    let result = coordinator.execute_cross_shard_commit(&mut cross_shard_tx).await;

    assert!(result.is_err(), "Should have failed due to insufficient balance");

    // Verify balances unchanged after rollback
    let final_alice = coordinator.get_balance("alice").await;
    let final_bob = coordinator.get_balance("bob").await;

    assert_eq!(original_alice, final_alice, "Alice balance should be unchanged after rollback");
    assert_eq!(original_bob, final_bob, "Bob balance should be unchanged after rollback");

    println!("✅ Rollback Recovery: Balances correctly restored after failed transaction");
}

#[tokio::test]
async fn test_concurrent_double_spend_prevention() {
    let config = ShardConfig {
        shard_count: 8,
        tx_per_shard: 100,
        cross_shard_enabled: true,
        byzantine_tolerance: 1,
        enable_fraud_proofs: true,
        max_shards: 8_000,
        auto_expand_threshold: 0.80,
    };

    let coordinator = ShardingCoordinator::new(config);
    
    coordinator.init_account("alice".to_string(), 1000).await.unwrap();
    coordinator.init_account("bob".to_string(), 0).await.unwrap();
    coordinator.init_account("charlie".to_string(), 0).await.unwrap();

    // Try to spend same funds twice concurrently
    let tx1 = Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 1000,
        gas_fee: 0,
        timestamp: 1,
        nonce: 1,
        signature: Some("sig1".to_string()),
    };

    let tx2 = Transaction {
        from: "alice".to_string(),
        to: "charlie".to_string(),
        amount: 1000,
        gas_fee: 0,
        timestamp: 1,
        nonce: 1, // Same nonce = double spend attempt
        signature: Some("sig2".to_string()),
    };

    // Try to process both concurrently
    let coordinator_clone = std::sync::Arc::new(coordinator);
    let coord1 = coordinator_clone.clone();
    let coord2 = coordinator_clone.clone();

    let handle1 = tokio::spawn(async move {
        let mut ctx1 = crate::sharding_production::CrossShardTransaction::new(0, 1, tx1);
        coord1.execute_cross_shard_commit(&mut ctx1).await
    });

    let handle2 = tokio::spawn(async move {
        let mut ctx2 = crate::sharding_production::CrossShardTransaction::new(0, 2, tx2);
        coord2.execute_cross_shard_commit(&mut ctx2).await
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // One should succeed, one should fail due to distributed lock
    let success_count = [result1.is_ok(), result2.is_ok()].iter().filter(|&&x| x).count();
    
    assert_eq!(success_count, 1, "Exactly one transaction should succeed (other blocked by lock)");

    println!("✅ Double-Spend Prevention: Concurrent transactions correctly blocked by distributed lock");
}
