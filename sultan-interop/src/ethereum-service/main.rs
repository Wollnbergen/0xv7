use sultan_interop::sultan::{chain_service_server::{ChainService, ChainServiceServer}, VerifyStateRequest, VerifyStateResponse};
use tonic::transport::Server;
use tracing::info;

use sultan_interop::sultan::{BlockInfo, GetBlockInfoRequest, GetBlockInfoResponse, GetStateProofRequest, GetStateProofResponse, SubscribeRequest};
use futures::Stream;
use std::pin::Pin;
use tonic::Status;

#[derive(Debug, Default)]
pub struct EthereumService;

#[tonic::async_trait]
impl ChainService for EthereumService {
    async fn verify_state(&self, _request: tonic::Request<VerifyStateRequest>) -> Result<tonic::Response<VerifyStateResponse>, tonic::Status> {
        info!("gRPC VerifyState request for chain: ethereum");
        Ok(tonic::Response::new(VerifyStateResponse { verified: true, message: "".to_string() }))
    }

    async fn get_block_info(&self, _request: tonic::Request<GetBlockInfoRequest>) -> Result<tonic::Response<GetBlockInfoResponse>, Status> {
        todo!()
    }

    async fn get_state_proof(&self, _request: tonic::Request<GetStateProofRequest>) -> Result<tonic::Response<GetStateProofResponse>, Status> {
        todo!()
    }

    type SubscribeToBlocksStream = Pin<Box<dyn Stream<Item = Result<BlockInfo, Status>> + Send>>;

    async fn subscribe_to_blocks(&self, _request: tonic::Request<SubscribeRequest>) -> Result<tonic::Response<Self::SubscribeToBlocksStream>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🚀 Initializing Ethereum Service with gRPC");
    let addr = "0.0.0.0:50051".parse()?;
    let service = EthereumService::default();
    info!("⚡ Starting Ethereum gRPC server on {}", addr);
    Server::builder()
        .add_service(ChainServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
