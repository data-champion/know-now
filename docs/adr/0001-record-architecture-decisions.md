# ADR-0001: Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §24

## Context

know-now has several long-lived architectural choices that need to be discoverable, justified, and revisitable: parser selection, template renderer model, frontend package manager, ownership boundaries, the multiple versioned compatibility surfaces (metadata schema, generator contract, lockfile schema, local API, renderer profile), and supply-chain posture. The PRD records *what* the current stance is in §24, but doesn't capture *why*, what alternatives were considered, or what the consequences are.

Without an explicit record, two failure modes recur over time:

1. Decisions are revisited from scratch because no one remembers the tradeoffs.
2. New contributors silently take a different direction because the existing one looks arbitrary.

Both are expensive on a project that promises **reproducible deterministic output across versions**.

## Decision

We use **Architecture Decision Records (ADRs)** to record significant architectural choices in this repository. The format follows Michael Nygard's original template, lightly adapted (status, date, deciders, related, context, decision, alternatives, consequences, references). ADRs live under `docs/adr/`. The process and triggering criteria are documented in [`docs/adr/README.md`](README.md).

The PRD §24 decisions table remains the high-level summary. ADRs are the place where decisions are explained, alternatives are weighed, and supersession is recorded.

## Alternatives considered

- **PRD-only decisions** (status quo): Keep all decisions inside the PRD. Rejected because (a) the PRD is already large and reads as product spec rather than design history, (b) supersession needs a discoverable trail without rewriting the PRD, and (c) ADRs are a well-understood industry pattern that lowers the barrier for new contributors and AI assistants.
- **Free-form design docs**: Allow unstructured `docs/design/` notes. Rejected because the lack of structure makes "is this current?" and "what was decided?" hard to answer at a glance.
- **Issue tracker only**: Record decisions in beads issue threads. Rejected because issues are about *work*; decisions need to outlive the work that produced them and remain readable without an issue-tracker round-trip.

## Consequences

Positive:

- Architectural choices are versioned with the codebase alongside the implementation that introduces them.
- New contributors can read `docs/adr/` and understand the design without reverse-engineering.
- Supersession history is explicit; we never lose the *why* behind a reversal.
- AI assistants can be pointed at the ADR set as a curated, high-signal context window.

Negative / costs:

- Slight overhead per architectural change — but the trigger criteria in the README scope this to genuinely architectural decisions.
- ADRs that go stale without being updated mislead readers. Mitigation: status field is mandatory, and supersession must be recorded explicitly.

## References

- Michael Nygard, "Documenting Architecture Decisions" — the original ADR pattern.
- PRD §24 — Decisions and remaining open decisions.
- [`docs/adr/README.md`](README.md) — process, triggers, and index.
