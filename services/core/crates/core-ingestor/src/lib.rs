//! Ingestor — 数据接入服务
//!
//! 拉取外部行情源(默认 YahooSource)并把每条 quote 通过 gRPC 推送到
//! Market 模块的 `MarketDataService.UpsertLatestQuote` 端点。
//!
//! 设计目标:可测试、可配置。`run_loop` 接受 `Box<dyn DataSource>` 与
//! 显式的 `Duration` 间隔,让单测能用 mock server 在毫秒级跑通。

use std::time::Duration;

use gpai_core_market::source::DataSource;
use gpai_proto_gen::gpai::market::v1::market_data_service_client::MarketDataServiceClient;
use gpai_proto_gen::gpai::market::v1::UpsertLatestQuoteRequest;
use tonic::transport::Channel;
use tracing::{error, info, warn};

/// 拉取 + 推送 单轮。出错时只记录、不 panic,让外层循环决定是否退出。
pub async fn tick_once(
    source: &dyn DataSource,
    client: &mut MarketDataServiceClient<Channel>,
    ids: &[String],
) -> anyhow::Result<usize> {
    let mut pushed = 0usize;
    for id in ids {
        let quote = match source.fetch_quote(id).await {
            Ok(q) => q,
            Err(e) => {
                warn!(instrument_id = %id, error = %e, "fetch_quote failed");
                continue;
            }
        };

        let proto: gpai_proto_gen::gpai::market::v1::Quote = quote.into();
        let req = UpsertLatestQuoteRequest { quote: Some(proto) };
        match client.upsert_latest_quote(req).await {
            Ok(_) => {
                pushed += 1;
                info!(instrument_id = %id, "quote pushed");
            }
            Err(e) => {
                error!(instrument_id = %id, error = %e, "upsert_latest_quote failed");
            }
        }
    }
    Ok(pushed)
}

/// 启动循环:`tick_once` + 等待 `poll_interval`,持续运行直到 `shutdown` 被 resolve。
///
/// * `source`        — 数据源
/// * `client`        — gRPC 客户端(已 `connect_lazy`)
/// * `ids`           — 要拉取的标的 ID 列表
/// * `poll_interval` — 两次拉取之间的间隔(单测里设小一些)
/// * `shutdown`      — 一旦 resolve,函数立刻返回
pub async fn run_loop(
    source: Box<dyn DataSource>,
    mut client: MarketDataServiceClient<Channel>,
    ids: Vec<String>,
    poll_interval: Duration,
    shutdown: impl std::future::Future<Output = ()> + Send,
) -> anyhow::Result<u64> {
    info!(?poll_interval, count = ids.len(), "ingestor loop starting");
    let mut iteration: u64 = 0;
    tokio::pin!(shutdown);

    loop {
        // 拉一次 + 推一次
        let _ = tick_once(source.as_ref(), &mut client, &ids).await?;
        iteration += 1;

        // 等待下一轮 / 等待 shutdown
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {}
            _ = &mut shutdown => {
                info!(iteration, "ingestor loop shutdown");
                return Ok(iteration);
            }
        }
    }
}
