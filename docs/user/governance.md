# Governance Metadata

PRD refs: §10.9, §19.1.

Governance metadata captures default ownership and compliance posture.

Supported governance object fields:
- `data_owner`
- `data_steward`
- `classification_default`
- `retention_default`

Entity- and attribute-level governance fields are documented in
`metadata-reference.md` (`owner`, `steward`, `classification`, `retention_policy`, `sensitivity`, `pii`).

## Example

```yaml
governance:
  data_owner: finance-data
  data_steward: governance-team
  classification_default: internal
  retention_default: retention_7y

entities:
  - name: payment
    owner: finance-data
    steward: governance-team
    classification: confidential
    retention_policy: retention_7y
    attributes:
      - name: card_last4
        logical_type: string
        sensitivity: high
        pii: true
```
