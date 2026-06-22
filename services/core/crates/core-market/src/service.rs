use crate::repo::QuoteRepo;
use crate::types::Quote;
use gpai_core_common::CoreResult;
use gpai_proto_gen::gpai::market::v1::{
    market_data_service_server::MarketDataService, GetQuoteRequest, GetQuoteResponse,
    ListInstrumentsRequest, ListInstrumentsResponse, UpsertLatestQuoteRequest,
    UpsertLatestQuoteResponse,
};
use gpai_proto_gen::gpai::instrument::v1::Instrument as InstrumentProto;
use gpai_proto_gen::gpai::common::v1::Market as MarketProto;
use tonic::{Request, Response, Status};
use std::sync::Arc;

pub struct MarketServiceImpl {
    pub repo: Arc<QuoteRepo>,
}

impl MarketServiceImpl {
    pub fn new(repo: Arc<QuoteRepo>) -> Self { Self { repo } }
}

#[tonic::async_trait]
impl MarketDataService for MarketServiceImpl {
    async fn get_quote(
        &self,
        req: Request<GetQuoteRequest>,
    ) -> Result<Response<GetQuoteResponse>, Status> {
        let id = req.into_inner().instrument_id;
        let q = self.repo.get(&id).await.map_err(to_status)?;
        Ok(Response::new(GetQuoteResponse { quote: Some(q.into()) }))
    }

    async fn upsert_latest_quote(
        &self,
        req: Request<UpsertLatestQuoteRequest>,
    ) -> Result<Response<UpsertLatestQuoteResponse>, Status> {
        let quote = req
            .into_inner()
            .quote
            .ok_or_else(|| Status::invalid_argument("quote is required"))?;
        let q: Quote = quote.try_into().map_err(|e: String| Status::internal(e))?;
        self.repo.upsert(&q).await.map_err(to_status)?;
        Ok(Response::new(UpsertLatestQuoteResponse { accepted: true }))
    }

    async fn list_instruments(
        &self,
        _req: Request<ListInstrumentsRequest>,
    ) -> Result<Response<ListInstrumentsResponse>, Status> {
        // 骨架阶段返回硬编码 1 条
        let instruments = vec![InstrumentProto {
            id: "US.AAPL.NASDAQ".into(),
            market: MarketProto::Us as i32,
            symbol: "AAPL".into(),
            exchange_code: "NASDAQ".into(),
            name_zh: "苹果".into(),
            name_en: "Apple Inc.".into(),
            asset_class: 1, // EQUITY
            currency: "USD".into(),
            timezone: "America/New_York".into(),
            lot_size: 1,
            delisted: false,
            listed_at: None,
        }];
        Ok(Response::new(ListInstrumentsResponse { instruments, page: None }))
    }
}

fn to_status(e: gpai_core_common::CoreError) -> Status {
    let code = match e {
        gpai_core_common::CoreError::NotFound(_) => 1,
        gpai_core_common::CoreError::InvalidArgument(_) => 2,
        gpai_core_common::CoreError::UpstreamUnavailable(_) => 6,
        gpai_core_common::CoreError::Internal(_) => 7,
    };
    Status::new(tonic::Code::Unknown, format!("code={} {}", code, e))
}
