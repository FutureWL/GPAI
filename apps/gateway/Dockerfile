# =============================================================================
# GPAI gateway — Go multi-stage Dockerfile
#
# Build context MUST be the monorepo root.
#
# Differences vs the original brief:
#   * `COPY go.work ./` is REMOVED — this repo has no go.work file (Go
#     module is fully self-contained at apps/gateway, with a `replace`
#     directive pointing at ../../gen/go).
#   * Adds an inline `buf generate` step because `gen/go/` is gitignored.
# =============================================================================

# -------- Builder stage --------
FROM golang:1.22-bookworm AS builder

WORKDIR /build

# Install protoc + buf + protoc-gen-go plugins so we can generate the
# gitignored `gen/go/` tree inside the build stage. The Go module's
# `replace` directive then resolves to this generated code on disk.
RUN apt-get update \
 && apt-get install -y --no-install-recommends protobuf-compiler git ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && go install github.com/bufbuild/buf/cmd/buf@v1.34.0 \
 && go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.34.2 \
 && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.4.0

ENV PATH="/go/bin:${PATH}"

# ---- Proto codegen layer ----
COPY proto proto
RUN buf generate proto/ --template proto/buf.gen.go.yaml --output gen/go

# ---- Dep cache layer ----
COPY apps/gateway/go.mod apps/gateway/go.sum apps/gateway/
COPY gen/go gen/go
RUN cd apps/gateway && go mod download

# ---- Real build layer ----
COPY apps/gateway apps/gateway
RUN cd apps/gateway \
 && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 \
    go build -trimpath -ldflags="-s -w" -o /out/gateway ./cmd/gateway

# -------- Runtime stage --------
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /out/gateway /gateway

EXPOSE 8080

USER nonroot:nonroot

ENTRYPOINT ["/gateway"]