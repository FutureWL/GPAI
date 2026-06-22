-- 0001_init.sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- exchanges
CREATE TABLE exchanges (
  code              TEXT PRIMARY KEY,
  name_zh           TEXT NOT NULL,
  name_en           TEXT NOT NULL,
  market            SMALLINT NOT NULL,
  timezone          TEXT NOT NULL,
  primary_currency  CHAR(3) NOT NULL
);

-- instruments
CREATE TABLE instruments (
  id                TEXT PRIMARY KEY,
  market            SMALLINT NOT NULL,
  symbol            TEXT NOT NULL,
  exchange_code     TEXT NOT NULL REFERENCES exchanges(code),
  name_zh           TEXT NOT NULL,
  name_en           TEXT,
  asset_class       SMALLINT NOT NULL,
  currency          CHAR(3) NOT NULL,
  timezone          TEXT NOT NULL,
  lot_size          INTEGER NOT NULL,
  delisted          BOOLEAN NOT NULL DEFAULT FALSE,
  listed_at         TIMESTAMPTZ,
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_instruments_market ON instruments(market) WHERE NOT delisted;

-- data_sources
CREATE TABLE data_sources (
  id                TEXT PRIMARY KEY,
  display_name      TEXT NOT NULL,
  enabled           BOOLEAN NOT NULL DEFAULT TRUE,
  config            JSONB,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- quotes_latest(骨架阶段核心表)
CREATE TABLE quotes_latest (
  instrument_id     TEXT PRIMARY KEY REFERENCES instruments(id),
  last_price        DOUBLE PRECISION NOT NULL,
  open              DOUBLE PRECISION NOT NULL,
  high              DOUBLE PRECISION NOT NULL,
  low               DOUBLE PRECISION NOT NULL,
  prev_close        DOUBLE PRECISION NOT NULL,
  volume            BIGINT NOT NULL,
  turnover          BIGINT NOT NULL,
  change            DOUBLE PRECISION NOT NULL,
  change_pct        DOUBLE PRECISION NOT NULL,
  ts                TIMESTAMPTZ NOT NULL,
  source_id         TEXT NOT NULL,
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 业务表骨架预留(组合/用户/租户)
CREATE TABLE tenants (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  slug              TEXT UNIQUE NOT NULL,
  display_name      TEXT NOT NULL,
  plan              SMALLINT NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id         UUID NOT NULL REFERENCES tenants(id),
  email             TEXT UNIQUE NOT NULL,
  password_hash     TEXT NOT NULL,
  display_name      TEXT,
  role              SMALLINT NOT NULL DEFAULT 0,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE portfolios (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id         UUID NOT NULL REFERENCES tenants(id),
  owner_user_id     UUID NOT NULL REFERENCES users(id),
  name              TEXT NOT NULL,
  base_currency     CHAR(3) NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE positions (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  portfolio_id      UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
  instrument_id     TEXT NOT NULL REFERENCES instruments(id),
  quantity          BIGINT NOT NULL,
  avg_cost          NUMERIC(20, 8) NOT NULL,
  opened_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE transactions (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  portfolio_id      UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
  instrument_id     TEXT NOT NULL REFERENCES instruments(id),
  side              SMALLINT NOT NULL,
  quantity          BIGINT NOT NULL,
  price             NUMERIC(20, 8) NOT NULL,
  fee               NUMERIC(20, 8) NOT NULL DEFAULT 0,
  executed_at       TIMESTAMPTZ NOT NULL,
  note              TEXT
);

CREATE TABLE ingestion_jobs (
  id                BIGSERIAL PRIMARY KEY,
  source_id         TEXT NOT NULL REFERENCES data_sources(id),
  market            SMALLINT NOT NULL,
  instrument_id     TEXT REFERENCES instruments(id),
  schedule          TEXT NOT NULL,
  enabled           BOOLEAN NOT NULL DEFAULT TRUE,
  last_run_at       TIMESTAMPTZ,
  last_status       SMALLINT
);

CREATE TABLE ingestion_runs (
  id                BIGSERIAL PRIMARY KEY,
  job_id            BIGINT NOT NULL REFERENCES ingestion_jobs(id),
  started_at        TIMESTAMPTZ NOT NULL,
  finished_at       TIMESTAMPTZ,
  status            SMALLINT NOT NULL,
  rows_written      BIGINT,
  error_message     TEXT
);