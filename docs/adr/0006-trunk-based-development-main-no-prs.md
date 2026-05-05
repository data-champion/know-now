# ADR-0006: Trunk-based development on `main` with no feature branches or PRs

- **Status:** Accepted
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §20.1, AGENTS.md §7.6, AGENTS.md §7.7

## Context

know-now is coordinated through beads and mcp-agent-mail threads, with small task-sized commits and explicit file reservations. The project wants high implementation velocity without hidden branch drift or long-lived integration work. Traditional feature-branch + PR workflows create duplicate state (branch review plus issue thread) and encourage large batches that are harder to validate against architecture invariants.

Implementer fresh-eyes self-review before commit (AGENTS.md §7.3 step 5) preserves review quality without requiring pull requests.

## Decision

know-now uses **trunk-based development on `main`**:

- All implementation work lands directly on `main`.
- Feature branches and pull requests are not part of the default workflow.
- Implementers stage only reserved bead paths, run a fresh-eyes self-review, then commit to `main`.
- Before pushing, implementers run `git pull --rebase` to keep history linear.

Conflict prevention is handled by mcp-agent-mail file reservations, not branch isolation.

## Alternatives considered

- **Feature branches + pull requests:** rejected because it duplicates coordination already handled by beads + agent-mail and increases cycle time for small bead-scoped changes.
- **Long-lived integration branches:** rejected because drift and merge risk accumulate, which conflicts with the project's deterministic and invariant-driven posture.
- **Direct commits without any review:** rejected because it weakens architecture-invariant enforcement; fresh-eyes self-review preserves a quality check while staying lightweight.

## Consequences

Positive:

- Faster bead-level delivery with less workflow overhead.
- Linear history by default (`rebase`, never merge commits for routine work).
- Coordination stays in one place: beads graph + per-bead threads + staged commits.

Negative / costs:

- Requires discipline: fine-grained commits and strict reservation hygiene are mandatory.
- Accidental broad staging is riskier on trunk and must be prevented by implementer fresh-eyes self-review.
- CI becomes even more critical because there is no PR buffer.

## References

- AGENTS.md §7.6, §7.7.
- PRD §20.1 (CI quality gate posture).
