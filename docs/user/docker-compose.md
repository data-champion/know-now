# Docker Compose

Run the full know-now local demo stack without installing Rust, Node, or PostgreSQL.

## Prerequisites

- Docker Engine 24+ with Compose V2 (`docker compose`, not `docker-compose`)

## Quick start

```bash
docker compose up
```

This builds and starts:

- **know-now-server** — the engine + dashboard at <http://127.0.0.1:3827>
- **postgres** — PostgreSQL 17 for generated DDL targets

The demo e-commerce project is mounted read-only from `fixtures/demo_ecommerce/`.

## Stopping

```bash
docker compose down
```

Metadata and `.knownow/` state persist in Docker volumes. To fully reset:

```bash
docker compose down --volumes
```

## Adding dbt

The dbt runner is opt-in. To include it:

```bash
docker compose --profile dbt up
```

This adds a `dbt-postgres` container that compiles the demo project through the dbt adapter.

## Network exposure

By default the server binds to `127.0.0.1` on the host — it is not reachable from other machines.

If you need network access (e.g., sharing the dashboard on a local network), copy the override file:

```bash
cp docker-compose.override.yml.example docker-compose.override.yml
docker compose up
```

Read the warnings in that file before using it. The know-now server is not designed for authenticated multi-user access.

## Platform notes

| Platform | Notes |
| -------- | ----- |
| Linux | Native Docker. Works out of the box. |
| macOS | Docker Desktop or Colima. Works out of the box. |
| Windows | WSL2 + Docker Desktop recommended. Run `docker compose` from a WSL2 shell. |

## Volumes

| Volume | Purpose |
| ------ | ------- |
| `pgdata` | PostgreSQL data directory |
| `knownow-state` | Engine state (`.knownow/` — audit log, run logs, cache) |

Both persist across `docker compose down` and are removed with `--volumes`.

## Logging

All services stream logs through Docker's logging driver:

```bash
docker compose logs                    # all services
docker compose logs know-now-server    # server only
docker compose logs -f                 # follow
```

## Building images

The Dockerfile uses a multi-stage build:

1. **rust-builder** — compiles the `know-now` binary with `--release --locked`
2. **web-builder** — builds the React dashboard with `pnpm build`
3. **runtime** — `debian:bookworm-slim` with just the binary + static assets

Rebuild after code changes:

```bash
docker compose build
docker compose up
```
