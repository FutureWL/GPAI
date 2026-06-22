use gpai_core_market::repo::QuoteRepo;
use gpai_core_market::Quote;
use chrono::Utc;
use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers::ContainerAsync;

async fn setup() -> (ContainerAsync<Postgres>, PgPool) {
    let pg = Postgres::default().start().await.expect("start postgres container");
    let host = pg.get_host().await.unwrap();
    let port = pg.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    // sqlx::query is a *prepared* statement and rejects multi-statement SQL.
    // Use the `Executor::execute` API with the raw connection so we can run
    // the multi-statement migration files.
    let migrations = [
        include_str!("../../../../../db/migrations/0001_init.sql"),
        include_str!("../../../../../db/migrations/0002_seed.sql"),
    ];
    for sql in migrations {
        sqlx::raw_sql(sql).execute(&pool).await.expect("apply migration");
    }
    (pg, pool)
}

#[tokio::test]
async fn upsert_then_get() {
    let (_c, pool) = setup().await;
    let repo = QuoteRepo::new(pool);

    let q = Quote {
        instrument_id: "US.AAPL.NASDAQ".into(),
        last_price: 200.0,
        open: 199.0, high: 201.0, low: 198.5, prev_close: 199.0,
        volume: 1_000_000, turnover: 200_000_000,
        change: 1.0, change_pct: 0.5,
        ts: Utc::now(),
    };
    repo.upsert(&q).await.unwrap();

    let got = repo.get("US.AAPL.NASDAQ").await.unwrap();
    assert!((got.last_price - 200.0).abs() < 0.001);
}

#[tokio::test]
async fn get_missing_returns_not_found() {
    let (_c, pool) = setup().await;
    let repo = QuoteRepo::new(pool);
    let err = repo.get("US.MSFT.NASDAQ").await.unwrap_err();
    assert!(matches!(err, gpai_core_common::CoreError::NotFound(_)));
}
