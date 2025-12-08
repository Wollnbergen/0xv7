mod test {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = create_block("Test Data");
        assert_eq!(block.data, "Test Data");
        assert!(block.hash.len() > 0);
    }

    #[test]
    fn test_blockchain_integration() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("First Block");
        assert_eq!(blockchain.blocks.len(), 1);
        assert_eq!(blockchain.blocks[0].data, "First Block");
    }

    #[test]
    fn test_blockchain_persistence() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("Second Block");
        let serialized = blockchain.serialize();
        let deserialized = Blockchain::deserialize(&serialized);
        assert_eq!(deserialized.blocks.len(), 1);
        assert_eq!(deserialized.blocks[0].data, "Second Block");
    }

    #[test]
    fn test_chain_validation() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block("Third Block");
        assert!(blockchain.is_valid());
    }
}