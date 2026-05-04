# Foundation E2E Driver

Entry point: `cargo xtask e2e foundation`

## What it checks

The foundation E2E is the "is the foundation healthy?" integration driver.
It verifies that all foundation-phase surfaces work together on a checkout.

| Step | Command | What it proves |
|------|---------|----------------|
| 1 | `cargo build --workspace` | Every crate compiles |
| 2 | `cargo test --workspace` | All tests pass |
| 3 | `cargo fmt --all -- --check` | Formatter clean |
| 4 | `cargo clippy --all-targets --all-features -- -D warnings` | No lint warnings |
| 5 | `cargo deny check` | License + advisory + banned-deps |
| 6 | `cargo test -p know_now_fitness` | Architecture fitness (PRD §17.6) |
| 7 | `pnpm install && typecheck && build` in `web/` | Frontend boot (skipped if `web/package.json` absent) |
| 8 | `br dep cycles` | Beads graph has no cycles (skipped if `br` unavailable) |
| 9 | `xtask docs check` | ADR index in sync |

## Running locally

```bash
cargo xtask e2e foundation
```

The driver prints per-step progress and a final count of steps passed.
Exit code 0 means all steps passed; non-zero means at least one failed.

## Interpreting a failure

Each step runs sequentially and fails fast. The output shows which step
failed and the underlying command's stderr. Fix that step before re-running.

Common failures:

- **Step 3 (fmt):** Run `cargo fmt --all` to fix.
- **Step 4 (clippy):** Read the warnings; they often point at unused imports or missing docs.
- **Step 5 (deny):** Check `deny.toml` for banned crate rules or advisory IDs.
- **Step 6 (fitness):** An architecture invariant from AGENTS.md §4 was violated. The assertion message names the specific invariant.
- **Step 7 (frontend):** Check `web/package.json` and `pnpm-lock.yaml`.
- **Step 8 (beads):** Run `br dep cycles` to see the cycle.

## Adding a new foundation gate

1. Add the step to `run_e2e_foundation()` in `xtask/src/lib.rs`.
2. Increment the step number in the progress output.
3. Update this table.
4. Run `cargo xtask e2e foundation` to verify.

## Banned-dep regression fixture

The `tests/fitness/banned_dep_regression/` directory contains a
`Cargo.toml.fixture` showing what a banned-dependency violation looks like.
Step 6 exercises this indirectly: the architecture fitness tests
(`crates/know_now_fitness/tests/architecture.rs`) verify that no generator
or non-metadata crate depends on banned YAML parser crates. The negative
controls in `crates/know_now_fitness/src/lib.rs` prove the detection
machinery works.

## Event shape schema

The log event shape is committed at
`tests/logging/expected_event_shape.schema.json` and validated by
`cargo xtask logs validate <events.jsonl>`.

## Related beads

- **know-now-42e.11** — this driver
- **know-now-42e.10** — structured logging facade
- **know-now-42e.4** — architecture fitness harness
- **know-now-42e.7** — xtask runner
