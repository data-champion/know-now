# dbt validation adapter guide

know-now can validate generated dbt projects by running dbt commands after generation. The adapter auto-detects the installed dbt tool and adapts its behavior.

PRD refs: §8.10, §17.3 (NFR-I7).

## Validation modes

| Mode | dbt required? | What it does | When to use |
|------|-------------|--------------|-------------|
| `none` | No | Skips validation | No dbt installed, CI without dbt |
| `dbt` | Yes | Auto-detects identity, runs `dbt parse` + `dbt compile` | Most teams |
| `dbt-core` | Yes | Requires dbt-core specifically | Pinned to dbt-core |
| `dbt-fusion` | Yes | Requires dbt-fusion specifically | Teams on dbt-fusion |
| `docker` | Yes (in image) | Runs dbt inside a Docker container | Hermetic CI, no local dbt |

The default mode is `none`. Generated dbt projects are always valid structurally — validation adds the guarantee that dbt itself can parse and compile the models.

## Configuration

Set the mode in your project config (`know-now.yml`):

```yaml
dbt_validation: dbt
```

Or use the full config form:

```yaml
dbt:
  mode: dbt
  executable: dbt        # default
  required_in_ci: false  # fail CI if dbt is missing
  docker_image: null     # only for mode: docker
```

The `dbt-existing-stack` profile defaults to `dbt_validation: warn`.

## Identity detection

When mode is `dbt`, `dbt-core`, or `dbt-fusion`, the adapter runs `dbt --version` and classifies the result:

| Identity | Detected when |
|----------|--------------|
| `core` | Version output contains "dbt-core" |
| `fusion` | Version output contains "dbt-fusion" or "dbt Fusion" |
| `compatible` | Version output starts with "dbt version" (generic) |
| `unknown` | Cannot parse version output |

With mode `dbt`, any identity is accepted. With `dbt-core` or `dbt-fusion`, the adapter requires the specific identity and fails with a clear error if mismatched.

## Docker mode

Docker mode runs dbt inside a container, mapping your generated project:

```yaml
dbt:
  mode: docker
  docker_image: ghcr.io/dbt-labs/dbt-core:1.7.0
```

In locked mode (`--locked`), the Docker image must be pinned by digest:

```yaml
dbt:
  mode: docker
  docker_image: ghcr.io/dbt-labs/dbt-core@sha256:abc123...
```

This ensures reproducibility — tag-based references can change between runs.

## CI patterns

### Mode: none (default, fastest)

No dbt required in CI. Generated dbt projects pass structural validation (valid YAML, valid Jinja SQL) but are not compiled by dbt.

```yaml
# .github/workflows/ci.yml
- run: know-now generate --target dbt
```

### Mode: dbt (recommended for teams with dbt)

```yaml
- uses: actions/setup-python@v5
  with:
    python-version: '3.11'
- run: pip install dbt-core dbt-postgres
- run: know-now generate --target dbt
```

### Mode: docker (hermetic CI)

```yaml
- run: |
    know-now generate --target dbt
  env:
    KNOWNOW_DBT_MODE: docker
    KNOWNOW_DBT_DOCKER_IMAGE: ghcr.io/dbt-labs/dbt-core:1.7.0
```

### Requiring dbt in CI

Set `required_in_ci: true` to fail the build when dbt is not found:

```yaml
dbt:
  mode: dbt
  required_in_ci: true
```

Without this flag, a missing dbt binary in mode `dbt` produces a warning rather than a failure.

## Diagnostics

When validation fails, the adapter reports diagnostics with file and line references:

```
error: dbt parse failed
  --> generated/dbt/models/marts/order.sql:5
  | Compilation Error: Model 'order' depends on a node named 'customer'
  | which was not found
```

These diagnostics appear in both text and JSON output formats, and are included in the `events.jsonl` structured log when running with `--verbose`.

## Validation flow

```text
generate --target dbt
  └─ Generate dbt artifacts (always)
      └─ Structural validation: YAML parse, SQL parse, Jinja balance
          └─ dbt adapter (if mode != none)
              ├─ detect: dbt --version → identity classification
              ├─ validate identity vs. configured mode
              ├─ dbt parse (project structure)
              └─ dbt compile (SQL resolution)
```

## Troubleshooting

**"dbt executable not found"** — dbt is not on PATH. Install dbt or set `mode: none`.

**"unsupported dbt identity for mode"** — you configured `dbt-core` but a different dbt variant is installed. Use `mode: dbt` to accept any variant, or install the required variant.

**"docker image must be pinned by digest in locked mode"** — use `image@sha256:...` instead of `image:tag` when running with `--locked`.
