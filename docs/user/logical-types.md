# Logical Types

PRD refs: §10.7, §19.1.

Logical types drive portable storage semantics for generated artifacts.

Current parser-recognized logical types:
- `integer`
- `bigint`
- `smallint`
- `decimal`
- `float`
- `double`
- `boolean`
- `string`
- `text`
- `date`
- `time`
- `timestamp`
- `timestamp_tz`
- `uuid`
- `json`
- `jsonb`
- `binary`
- `interval`
- `array`

## Example

```yaml
entities:
  - name: order
    attributes:
      - name: order_id
        logical_type: uuid
      - name: total_amount
        logical_type: decimal
      - name: is_paid
        logical_type: boolean
      - name: created_at
        logical_type: timestamp_tz
      - name: metadata
        logical_type: json
```
