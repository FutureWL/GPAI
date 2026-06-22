use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Market {
    Cn,
    Hk,
    Us,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    pub instrument_id: String,
    pub last_price: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub prev_close: f64,
    pub volume: i64,
    pub turnover: i64,
    pub change: f64,
    pub change_pct: f64,
    pub ts: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub id: String,
    pub market: Market,
    pub symbol: String,
    pub exchange_code: String,
    pub name_zh: String,
    pub name_en: Option<String>,
    pub currency: String,
    pub timezone: String,
    pub lot_size: i32,
}

// --- proto conversions ---

use gpai_proto_gen::gpai::common::v1::Market as MarketProto;
use gpai_proto_gen::gpai::market::v1::Quote as QuoteProto;
use gpai_proto_gen::google::protobuf::Timestamp as TimestampProto;

impl From<Market> for MarketProto {
    fn from(m: Market) -> Self {
        match m {
            Market::Cn => MarketProto::Cn,
            Market::Hk => MarketProto::Hk,
            Market::Us => MarketProto::Us,
        }
    }
}

impl From<Quote> for QuoteProto {
    fn from(q: Quote) -> Self {
        QuoteProto {
            instrument_id: q.instrument_id,
            last_price: q.last_price,
            open: q.open,
            high: q.high,
            low: q.low,
            prev_close: q.prev_close,
            volume: q.volume,
            turnover: q.turnover,
            change: q.change,
            change_pct: q.change_pct,
            ts: Some(TimestampProto {
                seconds: q.ts.timestamp(),
                nanos: q.ts.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

impl TryFrom<QuoteProto> for Quote {
    type Error = String;
    fn try_from(q: QuoteProto) -> Result<Self, Self::Error> {
        let ts = q.ts.ok_or("missing ts")?;
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
            .ok_or("invalid ts")?;
        Ok(Quote {
            instrument_id: q.instrument_id,
            last_price: q.last_price,
            open: q.open,
            high: q.high,
            low: q.low,
            prev_close: q.prev_close,
            volume: q.volume,
            turnover: q.turnover,
            change: q.change,
            change_pct: q.change_pct,
            ts: dt,
        })
    }
}