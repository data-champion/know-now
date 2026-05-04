# GitHub Actions template for know-now

Copy `know-now.yml` to `.github/workflows/know-now.yml` in your repository.

## Jobs

| Job | Trigger | What it does |
| --- | ------- | ------------ |
| `check` | Push to `main`, all PRs | Runs `know-now check --format json --locked` |
| `sarif` | Push to `main` only | Uploads SARIF to GitHub code scanning |

## Required permissions

- **`contents: read`** — checkout.
- **`checks: write`** — annotate PRs with diagnostics (optional; remove if not needed).
- **`security-events: write`** — upload SARIF results to code scanning (sarif job only).

## Version pinning

Set the `KNOW_NOW_VERSION` env var at the top of the workflow. This pins the binary download to a specific release. Update it when upgrading.

## Trade-offs

- The workflow downloads a prebuilt binary rather than building from source. This is faster (~seconds vs. minutes) but requires published release assets for your platform.
- The SARIF job runs only on `main` pushes to avoid duplicate uploads from PRs. Adjust the `if` condition if you want SARIF on every PR.
- If your repository tracks know-now as a Cargo workspace dependency (source build), replace the install step with `cargo build -p know-now` and enable Rust toolchain caching.
