# Architecture Decision Records

This directory holds **Architecture Decision Records** (ADRs) for know-now. ADRs capture significant architectural choices with their context, alternatives, and consequences, so that future maintainers don't have to reverse-engineer the *why* from the code.

The format is loosely based on Michael Nygard's original template, lightly adapted for this repo. A blank template lives at [`0000-template.md`](0000-template.md).

## When to write an ADR

Write an ADR when the change involves any of the following:

- Adopting, removing, or replacing a foundational dependency (parser, template engine, server framework, dashboard framework, package manager).
- Changing a versioned compatibility surface: metadata schema, generator contract, lockfile schema, local API contract, renderer profile.
- Introducing a new isolation boundary (new crate boundary, new permission model, new sandbox).
- Reversing a previous architectural decision.
- Choosing between two or more credible options where the tradeoff is non-obvious and the choice is durable.

You **do not** need an ADR for:

- Implementation details that don't change a public surface.
- Bug fixes.
- Refactors within an existing crate that don't affect the dependency graph.
- Routine dependency upgrades.

If in doubt, write the ADR. They are cheap; archaeology is expensive.

## Process

1. Copy [`0000-template.md`](0000-template.md) to `<NNNN>-<slug>.md` using the next available 4-digit number.
2. Fill in **Context**, **Decision**, **Alternatives considered**, **Consequences**, and **References**.
3. Set **Status** to `Proposed`. Open a beads issue (`br`) under the appropriate Epic and discuss in the bead's `[know-now-NN]` agent-mail thread (this repo is trunk-based — there are no PRs; see [`../../AGENTS.md`](../../AGENTS.md) §7.6).
4. On agreement, set status to `Accepted`. Stage the ADR file and request reviewer-agent review per [`../../AGENTS.md`](../../AGENTS.md) §7.8. After `Approved`, commit and push directly to `main`.
5. If a later ADR replaces this one, set status to `Superseded by ADR-NNNN` (do not delete the original — it remains the historical record).
6. Add an entry to the **Index** below in the same commit.
7. Reference the ADR from the relevant PRD section, from related code (where applicable), and from the introducing commit body.

ADRs are written in the past or present tense ("We chose…", "We use…"), not the future tense — once accepted, they describe what *is*, not what *will be*.

## Relationship with the PRD

The PRD §24 decisions table summarizes high-level decisions and their status. ADRs go deeper: context, alternatives, and consequences. Where both exist, the PRD names the decision and the ADR explains it. New decisions made after the PRD was last revised should appear here first; the PRD is amended when the decision becomes load-bearing for product or scope.

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-yaml-parser-serde-saphyr.md) | YAML parser: `serde-saphyr` confirmed | Confirmed |
| [0003](0003-yaml-authoring-subset.md) | Constrained YAML authoring subset | Accepted |
| [0004](0004-template-renderer-minijinja-restricted.md) | Restricted MiniJinja-based template renderer profile | Accepted |
| [0005](0005-frontend-package-manager-pnpm.md) | Frontend package manager: pnpm | Accepted |
| [0006](0006-trunk-based-development-main-no-prs.md) | Trunk-based development on `main` with no feature branches or PRs | Accepted |
| [0007](0007-review-gate-different-agent-via-mcp-agent-mail.md) | Code-review gate by a different agent via mcp-agent-mail | Accepted |
| [0008](0008-structured-logging-tracing.md) | Structured logging via `tracing` | Accepted |
