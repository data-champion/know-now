# Documentation index

This directory contains all repository documentation. Each file has a single responsibility — start with the PRD for product/architecture questions, and use the other guides for *how* to work in the repo.

## Source of truth

- [`PRD.md`](PRD.md) — Product Requirements Document. Canonical source for product, scope, architecture, requirements, NFRs, phase boundaries, and decisions. Everything else in this repo defers to it.

## For contributors

These live at the repository root for discoverability:

- [`../README.md`](../README.md) — project overview and entry point.
- [`../AGENTS.md`](../AGENTS.md) — invariants and conventions for humans and AI agents.
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — how to set up, change, commit, and propose work.

## Architecture decision records

- [`adr/`](adr/) — ADRs for significant architectural choices. Start with [`adr/README.md`](adr/README.md) for the process and index.

## Developer reference

- [`dev/repo-layout.md`](dev/repo-layout.md) — full top-level repository layout, ownership of each directory, and what is committed vs ignored.
- [`dev/versioning.md`](dev/versioning.md) — versioning policy across engine, metadata schema, generator contract, local API, lockfile schema, renderer profile, and packs.
- [`dev/commit-conventions.md`](dev/commit-conventions.md) — Conventional Commits guide tailored to this repo.

## User reference

Metadata authoring guides:

- [`user/metadata-reference.md`](user/metadata-reference.md)
- [`user/yaml-subset.md`](user/yaml-subset.md)
- [`user/logical-types.md`](user/logical-types.md)
- [`user/semantic-types.md`](user/semantic-types.md)
- [`user/governance.md`](user/governance.md)
- [`user/open-questions-assumptions.md`](user/open-questions-assumptions.md)
- [`user/domains-modules.md`](user/domains-modules.md)

Additional user docs (CLI details, dbt customization, policy packs, template packs, CI/CD recipes, troubleshooting) continue to land by phase. The full list of planned guides is in PRD §19.1.

## Operations reference (placeholder)

Operational topics (release process, supply-chain artifacts, compatibility matrix, support bundles, admin scan workflows) will appear under `docs/ops/` once the corresponding features ship. See PRD §17.8 (supply-chain), §20 (CI/CD), §16.7 (administration).

---

If you can't decide where a new doc belongs, ask in the relevant beads issue or PR — and prefer adding to an existing file over creating a new one. Documentation that nobody can find is documentation that doesn't exist.
