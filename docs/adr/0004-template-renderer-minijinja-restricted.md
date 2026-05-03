# ADR-0004: Restricted MiniJinja-based template renderer profile

- **Status:** Accepted
- **Date:** 2026-05-02
- **Deciders:** Maintainer
- **Related:** PRD §15.1, §15.1.1, §17.2 (NFR-S22..S25), §17.6, §16.4 EXT-004, §22 step 35

## Context

Phase 3 introduces declarative template packs so that consultants and teams can produce custom artifacts (internal API docs, organization-specific scaffolding) from the canonical generator contract. The challenge is doing this **without** opening up arbitrary code execution, network access, or filesystem escape — three failure modes that every "template plugin" system tends to grow into.

Constraints that shape the choice:

- Generators (built-in or template-driven) consume only the validated `GeneratorContract`. They never see raw YAML or internal Rust-only graph types.
- All writes go through `know_now_writer`. Templates return artifact descriptors only; the writer enforces path safety, ownership markers, manual-edit detection, stale handling, and atomic promotion (NFR-S17, PRD §9.3, §15.1.1).
- Template rendering must be **deterministic** for identical input.
- Output must be testable, fixture-able, and version-pinnable via the lockfile.

## Decision

Custom template packs are rendered through `know-now-minijinja-v1` — a **restricted, versioned renderer profile** built on top of the [`minijinja`](https://github.com/mitsuhiko/minijinja) crate. The profile exposes a curated, deterministic surface; MiniJinja is an *implementation detail*, not the public compatibility surface.

The profile contract:

- Renderer profile name and version: `know-now-minijinja-v1`.
- Undefined behavior: **strict** (undefined values are errors).
- **No** custom function/filter/test/loader registration for custom packs.
- **No** dynamic include paths.
- Static includes / inheritance allowed only when statically resolvable inside the pack root.
- **No** environment access, **no** filesystem reads outside the pack root, **no** filesystem writes (artifacts flow through the writer), **no** network, **no** process execution, **no** database access.
- Required limits: render fuel, output byte size, template byte size, include depth, output file count.
- Built-in pure filters only: `snake_case`, `kebab_case`, `pascal_case`, `upper_case`, `lower_case`, `indent`, `sort_by`, `join`, `default`, `json`, `yaml`, `markdown_escape`, `html_escape`.
- Forbidden: current time, random values, environment lookup, filesystem lookup, network fetch, process execution, database access, host-specific path expansion.
- Trust classification: built-in / approved / experimental / untrusted; `untrusted` packs cannot be used in `--locked` CI mode unless policy explicitly permits.
- Lockfile and manifest record renderer profile name and version, pack name, version, and content hash.

## Alternatives considered

- **Tera**: another mature Rust template engine. Comparable safety story to MiniJinja, but `minijinja` has better extension-point control (we need to *deny* registration, which it makes straightforward) and a smaller, easier-to-reason-about surface.
- **Handlebars-rs**: less expressive, less ergonomic for the artifact shapes we expect.
- **Custom DSL**: maximum control, but a huge implementation cost and an ecosystem-isolation tax for users. Templates that look like Jinja are easier to author and to review.
- **Allow MiniJinja's full feature set**: would make the renderer easy to ship but would require us to constantly chase escape hatches users find. The whole point of versioning the *profile* is to not promise MiniJinja's feature set.
- **WASI sandbox now (Phase 3)**: better isolation, much higher implementation cost. Tracked for Phase 5 (PRD §15.3) when plugin demand is clearer.

## Consequences

Positive:

- Custom packs can produce real artifacts without any of the typical "templating system → arbitrary code execution" failure modes.
- The renderer is a **versioned compatibility surface**, so we can evolve internals without breaking pack authors.
- Lockfile and manifest visibility makes drift, trust, and reproducibility administrator-observable.
- Architecture fitness tests can verify (and do — PRD §17.6) that template packs cannot bypass the writer or register native MiniJinja extensions.

Negative / costs:

- Pack authors will occasionally hit the wall of "X is allowed by Jinja but not by `know-now-minijinja-v1`". Mitigation: strong diagnostics including template path, line, column, pack name, profile, and contract version (PRD §15.1.1, EXT-004 acceptance #12).
- Renderer profile compatibility must be tested with fixtures and release-note diff summaries (NFR-M9). This is fixture work, not architectural complexity.
- Adding new built-in filters becomes a profile-version question, not a casual change.

## References

- PRD §15.1 — Phase 3 declarative template packs.
- PRD §15.1.1 — Restricted template renderer architecture.
- PRD §17.2 NFR-S22..S25 — template safety requirements.
- PRD §17.6 — architecture fitness tests for template packs.
- PRD §16.4 EXT-004 — declarative template pack acceptance.
- PRD §22 step 35 — build sequence position.
- [`minijinja`](https://github.com/mitsuhiko/minijinja) — implementation crate.
