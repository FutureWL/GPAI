#!/usr/bin/env bash
# scripts/dev-down.sh — stop the full local dev stack.
#
# Sends SIGTERM to each tracked PID, waits up to 5s, escalates to SIGKILL,
# then brings down docker compose (postgres + redis).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

shopt -s nullglob

# 1. Stop tracked background processes -----------------------------------
# Each service runs in its own process group (see dev-up.sh `setsid`), so
# we send signals to the negative PID to take down every child spawned by
# wrappers like `go run` and `pnpm` at once.
for pidfile in "$REPO_ROOT"/.pid.*; do
  name="$(basename "$pidfile" | sed 's/^\.pid\.//')"
  pid="$(cat "$pidfile" 2>/dev/null || true)"
  if [[ -z "${pid:-}" ]]; then
    rm -f "$pidfile"
    continue
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "[dev-down] $name (pid $pid) already gone"
    rm -f "$pidfile"
    continue
  fi
  echo "[dev-down] SIGTERM $name (pid $pid, pgroup)"
  kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true

  for _ in $(seq 1 5); do
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 1
  done

  if kill -0 "$pid" 2>/dev/null; then
    echo "[dev-down] SIGKILL $name (pid $pid, pgroup)"
    kill -KILL -- "-$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
  fi
  rm -f "$pidfile"
done

# 2. Docker compose down -------------------------------------------------
if [[ -f "$REPO_ROOT/deploy/docker-compose.dev.yml" ]]; then
  echo "[dev-down] docker compose down"
  docker compose -f "$REPO_ROOT/deploy/docker-compose.dev.yml" down
fi

echo "[dev-down] done"
