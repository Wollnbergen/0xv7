use sultan_coordinator::blockchain::ChainConfig;
use sultan_coordinator::rpc_server::RpcServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           SULTAN CHAIN RPC SERVER v1.0                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    let config = ChainConfig::default();
    let _server = RpcServer::new(config).await?;
    
    println!("🌐 RPC Server running on port 26657");
    println!("✅ Ready to accept requests");
    
    // For now, just keep running
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");
    
    Ok(())
}
