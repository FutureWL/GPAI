use gpai_core_common::ModuleRegistry;
use gpai_core_market::repo::QuoteRepo;
use gpai_core_market::service::MarketServiceImpl;
use gpai_proto_gen::gpai::market::v1::market_data_service_server::MarketDataServiceServer;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://gpai:gpai@localhost:5432/gpai".into());
    let pool = PgPoolOptions::new().connect(&database_url).await?;

    let registry = ModuleRegistry::new();
    let listener = registry.register("market").await?;

    let repo = Arc::new(QuoteRepo::new(pool));
    let svc = MarketServiceImpl::new(repo);

    tracing::info!(addr = %listener.local_addr()?, "MarketDataService starting");
    Server::builder()
        .add_service(MarketDataServiceServer::new(svc))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;
    Ok(())
}
