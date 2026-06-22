-- TimescaleDB hypertable 定义(Phase 1 启用,骨架不创建)
-- 参考 spec §4.2

-- SELECT create_hypertable('ohlcv_1m', 'ts', chunk_time_interval => INTERVAL '1 day');
-- SELECT create_hypertable('ohlcv_1d', 'ts', chunk_time_interval => INTERVAL '30 days');
-- SELECT create_hypertable('ingestion_health', 'ts', chunk_time_interval => INTERVAL '7 days');