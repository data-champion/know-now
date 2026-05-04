# Metadata Reference

PRD refs: §10, §10.3, §10.4, §10.5, §10.6, §10.7, §10.8, §10.9, §10.10, §19.1.

This page is the field-level reference for the current authoring metadata model in
`know_now_metadata::authoring::AuthoringMetadata`, aligned to the PRD metadata sections.

## Top-level fields

- `version`
- `project`
- `target_database`
- `policy`
- `domains`
- `modules`
- `entities`
- `relationships`
- `sources`
- `rules`
- `governance`
- `open_questions`
- `assumptions`

## `project`

- `name`
- `description`
- `owner`
- `tags`

## `target_database`

- `kind`
- `version`
- `compatibility_floor`

## `policy`

- `pack`
- `version`

## `domains[]`

- `id`
- `name`
- `description`
- `owner`

## `modules[]`

- `id`
- `name`
- `description`

## `entities[]`

- `id`
- `name`
- `display_name`
- `domain`
- `module`
- `owner`
- `steward`
- `classification`
- `retention_policy`
- `description`
- `type`
- `tags`
- `business_key`
- `attributes`

## `entities[].attributes[]`

- `id`
- `name`
- `logical_type`
- `semantic_type`
- `sensitivity`
- `pii`
- `required`
- `unique`
- `constraints`
- `description`
- `type`

## `relationships[]`

- `id`
- `from_entity`
- `to_entity`
- `cardinality`
- `from_key`
- `to_key`
- `description`

## `sources[]`

- `name`
- `kind`
- `description`
- `entities`
- `tables`

### `sources[].tables[]`

- `name`
- `entity`
- `schema`
- `columns`

### `sources[].tables[].columns[]`

- `source`
- `target`
- `transform`

## `rules[]`

- `id`
- `name`
- `entity`
- `attribute`
- `rule_type`
- `expression`
- `severity`
- `description`

## `governance`

- `data_owner`
- `data_steward`
- `classification_default`
- `retention_default`

## `open_questions[]`

- `id`
- `question`
- `context`
- `entity`
- `priority`

## `assumptions[]`

- `id`
- `statement`
- `rationale`
- `entity`
- `risk`

## Example

```yaml
version: "1.0"
project:
  name: ecommerce
  description: Metadata reference example.
  owner: data-team
  tags: [demo, docs]
target_database:
  kind: postgres
  version: "18"
  compatibility_floor: "16"
policy:
  pack: dc_standard
  version: "1.0"
domains:
  - id: dom_sales
    name: sales
    description: Sales domain.
    owner: sales-team
modules:
  - id: mod_core
    name: core
    description: Core module.
entities:
  - id: ent_customer
    name: customer
    display_name: Customer
    domain: dom_sales
    module: mod_core
    owner: data-team
    steward: steward-user
    classification: internal
    retention_policy: retention_7y
    description: Customer entity.
    type: dimension
    tags: [pii]
    business_key: [email]
    attributes:
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        sensitivity: high
        pii: true
        required: true
        unique: true
        constraints: [max_length:320]
        description: Customer email.
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_key: customer_id
    to_key: id
    description: Order to customer relationship.
sources:
  - name: crm
    kind: postgres
    description: CRM source.
    entities: [customer]
    tables:
      - name: customers
        entity: customer
        schema: public
        columns:
          - source: email_address
            target: email
            transform: lower(trim(value))
rules:
  - id: rule_email_not_null
    name: email_not_null
    entity: customer
    attribute: email
    rule_type: not_null
    expression: email IS NOT NULL
    severity: error
    description: Email must be present.
governance:
  data_owner: data-team
  data_steward: steward-user
  classification_default: internal
  retention_default: retention_5y
open_questions:
  - id: q_customer_deletion
    question: Should deleted customers remain queryable?
    context: GDPR workflow.
    entity: customer
    priority: high
assumptions:
  - id: asm_customer_email_required
    statement: Every customer has an email.
    rationale: Required by onboarding flow.
    entity: customer
    risk: medium
```
