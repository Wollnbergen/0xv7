use anyhow::Result;
use sultan_interop::sultan::chain_service_client::ChainServiceClient;
use sultan_interop::sultan::VerifyStateRequest;
use tonic::transport::Channel;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Initializing gRPC client for verify_state test");
    let channel = Channel::from_static("http://localhost:50054").connect().await?;
    let mut client = ChainServiceClient::new(channel);
    let request = tonic::Request::new(VerifyStateRequest {
        chain: "bitcoin".to_string(),
        proof: None, // For test, send None; for real, send Some(StateProof)
    });
    let response = client.verify_state(request).await?;
    info!("State verified successfully: {:?}", response.into_inner());
    Ok(())
}
