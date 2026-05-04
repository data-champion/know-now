# CI/CD recipes

Recipes for integrating `know-now check` into your CI pipeline. Each recipe corresponds to a template under [`examples/ci/`](../../examples/ci/).

PRD refs: §12.5 (check), §20.1 (CI).

## Core pattern

Every CI job follows the same pattern:

1. **Install** the know-now binary (pinned version).
2. **Run** `know-now check --format json --locked`.
3. **Optionally** run `know-now check --format sarif --locked` and upload results.

The `--locked` flag verifies that the lockfile matches the currently resolved versions. This catches version drift that could produce different output locally vs. in CI.

## GitHub Actions

Copy [`examples/ci/github-actions/know-now.yml`](../../examples/ci/github-actions/know-now.yml) to `.github/workflows/know-now.yml`.

```yaml
- name: Install know-now
  run: |
    curl -fsSL "https://github.com/data-champion/know-now/releases/download/v${KNOW_NOW_VERSION}/know-now-x86_64-unknown-linux-gnu.tar.gz" \
      | tar xz -C /usr/local/bin

- name: Validate metadata
  run: know-now check --format json --locked
```

The SARIF job uploads results to GitHub code scanning (requires `security-events: write`).

See [`examples/ci/github-actions/README.md`](../../examples/ci/github-actions/README.md) for required permissions.

## GitLab CI

Copy [`examples/ci/gitlab-ci/.gitlab-ci.yml`](../../examples/ci/gitlab-ci/.gitlab-ci.yml) to your repository root.

```yaml
know-now:check:
  stage: validate
  script:
    - know-now check --format json --locked
```

SARIF results are uploaded as `reports:sast` artifacts. The security dashboard requires GitLab Ultimate; on Free/Premium, the SARIF file is a downloadable artifact.

See [`examples/ci/gitlab-ci/README.md`](../../examples/ci/gitlab-ci/README.md) for details.

## CircleCI

Copy [`examples/ci/circleci/config.yml`](../../examples/ci/circleci/config.yml) to `.circleci/config.yml`.

The template uses a reusable `install-know-now` command and a pipeline parameter for version pinning.

See [`examples/ci/circleci/README.md`](../../examples/ci/circleci/README.md) for details.

## Buildkite

Copy [`examples/ci/buildkite/pipeline.yml`](../../examples/ci/buildkite/pipeline.yml) to `.buildkite/pipeline.yml`.

See [`examples/ci/buildkite/README.md`](../../examples/ci/buildkite/README.md) for details.

## Version pinning

All templates pin the know-now version via a variable at the top of the file. When upgrading:

1. Update the version variable in your CI config.
2. Run `know-now lock update` locally.
3. If the generator contract version changed (breaking), the lockfile update will require `--accept-contract-upgrade`. See the [lockfile guide](lockfile.md).
4. Commit both the CI config change and the updated `know-now.lock`.

## Platform targets

Templates assume `x86_64-unknown-linux-gnu`. For other platforms:

| Platform | Target |
|----------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` (`.zip` format) |

## Strict vs. soft SARIF

SARIF jobs use `continue-on-error` (GitHub), `allow_failure` (GitLab), or `soft_fail` (Buildkite) by default. Remove these for strict enforcement where check failures should block the pipeline.

## Source-build CI

If your repository includes know-now as a Cargo workspace dependency (building from source), replace the binary download step with:

```bash
cargo build -p know_now_cli
```

Enable Rust toolchain caching for faster builds. Do not use `cargo install know-now` (without `--locked`) in CI.
