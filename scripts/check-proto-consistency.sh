#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# buf v2 在 buf generate 时根据调用方式把 out 解析到不同位置:
# - 从仓库根调用 `buf generate proto/ --template buf.gen.ts.yaml` → out 解析到仓库根(gen/ts/...)
# - 从 proto/ 调用 `buf generate .` → out 解析到 proto/(proto/gen/ts/...)
# 两个可能位置都查一下,匹配上即可。
TS_FILE=""
for candidate in \
  "gen/ts/es/gpai/instrument/v1/instrument_pb.ts" \
  "proto/gen/ts/es/gpai/instrument/v1/instrument_pb.ts"; do
  if [ -f "$candidate" ]; then
    TS_FILE="$candidate"
    break
  fi
done

if [ -z "$TS_FILE" ]; then
  echo "✗ 找不到生成的 instrument_pb.ts(尝试过 gen/ts/es/... 和 proto/gen/ts/es/...)"
  echo "  请先运行 buf generate proto/ --template proto/buf.gen.ts.yaml"
  exit 1
fi

# 统计每种生成产物中 Instrument 消息的字段数,跨语言应一致
# 骨架阶段只检查 TS,其他语言在各自 CI workflow 中跑
# TS 用 `name: type;` 语法,proto 用 `type name = N;` 语法,要分别写 regex。
ts_fields=$(grep -cE "^\s+[a-zA-Z_][a-zA-Z0-9_]*\??:\s+(string|number|boolean|Timestamp|Market|AssetClass)" \
  "$TS_FILE" 2>/dev/null)

proto_fields=$(grep -E "^\s+(string|double|int32|int64|bool|google.protobuf.Timestamp|common.v1)" \
  proto/gpai/instrument/v1/instrument.proto 2>/dev/null | grep -v "^\s*//" | wc -l)

if [ "$ts_fields" -ne "$proto_fields" ]; then
  echo "✗ Instrument 字段数不一致:proto=$proto_fields, ts=$ts_fields (file=$TS_FILE)"
  exit 1
fi

echo "✓ proto consistency OK (Instrument: $proto_fields fields, file=$TS_FILE)"
