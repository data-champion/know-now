# GitLab CI template for know-now

Copy `.gitlab-ci.yml` to your repository root, or include it with:

```yaml
include:
  - local: 'path/to/.gitlab-ci.yml'
```

## Jobs

| Job | Trigger | What it does |
| --- | ------- | ------------ |
| `know-now:check` | MRs and default branch | Runs `know-now check --format json --locked` |
| `know-now:sarif` | Default branch only | Produces SARIF artifact for GitLab SAST |

## Required permissions

No special CI/CD permissions beyond default project access. The SARIF job uses GitLab's `reports:sast` artifact type, which requires GitLab Ultimate for the security dashboard integration. The artifact is still downloadable on all tiers.

## Version pinning

Set the `KNOW_NOW_VERSION` variable at the top of the file or override it in **Settings > CI/CD > Variables**.

## Trade-offs

- Uses `debian:bookworm-slim` as the base image. The `before_script` installs `curl` and `ca-certificates` since the slim image omits them. Substitute your own image if it already has these tools.
- The `&install-know-now` anchor avoids repeating the install step. If your pipeline has many jobs, consider a dedicated install stage with artifact passing instead.
- SARIF integration with GitLab's security dashboard requires Ultimate. On Free/Premium, the SARIF file is still available as a downloadable job artifact.
