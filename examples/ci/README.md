# CI integration examples

Runnable CI templates for integrating `know-now check` into your pipeline.

Each template provides two jobs:

1. **Metadata validation** — runs `know-now check --format json --locked` on every push and PR/MR.
2. **Code scanning (SARIF)** — runs `know-now check --format sarif --locked` on the default branch and uploads results where the platform supports it.

## Templates

| Platform | Template | Notes |
| -------- | -------- | ----- |
| [GitHub Actions](github-actions/) | `know-now.yml` | PR annotations via `checks: write`; SARIF upload to code scanning |
| [GitLab CI](gitlab-ci/) | `.gitlab-ci.yml` | SARIF via `reports:sast` artifact (Ultimate for dashboard) |
| [CircleCI](circleci/) | `config.yml` | Reusable command; SARIF as stored artifact |
| [Buildkite](buildkite/) | `pipeline.yml` | SARIF as build artifact |

## Installation approach

All templates download a prebuilt binary rather than building from source. This keeps CI fast (seconds, not minutes) and avoids requiring a Rust toolchain. If your project includes know-now as a workspace dependency, replace the download step with `cargo build -p know-now` and cache the Rust toolchain.

Do not use `cargo install know-now` (without `--locked`) in CI — it does not pin dependency versions.

## Customization

- **Version pinning**: each template defines a version variable at the top. Update it when upgrading.
- **Platform**: templates assume `x86_64-unknown-linux-gnu`. Adjust the download URL for other targets (macOS: `aarch64-apple-darwin`, Windows: `x86_64-pc-windows-msvc`).
- **Strict vs. soft SARIF**: SARIF jobs use `continue-on-error` / `allow_failure` / `soft_fail` by default. Remove these for strict enforcement.

See each template's README for platform-specific permissions and trade-offs.
