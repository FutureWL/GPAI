use crate::types::{Instrument, Quote};
use async_trait::async_trait;
use gpai_core_common::CoreResult;

pub mod mock;

pub use mock::MockSource;

#[async_trait]
pub trait DataSource: Send + Sync {
    fn source_id(&self) -> &str;

    /// 列出该数据源覆盖的所有标的
    async fn list_instruments(&self) -> CoreResult<Vec<Instrument>>;

    /// 拉取单个标的最新行情
    async fn fetch_quote(&self, instrument_id: &str) -> CoreResult<Quote>;
}