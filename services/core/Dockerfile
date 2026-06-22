# =============================================================================
# GPAI core — Rust multi-stage Dockerfile
#
# Build context MUST be the monorepo root (this Dockerfile uses
# `COPY Cargo.toml Cargo.lock ./`, `COPY services/core services/core`, and
# `COPY proto proto` so the workspace can resolve `proto-gen`'s
# `../../../../proto` path).
#
# Produces a single image `gpai/core` with TWO binaries:
#   /usr/local/bin/market-server  — gRPC MarketDataService
#   /usr/local/bin/ingestor       — YahooSource poller pushing via gRPC
#
# The runtime ENTRYPOINT is overridden in deploy/docker-compose.yml per
# service (core runs market-server, ingestor runs ingestor).
# =============================================================================

# -------- Builder stage --------
FROM rust:1.86-bookworm AS builder

WORKDIR /build

# tonic-build needs `protoc` plus the well-known Google proto files
# (`google/protobuf/timestamp.proto` etc.) on the include path.
# `protobuf-compiler` ships the `protoc` binary; `libprotobuf-dev`
# ships the well-known .proto files under `/usr/include/google/protobuf/`.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      protobuf-compiler libprotobuf-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# ---- Dep cache layer ----
# Copy only the workspace manifest and the crate sources that exist before
# the bin sources are populated. Then write stub `fn main() {}` files so
# `cargo build` will resolve and compile all dependencies without trying
# to link the real bins. A second `cargo build` (without the stubs) will
# rebuild only the bin targets on subsequent changes.
COPY Cargo.toml Cargo.lock ./
COPY services/core services/core
COPY proto proto

RUN mkdir -p services/core/crates/core-market/src/bin \
 && echo "fn main() {}" > services/core/crates/core-market/src/bin/market-server.rs \
 && echo "fn main() {}" > services/core/crates/core-ingestor/src/main.rs \
 && cargo build --release -p gpai-core-market --bin market-server \
 && cargo build --release -p gpai-core-ingestor --bin ingestor

# ---- Real build layer ----
# Replace the stub bins with the real source. cargo will rebuild only the
# bin targets (deps are cached from the layer above).
RUN true > services/core/crates/core-market/src/bin/market-server.rs \
 && rm    services/core/crates/core-ingestor/src/main.rs

COPY services/core/crates/core-market/src services/core/crates/core-market/src
COPY services/core/crates/core-ingestor/src services/core/crates/core-ingestor/src

RUN cargo build --release -p gpai-core-market --bin market-server \
 && cargo build --release -p gpai-core-ingestor --bin ingestor

# -------- Runtime stage --------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Non-root user for runtime
RUN groupadd --system --gid 1000 gpai \
 && useradd  --system --uid 1000 --gid gpai --no-create-home --shell /usr/sbin/nologin gpai

COPY --from=builder /build/target/release/market-server /usr/local/bin/market-server
COPY --from=builder /build/target/release/ingestor      /usr/local/bin/ingestor

USER gpai

# market-server gRPC
EXPOSE 50051

# Default to market-server. docker-compose.yml overrides this for the
# `ingestor` service.
ENTRYPOINT ["/usr/local/bin/market-server"]