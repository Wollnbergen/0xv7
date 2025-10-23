// filepath: /workspaces/0xv7/sultan-interop/bitcoin-service/src/main.rs
use anyhow::Result;
use sultan_interop::bitcoin::BitcoinBridge;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ bitcoin-service up");

    // Wire into the library module at src/bitcoin.rs
    let _bridge = BitcoinBridge::new().await?;
    // Optional dry-run; comment out if you don’t want it invoked on start

    // Keep the service alive until Ctrl+C
    signal::ctrl_c().await?;
    Ok(())
}
