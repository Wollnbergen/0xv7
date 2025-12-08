use sultan_chain::sdk::SultanSDK;
use sultan_chain::config::ChainConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ChainConfig::default();
    let sdk = SultanSDK::new(config, None).await?;
    
    println!("🏛️  Sultan SDK Example: Governance & Voting");
    println!("============================================\n");
    
    // Create a governance proposal
    println!("Creating governance proposal...");
    let proposal_id = sdk.proposal_create(
        "sultan1proposer123",
        "Increase Block Size",
        "Proposal to increase max block size from 1MB to 2MB for better throughput"
    ).await?;
    println!("✓ Proposal created with ID: {}\n", proposal_id);
    
    // Get proposal details
    let proposal = sdk.proposal_get(proposal_id).await?;
    println!("Proposal Details:");
    println!("  ID: {}", proposal["id"]);
    println!("  Title: {}", proposal["title"]);
    println!("  Proposer: {}", proposal["proposer"]);
    println!("  Description: {}", proposal["description"]);
    println!("  Status: {}\n", proposal["status"]);
    
    // Cast votes
    println!("Casting votes...");
    sdk.vote_on_proposal(proposal_id, "validator1", true).await?;
    sdk.vote_on_proposal(proposal_id, "validator2", true).await?;
    sdk.vote_on_proposal(proposal_id, "validator3", false).await?;
    println!("✓ Votes cast\n");
    
    // Tally votes
    let (yes_votes, no_votes) = sdk.votes_tally(proposal_id).await?;
    println!("Vote Tally:");
    println!("  Yes: {}", yes_votes);
    println!("  No: {}", no_votes);
    println!("  Result: {}\n", if yes_votes > no_votes { "PASSING" } else { "FAILING" });
    
    // List all proposals
    let all_proposals = sdk.get_all_proposals().await?;
    println!("All Proposals ({}):", all_proposals.len());
    for p in all_proposals {
        println!("  • {} - {} ({})", p["id"], p["title"], p["status"]);
    }
    
    println!("\n✅ Governance example completed!");
    
    Ok(())
}
