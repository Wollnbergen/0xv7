use sultan::chain_service_server::ChainServiceServer;
use sultan::interoperability_service_server::InteroperabilityServiceServer;
use sultan::event_service_server::EventServiceServer;
use tonic::transport::Server;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chain_impl = ChainService::new();
    let interop_impl = InteroperabilityService::new();
    let event_impl = EventService::new();

    let addr: std::net::SocketAddr = "0.0.0.0:50052".parse()?;

    Server::builder()
        .add_service(ChainServiceServer::new(chain_impl.clone()))
        .add_service(InteroperabilityServiceServer::new(interop_impl.clone()))
        .add_service(EventServiceServer::new(event_impl.clone()))
        .serve(addr)
        .await?;
    Ok(())
}