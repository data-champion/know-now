# Admin Scan

The `admin scan` command discovers know-now projects under a directory tree and aggregates their governance state. It is designed for platform teams managing multiple repositories.

## Usage

```bash
know-now admin scan /path/to/repos
```

The command recursively finds directories containing both `metadata/` and a project marker (`know-now.yml`, `know-now.yaml`, or `know-now.lock`).

### Output

For each discovered project:
- **Path** — location on disk
- **Lockfile status** — `locked`, `missing`, or `corrupt`
- **Engine version** — from the lockfile
- **Policy pack and version** — from the lockfile
- **Last generation timestamp** — from `.knownow/last_generation.json`
- **Drift classification** — if a catalog is provided (see below)

### JSON output

```bash
know-now admin scan /path/to/repos --format json
```

Returns a structured `ScanReport` with `scanned_root`, `projects[]`, and `catalog_used`.

## Drift classification with a catalog

Pass an approved-version catalog to classify each project's drift:

```bash
know-now admin scan /path/to/repos --catalog approved-versions.json
```

Each project is classified as `none`, `patch`, `minor`, `major`, `unknown`, or `unapproved` based on whether its engine version, metadata schema version, policy versions, and template versions match the catalog's approved set.

See [approved-versions.md](approved-versions.md) for the catalog format.

## Governance repo pattern

For organizations with many repositories, the recommended setup is:

1. Create a **governance repository** (e.g., `data-governance/`) containing:
   - `approved-versions.json` — the approved-version catalog
   - Custom policy packs
   - CI scripts that run `admin scan` across all repos

2. CI in the governance repo clones all project repos, runs:
   ```bash
   know-now admin scan ./repos --catalog approved-versions.json --format json
   ```

3. The JSON output feeds dashboards, alerts, or compliance reports.

This keeps governance configuration in one place, versioned and reviewable, separate from individual project repos.

## Catalog validation

Validate a catalog file independently:

```bash
know-now admin catalog-check approved-versions.json
```

This checks structural validity (required fields, version format) without scanning any projects.

## Skipped directories

The scan skips directories starting with `.`, `node_modules`, and `target` to avoid false positives and keep scan times reasonable.
