# YAML Authoring Subset

PRD refs: §10.2, §19.1.

know-now accepts a constrained YAML subset for safer parsing and better diagnostics.

Allowed patterns:
- top-level mapping
- scalar keys
- nested mappings/sequences in the supported metadata shape

Disallowed patterns:
- anchors
- aliases
- merge keys
- custom tags
- include directives
- multi-document files

## Valid Example

```yaml
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: uuid
      - name: email
        logical_type: string
        semantic_type: email
```

## Invalid: Anchor

```yaml
# expected-error: META-PAR-ANCHOR
defaults: &defaults
  logical_type: string
```

## Invalid: Alias

```yaml
# expected-error: META-PAR-ALIAS
items:
  - *defaults
```

## Invalid: Merge Key

```yaml
# expected-error: META-PAR-MERGE
base: &base
  logical_type: string
obj:
  <<: *base
```

## Invalid: Custom Tag

```yaml
# expected-error: META-PAR-TAG
value: !custom_type 42
```

## Invalid: Include Directive

```yaml
# expected-error: META-PAR-INCLUDE
items:
  - !include shared/file.yml
```

## Invalid: Multi-document YAML

```yaml
# expected-error: META-PAR-MULTIDOC
entities:
  - name: customer
---
entities:
  - name: order
```
