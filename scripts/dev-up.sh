#!/usr/bin/env bash
# scripts/dev-up.sh — start the full local dev stack.
#
# Brings up Postgres (TimescaleDB) + Redis via docker compose, waits for
# readiness, applies migrations, then starts the four app processes in
# the background:
#   - market-server (Rust gRPC, port 50051)
#   - ingestor     (Rust poll loop)
#   - gateway      (Go HTTP, port 8080)
#   - web          (Next.js, port 3000)
#
# PIDs are stored in `.pid.<service>`; logs in `logs/<service>.log`.
# All env comes from `.env` (copied from `deploy/.env.dev.example` if missing).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Make sure go (and other toolchains outside the default PATH) are reachable
# for background processes. We don't want to depend on the user's shell rc.
for tool_path in \
  /usr/local/go/bin \
  "$HOME/.cargo/bin" \
  "$HOME/.local/bin"; do
  case ":$PATH:" in
    *":$tool_path:"*) ;;
    *) [[ -d "$tool_path" ]] && export PATH="$tool_path:$PATH" ;;
  esac
done

LOG_DIR="$REPO_ROOT/logs"
PID_DIR="$REPO_ROOT"
mkdir -p "$LOG_DIR"

# 1. .env ----------------------------------------------------------------
if [[ ! -f "$REPO_ROOT/.env" ]]; then
  echo "[dev-up] .env not found — copying from deploy/.env.dev.example"
  cp "$REPO_ROOT/deploy/.env.dev.example" "$REPO_ROOT/.env"
fi

# shellcheck disable=SC1091
set -a; source "$REPO_ROOT/.env"; set +a

# 2. Docker compose (postgres + redis) -----------------------------------
echo "[dev-up] bringing up docker compose (postgres + redis)..."
docker compose -f "$REPO_ROOT/deploy/docker-compose.dev.yml" up -d

# 3. Wait for Postgres ---------------------------------------------------
# Pull host port for postgres out of DATABASE_URL (postgres://gpai:gpai@host:PORT/db)
pg_port="$(printf '%s' "$DATABASE_URL" | sed -E 's#.*@[^:]+:([0-9]+)/.*#\1#')"
pg_port="${pg_port:-5432}"
redis_port="$(printf '%s' "$REDIS_URL" | sed -E 's#.*:([0-9]+)/?$#\1#')"
redis_port="${redis_port:-6379}"

echo "[dev-up] waiting for postgres on 127.0.0.1:${pg_port} (max 30s)..."
for i in $(seq 1 30); do
  if (echo > "/dev/tcp/127.0.0.1/${pg_port}") >/dev/null 2>&1; then
    if command -v pg_isready >/dev/null 2>&1; then
      if pg_isready -h 127.0.0.1 -p "$pg_port" -U gpai >/dev/null 2>&1; then
        echo "[dev-up] postgres ready"
        break
      fi
    else
      echo "[dev-up] postgres port open"
      break
    fi
  fi
  sleep 1
  if [[ "$i" -eq 30 ]]; then
    echo "[dev-up] ERROR: postgres not ready after 30s" >&2
    exit 1
  fi
done

# 4. Wait for Redis ------------------------------------------------------
echo "[dev-up] waiting for redis on 127.0.0.1:${redis_port} (max 15s)..."
for i in $(seq 1 15); do
  if (echo > "/dev/tcp/127.0.0.1/${redis_port}") >/dev/null 2>&1; then
    echo "[dev-up] redis port open"
    break
  fi
  sleep 1
  if [[ "$i" -eq 15 ]]; then
    echo "[dev-up] ERROR: redis not ready after 15s" >&2
    exit 1
  fi
done

# 5. Migrations + seed ---------------------------------------------------
echo "[dev-up] applying migrations..."
"$REPO_ROOT/scripts/db-migrate.sh"
echo "[dev-up] seeding (best-effort)..."
if ! "$REPO_ROOT/scripts/db-seed.sh"; then
  echo "[dev-up] seed step failed (often the instruments table is missing) — continuing"
fi

# 6. Helper: start a background process ---------------------------------
# Usage: start_bg <name> <cwd> <cmd...>
# <cwd> may be "-" to inherit the repo root.
#
# The process is launched under `setsid` so it gets its own process group;
# dev-down then sends SIGTERM/SIGKILL to the whole group to reap children
# spawned by wrappers like `go run` (which forks the actual binary) and
# `pnpm` (which forks `next dev`).
start_bg() {
  local name="$1"; shift
  local cwd="$1"; shift
  local pidfile="$PID_DIR/.pid.$name"
  if [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
    echo "[dev-up] $name already running (pid $(cat "$pidfile"))"
    return
  fi
  rm -f "$pidfile"
  echo "[dev-up] starting $name: $*"
  if [[ "$cwd" == "-" ]]; then
    ( setsid "$@" ) > "$LOG_DIR/$name.log" 2>&1 &
  else
    ( cd "$cwd" && setsid "$@" ) > "$LOG_DIR/$name.log" 2>&1 &
  fi
  local pid=$!
  echo "$pid" > "$pidfile"
}

# 7. App processes -------------------------------------------------------
start_bg market-server - "$REPO_ROOT/target/debug/market-server"
start_bg ingestor      - "$REPO_ROOT/target/debug/ingestor"
start_bg gateway       "$REPO_ROOT/apps/gateway" go run ./cmd/gateway
start_bg web           - pnpm --filter @gpai/web dev

# 8. Port map ------------------------------------------------------------
echo
echo "[dev-up] port map:"
echo "  postgres   127.0.0.1:${pg_port}  (gpai / gpai)"
echo "  redis      127.0.0.1:${redis_port}"
echo "  market     127.0.0.1:50051 (gRPC)"
echo "  gateway    127.0.0.1:8080  (HTTP, /healthz)"
echo "  web        127.0.0.1:3000  (Next.js)"
echo
echo "[dev-up] logs:    $LOG_DIR/<service>.log"
echo "[dev-up] pids:    $PID_DIR/.pid.<service>"
echo "[dev-up] teardown: scripts/dev-down.sh"
