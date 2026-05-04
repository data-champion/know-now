# Lockfile and reproducibility

`know-now.lock` is a JSON file that pins the versions used during generation, so that the same metadata produces the same output regardless of which know-now version is installed. Commit it to version control.

PRD refs: §9.5.

## What the lockfile records

| Field | Purpose |
|-------|---------|
| `lockfile_schema_version` | Format version of the lockfile itself |
| `engine_version` | know-now CLI version that last updated the lock |
| `metadata_schema_version` | Metadata schema version |
| `generator_contract_version` | Generator contract version |
| `generators` | Map of generator names to their versions |
| `policy` | Policy pack name, version, and content hash |
| `target_compatibility` | Target database profiles (kind, version, compatibility floor) |
| `semantic_type_mappings` | Resolved semantic-to-logical type mappings |

## Commands

### `lock update`

Writes or updates `know-now.lock` from the currently resolved versions:

```bash
know-now lock update
```

Run this after upgrading know-now or changing your `know-now.yml` configuration.

### `lock check`

Verifies the lockfile matches current versions without modifying it:

```bash
know-now lock check
```

### `check --locked`

The recommended CI command. Runs the full check suite and additionally verifies lockfile consistency:

```bash
know-now check --locked
```

If the lockfile is missing, stale, or corrupt, `--locked` fails with a diagnostic code:

| Code | Meaning |
|------|---------|
| `LOCK-MISSING-004` | No lockfile found |
| `LOCK-CORRUPT-005` | Lockfile exists but cannot be parsed |
| `LOCK-STALE-003` | Lockfile fields do not match resolved versions |
| `LOCK-SCHEMA-001` | Lockfile schema version is not recognized |
| `LOCK-UNKNOWN-006` | Lockfile contains unrecognized fields (warning) |

## When to update the lockfile

- **After upgrading know-now** — the engine version changes.
- **After changing `know-now.yml`** — target database, policy, or generator settings may change resolved versions.
- **After a generator contract upgrade** — the contract version changes (see below).

## Generator contract upgrade workflow

When a new know-now version introduces a breaking generator contract change (major version bump), `lock update` will refuse to proceed:

```
LOCK-CONTRACT-002: generator contract version change is breaking
(0.1.0 -> 0.2.0); pass --accept-contract-upgrade to proceed
```

This is a safety gate. A contract version bump means generated output may change in structure or content.

### Migration steps

1. Read the release notes for the new contract version.
2. Run `lock update --accept-contract-upgrade`:

   ```bash
   know-now lock update --accept-contract-upgrade
   ```

3. Run `generate` and review the diff in `generated/`:

   ```bash
   know-now generate
   git diff generated/
   ```

4. If the output is acceptable, commit the updated lockfile and generated artifacts together:

   ```bash
   git add know-now.lock generated/
   git commit -m "chore: upgrade generator contract 0.1.0 -> 0.2.0"
   ```

5. CI will now pass with the new lockfile.

### What changes in a contract upgrade

A breaking contract change may affect:

- **DDL structure** — column types, constraint names, or ordering.
- **dbt model structure** — model file layout, schema YAML, or test definitions.
- **Manifest format** — additional fields or changed hash algorithms.
- **Output file paths** — renamed or reorganized output directories.

Non-breaking changes (new optional fields, additive features) do not bump the contract version and do not require `--accept-contract-upgrade`.

## Lockfile in `.gitignore`

Do **not** add `know-now.lock` to `.gitignore`. The lockfile is meant to be committed and reviewed by the team. It is the mechanism that ensures `check --locked` passes in CI — without it, different machines may resolve different versions and produce different output.

## Lockfile and profiles

The `demo` profile creates a lockfile automatically during `init --demo`. Other profiles create the lockfile on the first `lock update` or `generate` run.
