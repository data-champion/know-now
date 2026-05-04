# Doctor

`know-now doctor` runs automated health checks on your project, toolchain, and configuration.

## Usage

```sh
know-now doctor                   # text report
know-now doctor --format json     # machine-readable JSON
know-now doctor --check-updates   # also check for engine updates
```

## What Doctor Checks

- **Metadata directory** — exists and contains `.yml`/`.yaml` files.
- **Lockfile** — present, parseable, schema version compatible, not stale.
- **Configuration** — `know-now.yml` present and valid.
- **Generated output** — `generated/manifest.json` present and parseable.
- **Toolchain** — Rust toolchain available (when applicable).
- **ID coverage** — all entities, attributes, and relationships have stable IDs.

Each check produces a finding with a severity level:

| Severity | Meaning |
|----------|---------|
| ok | Check passed |
| warning | Non-blocking issue; generation will succeed but results may be suboptimal |
| error | Blocking issue; generation will fail or produce incorrect results |

## JSON Output Schema

When using `--format json`, doctor returns a `JsonEnvelope` with:

```json
{
  "version": "0.1.0",
  "command": "doctor",
  "result": "success",
  "payload": {
    "metadata_dir_exists": true,
    "metadata_file_count": 3,
    "lockfile_status": "ok",
    "config_status": "ok",
    "generated_status": "ok",
    "findings": [
      {
        "check": "metadata",
        "severity": "ok",
        "message": "metadata/ directory found with 3 files"
      }
    ]
  }
}
```

Downstream tools can parse this JSON to integrate doctor results into CI pipelines, dashboards, or monitoring systems.

## Common Doctor Findings

| Finding | Cause | Fix |
|---------|-------|-----|
| No metadata/ directory | Project not initialized | Run `know-now init` |
| Lockfile stale | Metadata changed since last lock | Run `know-now lock update` |
| Missing stable IDs | Entities/attributes lack `id:` fields | Run `know-now id backfill --apply` |
| No generated output | Never generated | Run `know-now generate` |
| Config not found | No `know-now.yml` | Run `know-now init` or create manually |

## Using Doctor in CI

```sh
know-now doctor --format json | jq '.payload.findings[] | select(.severity == "error")'
```

If any error-severity findings exist, fail the CI step.
