use gpai_core_market::source::MockSource;
use gpai_core_market::DataSource;

#[tokio::test]
async fn mock_returns_aapl_in_list() {
    let s = MockSource;
    let instruments = s.list_instruments().await.unwrap();
    assert_eq!(instruments.len(), 1);
    assert_eq!(instruments[0].id, "US.AAPL.NASDAQ");
}

#[tokio::test]
async fn mock_fetch_known_quote() {
    let s = MockSource;
    let q = s.fetch_quote("US.AAPL.NASDAQ").await.unwrap();
    assert_eq!(q.instrument_id, "US.AAPL.NASDAQ");
    assert!((q.last_price - 199.99).abs() < 0.01);
}

#[tokio::test]
async fn mock_fetch_unknown_returns_not_found() {
    let s = MockSource;
    let err = s.fetch_quote("US.MSFT.NASDAQ").await.unwrap_err();
    assert!(matches!(err, gpai_core_common::CoreError::NotFound(_)));
}