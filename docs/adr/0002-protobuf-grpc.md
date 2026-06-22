# ADR-0002: 选择 Protobuf + gRPC 作为跨语言契约

- 状态:已接受
- 日期:2026-06-22
- 决策者:Future. 魏来

## 背景

项目使用 4 种语言(TS、Python、Rust、Go),需要类型安全的服务间契约。

## 决策

采用 Protocol Buffers 定义消息与服务接口,生成各语言类型;gRPC 作为服务间通信协议。统一错误模型 `Error { code, message, details, request_id }`。

## 备选方案

### 方案 A:OpenAPI / REST

- 优点:Web 友好、工具成熟
- 缺点:类型一致性靠纪律;Python/Rust/Go 类型需手维护

### 方案 B:TS types + 各语言手写映射

- 缺点:跨语言类型一致性弱

## 后果

- Proto 是真相的单一来源,任何类型改动先改 proto
- 严格 buf lint + buf breaking CI 门禁
- TS 类型入库供前端 IDE 即时使用;其他语言构建时生成
- 包名 `gpai.<domain>.v1`,v1 永占,破坏性变更建 v2

## 参考

- spec §5 跨语言接口
- spec §5.3 错误模型