# Support Bundle

`know-now support` creates a sanitized diagnostic package as a single JSON file. This bundle is safe to share with maintainers for troubleshooting.

## Usage

```sh
know-now support                      # write bundle to .knownow/support-<timestamp>.json
know-now support --dry-run            # preview what will be collected
know-now support --format json        # JSON envelope output
```

## What the Bundle Contains

| Section | Contents |
|---------|----------|
| doctor | Full doctor report (health checks and findings) |
| lockfile_hash | SHA-256 of `know-now.lock` (not the lockfile itself) |
| config_summary | Configuration keys only (no values) |
| generator_versions | Version strings of all generators |
| recent_runs | Last 10 entries from the audit log |
| manifest_summary | Artifact count, generator names, hash (not content) |
| issues_summary | Count by status from `.knownow/issues.json` |
| environment | Safe environment variables only |

## What the Bundle Does NOT Contain

The support bundle is designed to be safe to share. It explicitly excludes:

- Metadata file contents (entity names, attributes, business logic)
- Generated artifact contents
- Configuration values (only keys are included)
- Lockfile contents (only a hash)
- Secrets, tokens, or credentials
- Full file paths (home directory is redacted)

## Environment Variable Allowlist

Only these environment variables are included:

`USER`, `LOGNAME`, `SHELL`, `TERM`, `LANG`, `LC_ALL`, `HOME`, `PATH`, `EDITOR`, `VISUAL`, `CARGO_PKG_VERSION`, `RUST_LOG`, `NO_COLOR`, `FORCE_COLOR`

Any variable containing `KEY`, `SECRET`, `TOKEN`, `PASSWORD`, `CREDENTIAL`, or `AUTH` in its name is always excluded.

## Path Redaction

All file paths in the bundle have the user's home directory replaced with `$HOME` to avoid leaking usernames or directory structures.

## Sharing a Bundle

```sh
# Create the bundle
know-now support

# The bundle is saved to .knownow/support-<timestamp>.json
# Share it via your preferred channel (issue tracker, email, etc.)
```

When reporting an issue, attach the support bundle for faster diagnosis. The maintainer can use the doctor findings and audit log to understand your project state without needing access to your metadata.
