# CircleCI template for know-now

Copy `config.yml` to `.circleci/config.yml` in your repository.

## Jobs

| Job | Trigger | What it does |
| --- | ------- | ------------ |
| `check` | All branches | Runs `know-now check --format json --locked` |
| `sarif` | `main` only | Produces SARIF artifact for download |

## Required permissions

No special project-level permissions beyond default checkout. The `cimg/base:current` image includes `curl` and `tar`.

## Version pinning

Set the `know-now-version` pipeline parameter at the top of the config. Override it in the CircleCI project settings or via the API.

## Trade-offs

- Uses CircleCI's `cimg/base` convenience image. Replace with your own if you need additional tools.
- SARIF output is stored as a build artifact. CircleCI does not have native SARIF integration; download the artifact for external processing.
- The reusable `install-know-now` command avoids repetition across jobs. For larger pipelines, consider caching the binary in a workspace.
