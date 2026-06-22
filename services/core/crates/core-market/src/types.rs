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