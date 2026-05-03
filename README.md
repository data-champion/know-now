# know-now

> **One metadata source of truth. Deterministic generated artifacts. Clear ownership boundaries. Safe regeneration. Fast feedback. Traceable decisions.**

know-now is a local-first, metadata-driven data platform generation engine. Users define the *what* of an analytical data platform — domains, modules, entities, attributes, relationships, source mappings, semantic types, business keys, quality rules, ownership, governance, and open questions — in YAML. know-now deterministically generates the *how*: PostgreSQL DDL, dbt projects, dbt tests, provider-neutral data quality contracts, Markdown documentation, Mermaid ER diagrams, manifests, and traceability metadata.

**Stack:** Rust (CLI + engine + local server) and TypeScript (dashboard).
**Distribution model:** Native CLI with prebuilt binaries and a Cargo source-build fallback.
**Runtime model:** Offline by default. No telemetry. No required cloud services.

---

## Repository status

This repository is in the **architecture-contract spike** phase (PRD Phase 1). The product specification is complete; the implementation has not yet started.

The canonical source of truth for product, architecture, and scope decisions is [`docs/PRD.md`](docs/PRD.md). All other documents reference, summarize, or instantiate the PRD — never override it.

| Phase | Status | Notes |
| ----- | ------ | ----- |
| 1 — architecture contract spike | not started | Workspace, parser spike, generator contract, writer PoC, deterministic manifest, one DDL + one Markdown artifact. |
| 2A — Rust CLI MVP | not started | First publicly usable CLI. |
| 2B — dbt, quality, diagrams, validation | not started | |
| 3 — change safety, dashboard, admin | not started | |
| 4+ — collaboration, intelligence | not started | |

See PRD §6 (Product phases) and §22 (Recommended build sequence).

---

## Quick links

- **Product spec:** [`docs/PRD.md`](docs/PRD.md)
- **Documentation index:** [`docs/README.md`](docs/README.md)
- **Agent / contributor guide:** [`AGENTS.md`](AGENTS.md)
- **Human contributor guide:** [`CONTRIBUTING.md`](CONTRIBUTING.md)
- **Architecture decision records:** [`docs/adr/`](docs/adr/)
- **Repo layout:** [`docs/dev/repo-layout.md`](docs/dev/repo-layout.md)
- **Versioning policy:** [`docs/dev/versioning.md`](docs/dev/versioning.md)
- **Commit conventions:** [`docs/dev/commit-conventions.md`](docs/dev/commit-conventions.md)

---

## Who this is for

| Persona | Primary surface |
| ------- | --------------- |
| Solo data consultant (Marco) | CLI, generated dbt/DDL/docs, dashboard demos, review exports |
| Data engineer / OSS adopter (Priya) | CLI, demo project, JSON Schema autocomplete, CI integration, declarative template packs |
| Non-technical stakeholder (David) | Local dashboard, review summary, generated docs, ER diagrams |
| Project administrator (Sanne) | `doctor`, `policy status`, `admin scan`, approved-version catalog |
| Maintainer | Rust workspace, fixtures, compatibility matrix, release pipeline |

See PRD §3 for the full personas.

---

## Installation (placeholder)

The CLI is not yet published. When releases land, the supported install paths will be:

```bash
# Recommended (requires cargo-binstall and published binary metadata)
cargo binstall know-now

# Source-build fallback
cargo install --locked know-now

# Direct binary download from GitHub releases (Linux, macOS, Windows)
```

See PRD §18 for distribution requirements and §20.3 for release artifacts.

---

## Local development (placeholder)

Once the workspace exists, contributors will use:

```bash
cargo build
cargo test
cargo xtask check

cd web
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

Until the workspace is scaffolded, see [`CONTRIBUTING.md`](CONTRIBUTING.md) for what to expect, [`AGENTS.md`](AGENTS.md) for invariants that must hold from the first line of code, and [`docs/dev/repo-layout.md`](docs/dev/repo-layout.md) for the planned top-level layout.

---

## Issue tracking

This repository uses [beads](https://github.com/) (CLI: `br`) for project-local issue tracking. Issues use the prefix `know-now`. See [`AGENTS.md`](AGENTS.md#issue-tracking-and-workflow) for the workflow rules used in this repo.

---

## Reporting issues and contributing

- Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before pushing changes. (Note: this repo is trunk-based — work lands directly on `main` after cross-agent review; there are no feature branches and no pull requests. See [`AGENTS.md`](AGENTS.md) §7.6 / §7.8.)
- Significant architectural choices are recorded as ADRs under [`docs/adr/`](docs/adr/).
- Product/scope questions belong against the PRD. Open an issue or proposal that cites the PRD section.

---

## License

License selection is tracked as an open decision (PRD §24). Generated output must remain unencumbered. Do not introduce dependencies that would compromise that until the project license is decided.
