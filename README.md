# GPAI

> 多市场(A 股 / 港股 / 美股)股票数据与投研平台 — 商业化产品方向

## 状态

**Phase 0 — 骨架阶段**(规划中)

仓库目前只包含**设计文档**。代码脚手架、proto 契约、CI 配置等将在实现计划通过后逐步落地。

## 文档

- **设计文档**:[`docs/superpowers/specs/2026-06-22-gpai-stock-platform-skeleton-design.md`](docs/superpowers/specs/2026-06-22-gpai-stock-platform-skeleton-design.md)
  - 完整定义骨架阶段(Phase 0)的目标、架构、领域模型、数据层、跨语言接口、前端、端到端切片、测试策略、部署与范围

## 核心决策一览

| 维度 | 选择 |
|------|------|
| 架构拓扑 | 模块化单体(后续可拆微服务) |
| 仓库组织 | Turborepo + pnpm workspaces |
| 跨语言接口 | Protocol Buffers + gRPC |
| 数据存储 | PostgreSQL + TimescaleDB + Redis |
| 核心进程壳 | Rust(可嵌 Python) |
| 部署形态 | SaaS + 本地双形态 |
| 前端栈 | Next.js 15 + tRPC + Tailwind + shadcn/ui |

## 后续阶段(预览)

- **Phase 1** — 数据平台:A 股 / 港股数据源、TimescaleDB 滚动聚合、WebSocket 推送
- **Phase 2** — 投研工具:选股器、技术指标库、财务数据接入
- **Phase 3** — 组合管理:组合 CRUD、持仓 / 交易记录、收益率分析
- **Phase 4** — 量化回测:策略框架、回测引擎、信号系统

每个阶段都是独立的 brainstorm → spec → plan → 实现循环。

## 贡献

仓库处于早期规划阶段,暂不接受外部 PR。Phase 0 实现计划通过后,会更新本 README 添加开发指南。
