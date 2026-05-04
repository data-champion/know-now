# dbt customization guide

know-now generates a complete dbt project from your metadata. This guide explains the generated structure, how to add custom logic, and how generated and custom code coexist.

PRD refs: §11.3, §9.6.

## Generated structure

```text
generated/dbt/
  dbt_project.yml            # Project definition, references custom/ paths
  models/
    staging/
      stg_<source>__<table>.sql   # One staging model per source table
    marts/
      <entity>.sql            # One mart model per entity (joins FK refs)
      schema.yml              # Column-level tests + docs for all marts
    sources.yml               # Source definitions from metadata
  tests/
    generic/
      is_valid_email.sql      # Semantic-type tests (when email attrs exist)
```

### dbt_project.yml

The generated `dbt_project.yml` includes paths that reach into your `custom/dbt/` directory:

```yaml
model-paths:
  - models
  - ../../../custom/dbt/models

macro-paths:
  - ../../../custom/dbt/macros

seed-paths:
  - ../../../custom/dbt/seeds
```

This means your custom models, macros, and seeds are automatically picked up by `dbt run` without touching the generated project file.

### Staging models

Each source table defined in your metadata produces a staging model at `models/staging/stg_<source>__<table>.sql`. The model selects from the `{{ source() }}` ref and applies any column transforms defined in your metadata:

```sql
with source as (
    select * from {{ source('crm_db', 'customers') }}
)

select
    cust_id,
    LOWER(cust_email) as email
from source
```

### Mart models

Each entity produces a mart model at `models/marts/<entity>.sql`. For entities with FK relationships, the mart joins related entities automatically:

```sql
select
    base.id,
    base.customer_id,
    base.total,
    customer.id as customer_id
from {{ ref('order') }} as base
left join {{ ref('customer') }} as customer
  on base.customer_id = customer.id
```

### Schema and tests

`models/marts/schema.yml` defines column-level dbt tests derived from your metadata:

- `not_null` for required attributes
- `unique` for unique attributes
- `relationships` for FK attributes (references the target entity's PK)
- `accepted_values` for `country_code` semantic types
- Custom generic tests (`is_valid_email`) for `email` semantic types
- `max_length` constraints translated to dbt tests
- `not_negative` for `currency_amount` semantic types

## Adding custom logic

Place your custom dbt files under `custom/dbt/` at the project root (create subdirectories as needed):

```text
custom/
  dbt/
    models/         # Your custom models (intermediate, reporting, etc.)
    macros/         # Custom macros and overrides
    seeds/          # Seed files
    profiles.yml    # Your connection profiles
```

know-now never reads or writes anything under `custom/`. The generated `dbt_project.yml` includes `custom/dbt/models` in its model paths, so your models are automatically included.

### Common patterns

**Adding an intermediate model:**
```text
custom/dbt/models/intermediate/int_active_customers.sql
```

**Overriding a macro:**
```text
custom/dbt/macros/generate_schema_name.sql
```

**Adding seeds:**
```text
custom/dbt/seeds/country_codes.csv
```

## Existing dbt stack coexistence

If you already have a dbt project, use the `dbt-existing-stack` profile:

```bash
know-now init my-project --profile dbt-existing-stack
```

This profile:
1. Generates dbt models under `generated/dbt/` (same structure)
2. Does not produce DDL (your dbt project owns the schema)
3. References your existing dbt project via the `custom/dbt/` paths
4. Sets `dbt_validation: warn` in config so know-now validates against your installed dbt

To integrate, add the generated model paths to your existing `dbt_project.yml`:

```yaml
model-paths:
  - models                    # your existing models
  - generated/dbt/models      # know-now generated models
```

## Regeneration safety

Generated dbt files under `generated/dbt/` are owned by know-now. Every `generate` run replaces them. If you edit a generated file directly, know-now detects the manual edit on next run and either warns or fails (depending on `--strict`).

To make changes that survive regeneration:
- Modify your metadata (entities, sources, relationships) and regenerate
- Use `custom/dbt/` for logic that doesn't derive from metadata
- Use `--accept-generated-overwrite` to force-overwrite detected edits

See [ownership.md](ownership.md) and [regeneration-safety.md](regeneration-safety.md) for full details.
