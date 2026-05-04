# Explain

`know-now explain` traces generated artifacts back to their metadata origins. Use it to understand what was generated, why, and from which metadata objects.

## Usage

```sh
# List all generated artifacts
know-now explain --list

# Trace a specific artifact by path
know-now explain --artifact generated/postgres/ddl/customer.sql

# Find all artifacts that reference a metadata object
know-now explain --object-id ent_customer

# JSON output
know-now explain --list --format json
```

## When to Use Explain

- **"What generated this file?"** — Use `--artifact <path>` to see which metadata objects, generators, and policy rules contributed.
- **"Where does this entity appear in generated output?"** — Use `--object-id <id>` to find all artifacts referencing it.
- **"What did the last generation produce?"** — Use `--list` for a full inventory.

## Output Fields

Each artifact entry shows:

| Field | Description |
|-------|-------------|
| path | Path to the generated file |
| kind | Artifact type (e.g., `ddl`, `dbt_model`, `docs`, `fixture`) |
| generator | Which generator produced it |
| generator_version | Version of the generator |
| content_hash | SHA-256 of the artifact content |
| metadata_object_ids | Which metadata objects contributed |
| trace | Detailed provenance (line spans, policy rules) |

## Error Codes

| Code | Meaning | Fix |
|------|---------|-----|
| EXPLAIN-NO-MANIFEST-001 | No `generated/manifest.json` | Run `know-now generate` first |
| EXPLAIN-PARSE-002 | Manifest is corrupt | Re-run `know-now generate` |

## Combining with Diff

To understand the impact of metadata changes on generated output:

```sh
# See what changed
know-now diff

# See which artifacts are affected
know-now diff --impact

# Trace a specific changed artifact
know-now explain --artifact generated/postgres/ddl/customer.sql
```
