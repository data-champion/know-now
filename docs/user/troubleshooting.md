# Troubleshooting

This page lists common errors by diagnostic code, with remediation steps.
For deeper investigation, see also:

- [doctor](doctor.md) — automated health checks
- [explain](explain.md) — trace generated artifacts back to metadata
- [support-bundle](support-bundle.md) — create a sanitized diagnostic package

## Metadata Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| META-001 | Generic metadata parse error | Check YAML syntax in `metadata/`. Run `know-now validate` for details. |
| META-ENT-001 | Entity validation failed (missing required fields) | Ensure each entity has a `name` and at least one attribute. |
| META-ENT-002 | Entity has duplicate attribute names | Rename or remove duplicate attributes within the entity. |
| META-REL-001 | Relationship references an unknown entity | Check `from_entity` and `to_entity` values match defined entity names. |
| META-REL-002 | Relationship validation failed | Verify cardinality is one of: one-to-one, one-to-many, many-to-one, many-to-many. |
| META-DOM-001 | Domain validation failed | Check domain definition in `metadata/`. |
| META-MOD-001 | Module validation failed | Check module definition in `metadata/`. |
| META-SRC-001 | Source system validation failed | Check source system definition in `metadata/`. |
| META-ASM-001 | Assembly-level metadata error | Check top-level metadata structure. |
| META-Q-001 | Quality rule validation failed | Check rule definitions — each needs `name`, `expression`, and `severity`. |
| META-ID-001 | Stable ID validation failed | Run `know-now id check` and `know-now id suggest` to fix. |

## Validation Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| VAL-001 | General validation failure | Run `know-now validate` for the full list of issues. |

## Diff Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| DIFF-BREAK-001 | Breaking changes detected in `--migration-safe` mode | Review the diff output. Either accept the changes or adjust metadata to be non-breaking. |
| DIFF-NO-BASELINE-002 | No previous generation found for comparison | Run `know-now generate` first to create a baseline, or use `--baseline manifest:<path>`. |
| DIFF-ID-003 | Missing stable IDs required by `--migration-safe` | Run `know-now id suggest` to generate IDs, then `know-now id backfill --apply`. |
| DIFF-GIT-001 | Git-based baseline not yet implemented | Use `--baseline last-generation` or `--baseline manifest:<path>` instead. |

## Explain Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| EXPLAIN-NO-MANIFEST-001 | No `generated/manifest.json` found | Run `know-now generate` first. |
| EXPLAIN-PARSE-002 | Manifest JSON could not be parsed | Check that `generated/manifest.json` is valid JSON. Re-run `know-now generate`. |

## Generation Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| GEN-001 | Generation failed | Check validate output first. Fix metadata issues, then re-run `know-now generate`. |

## Lockfile Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| LOCK-SCHEMA-001 | Lockfile schema version not supported | Upgrade know-now or regenerate the lockfile with `know-now lock update`. |
| LOCK-CONTRACT-002 | Lockfile contract version mismatch | Run `know-now lock update` to refresh. |
| LOCK-STALE-003 | Lockfile is stale (metadata changed since lock) | Run `know-now lock update` to regenerate. |
| LOCK-MISSING-004 | No lockfile found | Run `know-now lock update` to create one. Required for `--locked` mode. |
| LOCK-CORRUPT-005 | Lockfile is corrupt or unreadable | Delete and regenerate with `know-now lock update`. |
| LOCK-UNKNOWN-006 | Unknown lockfile error | Run `know-now doctor` for diagnostics. |
| DOC-LOCK-001 | Document-level lock error | Check lockfile structure. |
| DOC-LOCK-002 | Document-level lock version mismatch | Run `know-now lock update`. |
| DOC-META-001 | Document-level metadata error | Check lockfile metadata section. |

## Catalog Errors

| Code | Meaning | Remediation |
|------|---------|-------------|
| CATALOG-SCHEMA-001 | Missing `approved` section in catalog | Add the `approved:` key to your catalog file. |
| CATALOG-RANGE-002 | Invalid semver range | Check version range syntax (e.g., `1.0.x`, `1.x`, `1.0.3`). |
| CATALOG-TARGET-003 | Target has no floor and no allowed versions | Add either `floor` or `allowed` to the target spec. |
| CATALOG-TARGET-004 | Target floor version not in allowed list | Add the floor version to the `allowed` list, or adjust the floor. |

## Quick Diagnostic Steps

1. **Run `know-now doctor`** — checks toolchain, config, metadata, and lockfile health.
2. **Run `know-now validate`** — parses and validates all metadata files.
3. **Run `know-now id check`** — verifies stable IDs are present and valid.
4. **Run `know-now explain --list`** — shows all generated artifacts with provenance.
5. **Create a support bundle** — `know-now support` produces a sanitized JSON file safe to share.
