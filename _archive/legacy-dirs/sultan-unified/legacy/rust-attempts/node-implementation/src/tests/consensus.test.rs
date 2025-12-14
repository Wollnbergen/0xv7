fn test_consensus_mechanism() {
    // Setup the initial state of the blockchain
    let mut blockchain = Blockchain::new();

    // Test scenario 1: Valid block addition
    let block = Block::new(1, vec!["transaction1".to_string()]);
    assert!(blockchain.add_block(block.clone()).is_ok());
    assert_eq!(blockchain.get_latest_block().index, 1);

    // Test scenario 2: Invalid block addition (duplicate index)
    let result = blockchain.add_block(block);
    assert!(result.is_err());

    // Test scenario 3: Consensus after a fork
    let block2 = Block::new(2, vec!["transaction2".to_string()]);
    let block3 = Block::new(3, vec!["transaction3".to_string()]);
    blockchain.add_block(block2).unwrap();
    blockchain.add_block(block3).unwrap();

    // Simulate a fork
    let fork_block = Block::new(2, vec!["fork_transaction".to_string()]);
    assert!(blockchain.add_block(fork_block).is_err());

    // Validate consensus mechanism
    blockchain.resolve_conflicts();
    assert_eq!(blockchain.get_latest_block().index, 3);
}