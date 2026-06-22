//! Market-server 二进制入口
//!
//! 默认行为:
//! 1. 通过 `DATABASE_URL` 拉行情池(connect_lazy,数据库短暂不可用不阻塞启动)
//! 2. 绑定 `MARKET_ADDR`(默认 `127.0.0.1:50051`)
//! 3. 起 gRPC `MarketDataService`
//!
//! 环境变量:
//! * `DATABASE_URL` — Postgres 连接串
//! * `MARKET_ADDR` — 监听地址,默认 `127.0.0.1:50051`

use gpai_core_market::repo::QuoteRepo;
use gpai_core_market::service::MarketServiceImpl;
use gpai_proto_gen::gpai::market::v1::market_data_service_server::MarketDataServiceServer;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Lazy Postgres pool — 启动时 DB 还没 ready 不会 panic
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://gpai:gpai@localhost:5432/gpai".into());
    let pg_opts = match PgConnectOptions::from_str(&database_url) {
        Ok(opts) => opts,
        Err(e) => {
            tracing::warn!(error = %e, url = %database_url, "invalid DATABASE_URL; falling back to lazy pool with default options");
            PgConnectOptions::new()
        }
    };
    let pool = PgPoolOptions::new().connect_lazy_with(pg_opts);

    // 绑定 MARKET_ADDR(默认 127.0.0.1:50051)
    let market_addr = std::env::var("MARKET_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:50051".into());
    let listener = tokio::net::TcpListener::bind(&market_addr).await?;

    let repo = Arc::new(QuoteRepo::new(pool));
    let svc = MarketServiceImpl::new(repo);

    tracing::info!(addr = %listener.local_addr()?, "MarketDataService starting");
    Server::builder()
        .add_service(MarketDataServiceServer::new(svc))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;
    Ok(())
}
