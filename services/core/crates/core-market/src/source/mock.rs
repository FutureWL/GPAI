use crate::source::DataSource;
use crate::types::{Instrument, Market, Quote};
use async_trait::async_trait;
use chrono::Utc;
use gpai_core_common::CoreResult;

/// 用于本地无网测试的固定数据源
pub struct MockSource;

#[async_trait]
impl DataSource for MockSource {
    fn source_id(&self) -> &str { "mock" }

    async fn list_instruments(&self) -> CoreResult<Vec<Instrument>> {
        Ok(vec![Instrument {
            id: "US.AAPL.NASDAQ".into(),
            market: Market::Us,
            symbol: "AAPL".into(),
            exchange_code: "NASDAQ".into(),
            name_zh: "苹果".into(),
            name_en: Some("Apple Inc.".into()),
            currency: "USD".into(),
            timezone: "America/New_York".into(),
            lot_size: 1,
        }])
    }

    async fn fetch_quote(&self, instrument_id: &str) -> CoreResult<Quote> {
        if instrument_id != "US.AAPL.NASDAQ" {
            return Err(gpai_core_common::CoreError::NotFound(instrument_id.into()));
        }
        // 固定价格用于回归
        Ok(Quote {
            instrument_id: instrument_id.into(),
            last_price: 199.99,
            open: 198.50,
            high: 200.10,
            low: 197.80,
            prev_close: 198.20,
            volume: 50_000_000,
            turnover: 9_999_500_000,
            change: 1.79,
            change_pct: 0.90,
            ts: Utc::now(),
        })
    }
}