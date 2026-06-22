use crate::types::Quote;
use gpai_core_common::{CoreError, CoreResult};
use sqlx::PgPool;
use sqlx::Row;

/// Postgres-backed repository for the `quotes_latest` table.
#[derive(Clone)]
pub struct QuoteRepo {
    pool: PgPool,
}

impl QuoteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, q: &Quote) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO quotes_latest
                (instrument_id, last_price, open, high, low, prev_close,
                 volume, turnover, change, change_pct, ts, source_id, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'yahoo', NOW())
            ON CONFLICT (instrument_id) DO UPDATE SET
                last_price = EXCLUDED.last_price,
                open       = EXCLUDED.open,
                high       = EXCLUDED.high,
                low        = EXCLUDED.low,
                prev_close = EXCLUDED.prev_close,
                volume     = EXCLUDED.volume,
                turnover   = EXCLUDED.turnover,
                change     = EXCLUDED.change,
                change_pct = EXCLUDED.change_pct,
                ts         = EXCLUDED.ts,
                updated_at = NOW()
            "#,
        )
        .bind(&q.instrument_id)
        .bind(q.last_price)
        .bind(q.open)
        .bind(q.high)
        .bind(q.low)
        .bind(q.prev_close)
        .bind(q.volume)
        .bind(q.turnover)
        .bind(q.change)
        .bind(q.change_pct)
        .bind(q.ts)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, instrument_id: &str) -> CoreResult<Quote> {
        let row = sqlx::query(
            r#"
            SELECT instrument_id, last_price, open, high, low, prev_close,
                   volume, turnover, change, change_pct, ts
            FROM quotes_latest
            WHERE instrument_id = $1
            "#,
        )
        .bind(instrument_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?
        .ok_or_else(|| CoreError::NotFound(instrument_id.into()))?;

        let map_err = |e: sqlx::Error| CoreError::Internal(e.to_string());
        Ok(Quote {
            instrument_id: row.try_get("instrument_id").map_err(map_err)?,
            last_price: row.try_get("last_price").map_err(map_err)?,
            open: row.try_get("open").map_err(map_err)?,
            high: row.try_get("high").map_err(map_err)?,
            low: row.try_get("low").map_err(map_err)?,
            prev_close: row.try_get("prev_close").map_err(map_err)?,
            volume: row.try_get("volume").map_err(map_err)?,
            turnover: row.try_get("turnover").map_err(map_err)?,
            change: row.try_get("change").map_err(map_err)?,
            change_pct: row.try_get("change_pct").map_err(map_err)?,
            ts: row.try_get("ts").map_err(map_err)?,
        })
    }
}
