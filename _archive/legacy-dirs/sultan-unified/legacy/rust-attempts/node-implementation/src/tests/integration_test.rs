#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_zero_gas_fees() {
        let tx = Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            gas_fee: 0,
            timestamp: 0,
        };
        assert_eq!(tx.gas_fee, 0, "Gas fees must be zero!");
    }
    
    #[test]
    fn test_block_creation_with_zero_fees() {
        let mut blockchain = Blockchain::new();
        let tx = Transaction::new("sultan".to_string(), "user".to_string(), 1000);
        assert_eq!(tx.gas_fee, 0, "Transaction gas fee should be zero");
        
        blockchain.add_transaction(tx);
        let block = blockchain.create_block();
        
        assert_eq!(block.transactions[0].gas_fee, 0, "Block transactions should have zero gas");
    }
    
    #[test]
    fn test_blockchain_persistence() {
        let mut blockchain = Blockchain::new();
        
        // Add multiple transactions
        for i in 0..10 {
            let tx = Transaction::new(
                format!("user{}", i),
                format!("user{}", i+1),
                100 * i
            );
            blockchain.add_transaction(tx);
        }
        
        let block = blockchain.create_block();
        assert!(block.index > 0);
        assert_eq!(block.transactions.len(), 10);
        
        // Verify all have zero gas
        for tx in &block.transactions {
            assert_eq!(tx.gas_fee, 0);
        }
    }
}
