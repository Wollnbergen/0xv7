use sultan_coordinator::sdk::SultanSDK;
use sultan_coordinator::ChainConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut sdk = SultanSDK::new(
        ChainConfig {
            inflation_rate: 8.0,
            total_supply: 0,
            min_stake: 5000,
            shards: 8,
        },
        Some("127.0.0.1:9042"),
    )
    .await?;
    sdk.stake("validator_1", 5000).await?;
    sdk.query_apy(true).await?;
    sdk.cross_chain_swap("bitcoin", 1000).await?;
    sdk.mint_token("sultan1user123", 1000).await?;
    sdk.create_wallet("user123").await?;
    sdk.vote_on_proposal("42", true, "validator_1").await?;
    sdk.get_balance("sultan1user123").await?;
    Ok(())
}
