# know-now

> **One metadata source of truth. Deterministic generated artifacts. Clear ownership boundaries. Safe regeneration. Fast feedback. Traceable decisions.**

know-now is a local-first, metadata-driven data platform generation engine. Users define the *what* of an analytical data platform — domains, modules, entities, attributes, relationships, source mappings, semantic types, business keys, quality rules, ownership, governance, and open questions — in YAML. know-now deterministically generates the *how*: PostgreSQL DDL, dbt projects, dbt tests, provider-neutral data quality contracts, Markdown documentation, Mermaid ER diagrams, manifests, and traceability metadata.

**Stack:** Rust (CLI + engine + local server) and TypeScript (dashboard).
**Distribution model:** Native CLI with prebuilt binaries and a Cargo source-build fallback.
**Runtime model:** Offline by default. No telemetry. No required cloud services.

---

## Quick start

```bash
# Install (see docs/user/install.md for all options)
cargo binstall know-now

# Create a demo project with sample metadata
know-now init --demo
cd demo-project

# Validate metadata
know-now validate

# Run the recommended check suite
know-now check

# Check with lockfile verification (CI mode)
know-now check --locked
```

See [`docs/user/demo.md`](docs/user/demo.md) for a guided five-minute walkthrough.

---

## Repository status

This repository is **Phase 1 complete; Phase 2A active**. The product specification is complete; the Rust workspace builds, metadata parsing, validation, deterministic generation pipeline, and CLI are functional.

The canonical source of truth for product, architecture, and scope decisions is [`docs/PRD.md`](docs/PRD.md).

| Phase | Status | Notes |
| ----- | ------ | ----- |
| 1 — architecture contract spike | complete | Workspace, parser, generator contract, writer, deterministic manifest, DDL + Markdown artifacts. |
| 2A — Rust CLI MVP | active | First publicly usable CLI. |
| 2B — dbt, quality, diagrams, validation | not started | |
| 3 — change safety, dashboard, admin | not started | |
| 4+ — collaboration, intelligence | not started | |

See PRD §6 (Product phases) and §22 (Recommended build sequence).

---

## Quick links

- **Product spec:** [`docs/PRD.md`](docs/PRD.md)
- **Install guide:** [`docs/user/install.md`](docs/user/install.md)
- **Demo walkthrough:** [`docs/user/demo.md`](docs/user/demo.md)
- **Metadata reference:** [`docs/user/metadata-reference.md`](docs/user/metadata-reference.md)
- **Agent / contributor guide:** [`AGENTS.md`](AGENTS.md)
- **Human contributor guide:** [`CONTRIBUTING.md`](CONTRIBUTING.md)
- **Architecture decision records:** [`docs/adr/`](docs/adr/)
- **Repo layout:** [`docs/dev/repo-layout.md`](docs/dev/repo-layout.md)
- **CI integration examples:** [`examples/ci/`](examples/ci/)

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

## Installation

The CLI is distributed as prebuilt binaries and a Cargo source-build fallback.

```bash
# Recommended (requires cargo-binstall)
cargo binstall know-now

# Source-build fallback
cargo install --locked know-now

# Direct binary download from GitHub releases (Linux, macOS, Windows)
```

See [`docs/user/install.md`](docs/user/install.md) for prerequisites, verification, and platform-specific details.

---

## Local development

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

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for contributor setup and [`AGENTS.md`](AGENTS.md) for invariants.

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
