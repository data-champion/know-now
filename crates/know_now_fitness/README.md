# know_now_fitness — Architecture Fitness Tests

Mechanical enforcement of the architecture invariants in AGENTS.md §4 and
PRD §17.6.  These tests run in CI via `cargo test -p know_now_fitness` and
as part of `cargo xtask check`.

## How it works

The crate uses `cargo_metadata` to inspect the workspace dependency graph
at test time and asserts facts about which crates may depend on which.
Code-level invariants (API surface constraints) are enforced when the
relevant crates gain implementation.

## Adding a new invariant

1. Identify the invariant, the PRD/AGENTS.md section, and the crates it
   constrains.
2. Add a test function in `tests/architecture.rs` following the naming
   convention `invNN_short_description`.  Use the next available number.
3. Include a section comment block naming the invariant and its source.
4. The assertion message must name the violated invariant, cite the rule
   source, and list every offending crate.
5. If you need new helper functions (e.g., checking feature flags), add
   them to `src/lib.rs`.
6. Run `cargo test -p know_now_fitness` locally.  All tests must pass.
7. Never use `#[ignore]` on a fitness test.

## Test categories

- **Dependency-graph invariants** — checked via `transitive_deps` /
  `direct_deps` against the resolved `cargo metadata` output.
- **Code-level invariants** — structural proxies today, replaced by
  compile-time API gates or runtime guards as crates gain implementation.
- **Negative controls** — unit tests in `src/lib.rs` that verify the
  detection logic itself works (e.g., a known dependency is found).
