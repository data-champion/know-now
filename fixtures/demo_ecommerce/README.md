# Demo E-commerce Metadata Fixture

This fixture is a consulting-style baseline for Phase 1 and later compatibility tests.
It is intentionally synthetic and contains no real customer data.

## Purpose

The fixture exercises the core metadata model required by PRD Phase 1 and the
`know-now-upe.1` acceptance criteria:

- 8 entities across multiple domains/modules
- multiple source systems and source tables
- logical and semantic type coverage used by early generators
- governance metadata on key entities
- self-referencing and many-to-many relationships
- explicit open questions and assumptions for review workflows
- stable IDs for all modeled objects using the project prefix convention

## Scope Highlights

- Domains: `commercial`, `operations`
- Modules: `customers`, `orders`, `returns`, `catalog`, `supply`
- Entities:
  - `customer`
  - `order`
  - `product`
  - `inventory`
  - `supplier`
  - `return`
  - `customer_segment`
  - `address`
- Relationship patterns:
  - self-reference: `customer` parent/referrer hierarchy
  - many-to-many: `customer` ↔ `customer_segment`
- Sources:
  - `shopify`
  - `erp`

## Files

- `metadata/domains_and_modules.yml`
- `metadata/entities.yml`
- `metadata/relationships.yml`
- `metadata/sources.yml`
- `metadata/questions_and_assumptions.yml`
