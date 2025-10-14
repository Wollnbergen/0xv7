#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scylladb_connection() {
        // Debug: Print before and after DB connection
        println!("Connecting to ScyllaDB for test...");
        let result = scylla_db::ScyllaCluster::new(
            vec!["127.0.0.1:9042"],
            "test_keyspace"
        ).await;
        println!("ScyllaDB session creation result: {:?}", result);

        assert!(result.is_ok(), "ScyllaDB connection should succeed");
    }

    #[tokio::test]
    async fn test_load_basic() {
        // Basic load test
        let config = Config::default();
        let blockchain = Arc::new(SultanBlockchain::new(config).await.unwrap());

        let tester = load_testing::LoadTester::new(
            blockchain,
            100,  // 100 TPS
            10,   // 10 seconds
            5,    // 5 concurrent users
        );

        let report = tester.run_test().await.unwrap();
        assert!(report.actual_tps > 0);
    }
}