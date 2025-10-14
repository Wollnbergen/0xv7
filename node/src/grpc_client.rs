use anyhow::Result;
use sultan_interop::sultan::chain_service_client::ChainServiceClient;
use sultan_interop::sultan::VerifyStateRequest;
use tonic::transport::Channel;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let chains = vec![
        ("bitcoin", 50054),
        ("ethereum", 50051),
        ("solana", 50052),
        ("ton", 50053),
    ];
    for (chain, port) in chains {
        info!("Initializing gRPC client for {} verify_state test", chain);
        let addr = format!("http://localhost:{}", port);
        match Channel::builder(addr.parse()?).connect().await {
            Ok(channel) => {
                let mut client = ChainServiceClient::new(channel);
                let request = tonic::Request::new(VerifyStateRequest {
                    chain: chain.to_string(),
                    proof: None,
                });
                match client.verify_state(request).await {
                    Ok(response) => {
                        info!(
                            "State verified successfully for {}: {:?}",
                            chain,
                            response.into_inner()
                        );
                    }
                    Err(e) => {
                        info!("Failed to verify state for {}: {}", chain, e);
                    }
                }
            }
            Err(e) => {
                info!("Failed to connect to {} service on {}: {}", chain, addr, e);
            }
        }
    }
    Ok(())
}
