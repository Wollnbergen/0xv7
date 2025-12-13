// filepath: /workspaces/0xv7/node/src/bin/db_seed.rs
use anyhow::Result;
use sultan_coordinator::sdk::SultanSDK;
use sultan_coordinator::ChainConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let db = std::env::var("SULTAN_DB_ADDR").ok();
    let sdk = SultanSDK::new(
        ChainConfig { inflation_rate: 8.0, total_supply: 0, min_stake: 5000, shards: 8 },
        db.as_deref(),
    ).await?;

    sdk.validator_register(
        "validator_1",
        "0x1111111111111111111111111111111111111111",
        0,
        Some("mobile"),
    ).await?;

    sdk.proposal_create("p1", Some("Param change"), Some("Test proposal"), Some("open")).await?;

    println!("Seeded validator_1 and proposal p1");
    Ok(())
}
