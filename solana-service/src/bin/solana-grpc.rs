use tonic::{transport::Server, Request, Response, Status};

pub mod sultan {
    tonic::include_proto!("sultan");
}

use sultan::{
    chain_service_server::{ChainService, ChainServiceServer},
    BlockInfo, GetBlockInfoRequest, GetBlockInfoResponse,
    GetStateProofRequest, GetStateProofResponse,
    SubscribeRequest, VerifyStateRequest, VerifyStateResponse,
    StateProof,
};

#[derive(Debug, Default)]
pub struct SolanaGrpcService {}

#[tonic::async_trait]
impl ChainService for SolanaGrpcService {
    async fn get_block_info(
        &self,
        request: Request<GetBlockInfoRequest>,
    ) -> Result<Response<GetBlockInfoResponse>, Status> {
        let height = request.into_inner().height;
        
        println!("⚡ Solana gRPC: GetBlockInfo for slot {}", height);
        
        let block = BlockInfo {
            chain: "solana".to_string(),
            height,
            hash: format!("sol_{:064x}", height),
            state_root: format!("solstate_{:058x}", height),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        Ok(Response::new(GetBlockInfoResponse {
            block: Some(block),
        }))
    }

    async fn get_state_proof(
        &self,
        _request: Request<GetStateProofRequest>,
    ) -> Result<Response<GetStateProofResponse>, Status> {
        Ok(Response::new(GetStateProofResponse {
            proof: Some(StateProof::default()),
        }))
    }

    type SubscribeToBlocksStream = tokio_stream::wrappers::ReceiverStream<Result<BlockInfo, Status>>;

    async fn subscribe_to_blocks(
        &self,
        _request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeToBlocksStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        drop(tx);
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn verify_state(
        &self,
        _request: Request<VerifyStateRequest>,
    ) -> Result<Response<VerifyStateResponse>, Status> {
        Ok(Response::new(VerifyStateResponse {
            verified: true,
            message: "Solana state verified".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50052".parse()?;
    let service = SolanaGrpcService::default();

    println!("⚡ Solana gRPC server listening on {}", addr);

    Server::builder()
        .add_service(ChainServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
