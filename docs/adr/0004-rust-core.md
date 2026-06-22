# ADR-0004: 选择 Rust 作为核心进程壳

- 状态:已接受
- 日期:2026-06-22
- 决策者:Future. 魏来

## 背景

核心进程需承载多个业务模块 + gRPC server + 数据 IO,长期运行在生产。

## 决策

核心进程用 Rust 写,采用 tonic(异步 gRPC)+ tokio + sqlx。Python 模块通过 PyO3 嵌入(后续阶段启用)。

## 备选方案

- Python:开发快但运行性能弱、类型检查弱
- Go:简单可读,但金融场景后续需要 Rust 性能

## 后果

- 性能优势:可处理后续高频行情推送
- 编译型,部署简单(单 binary + scratch 镜像)
- 生态成熟(tonic / sqlx / tokio 都是事实标准)
- Python 暂留接口,Phase 1+ 启用

## 参考

- spec §2.3 多语言协作机制