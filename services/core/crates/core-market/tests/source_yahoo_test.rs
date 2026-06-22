use gpai_core_market::source::yahoo::YahooSource;
use gpai_core_market::DataSource;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture() -> String {
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

#[tokio::test]
async fn yahoo_parses_chart_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture()))
        .mount(&server)
        .await;

    let s = YahooSource::with_base_url(server.uri());
    let q = s.fetch_quote("US.AAPL.NASDAQ").await.unwrap();

    assert!((q.last_price - 230.45).abs() < 0.01);
    assert!((q.prev_close - 228.0).abs() < 0.01);
    assert!((q.change - 2.45).abs() < 0.01);
}

#[tokio::test]
async fn yahoo_404_returns_upstream_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let s = YahooSource::with_base_url(server.uri());
    let err = s.fetch_quote("US.AAPL.NASDAQ").await.unwrap_err();
    assert!(matches!(err, gpai_core_common::CoreError::UpstreamUnavailable(_)));
}