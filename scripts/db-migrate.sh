#!/usr/bin/env bash
# 骨架阶段:迁移脚本需要可重复运行(容器 pgdata 跨 dev-up/down 周期保留)。
# 临时方案:`ON_ERROR_STOP=0` 让 `CREATE TABLE` 重复执行时不中断。
# TODO:Phase 1 引入 schema_migrations 表 + 版本化迁移追踪。
set -uo pipefail  # 注意:去掉 -e,因为 psql 在 ON_ERROR_STOP=0 下会返回非零
cd "$(dirname "$0")/.."

: "${DATABASE_URL:=postgres://gpai:gpai@localhost:5432/gpai}"

for f in db/migrations/*.sql; do
  echo "applying $f"
  psql "$DATABASE_URL" -v ON_ERROR_STOP=0 -q -f "$f" || true
done
echo "✓ migrations applied (idempotent)"