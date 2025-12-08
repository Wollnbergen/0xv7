use anyhow::Result;
use sultan_coordinator::sdk::SultanSDK;
use sultan_coordinator::blockchain::ChainConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Sultan Chain SDK Demo");
    println!("========================\n");

    let config = ChainConfig::default();
    let sdk = SultanSDK::new(config, None).await?;

    // Create wallets
    println!("Creating wallets...");
    let alice_wallet = sdk.create_wallet("alice").await?;
    println!("✅ Alice wallet: {}", alice_wallet);

    // Check balance
    let balance = sdk.get_balance("alice").await?;
    println!("💰 Alice balance: {} SLTN", balance);

    // Transfer tokens (ZERO FEES!)
    println!("\n🔄 Transferring 100 SLTN from alice to bob (Fee: $0.00)");
    let tx_hash = sdk.transfer("alice", "bob", 100).await?;
    println!("✅ Transaction hash: {}", tx_hash);

    // Create governance proposal
    println!("\n📋 Creating governance proposal...");
    let proposal_id = sdk.proposal_create(
        "validator_1",
        "Reduce block time to 3 seconds",
        "This proposal aims to improve network speed"
    ).await?;
    println!("✅ Proposal ID: {}", proposal_id);

    // Vote on proposal - note the correct parameter order
    println!("🗳️ Voting on proposal...");
    sdk.vote_on_proposal(proposal_id, "validator_1", true).await?;
    println!("✅ Vote submitted");

    // Check APY
    let apy = sdk.query_apy(true).await?;
    println!("\n💎 Validator APY: {:.2}%", apy * 100.0);

    println!("\n✅ Sultan SDK Demo Complete!");
    Ok(())
}
