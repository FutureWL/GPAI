#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# 重新生成,失败即报错
buf generate proto/ --template proto/buf.gen.yaml

# 统计每种生成产物中 Instrument 消息的字段数,跨语言应一致
# 骨架阶段只检查 TS,其他语言在各自 CI workflow 中跑
ts_fields=$(grep -E "^\s+(string|double|int|bool|google.protobuf.Timestamp|common.v1)" \
  gen/ts/es/gpai/instrument/v1/instrument_pb.ts 2>/dev/null | wc -l)

proto_fields=$(grep -E "^\s+(string|double|int32|int64|bool|google.protobuf.Timestamp|common.v1)" \
  proto/gpai/instrument/v1/instrument.proto 2>/dev/null | grep -v "^\s*//" | wc -l)

if [ "$ts_fields" -ne "$proto_fields" ]; then
  echo "✗ Instrument 字段数不一致:proto=$proto_fields, ts=$ts_fields"
  exit 1
fi

echo "✓ proto consistency OK (Instrument: $proto_fields fields)"
