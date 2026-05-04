# Architecture Overview

This document describes know-now's architecture at a level suitable for contributors and evaluators. For the authoritative specification, see [PRD §8](PRD.md). For implementation invariants, see [AGENTS.md §4](../AGENTS.md).

## Pipeline

Metadata flows through a fixed pipeline. Each stage has a single owner crate and well-defined inputs/outputs.

```
metadata/  ──→  Parser  ──→  Validator  ──→  Contract  ──→  Generators  ──→  Writer  ──→  generated/
  (YAML)       (metadata)   (validate)     (contract)     (gen_*)          (writer)      (artifacts)
                                ↕                            ↕
                             Policy                      Templates
                            (policy)                    (templates)
```

1. **Parser** (`know_now_metadata`) — reads YAML files from `metadata/`, produces an `AuthoringMetadata` struct. The only crate that touches YAML parsers.
2. **Validator** (`know_now_validate`) — checks constraints, produces diagnostics, builds the canonical `ProjectGraph`.
3. **Contract** (`know_now_contract`) — projects the graph onto a versioned `GeneratorContract`. Generators consume this, never raw metadata.
4. **Generators** (`know_now_gen_*`) — each crate produces artifact descriptors for one target (Postgres DDL, dbt, docs, ER diagrams, fixtures, quality contracts). Generators do not depend on each other.
5. **Writer** (`know_now_writer`) — receives artifact descriptors, enforces path safety, detects manual edits, handles stale cleanup, and atomically promotes output to `generated/`.

## Crate map

| Crate | Responsibility |
| ----- | -------------- |
| `know_now_cli` | CLI binary, command dispatch, user-facing output |
| `know_now_core` | Project loading, pipeline orchestration |
| `know_now_metadata` | YAML parser, authoring model, span tracking |
| `know_now_validate` | Constraint checking, diagnostics, project graph |
| `know_now_contract` | Generator contract projection |
| `know_now_identity` | Stable ID generation and backfill |
| `know_now_codegen` | Generator trait definitions |
| `know_now_ir` | Typed DDL intermediate representation |
| `know_now_gen_postgres` | PostgreSQL DDL generator |
| `know_now_gen_dbt` | dbt project generator (models, sources, tests) |
| `know_now_gen_docs` | Markdown documentation generator |
| `know_now_gen_er` | Mermaid ER diagram generator |
| `know_now_gen_quality` | Quality contract generator |
| `know_now_gen_fixtures` | Synthetic fixture data generator |
| `know_now_writer` | Artifact writer, atomic promotion, manifest |
| `know_now_lock` | Lockfile (know-now.lock) serialization |
| `know_now_diff` | Metadata diff engine, change classification |
| `know_now_policy` | Policy pack engine, dc_standard rules |
| `know_now_templates` | Template pack renderer (restricted MiniJinja) |
| `know_now_cache` | Content-addressed generation cache |
| `know_now_toolchain` | dbt toolchain detection and validation |
| `know_now_diagnostics` | Diagnostic model, error codes |
| `know_now_audit` | Audit log (CLI command recording) |
| `know_now_catalog` | Approved-version catalog, drift classification |
| `know_now_server` | Local axum HTTP server, dashboard API |
| `know_now_fitness` | Architecture fitness tests (CI) |

## Key invariants

These are enforced by architecture fitness tests in CI (see [AGENTS.md §4](../AGENTS.md)):

- **Generators never read YAML.** They consume only the validated `GeneratorContract`.
- **Parser dependencies are isolated to `know_now_metadata`.** No other crate may depend on YAML parser crates.
- **Generator crates have no cross-dependencies.** Adding a new generator requires no changes to existing ones.
- **All writes go through `know_now_writer`.** No generator writes files directly.
- **Policy validation cannot mutate metadata.** Defaults are applied through explicit, traceable resolution.
- **Identical input produces byte-identical output** across all supported platforms.

## Ownership boundaries

| Path | Owner | Rule |
| ---- | ----- | ---- |
| `metadata/` | User | Never rewritten by the engine |
| `custom/` | User | Never written by know-now |
| `generated/` | Engine | Recreated atomically with manual-edit detection |
| `.knownow/` | Engine state | Cache, manifests, audit log, run logs |
| `know-now.yml` | User | Created by `init`, modified only by explicit commands |
| `know-now.lock` | Engine | Records resolved versions and hashes |

## Local server and dashboard

The local server (`know_now_server`) is a Rust axum application that serves:
- Read-only API at `/api/v1/*` (entities, relationships, manifest, docs, generation status, review state)
- The React/TypeScript dashboard (static assets built from `web/`)
- Launch-token → session-cookie authentication for browser access

The server binds to `127.0.0.1` by default. Network exposure requires explicit flags and is documented with warnings (PRD §13.2, AGENTS.md §8).

## Safety model

- No raw SQL string interpolation — all DDL goes through the typed IR and dialect emitters
- No telemetry, no outbound network calls in core commands
- Template packs run in a restricted renderer profile with fuel limits, no code execution, no filesystem escape
- Support bundles redact secrets
- Audit log records every CLI command
