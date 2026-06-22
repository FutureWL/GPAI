# ADR-0003: 选择 Turborepo + pnpm workspaces

- 状态:已接受
- 日期:2026-06-22
- 决策者:Future. 魏来

## 背景

需要统一编排 4 种语言的构建/测试/任务依赖。

## 决策

Turborepo 做任务编排与缓存,pnpm workspaces 管 Node/TS 依赖。

## 备选方案

- Nx:插件多但学习曲线陡
- 手写 Makefile / justfile:灵活但要自己拼装工具
- Cargo workspace 主导:Rust 是边角料不合适

## 后果

- `pnpm dev` / `pnpm build` / `pnpm test` 一行命令管理跨语言
- Turborepo 远程/本地缓存加速 CI
- 其他语言(Cargo / go.work / uv)各管自家依赖,Turbo 只管编排

## 参考

- spec §2.2 仓库目录
- Task 1 脚手架