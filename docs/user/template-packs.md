# Template Packs

Template packs generate custom artifacts from validated metadata using `know-now-minijinja-v1`, a restricted MiniJinja-based renderer profile. They are declarative — no arbitrary code execution, no network access, no filesystem escape.

## How it works

1. You write Jinja2 templates (`.j2` files) inside a pack directory
2. A `manifest.yml` declares the pack name, version, renderer, output path, and limits
3. know-now renders your templates against the `GeneratorContract` (validated metadata)
4. Output artifacts are written by the artifact writer (not by the templates directly)

## Pack structure

```
my-pack/
  manifest.yml
  templates/
    index.md.j2
    entity.md.j2
    partials/
      header.j2
```

## Manifest format

```yaml
name: internal-api-docs
version: "1.0"
target: docs
renderer:
  kind: know-now-minijinja
  profile: 1
output_dir: generated/api-docs
permissions:
  filesystem: output_only
  network: none
limits:
  max_templates: 100
  max_template_bytes: 262144
  max_output_files: 100
  max_output_bytes: 10485760
  max_fuel: 50000
  max_include_depth: 8
trust: experimental
```

### Required fields

| Field | Description |
| ----- | ----------- |
| `name` | Pack identifier |
| `version` | Semantic version |
| `target` | Artifact type (e.g., `docs`, `postgres`) |
| `renderer.kind` | Must be `know-now-minijinja` |
| `renderer.profile` | Must be `1` |
| `output_dir` | Output subdirectory under `generated/` |

### Limits (defaults shown)

| Limit | Default | Description |
| ----- | ------- | ----------- |
| `max_templates` | 100 | Max template files per pack |
| `max_template_bytes` | 262,144 (256 KiB) | Max single template file size |
| `max_output_files` | 100 | Max generated artifacts |
| `max_output_bytes` | 10,485,760 (10 MiB) | Max total output size |
| `max_fuel` | 50,000 | Render operation fuel budget |
| `max_include_depth` | 8 | Max include/inheritance nesting depth |

### Trust levels

| Level | Description |
| ----- | ----------- |
| `built_in` | Shipped with know-now |
| `approved` | Vetted by your organization |
| `experimental` | Not yet approved; warnings during generation |
| `untrusted` | Default; blocked in `--locked` CI mode unless policy permits |

### Licensing (required in `--locked` mode)

```yaml
licensing:
  license: MIT
  license_url: https://example.com/license
  license_review: Approved by legal 2026-01-15
```

## Available filters

Templates can use these built-in pure filters:

| Filter | Example | Result |
| ------ | ------- | ------ |
| `snake_case` | `{{ "CustomerOrder" \| snake_case }}` | `customer_order` |
| `kebab_case` | `{{ "customer_order" \| kebab_case }}` | `customer-order` |
| `pascal_case` | `{{ "customer_order" \| pascal_case }}` | `CustomerOrder` |
| `upper_case` | `{{ "hello" \| upper_case }}` | `HELLO` |
| `lower_case` | `{{ "HELLO" \| lower_case }}` | `hello` |
| `markdown_escape` | `{{ text \| markdown_escape }}` | Escapes Markdown special chars |
| `html_escape` | `{{ text \| html_escape }}` | Escapes HTML special chars |

## What you can do

- Use Jinja2 syntax: `{% for %}`, `{% if %}`, `{% set %}`, `{% block %}`
- Include other templates inside the pack root: `{% include "partials/header.j2" %}`
- Access the validated `GeneratorContract` context (entities, relationships, domains, etc.)
- Use the built-in pure filters listed above
- Template inheritance (all referenced templates must be inside the pack root)

## What is forbidden

Template packs **cannot**:

- Register custom functions, filters, tests, or loaders
- Read environment variables
- Run processes or execute shell commands
- Open network connections or make HTTP requests
- Access databases
- Write files directly (output goes through the artifact writer)
- Read files outside the template pack root
- Use dynamic include paths (`{% include variable_name %}` is rejected)
- Include templates that escape the pack root (`{% include "../../file" %}`)
- Access host-specific paths
- Generate timestamps, random values, or other non-deterministic data
- Use custom MiniJinja features beyond the `know-now-minijinja-v1` profile

## Validation enforced during rendering

- Template count, byte size, and include depth limits
- Output file count and total output byte size limits
- Render fuel exhaustion (prevents infinite loops)
- Include path escape detection
- Dynamic include rejection
- Output path safety (no `..`, no symlinks)
- Strict undefined behavior (undefined variables are errors)
- Output paths validated by the artifact writer

## Traceability

The generation manifest and lockfile record:
- Pack name and version
- Pack content hash (SHA-256)
- Renderer profile (`know-now-minijinja-v1`)

## Error handling

Template pack failures are reported separately from built-in generators. A template pack error does not crash built-in generation. Diagnostics include the template file path, line, column, pack name, renderer profile, and generator-contract version.

## Worked example

See [`examples/template-packs/internal-api-docs/`](../../examples/template-packs/internal-api-docs/) for a complete custom template pack.
