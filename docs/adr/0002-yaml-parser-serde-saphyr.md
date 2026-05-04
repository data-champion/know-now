# ADR-0002: YAML parser — `serde-saphyr` primary, `marked-yaml` fallback

- **Status:** Confirmed (Phase 1 spike validated 2026-05-04)
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §10.2, §17.2 (NFR-S13, NFR-S19, NFR-S21), §17.6 (NFR-M8), §22 step 2

## Context

know-now's metadata is YAML, edited by humans, validated by the engine, and ultimately the source of truth for deterministic generation. The parser choice affects:

- **Diagnostics quality** — file/line/column source spans, YAML paths, and clear errors for unsupported features. PRD §12.4 and the persona work depend on excellent diagnostics.
- **Safety** — duplicate-key rejection, parser budgets (file size, nesting, expansion), rejection of advanced YAML features that would compromise deterministic merging or future targeted patching.
- **Determinism** — stable mapping/sequence handling.
- **Maintenance posture** — the YAML ecosystem has a history of unmaintained or vulnerable parsers (NFR-S19).
- **Type ergonomics** — Serde-compatible deserialization into typed Rust metadata structs is preferred over hand-rolled YAML node walking.

Generators must never depend on the parser; YAML access is isolated to `know_now_metadata` (PRD §10.2, NFR-M8). The parser is therefore swappable without touching generator crates.

## Decision

The primary parser/deserializer is **`serde-saphyr`**. It provides Serde-compatible typed deserialization on top of the actively maintained `saphyr-parser` event stream and supports source spans, parser budgets, and event-level validation suitable for enforcing the know-now YAML subset (see ADR-0003).

If the Phase 1 parser spike cannot make `serde-saphyr` enforce the know-now YAML subset (anchors, aliases, merge keys, custom tags, include directives, multi-document files, duplicate keys, excessive nesting, excessive file sizes) **with high-quality source-aware diagnostics**, we fall back to **`marked-yaml`** as the secondary candidate. Direct `saphyr-parser` usage is reserved for a possible custom parser layer if neither option meets the bar.

## Alternatives considered

- **`serde_yaml`**: previously the de-facto standard. Rejected: project archived/unmaintained, and known to under-report diagnostic detail.
- **`serde_yml`**: a fork of `serde_yaml`. Rejected: maintenance posture and behavior under our diagnostic and budget requirements are not better enough to justify carrying it.
- **`yaml-rust2`**: maintained, but Serde integration and source-span ergonomics for typed metadata are weaker than `serde-saphyr` for our use case.
- **C/FFI YAML parsers** (`libfyaml`, `libyaml`): higher diagnostic potential, but the dependency policy disallows C/FFI parser dependencies absent explicit approval (NFR-S19). Risk/benefit does not favor it for a Rust workspace.
- **Custom parser on top of `saphyr-parser` events directly**: maximum control, highest implementation cost. Reserved as the fallback-of-last-resort if both ergonomics-focused options fail.

## Consequences

Positive:

- Typed deserialization aligns with the rest of the Rust workspace and reduces hand-rolled traversal code.
- Source-span and event-level validation lets us reject the unsupported subset *before* semantic validation, with diagnostics that include file, line, column, and YAML path.
- Parser concerns are isolated to one crate, so swapping later (per the fallback rule) does not ripple.

Negative / risks:

- `serde-saphyr` is younger and less battle-tested than `serde_yaml`. Mitigation: extensive parser fixtures (PRD §10.2 list and §20.1), the explicit fallback rule, and architecture-fitness tests verifying parser isolation.
- We commit to dependency-policy review (NFR-S19) on every parser-related upgrade.

Phase 1 exit criteria (PRD §23.1) include parser-spike validation: unsupported-feature diagnostics, duplicate-key rejection, and parser budget tests. If those fail, switching to `marked-yaml` per the fallback rule is in-scope and pre-approved by this ADR.

## Phase 1 Spike Validation (2026-05-04)

Evaluation against PRD §10.2 criteria:

| Criterion | Verdict | Notes |
|-----------|---------|-------|
| Subset enforcement quality | Pass | `validate_subset()` pre-scan rejects anchors, aliases, merge keys, custom tags, include directives, multi-document. 11 negative fixture tests. |
| Diagnostic quality | Pass | Error variants include `Location` (line/column). know-now's wrapper currently discards location from deserialization errors — fix is in-scope, not a parser limitation. |
| Source-span fidelity | Pass | `serde-saphyr::Error` variants carry `Location` with line/column. Pre-scan errors report line numbers. |
| Parser budget support | Pass | File size, nesting depth, node count, anchor count, alias count, keys-per-mapping — all enforced in `budgets.rs`. |
| Maintenance posture | Pass | v0.0.25, no advisories, actively maintained saphyr-rs project. `cargo deny check` clean. |
| Dependency depth | Acceptable | 11 transitive dependencies via serde-saphyr. No C/FFI. All MIT/Apache-2.0. |
| Audited license | Pass | MIT. All transitive deps pass `cargo deny check licenses`. |

**Decision: serde-saphyr confirmed.** The `marked-yaml` fallback is not needed. No switching required.

**Follow-up:** Extract line/column from `serde_saphyr::Error::location` in `parser.rs` deserialization error path to close the diagnostic quality gap (implementation issue, not a parser limitation).

## References

- PRD §10.2 — YAML authoring subset and parser requirements.
- PRD §17.2 NFR-S13, NFR-S19, NFR-S21 — parser-related security/safety requirements.
- PRD §17.6 NFR-M8 — parser dependency selection verified during dependency policy / fitness tests.
- PRD §22 step 2 — Phase 1 parser spike.
- PRD §23.1 — Phase 1 exit criteria.
- ADR-0003 — Constrained YAML authoring subset (this parser is what enforces it).
