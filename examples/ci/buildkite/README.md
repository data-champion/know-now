# Buildkite template for know-now

Copy `pipeline.yml` to `.buildkite/pipeline.yml` in your repository, or upload it dynamically:

```yaml
steps:
  - command: buildkite-agent pipeline upload .buildkite/pipeline.yml
```

## Steps

| Step | Trigger | What it does |
| ---- | ------- | ------------ |
| Metadata validation | All builds | Runs `know-now check --format json --locked` |
| Code scanning (SARIF) | `main` only | Produces SARIF artifact for download |

## Required permissions

No special agent or pipeline permissions. The agent must have network access to download the binary and `curl`/`tar` available.

## Version pinning

Set the `KNOW_NOW_VERSION` env var at the top of the pipeline. Override it in the Buildkite pipeline settings or via environment hooks.

## Trade-offs

- Each step downloads the binary independently. For pipelines with many steps, use an artifact or a pre-command hook to install once.
- SARIF output is stored as a Buildkite artifact. Buildkite does not have native SARIF integration; download via the API or UI for external processing.
- The SARIF step uses `soft_fail: true` so a check failure does not block the pipeline. Remove this if you want strict enforcement.
