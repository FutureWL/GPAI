# ADR-0005: 选择 TimescaleDB + Redis + PostgreSQL

- 状态:已接受
- 日期:2026-06-22
- 决策者:Future. 魏来

## 背景

数据层需同时支撑:业务数据(标的、用户、组合)+ 时序数据(K 线、行情快照)+ 实时缓存(自选股、限流、会话)。

## 决策

- PostgreSQL 16 + TimescaleDB 2.x 扩展:同时承载业务表与时序超表
- Redis 7:缓存 + Pub/Sub
- 单 Postgres+TimescaleDB 实例,Redis 单实例,骨架阶段不上集群

## 备选方案

- ClickHouse + Postgres:OLAP 强但运维复杂,骨架用不上
- DuckDB / Parquet:本地分析强,但不适合多客户端并发
- MongoDB:灵活但金融场景不如 SQL 严谨

## 后果

- TimescaleDB 在 PG 16 之上,业务表与时序表共存一份 migration
- 骨架阶段用 `quotes_latest` 普通表(upsert),不启用 hypertable
- 后续 Phase 启用 K 线 hypertable 与连续聚合
- Redis 用作实时缓存,`quote:*` 缓存 5 分钟 TTL

## 参考

- spec §4 数据层
- spec §7.3 切片交付文件