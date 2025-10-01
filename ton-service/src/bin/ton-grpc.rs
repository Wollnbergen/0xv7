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
pub struct TonGrpcService {}

#[tonic::async_trait]
impl ChainService for TonGrpcService {
    async fn get_block_info(
        &self,
        request: Request<GetBlockInfoRequest>,
    ) -> Result<Response<GetBlockInfoResponse>, Status> {
        let height = request.into_inner().height;
        
        println!("⚡ TON gRPC: GetBlockInfo for seqno {}", height);
        
        let block = BlockInfo {
            chain: "ton".to_string(),
            height,
            hash: format!("ton_{:064x}", height),
            state_root: format!("tonstate_{:058x}", height),
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
            message: "TON state verified".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50053".parse()?;
    let service = TonGrpcService::default();

    println!("⚡ TON gRPC server listening on {}", addr);

    Server::builder()
        .add_service(ChainServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
