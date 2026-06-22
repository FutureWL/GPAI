# GPAI Local Deploy

This directory holds the local-dev infrastructure and configuration for GPAI.

- `docker-compose.dev.yml` — Postgres (TimescaleDB) + Redis only. The four
  app processes (`market-server`, `ingestor`, `gateway`, `web`) are started
  by `scripts/dev-up.sh` so that logs and PIDs are easy to manage on the
  host.
- `.env.dev.example` — template for the local `.env` (gitignored). Copy it
  next to the repo root as `.env` or let `scripts/dev-up.sh` do that for
  you on first run.

## One-shot usage

```bash
./scripts/dev-up.sh     # bring up infra, migrate, start the 4 app procs
./scripts/dev-down.sh   # stop everything
```

Logs land in `logs/<service>.log`; PIDs in `.pid.<service>`.
