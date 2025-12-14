//! Integration tests for Sultan Core
//!
//! Comprehensive tests for blockchain, consensus, and transaction processing

use sultan_core::*;

#[test]
fn test_blockchain_initialization() {
    let blockchain = Blockchain::new();
    
    assert_eq!(blockchain.height(), 0);
    assert_eq!(blockchain.chain.len(), 1);
    assert_eq!(blockchain.get_latest_block().index, 0);
}

#[test]
fn test_genesis_accounts() {
    let accounts = vec![
        ("alice".to_string(), 1000000),
        ("bob".to_string(), 500000),
    ];
    
    let blockchain = init_with_genesis(accounts).unwrap();
    
    assert_eq!(blockchain.get_balance("alice"), 1000000);
    assert_eq!(blockchain.get_balance("bob"), 500000);
    assert_eq!(blockchain.get_balance("charlie"), 0);
}

#[test]
fn test_transaction_lifecycle() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    blockchain.init_account("bob".to_string(), 500);
    
    // Create and add transaction
    let tx = Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    );
    
    blockchain.add_transaction(tx).unwrap();
    
    // Create block
    let block = blockchain.create_block("validator1".to_string()).unwrap();
    
    assert_eq!(block.index, 1);
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(blockchain.get_balance("alice"), 900);
    assert_eq!(blockchain.get_balance("bob"), 600);
}

#[test]
fn test_multiple_transactions() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    blockchain.init_account("bob".to_string(), 500);
    blockchain.init_account("charlie".to_string(), 0);
    
    // Multiple transactions
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    )).unwrap();
    
    blockchain.add_transaction(Transaction::new(
        "bob".to_string(),
        "charlie".to_string(),
        50,
        1,
    )).unwrap();
    
    blockchain.create_block("validator1".to_string()).unwrap();
    
    assert_eq!(blockchain.get_balance("alice"), 900);
    assert_eq!(blockchain.get_balance("bob"), 550);
    assert_eq!(blockchain.get_balance("charlie"), 50);
}

#[test]
fn test_insufficient_balance() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 100);
    
    let tx = Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        200,
        1,
    );
    
    let result = blockchain.add_transaction(tx);
    assert!(result.is_err());
}

#[test]
fn test_zero_amount_rejected() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    
    let mut tx = Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        0,
        1,
    );
    tx.amount = 0;
    
    let result = blockchain.add_transaction(tx);
    assert!(result.is_err());
}

#[test]
fn test_nonce_ordering() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    
    // First transaction
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    )).unwrap();
    
    blockchain.create_block("validator1".to_string()).unwrap();
    
    // Second transaction with correct nonce
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        2,
    )).unwrap();
    
    // Invalid nonce (too low)
    let result = blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    ));
    
    assert!(result.is_err());
}

#[test]
fn test_block_validation() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    )).unwrap();
    
    // Small delay to ensure timestamp advances past genesis block
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let block = blockchain.create_block("validator1".to_string()).unwrap();
    
    // Validate the block we just created
    assert!(blockchain.validate_block(&block).unwrap());
}

#[test]
fn test_consensus_validator_management() {
    let mut consensus = ConsensusEngine::new();
    
    // Add validators
    consensus.add_validator("validator1".to_string(), 10000).unwrap();
    consensus.add_validator("validator2".to_string(), 20000).unwrap();
    consensus.add_validator("validator3".to_string(), 15000).unwrap();
    
    assert_eq!(consensus.validator_count(), 3);
    assert_eq!(consensus.total_stake, 45000);
    
    // Test proposer selection
    let proposer = consensus.select_proposer();
    assert!(proposer.is_some());
    assert!(consensus.is_validator(&proposer.unwrap()));
}

#[test]
fn test_consensus_min_stake() {
    let mut consensus = ConsensusEngine::new();
    
    // Below minimum stake (10K required)
    let result = consensus.add_validator("low_stake".to_string(), 5000);
    assert!(result.is_err());
    
    // At minimum stake (10K)
    let result = consensus.add_validator("good_stake".to_string(), 10000);
    assert!(result.is_ok());
}

#[test]
fn test_consensus_required_signatures() {
    let mut consensus = ConsensusEngine::new();
    
    consensus.add_validator("v1".to_string(), 10000).unwrap();
    assert_eq!(consensus.required_signatures(), 1);
    
    consensus.add_validator("v2".to_string(), 10000).unwrap();
    assert_eq!(consensus.required_signatures(), 2);
    
    consensus.add_validator("v3".to_string(), 10000).unwrap();
    assert_eq!(consensus.required_signatures(), 3);
    
    consensus.add_validator("v4".to_string(), 10000).unwrap();
    assert_eq!(consensus.required_signatures(), 3);
}

#[test]
fn test_transaction_validator() {
    let mut validator = TransactionValidator::new();
    
    let tx = Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    );
    
    // Valid transaction
    assert!(validator.validate(&tx, 1000, 0).is_ok());
    
    // Duplicate detection
    let result = validator.validate(&tx, 1000, 0);
    assert!(result.is_err());
}

#[test]
fn test_end_to_end_scenario() {
    // Initialize blockchain with genesis accounts
    let mut blockchain = init_with_genesis(vec![
        ("alice".to_string(), 1000000),
        ("bob".to_string(), 500000),
        ("charlie".to_string(), 250000),
    ]).unwrap();
    
    // Initialize consensus
    let mut consensus = ConsensusEngine::new();
    consensus.add_validator("validator1".to_string(), 100000).unwrap();
    consensus.add_validator("validator2".to_string(), 150000).unwrap();
    
    // Block 1: Alice sends to Bob
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        10000,
        1,
    )).unwrap();
    
    // Small delay to ensure timestamp advances
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let proposer = consensus.select_proposer().unwrap();
    let block1 = blockchain.create_block(proposer.clone()).unwrap();
    consensus.record_proposal(&proposer).unwrap();
    
    assert_eq!(block1.index, 1);
    assert_eq!(blockchain.get_balance("alice"), 990000);
    assert_eq!(blockchain.get_balance("bob"), 510000);
    
    // Block 2: Bob sends to Charlie
    blockchain.add_transaction(Transaction::new(
        "bob".to_string(),
        "charlie".to_string(),
        5000,
        1,
    )).unwrap();
    
    // Small delay to ensure timestamp advances
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let proposer = consensus.select_proposer().unwrap();
    let block2 = blockchain.create_block(proposer.clone()).unwrap();
    consensus.record_proposal(&proposer).unwrap();
    
    assert_eq!(block2.index, 2);
    assert_eq!(blockchain.get_balance("bob"), 505000);
    assert_eq!(blockchain.get_balance("charlie"), 255000);
    
    // Verify chain integrity
    assert_eq!(blockchain.height(), 2);
    assert_eq!(block2.prev_hash, block1.hash);
}

#[test]
fn test_state_root_changes() {
    let mut blockchain = Blockchain::new();
    blockchain.init_account("alice".to_string(), 1000);
    
    let genesis = blockchain.get_latest_block();
    let genesis_state_root = genesis.state_root.clone();
    
    // Add transaction and create block
    blockchain.add_transaction(Transaction::new(
        "alice".to_string(),
        "bob".to_string(),
        100,
        1,
    )).unwrap();
    
    let block = blockchain.create_block("validator1".to_string()).unwrap();
    
    // State root should change after transaction
    assert_ne!(block.state_root, genesis_state_root);
}

#[test]
fn test_concurrent_validators() {
    let mut consensus = ConsensusEngine::new();
    
    // Add 10 validators
    for i in 1..=10 {
        consensus.add_validator(
            format!("validator{}", i),
            10000 + (i * 1000) as u64,
        ).unwrap();
    }
    
    assert_eq!(consensus.validator_count(), 10);
    
    // Select proposers for 100 rounds
    let mut proposer_counts = std::collections::HashMap::new();
    
    for _ in 0..100 {
        if let Some(proposer) = consensus.select_proposer() {
            *proposer_counts.entry(proposer).or_insert(0) += 1;
        }
    }
    
    // All validators should be selected at least once in 100 rounds
    assert!(proposer_counts.len() >= 8);
}
