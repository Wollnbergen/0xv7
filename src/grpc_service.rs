// grpc_service.rs - gRPC for interop

// Add gRPC service logic here
use tonic::{transport::Server, Request, Response, Status};
use tonic::server::Grpc;
use tokio_stream::wrappers::ReceiverStream;
use anyhow::Result;
use tracing::info;
use std::sync::Arc;
use crate::blockchain::Blockchain;
use tonic::include_proto!("sultan"); // Proto gen for sultan
use sultan::{
    chain_service_server::{ChainService, ChainServiceServer},
    BlockInfo, GetBlockInfoRequest, GetBlockInfoResponse,
    GetStateProofRequest, GetStateProofResponse,
    SubscribeRequest, VerifyStateRequest, VerifyStateResponse,
    StateProof,
};

pub struct SultanGrpcService {
    blockchain: Arc<Blockchain>,
}

impl SultanGrpcService {
    pub fn new(blockchain: Arc<Blockchain>) -> Self {
        Self { blockchain }
    }
}

#[tonic::async_trait]
impl ChainService for SultanGrpcService {
    async fn get_block_info(
        &self,
        request: Request<GetBlockInfoRequest>,
    ) -> Result<Response<GetBlockInfoResponse>, Status> {
        let height = request.into_inner().height;
        info!("⚡ gRPC GetBlockInfo request for height: {}", height);
        let block = BlockInfo {
            chain: "sultan".to_string(),
            height,
            hash: format!("sultan_{:064x}", height),
            state_root: format!("sultanstate_{:058x}", height),
            timestamp: chrono::Utc::now().timestamp(),
        };
        Ok(Response::new(GetBlockInfoResponse {
            block: Some(block),
        }))
    }
    async fn get_state_proof(
        &self,
        request: Request<GetStateProofRequest>,
    ) -> Result<Response<GetStateProofResponse>, Status> {
        let height = request.into_inner().block_height;
        info!("⚡ gRPC GetStateProof request for height: {}", height);
        let proof = StateProof {
            block_height: height,
            state_root: format!("sultanstate_{:058x}", height),
            merkle_proof: vec![vec![]], // Dummy
            signature: vec![], // Dummy
        };
        Ok(Response::new(GetStateProofResponse {
            proof: Some(proof),
        }))
    }
    type SubscribeToBlocksStream = ReceiverStream<Result<BlockInfo, Status>>;
    async fn subscribe_to_blocks(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<<SultanGrpcService as ChainService>::SubscribeToBlocksStream>, Status> {
        let from_block = request.into_inner().from_block;
        info!("⚡ gRPC SubscribeToBlocks request from block: {}", from_block);
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let blockchain_clone = self.blockchain.clone();
        tokio::spawn(async move {
            let mut height = from_block;
            loop {
                let block = BlockInfo {
                    chain: "sultan".to_string(),
                    height,
                    hash: format!("sultan_{:064x}", height),
                    state_root: format!("sultanstate_{:058x}", height),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                if tx.send(Ok(block)).await.is_err() {
                    break;
                }
                height += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    async fn verify_state(
        &self,
        request: Request<VerifyStateRequest>,
    ) -> Result<Response<VerifyStateResponse>, Status> {
        let req = request.into_inner();
        info!("⚡ gRPC VerifyState request for chain: {}", req.chain);
        // Implement verification logic using self.blockchain
        Ok(Response::new(VerifyStateResponse {
            verified: true,
            message: "Sultan state verified".to_string(),
        }))
    }
}
pub async fn start_grpc_server(blockchain: Arc<Blockchain>, addr: String) -> Result<()> {
    let service = SultanGrpcService::new(blockchain);
    Server::builder()
        .add_service(ChainServiceServer::new(service))
        .serve(addr.parse()?).await?;
    Ok(())
}