use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Sultan SDK with default config
    let config = ChainConfig::default();
    let sdk = SultanSDK::new(config, None).await?;
    
    println!("🚀 Sultan SDK Example: Basic Wallet Operations");
    println!("===============================================\n");
    
    // Create wallets
    println!("Creating wallets...");
    let alice = sdk.create_wallet("alice").await?;
    let bob = sdk.create_wallet("bob").await?;
    println!("✓ Alice: {}", alice);
    println!("✓ Bob: {}\n", bob);
    
    // Check balances
    let alice_balance = sdk.get_balance(&alice).await?;
    let bob_balance = sdk.get_balance(&bob).await?;
    println!("Initial balances:");
    println!("  Alice: {} SLTN", alice_balance);
    println!("  Bob: {} SLTN\n", bob_balance);
    
    // Transfer with zero fees
    println!("Transferring 500 SLTN from Alice to Bob (ZERO FEES)...");
    let tx_hash = sdk.transfer(&alice, &bob, 500).await?;
    println!("✓ Transaction hash: {}\n", tx_hash);
    
    // Check updated balances
    let alice_balance = sdk.get_balance(&alice).await?;
    let bob_balance = sdk.get_balance(&bob).await?;
    println!("Updated balances:");
    println!("  Alice: {} SLTN", alice_balance);
    println!("  Bob: {} SLTN\n", bob_balance);
    
    // Query block height
    let height = sdk.get_block_height().await?;
    println!("Current block height: {}", height);
    
    // Transaction count
    let tx_count = sdk.get_transaction_count(&alice).await?;
    println!("Alice's transaction count: {}\n", tx_count);
    
    println!("✅ Example completed successfully!");
    
    Ok(())
}
