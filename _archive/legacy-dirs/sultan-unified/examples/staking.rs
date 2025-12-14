use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ChainConfig::default();
    config.min_stake = 10000;
    config.inflation_rate = 0.08; // 8% APY
    
    let sdk = SultanSDK::new(config, None).await?;
    
    println!("💰 Sultan SDK Example: Staking & Rewards");
    println!("=========================================\n");
    
    // Query APY rates
    println!("Current APY Rates:");
    let validator_apy = sdk.query_apy(true).await?;
    let delegator_apy = sdk.query_apy(false).await?;
    println!("  Validator: {:.2}%", validator_apy * 100.0);
    println!("  Delegator: {:.2}%\n", delegator_apy * 100.0);
    
    // Create validator wallet
    let validator = sdk.create_wallet("my_validator").await?;
    println!("Validator wallet: {}", validator);
    
    // Stake tokens
    let stake_amount = 50000;
    println!("\nStaking {} SLTN...", stake_amount);
    match sdk.stake(&validator, stake_amount).await {
        Ok(_) => println!("✓ Successfully staked!"),
        Err(e) => println!("✗ Staking failed: {}", e),
    }
    
    // Try staking below minimum (should fail)
    println!("\nAttempting to stake below minimum (5000 SLTN)...");
    match sdk.stake(&validator, 5000).await {
        Ok(_) => println!("✗ Unexpected success"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }
    
    // Register as validator
    println!("\nRegistering validator...");
    let result = sdk.validator_register(&validator, stake_amount).await?;
    println!("✓ {}", result);
    
    println!("\n✅ Staking example completed!");
    
    Ok(())
}
