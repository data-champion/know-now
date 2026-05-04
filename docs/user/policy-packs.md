# Policy Packs

Policy packs encode reusable project standards as declarative, non-mutating rules. They validate metadata, provide defaults, and classify findings — they cannot execute code or modify files.

## Built-in: dc_standard

Every project starts with `dc_standard` v1.0 (always available, no configuration needed). It defines baseline rules:

| Code | Applies to | Severity | Description |
| ---- | ---------- | -------- | ----------- |
| POL-NAM-001 | entity | warning | Entity names must be snake_case |
| POL-NAM-002 | attribute | warning | Attribute names must be snake_case |
| POL-NAM-003 | module | warning | Module IDs must be snake_case |
| POL-NAM-004 | all | warning | Identifiers must be lowercase ASCII + digits + underscores |
| POL-ENT-001 | entity | warning | Entities must have a required+unique attribute or business_key |
| POL-ENT-002 | entity | warning | Entities must define business_key |
| POL-DOC-001 | entity | warning | Entities must have descriptions |
| POL-DOC-002 | attribute | warning | Required attributes must have descriptions |

## Custom policy pack format

A policy pack is a YAML manifest with a `name`, `version`, `severity_profile`, and a list of `rules`.

```yaml
name: my-org-standards
version: "1.0"
description: Organization-wide metadata conventions
severity_profile: standard

rules:
  - id: CORP-001
    severity: error
    applies_to: entity
    expression:
      kind: attribute_presence
      attribute: description
    rationale: All entities must be documented for stakeholder review.
    remediation: Add a description field to the entity.

  - id: CORP-002
    severity: warning
    applies_to: entity
    expression:
      kind: naming_convention
      pattern: "^[a-z][a-z0-9_]*$"
    rationale: Snake_case naming keeps DDL identifiers clean.
    remediation: Rename the entity to use snake_case.

  - id: CORP-003
    severity: warning
    applies_to: attribute
    expression:
      kind: enum_membership
      field: logical_type
      allowed: [string, integer, boolean, decimal, date, timestamp, uuid, text]
    rationale: Restrict logical types to the approved set.
    remediation: Use one of the allowed logical types.
```

### Severity profiles

| Profile | Behavior |
| ------- | -------- |
| `standard` | Default. Warnings do not block generation. |
| `strict` | Warnings promoted to errors where possible. |
| `relaxed` | Errors demoted to warnings where possible. |

### Rule expression types

**attribute_presence** — checks a metadata field exists:

```yaml
expression:
  kind: attribute_presence
  attribute: description
```

**naming_convention** — validates names against a regex:

```yaml
expression:
  kind: naming_convention
  pattern: "^[a-z][a-z0-9_]*$"
```

**enum_membership** — restricts a field to an allowed set:

```yaml
expression:
  kind: enum_membership
  field: logical_type
  allowed: [string, integer, boolean]
```

**cardinality_check** — validates relationship cardinality:

```yaml
expression:
  kind: cardinality_check
  allowed: [many_to_one, one_to_many]
```

**tag_presence** — requires a specific tag:

```yaml
expression:
  kind: tag_presence
  tag: owner
```

### applies_to targets

`entity`, `attribute`, `relationship`, `module`, `domain`, `source_system`, `quality_rule`

## What is forbidden

Policy packs **cannot**:

- Execute arbitrary code
- Mutate raw metadata or the canonical project graph
- Write files directly
- Access the network
- Read environment variables
- Access databases
- Run processes
- Register custom functions or filters

Policy-provided defaults are applied through an explicit, traceable resolution step. Every artifact reports which policy defaults and findings affected it.

## Version pinning and drift

Policy pack versions are recorded in `know-now.lock`. The `policy status` command reports drift:

```bash
know-now policy status
```

Drift classifications: `none`, `patch`, `minor`, `major`, `unknown`, `unapproved`.

In `--locked` mode, unknown policy features are rejected unless explicitly permitted.

## Explaining findings

```bash
know-now policy explain POL-NAM-001
```

Shows the rule name, rationale, severity, remediation, and organization-specific explanation text.

## Traceability

The generation manifest records for each policy pack:
- Pack name
- Pack version
- Pack content hash (SHA-256)

## Worked example

See [`examples/policy-packs/minimal-org/`](../../examples/policy-packs/minimal-org/) for a complete custom policy pack.
