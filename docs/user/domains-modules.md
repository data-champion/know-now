# Domains and Modules

PRD refs: §10.3, §19.1.

Domains and modules provide organizational structure for metadata.

Domain fields:
- `id`
- `name`
- `description`
- `owner`

Module fields:
- `id`
- `name`
- `description`

Entities can reference a `domain` and `module`.

## Example

```yaml
domains:
  - id: dom_commercial
    name: commercial
    description: Revenue and customer-facing concepts.
    owner: sales-ops

modules:
  - id: mod_orders
    name: orders
    description: Order lifecycle and fulfillment.

entities:
  - name: order
    domain: dom_commercial
    module: mod_orders
    attributes:
      - name: order_id
        logical_type: uuid
```
