#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# 加载 .env 或用默认
: "${DATABASE_URL:=postgres://gpai:gpai@localhost:5432/gpai}"

# 用 psql 直接执行,避免引入额外工具
for f in db/migrations/*.sql; do
  echo "applying $f"
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$f"
done
echo "✓ migrations applied"