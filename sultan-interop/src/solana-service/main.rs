use sultan_interop::sultan::{
    chain_service_server::{ChainService, ChainServiceServer},
    VerifyStateRequest, VerifyStateResponse,
};
use tonic::transport::Server;
use tracing::info;

use futures::Stream;
use std::pin::Pin;
use sultan_interop::sultan::{
    BlockInfo, GetBlockInfoRequest, GetBlockInfoResponse, GetStateProofRequest,
    GetStateProofResponse, SubscribeRequest,
};
use tonic::Status;

#[derive(Debug, Default)]
pub struct SolanaService;

#[tonic::async_trait]
impl ChainService for SolanaService {
    async fn verify_state(
        &self,
        request: tonic::Request<VerifyStateRequest>,
    ) -> Result<tonic::Response<VerifyStateResponse>, tonic::Status> {
        let req = request.into_inner();
        info!("gRPC VerifyState request for chain: {}", req.chain);
        Ok(tonic::Response::new(VerifyStateResponse {
            verified: true,
            message: "".to_string(),
        }))
    }

    async fn get_block_info(
        &self,
        _request: tonic::Request<GetBlockInfoRequest>,
    ) -> Result<tonic::Response<GetBlockInfoResponse>, Status> {
        todo!()
    }

    async fn get_state_proof(
        &self,
        _request: tonic::Request<GetStateProofRequest>,
    ) -> Result<tonic::Response<GetStateProofResponse>, Status> {
        todo!()
    }

    type SubscribeToBlocksStream = Pin<Box<dyn Stream<Item = Result<BlockInfo, Status>> + Send>>;

    async fn subscribe_to_blocks(
        &self,
        _request: tonic::Request<SubscribeRequest>,
    ) -> Result<tonic::Response<Self::SubscribeToBlocksStream>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🚀 Initializing Solana Service with gRPC");

    let addr_str =
        std::env::var("SOLANA_GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50052".to_string());
    let addr = addr_str.parse()?;

    info!("⚡ Starting Solana gRPC server on {}", addr);
    let service = SolanaService; // unit struct, no Default call needed

    Server::builder()
        .add_service(ChainServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
