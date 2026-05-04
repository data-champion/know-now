# ADR-0008: Structured logging via `tracing`

## Status

Accepted

## Context

know-now needs structured, span-based logging for pipeline stage timing
(NFR-P12), CI event assertions, support bundle capture, and
`--verbose`/`--debug`/`--format json` CLI knobs (PRD §12.2).

The choice of logging facade affects every crate in the workspace because
library crates emit spans and events, while the CLI/server subscriber
configures output.

## Decision

Use `tracing` (facade) + `tracing-subscriber` (output) as the structured
logging stack.

- The facade (`tracing`) is a dependency of library crates that emit events.
- The subscriber (`tracing-subscriber`) is configured only in entry points
  (CLI binary, server binary, test harness).
- Library crates never call `tracing::subscriber::set_global_default`; they
  accept whatever subscriber the caller installed.

### Banned alternatives

- `log` + `env_logger` as the primary facade. `env_logger` lacks span
  support, structured fields, and per-layer filtering. `log` is acceptable
  as a compatibility bridge (via `tracing-log`) but not as the primary API.

## Alternatives considered

| Option | Pros | Cons |
|--------|------|------|
| `log` + `env_logger` | Simpler API, smaller dep | No spans, no structured fields, no JSON output |
| `slog` | Structured, mature | Heavier API surface, less ecosystem adoption than tracing |
| `tracing` | Spans, structured, async-ready, rich ecosystem | Slightly larger API surface |

## Consequences

- All production crates use `tracing::info!`, `tracing::info_span!`, etc.
- `println!`/`eprintln!` are banned in production crate source
  (enforced by fitness test).
- The JSON event schema is versioned via `LOG_SCHEMA_VERSION` in
  `know_now_diagnostics`.
- `cargo deny check` must remain green with `tracing` + `tracing-subscriber`
  in the dependency tree.

## References

- PRD §12.2 (standard CLI flags)
- PRD §17.1 (stage budgets)
- NFR-P12 (pipeline timing breakdown)
- AGENTS.md §4.2 (volatile vs deterministic)
- know-now-42e.10 (this bead)
