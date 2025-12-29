//! Integration tests for sharding implementation
//! Uses production sharding code (sharding_production.rs, sharded_blockchain_production.rs)

#[cfg(test)]
mod sharding_tests {
    use sultan_core::{ShardedBlockchain, ShardConfig, Transaction, SultanBlockchain};
    use sultan_core::sharding_production::ShardConfig as ProdShardConfig;

    #[tokio::test]
    async fn test_sharding_initialization() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 1000,
            cross_shard_enabled: false,
        };

        let blockchain = ShardedBlockchain::new(config.clone());
        
        assert_eq!(blockchain.get_height(), 0);
        assert_eq!(blockchain.sharding.config.shard_count, 10);
        assert_eq!(blockchain.sharding.config.tx_per_shard, 1000);
        
        let stats = blockchain.get_stats();
        assert_eq!(stats.shard_count, 10);
        
        println!("✅ Sharding initialization test passed");
    }

    #[tokio::test]
    async fn test_account_initialization() {
        let config = ShardConfig::default();
        let blockchain = ShardedBlockchain::new(config);
        
        // Initialize accounts
        blockchain.init_account("alice".to_string(), 1_000_000).unwrap();
        blockchain.init_account("bob".to_string(), 500_000).unwrap();
        
        // Check balances
        assert_eq!(blockchain.get_balance("alice"), 1_000_000);
        assert_eq!(blockchain.get_balance("bob"), 500_000);
        assert_eq!(blockchain.get_balance("charlie"), 0);
        
        println!("✅ Account initialization test passed");
    }

    #[tokio::test]
    async fn test_transaction_processing() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 1000,
            cross_shard_enabled: false,
        };
        
        let blockchain = ShardedBlockchain::new(config);
        
        // Initialize accounts
        blockchain.init_account("alice".to_string(), 1_000_000).unwrap();
        blockchain.init_account("bob".to_string(), 0).unwrap();
        
        // Create transactions
        let mut transactions = vec![];
        for i in 0..100 {
            transactions.push(Transaction {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
                gas_fee: 0,
                timestamp: i,
                nonce: i as u64,
                signature: None,
                public_key: None,
                memo: None,
            });
        }
        
        // Process transactions
        let processed = blockchain.process_transactions(transactions).await.unwrap();
        
        assert_eq!(processed.len(), 100);
        
        println!("✅ Transaction processing test passed (100 txs)");
    }

    #[tokio::test]
    async fn test_parallel_shard_processing() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 1000,
            cross_shard_enabled: false,
        };
        
        let blockchain = ShardedBlockchain::new(config);
        
        // Initialize multiple accounts across different shards
        for i in 0..10 {
            let addr = format!("user{}", i);
            blockchain.init_account(addr, 1_000_000).unwrap();
        }
        
        // Create transactions distributed across shards
        let mut transactions = vec![];
        for i in 0..1000 {
            let from = format!("user{}", i % 10);
            let to = format!("user{}", (i + 1) % 10);
            
            transactions.push(Transaction {
                from,
                to,
                amount: 1,
                gas_fee: 0,
                timestamp: i as u64,
                nonce: i as u64,
                signature: None,
                public_key: None,
                memo: None,
            });
        }
        
        let start = std::time::Instant::now();
        let processed = blockchain.process_transactions(transactions).await.unwrap();
        let duration = start.elapsed();
        
        println!("  Processed {} txs in {:?}", processed.len(), duration);
        println!("  TPS: {:.0}", processed.len() as f64 / duration.as_secs_f64());
        
        assert!(processed.len() > 0);
        
        println!("✅ Parallel shard processing test passed");
    }

    #[tokio::test]
    async fn test_block_creation() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 1000,
            cross_shard_enabled: false,
        };
        
        let mut blockchain = ShardedBlockchain::new(config);
        
        // Initialize accounts
        blockchain.init_account("alice".to_string(), 1_000_000).unwrap();
        blockchain.init_account("bob".to_string(), 0).unwrap();
        
        // Create transactions
        let mut transactions = vec![];
        for i in 0..50 {
            transactions.push(Transaction {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
                gas_fee: 0,
                timestamp: i,
                nonce: i as u64,
                signature: None,
                public_key: None,
                memo: None,
            });
        }
        
        // Create block with transactions
        let block = blockchain.create_block(transactions, "validator1".to_string()).await.unwrap();
        blockchain.add_block(block.clone()).unwrap();
        
        assert_eq!(blockchain.get_height(), 1);
        assert_eq!(block.validator, "validator1");
        
        println!("✅ Block creation test passed");
    }

    #[tokio::test]
    async fn test_tps_capacity() {
        // Production TPS formula: (shard_count × tx_per_shard) / 2 (for 2-second blocks)
        // Using production SultanBlockchain (sharded_blockchain_production.rs)
        let test_configs = vec![
            (10, 10_000, 50_000),    // 10 × 10,000 / 2 = 50,000 TPS
            (50, 10_000, 250_000),   // 50 × 10,000 / 2 = 250,000 TPS
            (100, 10_000, 500_000),  // 100 × 10,000 / 2 = 500,000 TPS
        ];
        
        for (shards, tx_per_shard, expected_tps) in test_configs {
            let config = ProdShardConfig {
                shard_count: shards,
                tx_per_shard,
                max_shards: 8000,
                auto_expand_threshold: 0.8,
                cross_shard_enabled: false,
                byzantine_tolerance: 1,
                enable_fraud_proofs: true,
            };
            
            let blockchain = SultanBlockchain::new(config);
            let tps_capacity = blockchain.get_tps_capacity().await;
            
            assert_eq!(tps_capacity, expected_tps, 
                "TPS mismatch for {} shards × {} tx/shard", shards, tx_per_shard);
            
            println!("  {} shards × {} tx/shard = {} TPS ✅", shards, tx_per_shard, tps_capacity);
        }
        
        println!("✅ TPS capacity test passed (production formula)");
    }

    #[tokio::test]
    async fn test_shard_stats() {
        // Using production SultanBlockchain for stats
        let config = ProdShardConfig {
            shard_count: 100,
            tx_per_shard: 10_000,
            max_shards: 8000,
            auto_expand_threshold: 0.8,
            cross_shard_enabled: true,
            byzantine_tolerance: 1,
            enable_fraud_proofs: true,
        };
        
        let blockchain = SultanBlockchain::new(config);
        
        let stats = blockchain.get_stats().await;
        
        assert_eq!(stats.shard_count, 100);
        // Production formula: 100 × 10,000 / 2 = 500,000 TPS
        assert_eq!(stats.estimated_tps, 500_000);
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.total_processed, 0);
        
        println!("  Shard Count:       {}", stats.shard_count);
        println!("  Estimated TPS:     {}", stats.estimated_tps);
        println!("  Total Processed:   {}", stats.total_processed);
        
        println!("✅ Shard stats test passed (production)");
    }

    #[tokio::test]
    async fn test_high_volume_transactions() {
        let config = ShardConfig {
            shard_count: 10,
            tx_per_shard: 10_000,
            cross_shard_enabled: false,
        };
        
        let blockchain = ShardedBlockchain::new(config);
        
        // Initialize accounts
        blockchain.init_account("sender".to_string(), 10_000_000).unwrap();
        blockchain.init_account("receiver".to_string(), 0).unwrap();
        
        // Create 10,000 transactions
        let mut transactions = vec![];
        for i in 0..10_000 {
            transactions.push(Transaction {
                from: "sender".to_string(),
                to: "receiver".to_string(),
                amount: 1,
                gas_fee: 0,
                timestamp: i,
                nonce: i as u64,
                signature: None,
                public_key: None,
                memo: None,
            });
        }
        
        let start = std::time::Instant::now();
        let processed = blockchain.process_transactions(transactions).await.unwrap();
        let duration = start.elapsed();
        
        let tps = processed.len() as f64 / duration.as_secs_f64();
        
        println!("  Processed:   {} transactions", processed.len());
        println!("  Duration:    {:?}", duration);
        println!("  Actual TPS:  {:.0}", tps);
        
        assert!(processed.len() > 0);
        assert!(tps > 1000.0, "TPS should be > 1000 with sharding");
        
        println!("✅ High volume transaction test passed");
    }
}
