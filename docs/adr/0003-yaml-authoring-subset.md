# ADR-0003: Constrained YAML authoring subset

- **Status:** Accepted
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §10.2, §17.2 (NFR-S21), §17.6 (NFR-M8), ADR-0002

## Context

YAML's full feature set includes anchors, aliases, merge keys, custom tags, include-style directives, and multi-document files. These features are technically valid YAML, but they:

- Create ambiguous semantics that break **deterministic merging** and future **targeted metadata patching** (PRD §10.11).
- Make **source-aware diagnostics** harder to author and to read.
- Enable footguns (alias loops, runaway expansion) that complicate **parser budgets** (NFR-S21).
- Vary in implementation across YAML parsers and editors, threatening **portability** and **reproducibility**.

know-now's metadata is also intended to remain readable by editors, CI, documentation tooling, and a future visual UI without requiring those tools to understand the full YAML spec.

## Decision

know-now defines and enforces a **constrained YAML authoring subset** for all metadata files:

- Top-level mapping documents only.
- Scalar string keys.
- Strings, numbers, booleans, nulls, sequences, and mappings as values.
- **No anchors.**
- **No aliases.**
- **No merge keys** (`<<:`).
- **No custom tags** (`!!something`, `!Custom`).
- **No include directives.**
- **No multi-document files** (no `---` separators in metadata).
- **Duplicate keys are errors.**
- **Excessive nesting** is rejected.
- **Excessive file size** is rejected.
- **Unsupported scalar forms** that produce ambiguous metadata semantics are rejected.

Enforcement happens at the parser layer (per ADR-0002), so unsupported features produce source-aware diagnostics with file, line, column, and YAML path **before** semantic validation runs.

## Alternatives considered

- **Full YAML 1.2**: maximum expressiveness, but breaks every motivation in the Context section.
- **Allow anchors/aliases for DRY metadata**: a frequent ask. Rejected because it complicates targeted patching, manifesting, and incremental generation, and because policy packs and shared metadata fragments are better answers than YAML mechanics.
- **Switch to JSON or TOML**: JSON is hostile to humans for metadata of this shape; TOML doesn't fit nested document structures well. The constrained YAML subset preserves the readability win without the footguns.
- **Strict YAML profile defined only by docs (no parser enforcement)**: rejected — silent acceptance of unsupported features defeats the purpose. The subset must be enforced by the parser/event layer with diagnostics.

## Consequences

Positive:

- Predictable metadata shape across editors, CI, and tools.
- Fast, reliable source-aware diagnostics.
- Future metadata-writing commands (`id backfill --apply`, visual UI patches) are tractable.
- Easier to teach and review.

Negative / costs:

- Some legitimate convenience patterns (anchor reuse) are unavailable. We provide policy packs, defaults, and (later) shared metadata constructs as alternatives.
- The subset requires explicit parser configuration / event validation, which adds implementation cost to `know_now_metadata`. The cost is bounded; the parser fixtures listed in PRD §10.2 cover the unsupported set.

## References

- PRD §10.2 — YAML authoring subset and parser requirements.
- PRD §17.2 NFR-S21 — pre-scan/event validation rejecting unsupported features.
- ADR-0002 — Parser selection (the parser is what enforces this subset).
