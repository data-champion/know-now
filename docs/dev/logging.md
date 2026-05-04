# Structured Logging

know-now uses `tracing` as its structured logging facade and `tracing-subscriber` for output configuration. See ADR-0008 for the decision rationale.

## Crate layout

- **`know_now_diagnostics`** — owns the subscriber configuration, stage taxonomy, log schema version, and test affordances. Consumed by all crates that emit spans or events.
- **`know_now_audit`** — owns redaction patterns (`know_now_audit::redaction`). The diagnostics subscriber pipeline routes events through audit redaction before serialization.
- Library crates **never** call `tracing::subscriber::set_global_default`. They accept whatever subscriber the caller installed.

## Schema version

`know_now_diagnostics::LOG_SCHEMA_VERSION` (currently `"0.1.0"`) pins the JSON event format so downstream tools (CI, dashboard, support bundle) can rely on field presence.

## Stage span taxonomy

Every pipeline stage from PRD §17.1 is represented as a named tracing span. The canonical list lives in `know_now_diagnostics::stage::Stage`:

| Enum variant | Span name |
|---|---|
| `Discovery` | `discovery` |
| `Parsing` | `parsing` |
| `YamlSubset` | `yaml_subset` |
| `Deserialize` | `deserialize` |
| `Semantic` | `semantic` |
| `Policy` | `policy` |
| `DefaultResolution` | `default_resolution` |
| `Contract` | `contract` |
| `Capabilities` | `capabilities` |
| `Planning` | `planning` |
| `Generation` | `generation` |
| `Validation` | `validation` |
| `ManualEditDetection` | `manual_edit_detection` |
| `PathSafety` | `path_safety` |
| `StalePlan` | `stale_plan` |
| `AtomicWrite` | `atomic_write` |
| `Manifesting` | `manifesting` |
| `RunLog` | `run_log` |

Stage names are exported as a public enum so test fixtures cannot drift from production names without a compile error.

## Adding a new stage span

1. Add a variant to `Stage` in `crates/know_now_diagnostics/src/stage.rs`.
2. Add the corresponding `as_str()` arm and entry in `Stage::ALL`.
3. Update `stage_count_matches_prd_17_1` test to match the new count.
4. Update this table.
5. Bump `LOG_SCHEMA_VERSION` if the new stage changes the event shape for existing consumers.

## Structured fields

Each stage span carries at minimum:

- `name` — the stage name string from `Stage::as_str()`.

Pipeline-level spans additionally carry:

- `project_root_hash` — SHA-256 of the project root path (never the path itself — NFR-S9).
- `generator` — generator name (on generation-phase spans).
- `artifact_count` — number of artifacts produced (on generation-phase spans).
- `duration_ms` — recorded automatically by `tracing-subscriber` span close events.

## Output stream conventions

- All structured events go to **stderr**. stdout is reserved for command results so `--format json | jq …` pipelines remain clean.
- Pretty-printed text is the default on a TTY.
- `--no-color` disables ANSI even on a TTY.
- `--format json` emits JSON-Lines on stderr.

## CLI flags

| Flag | Effect |
|---|---|
| `--verbose` | Lifts log level to INFO |
| `--debug` | Lifts log level to DEBUG |
| `--format text\|json\|sarif` | Output format (sarif deferred to Phase 2A) |
| `--no-color` | Disables ANSI escape codes |

These are defined as types in `know_now_diagnostics::logging` (`OutputFormat`, `Verbosity`, `LogConfig`). CLI argument parsing maps to these types; the `init_subscriber()` function configures the subscriber.

## Determinism boundary

Log events **may** contain timestamps, `run_id`, and `duration_ms` because they are volatile run state (NFR-R13, AGENTS.md §4.2). The deterministic manifest (`generated/manifest.json`) and generated artifacts **must never** include log content. Architecture fitness tests enforce this.

## Test affordances

For in-process test assertions, `know_now_diagnostics::test_support` (behind `cfg(test)` or `feature = "test-support"`) provides:

- `install_test_subscriber()` — returns a `(DefaultGuard, CapturedEvents)` pair. The guard scopes the subscriber to the current thread.
- `CapturedEvents::events()` — returns all captured events as `Vec<serde_json::Value>`.
- `CapturedEvents::stage_names()` — extracts span names from captured events.
- `assert_stage_sequence!(events, expected)` — asserts the §17.1 stage span sequence appeared in order.
- `assert_no_secrets!(events)` — runs the `know_now_audit::redaction` pattern catalog over captured events and fails if any secret is detected.

## Redaction

Events pass through `know_now_audit::redaction` before serialization. See `docs/dev/security.md` for pattern details.

## Server integration

This foundation bead defines request-id span field names and test helpers. Axum middleware, `X-Request-Id` propagation, launch-token logging, and CORS/CSP reject logging are implemented by server beads (know-now-hrw.13, know-now-hrw.27).

Request-id validation rules:
- ASCII only, max 64 characters.
- Redaction applies to request-id values that match secret patterns.
