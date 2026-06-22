use crate::source::DataSource;
use crate::types::{Instrument, Market, Quote};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use gpai_core_common::{CoreError, CoreResult};
use serde::Deserialize;
use std::time::Duration;

const BASE: &str = "https://query1.finance.yahoo.com";

pub struct YahooSource {
    client: reqwest::Client,
    base_url: String,
}

impl YahooSource {
    pub fn new() -> Self {
        Self::with_base_url(BASE.to_string())
    }

    /// 测试用:指向 mock server
    #[doc(hidden)]
    pub fn with_base_url(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("gpai/0.1 (+https://github.com/FutureWL/GPAI)")
            .build()
            .expect("reqwest client");
        Self { client, base_url }
    }

    fn base(&self) -> &str {
        &self.base_url
    }
}

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: ChartContainer,
}

#[derive(Debug, Deserialize)]
struct ChartContainer {
    result: Vec<ChartResult>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: ChartMeta,
}

#[derive(Debug, Deserialize)]
struct ChartMeta {
    symbol: String,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
    #[serde(rename = "regularMarketTime")]
    regular_market_time: i64,
    #[serde(rename = "chartPreviousClosePrice", default)]
    prev_close: f64,
    #[serde(rename = "regularMarketDayHigh", default)]
    high: f64,
    #[serde(rename = "regularMarketDayLow", default)]
    low: f64,
    #[serde(rename = "regularMarketOpen", default)]
    open: f64,
    #[serde(rename = "regularMarketVolume", default)]
    volume: i64,
    currency: Option<String>,
    exchangeName: Option<String>,
}

#[async_trait]
impl DataSource for YahooSource {
    fn source_id(&self) -> &str { "yahoo" }

    async fn list_instruments(&self) -> CoreResult<Vec<Instrument>> {
        // Yahoo 没有官方 list 端点;骨架阶段只覆盖硬编码列表
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
        // 从 "US.AAPL.NASDAQ" 提取 "AAPL"
        let symbol = instrument_id
            .split('.')
            .nth(1)
            .ok_or_else(|| CoreError::InvalidArgument(instrument_id.into()))?;

        let url = format!("{}/v8/finance/chart/{}", self.base(), symbol);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::UpstreamUnavailable(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CoreError::UpstreamUnavailable(format!(
                "yahoo returned {}",
                resp.status()
            )));
        }

        let body: ChartResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::UpstreamUnavailable(e.to_string()))?;

        let meta = body
            .chart
            .result
            .into_iter()
            .next()
            .ok_or_else(|| CoreError::UpstreamUnavailable("empty result".into()))?
            .meta;

        let prev_close = if meta.prev_close == 0.0 {
            meta.regular_market_price
        } else {
            meta.prev_close
        };

        let change = meta.regular_market_price - prev_close;
        let change_pct = if prev_close != 0.0 {
            (change / prev_close) * 100.0
        } else {
            0.0
        };

        Ok(Quote {
            instrument_id: instrument_id.to_string(),
            last_price: meta.regular_market_price,
            open: meta.open,
            high: meta.high,
            low: meta.low,
            prev_close,
            volume: meta.volume,
            turnover: 0,
            change,
            change_pct,
            ts: Utc.timestamp_opt(meta.regular_market_time, 0)
                .single()
                .unwrap_or_else(Utc::now),
        })
    }
}