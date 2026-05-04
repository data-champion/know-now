# Five-minute demo

This walkthrough creates a demo project, validates its metadata, and runs the recommended check suite. Total time: under five minutes.

## 1. Create a demo project

```bash
know-now init --demo
```

This creates a `demo-project/` directory with:

```
demo-project/
  know-now.yml          # Project configuration (postgres, dc_standard policy)
  know-now.lock         # Lockfile pinning engine and contract versions
  metadata/
    project.yml         # E-commerce data model: domains, modules, governance
    entities.yml        # Customers, orders, products, order_items
    relationships.yml   # Entity relationships (customer->order, etc.)
    sources.yml         # Source table mappings
  generated/            # Engine-generated artifacts (populated by generate)
  custom/               # User-maintained files (never overwritten)
  README.md             # Project-specific quick start
```

The demo uses the `demo` profile, which includes a sample e-commerce data model with customers, orders, products, and order items across sales and catalog domains.

## 2. Enter the project

```bash
cd demo-project
```

## 3. Validate metadata

```bash
know-now validate
```

Expected output:

```
Validation passed. No issues found.
```

This parses all YAML metadata files, checks identifiers, builds the project graph, and evaluates the `dc_standard` policy pack. Any errors (missing fields, invalid references, policy violations) appear as diagnostics with codes, file locations, and help text.

## 4. Run the check suite

```bash
know-now check
```

Expected output:

```
Check passed. Project is ready to generate.
```

`check` runs the same validation as `validate` and is the recommended command for CI pipelines. It supports `--locked` for lockfile verification and `--format json` for machine-readable output.

## 5. Verify lockfile consistency

```bash
know-now check --locked
```

The `--locked` flag verifies that the lockfile matches the currently resolved engine and contract versions. This is what CI pipelines should use.

## 6. Explore the metadata

Open `metadata/project.yml` to see the project structure:

- **Domains:** `sales` (customer and order management), `catalog` (product catalog and inventory)
- **Modules:** `core` (core business entities)
- **Governance:** data owner, data steward, classification, retention policy

Open `metadata/entities.yml` to see entity definitions with attributes, business keys, logical types, and semantic types.

## 7. Inspect project configuration

```bash
know-now config inspect
```

Shows the resolved project configuration: root path, config file, metadata file count, lockfile state, engine version, available generators, and active policy.

## Next steps

- **Edit metadata:** modify `metadata/entities.yml` to add attributes or entities, then re-run `validate`.
- **CI integration:** copy a template from [`examples/ci/`](../../examples/ci/) to your CI pipeline.
- **Metadata reference:** see [`metadata-reference.md`](metadata-reference.md) for the full YAML schema.
- **YAML subset:** see [`yaml-subset.md`](yaml-subset.md) for the supported YAML features.
