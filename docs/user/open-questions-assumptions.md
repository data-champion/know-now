# Open Questions and Assumptions

PRD refs: §10.10, §19.1.

Use open questions to track unresolved design decisions and assumptions to capture risk-bearing beliefs.

## Open Questions Fields

- `id`
- `question`
- `context`
- `entity`
- `priority`

## Assumptions Fields

- `id`
- `statement`
- `rationale`
- `entity`
- `risk`

## Example

```yaml
open_questions:
  - id: q_refunds
    question: Should refunded orders be excluded from LTV?
    context: KPI definition for stakeholder reporting.
    entity: customer
    priority: high

assumptions:
  - id: asm_email_unique
    statement: Customer email is unique across active customers.
    rationale: Required by identity merge strategy.
    entity: customer
    risk: medium
```
