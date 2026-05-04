# Generated output reference

know-now generates artifacts under `generated/` based on your metadata, target database, and active generators. This document describes what gets generated, per phase and per profile.

PRD refs: §11, §9.1.

## Output structure

```text
generated/
  ddl/              # PostgreSQL DDL (CREATE TABLE, constraints, indexes)
  dbt/              # dbt project (models, schema, sources, tests)
  quality_contracts/ # Provider-neutral data quality rules
  docs/             # Markdown documentation (entity pages, domain pages)
  diagrams/         # Mermaid ER diagrams
  review/           # Review summaries and exports
  manifest.json     # Deterministic manifest (content hashes, artifact list)
```

## What each generator produces

### PostgreSQL DDL (`ddl/`)

One `.sql` file per entity. Each file contains a `CREATE TABLE` statement with typed columns, constraints, and comments derived from metadata attributes, logical types, and semantic types.

Generated from: `metadata/entities.yml` attributes, `metadata/project.yml` target database settings.

### dbt project (`dbt/`)

A complete dbt project with models, schema YAML, source definitions, and generated tests. Custom dbt files under `custom/dbt/` are not touched.

Generated from: entities, sources, relationships, and quality rules in metadata.

### Quality contracts (`quality_contracts/`)

Provider-neutral data quality rule definitions derived from `metadata/rules.yml` and policy-applied defaults.

### Documentation (`docs/`)

Markdown pages for the project summary, each domain, each module, and each entity. Includes attribute tables, relationship lists, and governance metadata.

### Diagrams (`diagrams/`)

Mermaid ER diagrams showing entity relationships within and across domains.

### Review summaries (`review/`)

Stakeholder-facing review exports and summaries.

### Manifest (`manifest.json`)

A deterministic manifest recording every generated artifact with its content hash, generator, and contract version. Used for:
- Stale artifact detection (comparing previous vs. current manifest)
- Reproducibility verification (`--locked` mode)
- Change detection for incremental generation

The manifest does not contain timestamps, machine-local paths, usernames, or environment-specific data.

## Generators by phase

| Phase | Generators active | Notes |
|-------|-------------------|-------|
| 1 | PostgreSQL DDL, Markdown docs | Minimal viable generation |
| 2A | + dbt project, quality contracts | First publicly usable output |
| 2B | + diagrams, review | Full artifact set |
| 3+ | + change safety, admin features | Dashboard and collaboration |

## Generators by profile

| Profile | DDL | dbt | Quality | Docs | Diagrams |
|---------|-----|-----|---------|------|----------|
| `minimal` | Yes | No | No | Yes | No |
| `consultant-postgres-dbt` | Yes | Yes | Yes | Yes | Yes |
| `dbt-existing-stack` | No | Yes | Yes | Yes | No |
| `governed-team` | Yes | Yes | Yes | Yes | Yes |
| `demo` | Yes | No | No | Yes | No |

## Determinism guarantee

Identical input metadata produces byte-identical generated output across Linux, macOS, and Windows. The manifest, file content, and file ordering are all deterministic. This is enforced by CI tests on all three platforms.

## Regeneration safety

See [`ownership.md`](ownership.md) for the ownership boundaries that protect your files during regeneration, including manual-edit detection, atomic promotion, and stale artifact handling.
