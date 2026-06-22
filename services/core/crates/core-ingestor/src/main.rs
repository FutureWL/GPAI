//! Ingestor 二进制入口
//!
//! 默认行为:
//! 1. 用 `gpai_core_market::source::YahooSource::new()` 拉行情
//! 2. 通过 `gpai_core_common::ModuleRegistry` 拿 market 模块地址
//! 3. 每 30s 拉一次并通过 gRPC 推送给 market
//!
//! 环境变量:
//! * `INGESTOR_POLL_SECS` — 覆盖默认 30s
//! * `INGESTOR_INSTRUMENTS` — 逗号分隔的标的 ID 列表

use std::time::Duration;

use gpai_core_common::ModuleRegistry;
use gpai_core_market::source::yahoo::YahooSource;
use gpai_core_market::DataSource;
use gpai_proto_gen::gpai::market::v1::market_data_service_client::MarketDataServiceClient;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 1. 解析配置
    let poll_interval = Duration::from_secs(
        std::env::var("INGESTOR_POLL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    );
    let ids: Vec<String> = std::env::var("INGESTOR_INSTRUMENTS")
        .unwrap_or_else(|_| "US.AAPL.NASDAQ".into())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 2. 数据源
    let source: Box<dyn DataSource> = Box::new(YahooSource::new());

    // 3. gRPC client — 走模块注册中心
    let registry = ModuleRegistry::new();
    let market_addr = registry
        .get("market")
        .ok_or_else(|| anyhow::anyhow!("market module not registered"))?;
    let url = format!("http://{}", market_addr);
    let endpoint = Channel::from_shared(url)?
        .connect_timeout(Duration::from_secs(5));
    let client = MarketDataServiceClient::new(endpoint.connect_lazy());

    // 4. 跑循环
    let shutdown = async {
        // 等待 Ctrl-C
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Ctrl-C received, shutting down");
    };

    gpai_core_ingestor::run_loop(source, client, ids, poll_interval, shutdown).await?;
    Ok(())
}
