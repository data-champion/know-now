# ADR-0005: Frontend package manager — pnpm

- **Status:** Accepted
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §8.2, §17.4 (NFR-I9), §17.6 (NFR-M7), §18.2

## Context

The Phase 3 dashboard is a TypeScript + React + Vite application under `web/`. Choosing a frontend package manager affects:

- **Reproducibility** — lockfile semantics and respect for resolution decisions across machines and CI.
- **Install speed and disk usage** — the CLI/dashboard developer loop should not be heavyweight.
- **Determinism** — `--frozen-lockfile`-style behavior must be available and reliable in CI.
- **Supply-chain posture** — strict-by-default package access and predictable hoisting reduce install-time foot-guns.
- **Compatibility with how the dashboard is shipped** — bundled assets travel with the Rust server (PRD §13.4, NFR-I9), so install correctness matters at release time, not just locally.

## Decision

Use **pnpm** as the frontend package manager.

- The pnpm version is pinned in `web/package.json` via the `packageManager` field.
- `web/pnpm-lock.yaml` is committed and reviewed like any other dependency artifact.
- CI installs use `pnpm install --frozen-lockfile`.
- `package-lock.json`, `yarn.lock`, `bun.lock`, and `bun.lockb` must **not** be committed under `web/`.
- CI explicitly installs or activates the pinned pnpm version rather than assuming Corepack is available with the active Node.js.
- The Node.js requirement in `engines.node` tracks the active Vite minimum.

## Alternatives considered

- **npm**: ubiquitous, but `package-lock.json` semantics have produced reproducibility surprises across npm versions, and npm's hoisting model is more permissive than we want.
- **Yarn (Berry)**: capable, but the workspace integrates cleanly with pnpm's strict-by-default model, and Yarn's PnP mode would conflict with editor/tooling assumptions in the dashboard.
- **Bun**: fast, but the install/runtime story is still moving and the lockfile format is younger; we don't want to underwrite that risk for a dashboard that has to ship bundled with the Rust server.
- **No package manager pinning** (rely on whatever the developer has): rejected — this is precisely the failure mode `know-now.lock` exists to prevent on the Rust side, and we hold the dashboard to the same standard.

## Consequences

Positive:

- Strict node_modules layout reduces "works on my machine" failures.
- Lockfile semantics are well-understood and respected by `--frozen-lockfile`.
- Dashboard build artifacts are reproducible, which matters for Phase 3 releases that bundle them with the server (PRD §13.4, §20.3).
- Architecture fitness tests can require pnpm-specific behavior (NFR-M7) without ambiguity.

Negative / costs:

- Some contributors will need to install pnpm explicitly. The pinned `packageManager` field plus a one-line `npm install -g pnpm@<version>` (or `corepack enable && corepack use pnpm@<version>`) keeps this small.
- We commit to keeping the pinned pnpm version in sync with the Node.js / Vite combination supported in CI.

## References

- PRD §8.2 — Rust workspace + `web/` layout.
- PRD §17.4 NFR-I9 — pnpm version pinned and frozen-lockfile CI.
- PRD §17.6 NFR-M7 — frontend dependency installation policy.
- PRD §18.2 — contributor setup, frontend package-manager policy.
- PRD §20.1 — frontend CI rules.
