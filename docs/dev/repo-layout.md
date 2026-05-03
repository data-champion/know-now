# Repository layout

This document describes the **planned** top-level layout of the know-now repository, the ownership of each directory, and what is committed vs ignored. It complements the **product project layout** described in PRD §9 (which describes how a generated *consumer* project is laid out, not this repo).

The repository is currently pre-implementation; many of the directories below do not yet exist. They will appear as Phase 1 and Phase 2A land.

## Top-level

```text
know-now/
├── README.md                       # Project overview and entry point
├── AGENTS.md                       # Invariants and conventions for humans/AI agents
├── CONTRIBUTING.md                 # Human contributor guide
├── LICENSE                         # (TBD — license selection is an open decision)
├── CHANGELOG.md                    # Changelog (Phase 2A onward)
├── Cargo.toml                      # Rust workspace manifest
├── Cargo.lock                      # Committed: reproducibility for binaries
├── rust-toolchain.toml             # Pinned Rust toolchain
├── deny.toml                       # cargo-deny config (licenses, advisories, bans, sources)
├── .editorconfig                   # Editor defaults
├── .gitattributes                  # Line endings, generated-marker hints
├── .gitignore                      # Top-level ignores
├── crates/                         # Rust workspace crates (see PRD §8.2)
├── web/                            # TypeScript dashboard (Phase 3)
├── examples/                       # Bundled example projects
├── fixtures/                       # Compatibility fixtures, parser fixtures, snapshots
├── docs/                           # All documentation
├── .beads/                         # beads issue-tracker DB and config
└── .claude/                        # Claude Code per-repo settings (gitignored content as needed)
```

## `crates/` — Rust workspace

The crate set is defined in PRD §8.2 and reproduced here for orientation. Each crate has a single, named responsibility; cross-crate boundaries are part of the architecture invariants (see [`AGENTS.md`](../../AGENTS.md) §4).

```text
crates/
├── know_now_cli/           # CLI entrypoint
├── know_now_core/          # orchestration, project loading, generation plans
├── know_now_diagnostics/   # diagnostics, source spans, renderers (text/JSON/SARIF)
├── know_now_metadata/      # metadata types, parsing, source spans, schema generation
│                           # (the ONLY crate allowed to depend on YAML parser deps)
├── know_now_contract/      # stable generator/API contract schemas
├── know_now_identity/      # stable object IDs, ID suggestions, rename matching
├── know_now_validate/      # semantic validation and policy validation
├── know_now_codegen/       # generator traits and artifact model
├── know_now_ir/            # typed SQL/DDL/documentation IR
├── know_now_writer/        # staging, path safety, ownership markers, atomic promotion
│                           # (the ONLY crate allowed to write to disk)
├── know_now_lock/          # lockfile schema, resolution, locked/unlocked checks
├── know_now_gen_postgres/  # PostgreSQL DDL generator
├── know_now_gen_dbt/       # dbt generator
├── know_now_gen_quality/   # quality contracts and dbt tests
├── know_now_gen_docs/      # Markdown and Mermaid generation
├── know_now_diff/          # graph diffing, ID matching, change classification
├── know_now_server/        # local axum API server
├── know_now_policy/        # policy pack loading and evaluation
├── know_now_templates/     # restricted MiniJinja-based template rendering
├── know_now_cache/         # content-addressed cache and dependency tracking
├── know_now_toolchain/     # external toolchain adapters (e.g. dbt validation)
├── know_now_audit/         # audit events, redaction, support-bundle summaries
└── xtask/                  # release, fixture, benchmark, maintenance tasks
```

**Hard rules** (see [`AGENTS.md`](../../AGENTS.md) §4 and PRD §17.6 for the full set):

- YAML parser dependencies only in `know_now_metadata`.
- All file writes go through `know_now_writer`.
- Generator crates have no direct cross-dependencies.
- Built-in generators and template packs do not write files; they return artifact descriptors.
- Policy validation cannot mutate metadata or the canonical graph.

## `web/` — Dashboard

```text
web/
├── package.json            # `packageManager` pins exact pnpm version
├── pnpm-lock.yaml          # COMMITTED. Other lockfiles are NOT permitted here.
├── tsconfig.json
├── vite.config.ts
├── src/
└── public/
```

The dashboard consumes the local API contract only via documented endpoints (or a generated TypeScript client). It does not reach into Rust internals. Build artifacts (`dist/`) are release outputs and are not committed.

## `examples/` — Bundled example projects

Realistic consulting-style projects that are runnable with `know-now init --demo` or referenced by `examples list`. These are committed so that demos and CI compatibility fixtures are reproducible.

## `fixtures/` — Test fixtures

```text
fixtures/
├── parser/                 # YAML parser fixtures (valid_*, anchor, alias, merge_key, ...)
├── compatibility/          # Per-PRD §20.2 compatibility fixtures
└── snapshots/              # Generator output snapshots
```

Each invalid parser fixture snapshots both text and JSON diagnostics (PRD §20.1).

## `docs/` — Documentation

```text
docs/
├── README.md               # Documentation index
├── PRD.md                  # SOURCE OF TRUTH for product/architecture/scope
├── adr/                    # Architecture Decision Records
│   ├── README.md
│   ├── 0000-template.md
│   └── NNNN-*.md
├── dev/                    # Contributor / maintainer reference
│   ├── repo-layout.md      # (this file)
│   ├── versioning.md
│   └── commit-conventions.md
├── user/                   # (placeholder) end-user reference, added with Phase 2A/2B features
└── ops/                    # (placeholder) release/operations reference
```

## `.beads/` — Issue tracking

beads-managed local issue database and config. The `beads.db*` files and operational state are gitignored per `.beads/.gitignore`; only `config.yaml`, `issues.jsonl`, and `metadata.json` are tracked.

## What is committed vs ignored

Committed:

- All source under `crates/`, `web/src/`, `examples/`, `fixtures/`, `docs/`.
- `Cargo.lock`, `web/pnpm-lock.yaml`, `rust-toolchain.toml`, `deny.toml`, `.editorconfig`, `.gitignore`, `.gitattributes`.
- ADRs and PRD.
- beads `config.yaml`, `issues.jsonl`, `metadata.json`.

Ignored:

- `target/` (Rust build output).
- `node_modules/`, `web/dist/`, `web/.vite/`.
- beads operational state (`beads.db*`, `daemon.*`, `bd.sock`, etc — see `.beads/.gitignore`).
- Local Claude Code session state.
- Per-machine editor / IDE state.

## What about generated *consumer* projects?

PRD §9.1 describes the layout of a generated **consumer** project (`my-knownow-project/` with `metadata/`, `generated/`, `custom/`, `.knownow/`). That layout is the *output* of `know-now init` — it does **not** describe this repository. Don't conflate the two.
