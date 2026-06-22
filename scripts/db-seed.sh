#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${DATABASE_URL:=postgres://gpai:gpai@localhost:5432/gpai}"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f db/migrations/0002_seed.sql
echo "✓ seed data applied"