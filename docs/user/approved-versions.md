# Approved-Version Catalog

The approved-version catalog is a JSON file that defines which engine versions, metadata schemas, policies, templates, and targets are approved for use in your organization. It is used by `admin scan --catalog` to classify project drift.

## Format

```json
{
  "approved": {
    "engines": {
      "know-now": ["0.1.0", "0.2.0", "1.0.0"]
    },
    "metadata_schema_versions": ["1.0"],
    "generator_contract_versions": ["1.0"],
    "policies": {
      "dc_standard": ["1.0"],
      "my-org-standards": ["1.0", "1.1"]
    },
    "templates": {
      "internal-api-docs": ["1.0"]
    },
    "template_renderers": {
      "know-now-minijinja": ["1"]
    },
    "targets": {
      "postgres": {
        "floor": "15",
        "allowed": ["15", "16", "17"]
      }
    }
  }
}
```

## Fields

### engines

Maps engine name to a list of approved version strings. Projects running unapproved engine versions are flagged.

### metadata_schema_versions

List of approved metadata schema versions. Projects using unlisted schema versions are flagged.

### generator_contract_versions

List of approved generator contract versions.

### policies

Maps policy pack name to approved version lists. Projects using unapproved policy versions are flagged as drifting.

### templates

Maps template pack name to approved version lists.

### template_renderers

Maps renderer name to approved profile versions.

### targets

Maps target name (e.g., `postgres`) to a spec with:
- `floor` — minimum allowed version (optional)
- `allowed` — explicit list of allowed versions

## Drift classifications

When `admin scan` compares a project against the catalog, it classifies drift as:

| Classification | Meaning |
| -------------- | ------- |
| `none` | All versions match the approved set |
| `patch` | Differs by a patch version (e.g., 1.0.1 vs 1.0.0) |
| `minor` | Differs by a minor version (e.g., 1.1.0 vs 1.0.0) |
| `major` | Differs by a major version (e.g., 2.0.0 vs 1.0.0) |
| `unknown` | Version not found in the catalog at all |
| `unapproved` | Explicitly not in the approved set |

## Drift workflow

1. **Set up the catalog** — create `approved-versions.json` in your governance repo
2. **Validate it** — `know-now admin catalog-check approved-versions.json`
3. **Scan projects** — `know-now admin scan /repos --catalog approved-versions.json`
4. **Review drift** — projects with `major`, `unknown`, or `unapproved` drift need attention
5. **Update the catalog** — after testing new versions, add them to the approved set

## Governance repo setup

Keep the catalog in a dedicated governance repository alongside your custom policy packs:

```
data-governance/
  approved-versions.json
  policy-packs/
    my-org-standards/
      policy.yml
  ci/
    scan-all-repos.sh
```

This keeps governance configuration versioned, reviewable, and separate from individual project repos. See [admin-scan.md](admin-scan.md) for the scan workflow.
