# GPAI 股票平台 — 骨架阶段设计文档

| 项目 | 内容 |
|------|------|
| 文档类型 | 架构设计 / 实现规格 |
| 状态 | 待用户审阅 |
| 创建日期 | 2026-06-22 |
| 范围 | 骨架阶段(Phase 0) |
| 后续阶段 | 数据平台 / 投研 / 组合管理 / 量化回测(独立 spec) |

---

## 1. 背景与目标

### 1.1 项目定位

GPAI 是一个面向**多市场**(A 股、港股、美股)的**商业化股票数据与投研平台**。本设计文档定义**骨架阶段(Phase 0)**的范围与实现策略。

骨架阶段不交付完整产品,而是建立可演进的工程基座:

- 多语言协作的 monorepo
- 跨语言契约(Protobuf + gRPC)
- 领域模型与数据层基础
- 单一端到端垂直切片,证明全栈可跑通
- 双部署形态(SaaS + 本地)的基础设施雏形

后续每个子系统(数据平台、投研、组合管理、量化回测)作为独立 spec 增量交付。

### 1.2 商业目标

- 长期做成可商业化的产品
- 初期阶段(本骨架)不涉及真实交易,只做行情与分析,合规风险最低
- 架构必须支持从单用户本地部署到多租户 SaaS 的两端形态

### 1.3 关键决策概览

| 决策点 | 选择 | 替代方案 | 选择理由 |
|--------|------|---------|---------|
| 架构拓扑 | 模块化单体 | 微服务 / 混合 | 初期迭代最快,模块边界清晰,后期可拆服务 |
| 仓库组织 | Turborepo + pnpm workspaces | Nx / 手写 Makefile / Cargo workspace | 多语言生态成熟,缓存好用 |
| 跨语言接口 | Protocol Buffers + gRPC | OpenAPI / JSON Schema / TS types | 金融科技行业标准,类型安全,性能好 |
| 核心进程壳语言 | Rust | Python / Go | 性能好 + tonic gRPC 生态成熟 + 可嵌 Python |
| 数据存储 | PostgreSQL + TimescaleDB + Redis | ClickHouse / DuckDB / MongoDB | 金融场景稳健组合 |
| 部署形态 | SaaS + 本地双形态 | 仅 SaaS / 仅本地 | 用户已确认两者都做 |
| 鉴权(SaaS) | Auth.js v5 + 多租户 | Clerk / 自建 | 开源、可控、成本低 |
| 前端栈 | Next.js 15 + tRPC + Tailwind + shadcn | Remix / SvelteKit | 生态最广、TypeScript 一致性高 |

详细 ADR 见 `docs/adr/`。

---

## 2. 整体架构

### 2.1 运行时拓扑

```
                        ┌─────────────────────────────┐
   Browser ───HTTPS───▶ │   Web App (Next.js, TS)     │
                        └────────────┬────────────────┘
                                     │ REST / WebSocket
                                     ▼
                        ┌─────────────────────────────┐
                        │   API Gateway (Go)          │
                        │   鉴权 / 限流 / 租户路由     │
                        └────────────┬────────────────┘
                                     │ gRPC (in-proc 或 TCP)
                                     ▼
        ┌──────────────────────────────────────────────────────┐
        │         Core Monolith(单进程,模块化)                 │
        │  ┌────────────┐ ┌────────────┐ ┌────────────┐        │
        │  │ Market     │ │ Analysis   │ │ Portfolio  │        │
        │  │ (Rust)     │ │ (骨架预留) │ │ (骨架预留) │        │
        │  └────────────┘ └────────────┘ └────────────┘        │
        │  ┌────────────┐ ┌────────────┐                       │
        │  │ Ingestor   │ │ Scheduler  │                       │
        │  │ (Rust)     │ │ (骨架预留) │                       │
        │  └────────────┘ └────────────┘                       │
        └────────────┬────────────────────────────┬────────────┘
                     ▼                            ▼
            ┌────────────────┐            ┌────────────────┐
            │ PostgreSQL +   │            │ Redis          │
            │ TimescaleDB    │            │ 缓存 / Pub-Sub │
            └────────────────┘            └────────────────┘
```

骨架阶段仅实现 Market 模块与 Ingestor,其他模块留接口和目录。

### 2.2 仓库目录结构

```
GPAI/
├── proto/                          # 跨语言契约源头
│   ├── buf.yaml
│   ├── buf.gen.yaml
│   ├── common/v1/{types,errors,pagination}.proto
│   ├── instrument/v1/{instrument,instrument_service}.proto
│   ├── market/v1/{quote,ohlcv,market_data_service,calendar}.proto
│   ├── portfolio/v1/{position,transaction,portfolio_service}.proto
│   └── ingestion/v1/{job,ingestion_service}.proto
│
├── gen/                            # 生成的代码
│   ├── ts/         # 提交到 git
│   ├── python/     # gitignore
│   ├── go/         # gitignore
│   └── rust/       # gitignore
│
├── apps/
│   ├── web/                        # Next.js 15,TS
│   └── gateway/                    # Go,API Gateway
│
├── services/
│   └── core/                       # 模块化单体(Rust 壳)
│       ├── Cargo.toml              # workspace 根
│       ├── crates/
│       │   ├── core-common/        # 共享工具、日志、配置
│       │   ├── core-market/        # 行情模块(gRPC server)
│       │   ├── core-analysis/      # 分析模块(骨架占位)
│       │   ├── core-portfolio/     # 组合模块(骨架占位)
│       │   └── core-ingestor/      # 数据采集器
│       └── py/                     # Python 模块(后续阶段)
│           └── README.md
│
├── packages/
│   ├── ui/                         # 共享 React 组件
│   ├── config-eslint/              # 共享 ESLint 配置
│   ├── config-ts/                  # 共享 tsconfig
│   └── infra/                      # IaC 模板
│
├── db/
│   ├── migrations/                 # SQL 文件
│   ├── seeds/                      # 测试种子数据
│   └── schemas/                    # TimescaleDB hypertable DDL
│
├── deploy/
│   ├── docker-compose.dev.yml      # 本地开发
│   ├── docker-compose.yml          # 本地生产部署
│   ├── install.sh                  # 一键安装脚本
│   ├── saas/                       # SaaS K8s manifests
│   └── onprem/                     # 本地部署文档
│
├── docs/
│   ├── architecture/               # 架构图(Structurizr DSL 或 mermaid)
│   ├── adr/                        # 架构决策记录
│   └── superpowers/specs/          # 设计 / 规划文档
│
├── scripts/
│   ├── check-proto-consistency.sh
│   ├── dev-up.sh
│   └── dev-down.sh
│
├── e2e/                            # Playwright E2E
│
├── .github/workflows/
│   ├── ci-rust.yml
│   ├── ci-go.yml
│   ├── ci-python.yml
│   ├── ci-web.yml
│   └── ci-proto.yml
│
├── turbo.json
├── pnpm-workspace.yaml
├── Cargo.toml                      # workspace 根
├── go.work
├── pyproject.toml
├── buf.yaml
└── README.md
```

### 2.3 多语言协作机制

- **Rust 进程内嵌 Python**:核心进程为 Rust binary,通过 PyO3 嵌入 Python 运行时(Python 模块留待后续阶段启用)
- **进程内 gRPC**:各模块在 `127.0.0.1:0` 跑 tonic gRPC server,模块间走 gRPC client,真实 RPC 边界
- **Go 服务**:Ingestor 和 Gateway 用 Go 写;骨架阶段 Ingestor 用 Rust(简化),Gateway 用 Go
- **TypeScript**:仅前端,消费 proto 生成的类型
- **构建编排**:Turborepo 任务图,Cargo/pnpm/go 各管自己的依赖,Turbo 管跨语言编排

### 2.4 模块边界纪律

- 模块**永远**通过 gRPC client 调其他模块,不直接 `use` 内部结构体
- 每个模块一个 `crates/core-X/` 目录,有自己的 `Cargo.toml` 和 proto 子集
- 跨模块类型引用必须从 proto 生成,不允许在多个模块各自定义同名 struct
- 后期拆服务:把 transport 从 in-proc 换成 TCP,业务代码零改动

---

## 3. 领域模型

### 3.1 核心枚举

```protobuf
enum Market {
  MARKET_UNSPECIFIED = 0;
  MARKET_CN = 1;     // A 股(沪深)
  MARKET_HK = 2;     // 港股
  MARKET_US = 3;     // 美股
}

enum AssetClass {
  ASSET_CLASS_UNSPECIFIED = 0;
  ASSET_CLASS_EQUITY = 1;
  ASSET_CLASS_ETF = 2;
  ASSET_CLASS_INDEX = 3;
  ASSET_CLASS_FUTURES = 4;
  ASSET_CLASS_OPTION = 5;
  ASSET_CLASS_CRYPTO = 6;
}

enum Interval {
  INTERVAL_UNSPECIFIED = 0;
  INTERVAL_1M = 1;
  INTERVAL_5M = 2;
  INTERVAL_15M = 3;
  INTERVAL_1H = 4;
  INTERVAL_1D = 5;
  INTERVAL_1W = 6;
  INTERVAL_1MO = 7;
}
```

### 3.2 核心消息

#### Instrument(标的)

```protobuf
message Instrument {
  string id = 1;                  // 内部 ID,格式 "{Market}.{Symbol}.{ExchangeCode}"
  Market market = 2;
  string symbol = 3;              // 原始代码,如 "600519"
  string exchange_code = 4;       // "SH" / "SZ" / "HK" / "NASDAQ" / "NYSE"
  string name_zh = 5;
  string name_en = 6;
  AssetClass asset_class = 7;
  string currency = 8;            // ISO 4217
  string timezone = 9;            // IANA tz,Asia/Shanghai / America/New_York
  int32 lot_size = 10;            // 每手股数
  bool delisted = 11;
  google.protobuf.Timestamp listed_at = 12;
}
```

**统一 ID 格式**:`{Market枚举简码}.{symbol}.{exchange_code}`,例:

- A 股贵州茅台:`CN.600519.SH`
- 港股腾讯:`HK.00700.HK`
- 美股苹果:`US.AAPL.NASDAQ`

#### Quote(实时行情)

```protobuf
message Quote {
  string instrument_id = 1;
  double last_price = 2;
  double open = 3;
  double high = 4;
  double low = 5;
  double prev_close = 6;
  int64 volume = 7;
  int64 turnover = 8;             // 原币种最小单位(分/仙/美分)
  double change = 9;
  double change_pct = 10;
  google.protobuf.Timestamp ts = 11;  // 服务器时间戳(UTC)
}
```

#### OHLCV(K线)

```protobuf
message OHLCV {
  string instrument_id = 1;
  Interval interval = 2;
  google.protobuf.Timestamp open_time = 3;  // K线起点(UTC)
  double open = 4;
  double high = 5;
  double low = 6;
  double close = 7;
  int64 volume = 8;
  int64 turnover = 9;
}
```

### 3.3 时间处理约定

- **所有时间存储为 UTC**(`google.protobuf.Timestamp`)
- **展示层**按 `Instrument.timezone` 字段转换到本地时区
- 各市场交易时间窗口在 `market_calendars` 表中独立维护
- 美股夏令时通过 IANA tz 自动处理,代码不写死偏移
- 节假日和半日交易日(A 股春节、美股感恩节后)在 `market_calendars` 显式列出

---

## 4. 数据层

### 4.1 PostgreSQL Schema(业务表)

```sql
-- 交易所
CREATE TABLE exchanges (
  code              TEXT PRIMARY KEY,         -- SH / SZ / HK / NYSE / NASDAQ
  name_zh           TEXT NOT NULL,
  name_en           TEXT NOT NULL,
  market            SMALLINT NOT NULL,        -- Market 枚举值
  timezone          TEXT NOT NULL,            -- IANA tz
  primary_currency  CHAR(3) NOT NULL
);

-- 标的字典
CREATE TABLE instruments (
  id                TEXT PRIMARY KEY,         -- CN.600519.SH
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
CREATE INDEX idx_instruments_symbol ON instruments(symbol);

-- 数据源注册
CREATE TABLE data_sources (
  id                TEXT PRIMARY KEY,         -- yahoo / akshare / tushare / mock
  display_name      TEXT NOT NULL,
  enabled           BOOLEAN NOT NULL DEFAULT TRUE,
  config            JSONB,                    -- API key 等
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 摄取任务与运行记录
CREATE TABLE ingestion_jobs (
  id                BIGSERIAL PRIMARY KEY,
  source_id         TEXT NOT NULL REFERENCES data_sources(id),
  market            SMALLINT NOT NULL,
  instrument_id     TEXT REFERENCES instruments(id),
  schedule          TEXT NOT NULL,            -- cron 表达式
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

-- 用户与多租户(SaaS)
CREATE TABLE tenants (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  slug              TEXT UNIQUE NOT NULL,     -- URL slug
  display_name      TEXT NOT NULL,
  plan              SMALLINT NOT NULL,        -- 0=free / 1=pro / 2=enterprise
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id         UUID NOT NULL REFERENCES tenants(id),
  email             TEXT UNIQUE NOT NULL,
  password_hash     TEXT NOT NULL,
  display_name      TEXT,
  role              SMALLINT NOT NULL DEFAULT 0,  -- 0=member / 1=admin
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 组合管理(骨架阶段表结构预留,接口未实现)
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
  side              SMALLINT NOT NULL,        -- 0=buy / 1=sell
  quantity          BIGINT NOT NULL,
  price             NUMERIC(20, 8) NOT NULL,
  fee               NUMERIC(20, 8) NOT NULL DEFAULT 0,
  executed_at       TIMESTAMPTZ NOT NULL,
  note              TEXT
);
```

### 4.2 TimescaleDB 超表(时序数据)

```sql
-- 启用 TimescaleDB 扩展
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- 最新行情(普通表,upsert 模式)
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

-- K 线 1 分钟
CREATE TABLE ohlcv_1m (
  instrument_id     TEXT NOT NULL,
  ts                TIMESTAMPTZ NOT NULL,
  open              DOUBLE PRECISION NOT NULL,
  high              DOUBLE PRECISION NOT NULL,
  low               DOUBLE PRECISION NOT NULL,
  close             DOUBLE PRECISION NOT NULL,
  volume            BIGINT NOT NULL,
  turnover          BIGINT NOT NULL,
  PRIMARY KEY (instrument_id, ts)
);
SELECT create_hypertable('ohlcv_1m', 'ts', chunk_time_interval => INTERVAL '1 day');

-- K 线 1 天(从 1m 滚动聚合而来,或独立存储)
CREATE TABLE ohlcv_1d (
  instrument_id     TEXT NOT NULL,
  ts                TIMESTAMPTZ NOT NULL,
  open              DOUBLE PRECISION NOT NULL,
  high              DOUBLE PRECISION NOT NULL,
  low               DOUBLE PRECISION NOT NULL,
  close             DOUBLE PRECISION NOT NULL,
  volume            BIGINT NOT NULL,
  turnover          BIGINT NOT NULL,
  PRIMARY KEY (instrument_id, ts)
);
SELECT create_hypertable('ohlcv_1d', 'ts', chunk_time_interval => INTERVAL '30 days');

-- 摄取健康指标(hypertable)
CREATE TABLE ingestion_health (
  source_id         TEXT NOT NULL,
  ts                TIMESTAMPTZ NOT NULL,
  latency_ms        INTEGER NOT NULL,
  success           BOOLEAN NOT NULL,
  error_code        TEXT
);
SELECT create_hypertable('ingestion_health', 'ts', chunk_time_interval => INTERVAL '7 days');

-- 市场交易日历
CREATE TABLE market_calendars (
  market            SMALLINT NOT NULL,
  date              DATE NOT NULL,
  is_trading_day    BOOLEAN NOT NULL,
  sessions          JSONB NOT NULL,           -- [{"open":"09:30","close":"11:30"}, ...]
  PRIMARY KEY (market, date)
);
```

骨架阶段仅启用 `quotes_latest` 表(用于 Hello Quote 切片),`ohlcv_*` 与 `ingestion_health` 表在骨架阶段**不**使用。

### 4.3 Redis 用法

| Key 模式 | 用途 | TTL |
|---------|------|-----|
| `quote:{instrument_id}` | 最新行情 JSON 缓存 | 5 分钟 |
| `instruments:active` | Set,正在交易的标的 | 永不过期,启动时重建 |
| `rate_limit:{user_id}:{endpoint}` | 限流计数器 | 1 分钟 |
| `session:{token}` | 会话数据 | 7 天 |
| `pub:quote:{market}` | Pub/Sub 频道,推实时行情 | - |

骨架阶段使用 `quote:*` 缓存,其他 key 在后续阶段启用。

### 4.4 数据源适配层

```rust
// services/core/crates/core-market/src/source.rs
#[async_trait]
pub trait DataSource: Send + Sync {
    fn source_id(&self) -> &str;
    fn market(&self) -> Market;
    async fn list_instruments(&self) -> Result<Vec<Instrument>, SourceError>;
    async fn fetch_quote(&self, instrument_id: &str) -> Result<Quote, SourceError>;
    async fn fetch_ohlcv(
        &self,
        instrument_id: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<Vec<OHLCV>, SourceError>;
}
```

**骨架阶段实现**:

| Source | market | 说明 |
|--------|--------|------|
| `MockSource` | US | 固定假数据,用于本地无网测试;`US.AAPL.NASDAQ` 返回 100.00~300.00 区间随机价格(用于切片自检) |
| `YahooSource` | US / HK | 用 Rust 直接 HTTPS 调 `query1.finance.yahoo.com/v8/finance/chart/{symbol}`,15 分钟延迟;切片仅验证 `US.AAPL.NASDAQ` |

骨架阶段**不实现** A 股数据源,留给 Phase 1。

---

## 5. 跨语言接口

### 5.1 Proto 仓库布局

```
proto/
├── buf.yaml                       # Buf 配置
├── buf.gen.yaml                   # 模板
├── common/v1/
│   ├── types.proto                # Market / AssetClass / Interval
│   ├── errors.proto               # Error / Result<T>
│   └── pagination.proto           # PageRequest / PageResponse
├── instrument/v1/
│   ├── instrument.proto
│   └── instrument_service.proto
├── market/v1/
│   ├── quote.proto
│   ├── ohlcv.proto
│   ├── market_data_service.proto
│   └── calendar.proto
├── portfolio/v1/
│   ├── position.proto
│   ├── transaction.proto
│   └── portfolio_service.proto
└── ingestion/v1/
    ├── job.proto
    └── ingestion_service.proto
```

### 5.2 包名与版本化

- 包名格式:`gpai.<domain>.v1`(例 `gpai.market.v1`)
- 破坏性变更必须**新建**包(v2),旧包保留至少 1 个主版本周期
- 字段永不删除或改类型,只能加新字段 + `reserved` 标记
- CI 用 `buf breaking --against '.git#branch=main'` 检测破坏性变更

### 5.3 统一错误模型

```protobuf
message Error {
  enum Code {
    CODE_UNSPECIFIED = 0;
    CODE_NOT_FOUND = 1;
    CODE_INVALID_ARGUMENT = 2;
    CODE_UNAUTHENTICATED = 3;
    CODE_PERMISSION_DENIED = 4;
    CODE_RATE_LIMITED = 5;
    CODE_UPSTREAM_UNAVAILABLE = 6;
    CODE_INTERNAL = 7;
    CODE_CONFLICT = 8;
  }
  Code code = 1;
  string message = 2;
  map<string, string> details = 3;
  string request_id = 4;
}
```

各语言 SDK 把 `Error.code` 映射成本语言的异常或 Result 类型。

### 5.4 gRPC 服务定义

```protobuf
// proto/market/v1/market_data_service.proto
service MarketDataService {
  rpc GetQuote(GetQuoteRequest) returns (GetQuoteResponse);
  rpc GetOHLCV(GetOHLCVRequest) returns (GetOHLCVResponse);
  rpc ListInstruments(ListInstrumentsRequest) returns (ListInstrumentsResponse);
}

message GetQuoteRequest { string instrument_id = 1; }
message GetQuoteResponse { Quote quote = 1; }

message GetOHLCVRequest {
  string instrument_id = 1;
  Interval interval = 2;
  google.protobuf.Timestamp from = 3;
  google.protobuf.Timestamp to = 4;
}
message GetOHLCVResponse { repeated OHLCV bars = 1; }
```

骨架阶段只实现 `GetQuote` / `UpsertLatestQuote` / `ListInstruments`,`GetOHLCV` 定义在 proto 中但不实现。

### 5.5 代码生成策略

| 语言 | 工具 | 输出位置 | 入库 |
|------|------|---------|------|
| TypeScript | `buf generate --template buf.gen.ts.yaml` | `gen/ts/` | **是** |
| Python | `buf generate --template buf.gen.py.yaml` | `gen/python/` | 否 |
| Go | `buf generate --template buf.gen.go.yaml` | `gen/go/` | 否 |
| Rust | `tonic-build` + `buf` | `services/core/crates/.../proto/` | 否 |

**TypeScript 入库理由**:前端在 Next.js 中频繁使用类型,提交后 IDE 立即可见,无构建等待。

### 5.6 in-proc gRPC

核心进程内,各模块在 `127.0.0.1:0` 起 tonic gRPC server,端口由模块注册中心分配:

```rust
// services/core/crates/core-common/src/registry.rs
pub struct ModuleRegistry {
    modules: HashMap<String, SocketAddr>,
}

impl ModuleRegistry {
    pub async fn register(&mut self, name: &str) -> Result<SocketAddr>;
    pub fn get(&self, name: &str) -> Option<SocketAddr>;
}
```

模块启动时注册,其他模块通过 registry 取地址,调 client。骨架阶段 Ingestor 直接调 Market 模块的 gRPC client 写 `quotes_latest`。

---

## 6. Web App 前端

### 6.1 技术栈

- **Next.js 15**(App Router)+ React 19 + TypeScript
- **tRPC** 端到端类型安全
- **Tailwind CSS + shadcn/ui** 基础组件
- **Recharts** 图表
- **TanStack Query** 客户端缓存
- **Zustand** UI 状态
- **Auth.js v5** 鉴权(SaaS)

### 6.2 路由与页面

| 路由 | 用途 | 骨架实现 |
|------|------|---------|
| `/` | 首页 / 登录入口 | ✅ |
| `/login` | 登录 | ✅(仅 SaaS 模式) |
| `/dashboard` | 自选股 + 大盘指数 | ⏳ Phase 1 |
| `/portfolio` | 组合列表 | ⏳ Phase 3 |
| `/portfolio/[id]` | 组合详情 | ⏳ Phase 3 |
| `/settings` | 设置 | ⏳ Phase 2 |
| `/markets` | 市场列表 | ⏳ Phase 1 |
| `/markets/[instrument]` | 个股详情(行情 + K线) | ✅(K 线简化为占位) |

骨架阶段交付:`/`、`/login`、`/markets/US.AAPL`(K 线区域显示 "K 线图表 — 下一迭代")。

### 6.3 个股详情页(骨架核心页面)

```tsx
// apps/web/app/markets/[instrument]/page.tsx
export default async function InstrumentPage({
  params,
}: {
  params: { instrument: string };
}) {
  const quote = await trpc.market.getQuote({ instrumentId: params.instrument });
  return (
    <main className="grid grid-cols-1 lg:grid-cols-3 gap-6 p-6">
      <header className="col-span-full">
        <h1 className="text-4xl font-mono tabular-nums">
          {quote.instrumentId}
        </h1>
        <p className="text-6xl font-mono tabular-nums text-green-400">
          ${quote.lastPrice.toFixed(2)}
        </p>
        <p className="text-xl font-mono tabular-nums">
          {quote.change >= 0 ? '▲' : '▼'} {Math.abs(quote.change).toFixed(2)}
          {' '}
          ({quote.changePct.toFixed(2)}%)
        </p>
      </header>
      <section className="col-span-2 border rounded p-4">
        <h2>K 线图</h2>
        <p className="text-zinc-500">K 线图表 — 下一迭代交付</p>
      </section>
      <section className="border rounded p-4">
        <h2>财务数据</h2>
        <p className="text-zinc-500">财务标签页 — 后续 spec 交付</p>
      </section>
    </main>
  );
}
```

**视觉方向**:金融终端风(深色、高对比、等宽数字、数据密度高)。骨架阶段用 shadcn 默认 dark theme 占位,品牌阶段重设。

### 6.4 数据获取流

```
Browser
  │ (RSC) 直接调 API Gateway
  ▼
Next.js Server Component
  │ tRPC client (with auth header)
  ▼
API Gateway
  │ gRPC client
  ▼
Core Monolith (MarketDataService.GetQuote)
  │ SQL
  ▼
TimescaleDB / PostgreSQL
```

骨架阶段用 RSC + 30 秒轮询;WebSocket 实时推送在 Phase 1 加入。

### 6.5 多租户(SaaS)

- 域名形式:`{tenant}.gpai.app`,在 Next.js middleware 解析 tenant
- 所有数据查询带 `tenant_id` 过滤,DB 层强制
- 骨架阶段只实现单租户占位,多租户机制留到 Phase 1

---

## 7. 端到端最小切片("Hello Quote")

### 7.1 切片范围

骨架阶段的**唯一硬验收**:从外部数据源到浏览器,数据走完全栈,真实可见。

**切片目标**:浏览器打开 `http://localhost:3000/markets/US.AAPL`,看到 AAPL 真实价格(从 Yahoo Finance 拉取)。

### 7.2 数据流

```
1. Ingestor(骨架:Rust 进程内的 tokio task)每 30 秒:
   a. 调 YahooSource.fetch_quote("AAPL")
   b. 解析为 Quote 消息
   c. 调 MarketDataService gRPC (in-proc):UpsertLatestQuote
   d. Service 写 TimescaleDB quotes_latest
   e. Service 写 Redis quote:US.AAPL

2. 浏览器请求 GET /markets/US.AAPL:
   a. Next.js RSC 调 tRPC market.getQuote
   b. tRPC client 调 API Gateway REST /v1/quotes/US.AAPL
   c. Gateway gRPC 调 Core MarketDataService.GetQuote
   d. Service 读 TimescaleDB quotes_latest
   e. 返回 Quote 消息
   f. Next.js 渲染页面
```

### 7.3 切片交付的具体文件

| 文件 | 用途 |
|------|------|
| `proto/market/v1/quote.proto` | Quote 消息 |
| `proto/market/v1/market_data_service.proto` | GetQuote / UpsertLatestQuote / ListInstruments |
| `proto/common/v1/types.proto` | Market 枚举 |
| `services/core/crates/core-market/src/source/yahoo.rs` | Yahoo 数据源实现 |
| `services/core/crates/core-market/src/source/mock.rs` | Mock 数据源 |
| `services/core/crates/core-market/src/service.rs` | MarketDataService gRPC server |
| `services/core/crates/core-ingestor/src/main.rs` | 定时拉取循环 |
| `apps/gateway/internal/handler/quote.go` | REST handler:`GET /v1/quotes/{id}` |
| `apps/web/app/markets/[instrument]/page.tsx` | 个股详情页 |
| `apps/web/app/page.tsx` | 首页(指向 AAPL 链接) |
| `db/migrations/0001_init.sql` | 全部表结构 |
| `db/seeds/us_aapl.sql` | AAPL 种子数据 |
| `deploy/docker-compose.dev.yml` | 本地开发依赖 |
| `e2e/hello-quote.spec.ts` | Playwright E2E 测试 |

### 7.4 切片完成定义(DoD)

- [ ] `pnpm dev` 一行起全栈(core / gateway / web / postgres / timescaledb / redis)
- [ ] 浏览器 `http://localhost:3000/markets/US.AAPL` 显示 AAPL 真实价格
- [ ] 价格每 30 秒自动刷新(后台 Ingestor 拉取)
- [ ] CI 跑通单测 + 集成 + E2E
- [ ] 覆盖率 ≥ 80%(各语言独立统计)
- [ ] Playwright E2E 测试在 CI 中通过

### 7.5 切片显式不做

- WebSocket 实时推送(30s 轮询足够)
- A 股/港股数据源
- K 线图(显示占位)
- 用户登录(SaaS 模式直接访问;骨架默认无鉴权中间件)
- 组合管理任何功能
- 多租户隔离(SaaS 模式单租户占位)
- 回测、策略

---

## 8. 测试策略

### 8.1 测试分层

| 层级 | 工具 | 覆盖目标 |
|------|------|---------|
| 单元测试 | Rust:`cargo test` / Go:`go test` / TS:`vitest` / Python:`pytest` | ≥ 80% |
| 集成测试 | `testcontainers`(Rust/Go 起一次性 DB 容器) | 关键模块 100% |
| 跨语言契约 | `buf breaking` + `scripts/check-proto-consistency.sh` | 100% |
| API 测试 | `gRPCurl`(gRPC)+ MSW(TS REST mock) | 100% endpoint |
| E2E | Playwright | Hello Quote 切片 |

### 8.2 跨语言一致性检查

`scripts/check-proto-consistency.sh`:

```bash
#!/bin/bash
set -e

# 1. 重新生成所有语言的代码
buf generate

# 2. 统计每个生成的 Instrument 消息字段数
for lang in ts go rust; do
  count=$(gen_count_fields_$lang gpai.instrument.v1.Instrument)
  echo "$lang: $count"
done | awk '{print $2}' | sort -u | wc -l
# 输出必须 = 1,否则报错:跨语言字段数不一致
```

### 8.3 测试数据库

- 集成测试用 `testcontainers` 启动一次性 TimescaleDB + Redis
- 自动跑 migration + seed
- 测试结束销毁容器

### 8.4 Mock 策略

- **外部 HTTP API**(Yahoo、券商等)必须 mock,使用 `wiremock` 或类似工具
- **数据库**用 testcontainers 真实实例,不用 mock
- **gRPC 跨模块**用 in-memory channel,真实跑 server

### 8.5 E2E 切片测试

```typescript
// e2e/hello-quote.spec.ts
import { test, expect } from '@playwright/test';

test('AAPL 页面显示真实行情', async ({ page }) => {
  await page.goto('/markets/US.AAPL');
  const price = page.getByTestId('quote-last-price');
  await expect(price).toBeVisible();
  await expect(price).toHaveText(/^\$\d+\.\d{2}$/);
});
```

### 8.6 CI 矩阵

| Workflow | 触发 | 任务 |
|----------|------|------|
| `ci-proto.yml` | proto/** 变更 | buf lint + buf breaking + 一致性检查 |
| `ci-rust.yml` | services/core/** 变更 | cargo test + cargo clippy + 覆盖率 |
| `ci-go.yml` | apps/gateway/** 变更 | go test + golangci-lint + 覆盖率 |
| `ci-web.yml` | apps/web/** 变更 | tsc + vitest + eslint + 覆盖率 |
| `e2e.yml` | 全部通过后 | Playwright run,产物上传 |

---

## 9. 部署

### 9.1 本地开发

```bash
git clone https://github.com/yourorg/gpai
cd gpai
pnpm install
cp .env.example .env
./scripts/dev-up.sh
# 浏览器打开 http://localhost:3000
```

`scripts/dev-up.sh`:

1. `docker compose -f deploy/docker-compose.dev.yml up -d`(起 Postgres+TimescaleDB+Redis)
2. `pnpm db:migrate`(跑 migration)
3. `pnpm db:seed`(种子数据)
4. `pnpm dev`(起 core / gateway / web,各自 watch 模式)

### 9.2 本地生产部署(用户自部署)

`deploy/docker-compose.yml` + `deploy/install.sh`:

```yaml
services:
  postgres: { image: timescale/timescaledb:latest-pg16 }
  redis: { image: redis:7-alpine }
  core: { image: gpai/core:latest, depends_on: [postgres, redis] }
  gateway: { image: gpai/gateway:latest, depends_on: [core] }
  web: { image: gpai/web:latest, depends_on: [gateway] }
```

`install.sh`:

1. 检测 Docker 版本(≥ 24)
2. 生成 `.env`(随机密码)
3. `docker compose up -d`
4. 等待健康检查通过
5. 跑 migration
6. 打印访问 URL 与默认管理员凭据

### 9.3 SaaS 部署

- **Web**:Vercel 多区域
- **Gateway / Core**:K8s(自建 EKS / GKE)或 Fly.io
- **DB**:Timescale Cloud + Postgres 托管 + Redis 托管
- **IaC**:Terraform 管云资源
- **GitOps**:ArgoCD 同步 K8s manifests

骨架阶段**不**交付 SaaS 部署细节,只交付:
- `deploy/saas/` 目录含示例 K8s manifests
- Dockerfile 多阶段构建(每个 app 一份)
- CI 中包含 `docker buildx` 镜像推送

### 9.4 Feature Flag 区分形态

```rust
// services/core/crates/core-common/src/auth.rs
#[cfg(feature = "saas")]
pub mod auth {
    // Auth.js JWT 验证 + tenant 解析
}

#[cfg(feature = "onprem")]
pub mod auth {
    // 本地账户 session
}
```

构建命令:
- `cargo build --features saas --release`
- `cargo build --features onprem --release`

骨架阶段**不实现**鉴权(切片不要求),feature flag 目录预留。

### 9.5 密钥管理

- `.env.example` 入仓,`.env` gitignore
- 本地部署:`install.sh` 自动生成 `.env` 强密码
- SaaS:K8s External Secrets Operator + AWS Secrets Manager / GCP Secret Manager
- 任何密钥/Token 绝不入仓(包括 README 与 ADR)

---

## 10. 范围与交付

### 10.1 骨架阶段 DoD(总)

- [ ] `pnpm dev` 一行起全栈
- [ ] Hello Quote 切片通过 E2E 测试
- [ ] CI 全绿(单测 + 集成 + E2E + proto 检查)
- [ ] 各语言覆盖率 ≥ 80%
- [ ] 5 个 ADR 提交到 `docs/adr/`
- [ ] `docs/architecture/` 至少 1 张架构图(mermaid)
- [ ] `README.md` 含本地启动 + 部署说明
- [ ] `docker compose -f deploy/docker-compose.yml up` 在干净机器可跑
- [ ] Mock + Yahoo 数据源实现并测试
- [ ] Proto 含 5 个 service 定义(common / instrument / market / portfolio / ingestion)
- [ ] 双部署形态 Docker 镜像构建通过

### 10.2 显式不做(YAGNI)

骨架阶段**不交付**:

- 任何券商集成 / 真实下单
- WebSocket 实时推送
- 加密货币 / 期货 / 期权
- 多用户协作 / 分享组合
- 移动端
- 暗色模式以外的主题
- 复杂风控 / 审计
- 通知(邮件 / 微信 / 推送)
- 鉴权(SaaS 与本地都跳过,默认开放)
- A 股数据源
- K 线图(占位)
- 多租户隔离
- 组合管理任何功能
- 回测 / 策略

### 10.3 风险登记

| 风险 | 等级 | 缓解 |
|------|------|------|
| 多语言构建链复杂度 | 高 | Turborepo 缓存 + CI 矩阵,分模块并行构建 |
| Proto 演进兼容 | 中 | 严格 buf lint + breaking 检查 + 跨语言一致性脚本 |
| TimescaleDB 本地 arm64 镜像 | 中 | docker-compose 用 amd64 镜像 + volume 持久化 |
| Yahoo Finance 限流 / 反爬 | 中 | 切片只拉 1 个标的,先验证通路;限流反爬放 Phase 1 |
| Rust + Python(PyO3)集成复杂度 | 中 | 切片用纯 Rust 实现 Yahoo 适配器,Python 留下一阶段 |
| gRPC in-proc 性能开销 | 低 | 单条 quote < 1ms,可忽略;有真问题再换 ArcSwap 直调 |
| SaaS + 本地双形态代码分叉 | 高 | feature flag + 严格 ADR;定期同步代码 |

### 10.4 后续 spec 路线图(预览,非本 spec 范围)

- **Phase 1 — 数据平台**:A 股 / 港股数据源、TimescaleDB 滚动聚合、WebSocket 推送、限流降级
- **Phase 2 — 投研工具**:选股器、技术指标库、财务数据接入
- **Phase 3 — 组合管理**:组合 CRUD、持仓 / 交易记录、收益率分析
- **Phase 4 — 量化回测**:策略框架、回测引擎、信号系统

每个 Phase 独立 brainstorm → spec → plan → 实现循环。

---

## 11. 决策记录(ADR 索引)

骨架阶段需创建以下 ADR,放在 `docs/adr/`:

- **ADR-001**:选择模块化单体而非微服务
- **ADR-002**:选择 Protobuf + gRPC 作为跨语言契约
- **ADR-003**:选择 Turborepo + pnpm workspaces
- **ADR-004**:选择 Rust 作为核心进程壳
- **ADR-005**:选择 TimescaleDB + Redis + PostgreSQL

模板:MADR 4.0。

---

## 12. 验收确认

本文档作为骨架阶段 spec 的契约。任何后续实现必须遵守本设计,如有偏离需先更新本文档并重新评审。

下一步:用户审阅本 spec,通过后调用 writing-plans 技能,产出实现计划。
