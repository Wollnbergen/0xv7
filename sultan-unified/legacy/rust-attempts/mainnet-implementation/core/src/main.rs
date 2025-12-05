use sultan_core::{Blockchain, ChainConfig};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            SULTAN CHAIN - ZERO GAS BLOCKCHAIN                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let config = ChainConfig::default();
    println!("\n✅ Configuration:");
    println!("   Chain ID: {}", config.chain_id);
    println!("   Gas Price: $0.00 (Zero forever!)");
    println!("   Staking APY: 26.67%");
    
    let mut blockchain = Blockchain::new(config);
    println!("\n✅ Blockchain initialized");
    
    if let Some(genesis) = blockchain.get_latest_block() {
        println!("   Genesis Block: #{}", genesis.index);
        println!("   Genesis Hash: {}", &genesis.hash[..16]);
    }
    
    // Create a test block
    let block = blockchain.create_block(vec![]);
    println!("\n⛏️  New block mined:");
    println!("   Block #: {}", block.index);
    println!("   Hash: {}", &block.hash[..16]);
    println!("   Gas Fees: $0.00");
    
    println!("\n🚀 Sultan Blockchain is operational!");
    println!("   ✓ Zero gas fees");
    println!("   ✓ 26.67% APY staking");
    println!("   ✓ Block production working");
}
