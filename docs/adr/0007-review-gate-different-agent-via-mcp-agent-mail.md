# ADR-0007: Code-review gate by a different agent via mcp-agent-mail

- **Status:** Superseded — policy reversed on 2026-05-05; the cross-agent review gate has been removed in favor of fresh-eyes self-review before commit. See AGENTS.md §7.3 step 5 and §7.6.
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** AGENTS.md §7.4, PRD §17.6, PRD §20.1

## Context

know-now uses trunk-based development with direct commits to `main`. Without pull requests, the project still needs a durable review gate that is explicit, auditable, and attached to bead-scoped work. Review quality must remain architecture-aware (AGENTS invariants) and independent from the implementer's own assessment.

Because implementation is coordinated through mcp-agent-mail threads and file reservations, the same system can carry review requests, findings, and final verdicts without introducing a second workflow tool.

## Decision

know-now requires a **separate reviewer-agent** to review each implementer's staged diff before commit:

- The implementer stages only reserved bead paths and posts a review request in the bead thread.
- The reviewer agent inspects `git diff --cached` and checks behavior against AGENTS invariants and relevant PRD sections.
- The reviewer returns one explicit verdict in-thread: `Approved`, `Changes requested`, or `Blocked`.
- The implementer may commit only after `Approved`, unless a maintainer override explicitly instructs otherwise.

The review record lives in mcp-agent-mail thread history, which is the authoritative audit trail in this workflow.

## Alternatives considered

- **Self-review only:** rejected because it removes independence and increases blind-spot risk.
- **Human-only ad hoc review in chat:** rejected because review outcomes become hard to audit and are not reliably tied to bead IDs.
- **Branch/PR review gate:** rejected as incompatible with the chosen trunk-based workflow and duplicate coordination overhead.

## Consequences

Positive:

- Preserves a strong review gate while staying fully trunk-based.
- Keeps review artifacts (request, findings, verdict) searchable and bead-linked.
- Encourages small staged payloads aligned with reservations, which improves review depth.

Negative / costs:

- Requires reviewer-agent availability to avoid throughput bottlenecks.
- Adds coordination overhead in thread hygiene and agent contact setup.
- Tooling outages in agent-mail can block review messaging and must be treated as operational incidents.

## References

- AGENTS.md §7.4 (mcp-agent-mail usage).
- PRD §17.6 (architecture fitness and enforcement posture).
- PRD §20.1 (CI/review quality expectations).
