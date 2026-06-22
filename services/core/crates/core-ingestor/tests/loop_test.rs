//! Ingestor 端到端测试
//!
//! 流程:
//! 1. wiremock 模拟 Yahoo AAPL chart 端点
//! 2. wiremock 模拟 Market 模块的 gRPC `UpsertLatestQuote`(用 `matcher` 匹配 path)
//! 3. 启动 run_loop,50ms 一轮,跑 ~250ms
//! 4. 验证两个 server 至少各被命中 1 次
//!
//! 这里的关键约束:不能让 run_loop 在循环里"空转"。YahooSource 真正发 HTTP,
//! gRPC 端点真正收 RPC。两边都收到请求 ⇒ 拉取 + 推送闭环成立。

use std::time::Duration;

use gpai_core_market::source::yahoo::YahooSource;
use gpai_core_market::DataSource;
use gpai_proto_gen::gpai::market::v1::market_data_service_client::MarketDataServiceClient;
use gpai_proto_gen::gpai::market::v1::market_data_service_server::MarketDataServiceServer;
use gpai_proto_gen::gpai::market::v1::{
    market_data_service_server::MarketDataService, UpsertLatestQuoteRequest,
    UpsertLatestQuoteResponse,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mock market server:计数收到的 upsert 数,返回 `accepted: true`。
struct MockMarketService {
    counter: Arc<AtomicU32>,
}

#[tonic::async_trait]
impl MarketDataService for MockMarketService {
    async fn upsert_latest_quote(
        &self,
        _req: Request<UpsertLatestQuoteRequest>,
    ) -> Result<Response<UpsertLatestQuoteResponse>, Status> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(Response::new(UpsertLatestQuoteResponse { accepted: true }))
    }

    // 未使用的方法,留空返回
    async fn get_quote(
        &self,
        _: Request<gpai_proto_gen::gpai::market::v1::GetQuoteRequest>,
    ) -> Result<Response<gpai_proto_gen::gpai::market::v1::GetQuoteResponse>, Status> {
        unimplemented!()
    }
    async fn list_instruments(
        &self,
        _: Request<gpai_proto_gen::gpai::market::v1::ListInstrumentsRequest>,
    ) -> Result<Response<gpai_proto_gen::gpai::market::v1::ListInstrumentsResponse>, Status>
    {
        unimplemented!()
    }
}

fn yahoo_fixture() -> String {
    serde_json::json!({
        "chart": {
            "result": [{
                "meta": {
                    "symbol": "AAPL",
                    "regularMarketPrice": 230.45,
                    "regularMarketTime": 1716000000,
                    "chartPreviousClosePrice": 228.00,
                    "regularMarketDayHigh": 231.0,
                    "regularMarketDayLow": 227.5,
                    "regularMarketOpen": 228.5,
                    "regularMarketVolume": 12345678,
                    "currency": "USD",
                    "exchangeName": "NIM"
                }
            }],
            "error": null
        }
    })
    .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_loop_pulls_from_yahoo_and_pushes_to_market() {
    // 1. Yahoo mock
    let yahoo = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(yahoo_fixture()))
        .expect(1..)
        .mount(&yahoo)
        .await;

    // 2. Market gRPC mock (用真实 tonic server 跑在 127.0.0.1 随机端口)
    let counter = Arc::new(AtomicU32::new(0));
    let svc = MockMarketService {
        counter: counter.clone(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let incoming = TcpListenerStream::new(listener);
    tokio::spawn(async move {
        Server::builder()
            .add_service(MarketDataServiceServer::new(svc))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // 3. 装 source + client
    let source: Box<dyn DataSource> = Box::new(YahooSource::with_base_url(yahoo.uri()));
    let endpoint = tonic::transport::Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect_timeout(Duration::from_secs(5));
    let client = MarketDataServiceClient::new(endpoint.connect_lazy());

    // 4. 跑 ~250ms 后 shutdown
    let ids = vec!["US.AAPL.NASDAQ".to_string()];
    let shutdown = async {
        tokio::time::sleep(Duration::from_millis(250)).await;
    };
    let iters = gpai_core_ingestor::run_loop(
        source,
        client,
        ids,
        Duration::from_millis(50),
        shutdown,
    )
    .await
    .expect("run_loop");

    // 5. 断言:至少跑了两轮(250/50 = 5,确保留 margin 走至少 2)
    assert!(iters >= 2, "expected >=2 iterations, got {}", iters);
    // 至少 1 次推送被 market mock 收到
    let upserts = counter.load(Ordering::SeqCst);
    assert!(upserts >= 1, "expected >=1 upsert, got {}", upserts);
}
