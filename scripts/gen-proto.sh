#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# 1. Lint
buf lint proto/

# 2. Generate TypeScript
buf generate proto/ --template proto/buf.gen.yaml

# 3. Format
cd gen/ts && npx prettier --write "**/*.ts" 2>/dev/null || true

echo "✓ proto generated to gen/ts/"
