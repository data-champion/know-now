# Semantic Types

PRD refs: §10.8, §19.1.

Semantic types express business meaning and downstream quality/documentation intent.

Current parser-recognized semantic types:
- `email`
- `phone`
- `url`
- `currency`
- `percentage`
- `ip_address`
- `mac_address`
- `ssn`
- `credit_card`
- `postal_code`
- `country`
- `language`
- `latitude`
- `longitude`
- `geo_point`
- `file_path`
- `mime_type`
- `markdown`
- `html`

## Example

```yaml
entities:
  - name: customer
    attributes:
      - name: email
        logical_type: string
        semantic_type: email
      - name: phone
        logical_type: string
        semantic_type: phone
      - name: website
        logical_type: string
        semantic_type: url
      - name: country_code
        logical_type: string
        semantic_type: country
      - name: postal_code
        logical_type: string
        semantic_type: postal_code
```
