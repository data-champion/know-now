//! Stable object identity crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).
#![allow(clippy::missing_errors_doc)]

use std::collections::BTreeSet;
use std::fmt::Write;

use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_metadata::authoring::AuthoringMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdCheckResult {
    pub missing: Vec<MissingId>,
    pub invalid: Vec<InvalidId>,
    pub duplicate: Vec<DuplicateId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingId {
    pub object_type: String,
    pub name: String,
    pub yaml_path: String,
    pub suggested_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidId {
    pub id: String,
    pub object_type: String,
    pub name: String,
    pub yaml_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateId {
    pub id: String,
    pub locations: Vec<String>,
}

struct CheckAccumulator {
    missing: Vec<MissingId>,
    invalid: Vec<InvalidId>,
    duplicates: Vec<DuplicateId>,
    diagnostics: Vec<Diagnostic>,
    seen: BTreeSet<String>,
}

impl CheckAccumulator {
    fn new() -> Self {
        Self {
            missing: Vec::new(),
            invalid: Vec::new(),
            duplicates: Vec::new(),
            diagnostics: Vec::new(),
            seen: BTreeSet::new(),
        }
    }

    fn record_invalid(
        &mut self,
        id: &str,
        object_type: &str,
        name: &str,
        path: &str,
        reason: &str,
    ) {
        self.invalid.push(InvalidId {
            id: id.to_owned(),
            object_type: object_type.into(),
            name: name.to_owned(),
            yaml_path: path.to_owned(),
            reason: reason.to_owned(),
        });
        self.diagnostics.push(
            Diagnostic::new(
                Severity::Warning,
                "ID-FMT-001",
                format!("{object_type} id '{id}': {reason}"),
            )
            .with_yaml_path(path),
        );
    }

    fn record_missing(
        &mut self,
        object_type: &str,
        name: &str,
        path: &str,
        suggested: String,
        help: &str,
    ) {
        self.missing.push(MissingId {
            object_type: object_type.into(),
            name: name.to_owned(),
            yaml_path: format!("{path}.id"),
            suggested_id: suggested,
        });
        self.diagnostics.push(
            Diagnostic::new(
                Severity::Warning,
                "ID-MISSING-001",
                format!("{object_type} '{name}' has no stable id"),
            )
            .with_yaml_path(path)
            .with_help(help),
        );
    }

    fn check_duplicate(&mut self, id: &str, path: &str) {
        if !self.seen.insert(id.to_owned()) {
            if let Some(dup) = self.duplicates.iter_mut().find(|d| d.id == id) {
                dup.locations.push(path.to_owned());
            } else {
                self.duplicates.push(DuplicateId {
                    id: id.to_owned(),
                    locations: vec![path.to_owned()],
                });
            }
            self.diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "ID-DUP-001",
                    format!("duplicate id '{id}'"),
                )
                .with_yaml_path(path),
            );
        }
    }

    fn check_present_id(
        &mut self,
        id: &str,
        object_type: &str,
        name: &str,
        path: &str,
        prefix: &str,
    ) {
        if let Some(reason) = validate_id_format(id, prefix) {
            self.record_invalid(id, object_type, name, path, &reason);
        }
        self.check_duplicate(id, path);
    }

    fn into_result(self) -> (IdCheckResult, Vec<Diagnostic>) {
        let result = IdCheckResult {
            missing: self.missing,
            invalid: self.invalid,
            duplicate: self.duplicates,
        };
        (result, self.diagnostics)
    }
}

pub fn check_ids(metadata: &AuthoringMetadata) -> (IdCheckResult, Vec<Diagnostic>) {
    let mut acc = CheckAccumulator::new();
    check_domains(metadata, &mut acc);
    check_modules(metadata, &mut acc);
    check_entities(metadata, &mut acc);
    check_relationships(metadata, &mut acc);
    check_rules(metadata, &mut acc);
    acc.into_result()
}

fn check_domains(metadata: &AuthoringMetadata, acc: &mut CheckAccumulator) {
    for (idx, domain) in metadata.domains.iter().enumerate() {
        let path = format!("domains[{idx}].id");
        acc.check_present_id(&domain.id, "domain", &domain.name, &path, "dom_");
    }
}

fn check_modules(metadata: &AuthoringMetadata, acc: &mut CheckAccumulator) {
    for (idx, module) in metadata.modules.iter().enumerate() {
        let path = format!("modules[{idx}].id");
        acc.check_present_id(&module.id, "module", &module.name, &path, "mod_");
    }
}

fn check_entities(metadata: &AuthoringMetadata, acc: &mut CheckAccumulator) {
    for (ent_idx, entity) in metadata.entities.iter().enumerate() {
        let ent_path = format!("entities[{ent_idx}]");
        if let Some(id) = &entity.id {
            let path = format!("{ent_path}.id");
            acc.check_present_id(id, "entity", &entity.name, &path, "ent_");
        } else {
            let suggested = suggest_entity_id(&entity.name, entity.domain.as_deref());
            acc.record_missing(
                "entity",
                &entity.name,
                &ent_path,
                suggested,
                "Add an 'id' field, e.g., 'id: ent_<name>'.",
            );
        }

        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            let attr_path = format!("{ent_path}.attributes[{attr_idx}]");
            if let Some(id) = &attr.id {
                let path = format!("{attr_path}.id");
                acc.check_present_id(id, "attribute", &attr.name, &path, "attr_");
            } else {
                let suggested = suggest_attribute_id(&entity.name, &attr.name);
                acc.record_missing(
                    "attribute",
                    &attr.name,
                    &attr_path,
                    suggested,
                    "Add an 'id' field, e.g., 'id: attr_<entity>_<name>'.",
                );
            }
        }
    }
}

fn check_relationships(metadata: &AuthoringMetadata, acc: &mut CheckAccumulator) {
    for (idx, rel) in metadata.relationships.iter().enumerate() {
        let rel_path = format!("relationships[{idx}]");
        let display_name = format!("{}→{}", rel.from_entity, rel.to_entity);
        if let Some(id) = &rel.id {
            let path = format!("{rel_path}.id");
            acc.check_present_id(id, "relationship", &display_name, &path, "rel_");
        } else {
            let suggested = suggest_relationship_id(&rel.from_entity, &rel.to_entity);
            acc.record_missing(
                "relationship",
                &display_name,
                &rel_path,
                suggested,
                "Add an 'id' field, e.g., 'id: rel_<from>_<to>'.",
            );
        }
    }
}

fn check_rules(metadata: &AuthoringMetadata, acc: &mut CheckAccumulator) {
    for (idx, rule) in metadata.rules.iter().enumerate() {
        let rule_path = format!("rules[{idx}]");
        if let Some(id) = &rule.id {
            let path = format!("{rule_path}.id");
            acc.check_present_id(id, "quality_rule", &rule.name, &path, "rule_");
        } else {
            let suggested = suggest_rule_id(&rule.name);
            acc.record_missing(
                "quality_rule",
                &rule.name,
                &rule_path,
                suggested,
                "Add an 'id' field, e.g., 'id: rule_<name>'.",
            );
        }
    }
}

fn validate_id_format(id: &str, expected_prefix: &str) -> Option<String> {
    if id.is_empty() {
        return Some("id is empty".into());
    }
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Some(
            "id must contain only lowercase ASCII letters, digits, and underscores".into(),
        );
    }
    if !id.starts_with(expected_prefix) {
        return Some(format!("id should start with '{expected_prefix}'"));
    }
    None
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

#[must_use]
pub fn suggest_entity_id(name: &str, domain: Option<&str>) -> String {
    let norm = normalize_name(name);
    domain.map_or_else(
        || format!("ent_{norm}"),
        |d| format!("ent_{}_{norm}", normalize_name(d)),
    )
}

#[must_use]
pub fn suggest_attribute_id(entity_name: &str, attr_name: &str) -> String {
    format!(
        "attr_{}_{}",
        normalize_name(entity_name),
        normalize_name(attr_name)
    )
}

#[must_use]
pub fn suggest_relationship_id(from_entity: &str, to_entity: &str) -> String {
    format!(
        "rel_{}_{}",
        normalize_name(from_entity),
        normalize_name(to_entity)
    )
}

#[must_use]
pub fn suggest_rule_id(rule_name: &str) -> String {
    format!("rule_{}", normalize_name(rule_name))
}

pub fn suggest_all_ids(metadata: &AuthoringMetadata) -> Vec<MissingId> {
    let mut suggestions = Vec::new();

    for (ent_idx, entity) in metadata.entities.iter().enumerate() {
        if entity.id.is_none() {
            suggestions.push(MissingId {
                object_type: "entity".into(),
                name: entity.name.clone(),
                yaml_path: format!("entities[{ent_idx}].id"),
                suggested_id: suggest_entity_id(&entity.name, entity.domain.as_deref()),
            });
        }
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            if attr.id.is_none() {
                suggestions.push(MissingId {
                    object_type: "attribute".into(),
                    name: attr.name.clone(),
                    yaml_path: format!("entities[{ent_idx}].attributes[{attr_idx}].id"),
                    suggested_id: suggest_attribute_id(&entity.name, &attr.name),
                });
            }
        }
    }

    for (idx, rel) in metadata.relationships.iter().enumerate() {
        if rel.id.is_none() {
            suggestions.push(MissingId {
                object_type: "relationship".into(),
                name: format!("{}→{}", rel.from_entity, rel.to_entity),
                yaml_path: format!("relationships[{idx}].id"),
                suggested_id: suggest_relationship_id(&rel.from_entity, &rel.to_entity),
            });
        }
    }

    for (idx, rule) in metadata.rules.iter().enumerate() {
        if rule.id.is_none() {
            suggestions.push(MissingId {
                object_type: "quality_rule".into(),
                name: rule.name.clone(),
                yaml_path: format!("rules[{idx}].id"),
                suggested_id: suggest_rule_id(&rule.name),
            });
        }
    }

    suggestions
}

#[must_use]
pub fn backfill_preview(metadata: &AuthoringMetadata) -> String {
    let suggestions = suggest_all_ids(metadata);
    if suggestions.is_empty() {
        return "No missing IDs found.\n".into();
    }
    let mut out = String::new();
    for s in &suggestions {
        let _ = writeln!(
            out,
            "  {}: {} → id: {}",
            s.yaml_path, s.name, s.suggested_id
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use know_now_metadata::test_support::parse_yaml_metadata;

    use super::*;

    #[test]
    fn all_ids_present_no_missing() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
",
        );
        let (result, diags) = check_ids(&meta);
        assert!(result.missing.is_empty());
        assert!(result.duplicate.is_empty());
        assert!(!diags.iter().any(|d| d.code == "ID-MISSING-001"));
    }

    #[test]
    fn missing_entity_id() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
",
        );
        let (result, diags) = check_ids(&meta);
        assert_eq!(result.missing.len(), 1);
        assert_eq!(result.missing[0].object_type, "entity");
        assert!(diags.iter().any(|d| d.code == "ID-MISSING-001"));
    }

    #[test]
    fn missing_attribute_id() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes:
      - name: id
        logical_type: integer
",
        );
        let (result, _diags) = check_ids(&meta);
        assert_eq!(result.missing.len(), 1);
        assert_eq!(result.missing[0].object_type, "attribute");
    }

    #[test]
    fn missing_relationship_id() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes: []
relationships:
  - from_entity: order
    to_entity: customer
",
        );
        let (result, _diags) = check_ids(&meta);
        assert!(result
            .missing
            .iter()
            .any(|m| m.object_type == "relationship"));
    }

    #[test]
    fn duplicate_id_detected() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes: []
  - id: ent_customer
    name: order
    attributes: []
",
        );
        let (result, diags) = check_ids(&meta);
        assert!(!result.duplicate.is_empty());
        assert!(diags.iter().any(|d| d.code == "ID-DUP-001"));
    }

    #[test]
    fn invalid_id_format_uppercase() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: Ent_Customer
    name: customer
    attributes: []
",
        );
        let (result, diags) = check_ids(&meta);
        assert!(!result.invalid.is_empty());
        assert!(diags.iter().any(|d| d.code == "ID-FMT-001"));
    }

    #[test]
    fn invalid_id_wrong_prefix() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: rel_customer
    name: customer
    attributes: []
",
        );
        let (result, _diags) = check_ids(&meta);
        assert!(!result.invalid.is_empty());
        assert!(result.invalid[0].reason.contains("ent_"));
    }

    #[test]
    fn suggest_entity_id_simple() {
        assert_eq!(suggest_entity_id("customer", None), "ent_customer");
    }

    #[test]
    fn suggest_entity_id_with_domain() {
        assert_eq!(
            suggest_entity_id("customer", Some("sales")),
            "ent_sales_customer"
        );
    }

    #[test]
    fn suggest_entity_id_normalizes() {
        assert_eq!(
            suggest_entity_id("Customer Order", None),
            "ent_customer_order"
        );
    }

    #[test]
    fn suggest_attribute_id_format() {
        assert_eq!(
            suggest_attribute_id("customer", "email"),
            "attr_customer_email"
        );
    }

    #[test]
    fn suggest_relationship_id_format() {
        assert_eq!(
            suggest_relationship_id("order", "customer"),
            "rel_order_customer"
        );
    }

    #[test]
    fn suggest_rule_id_format() {
        assert_eq!(suggest_rule_id("email_format"), "rule_email_format");
    }

    #[test]
    fn suggest_all_ids_deterministic() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
      - name: email
        logical_type: string
relationships:
  - from_entity: order
    to_entity: customer
",
        );
        let run1 = suggest_all_ids(&meta);
        let run2 = suggest_all_ids(&meta);
        assert_eq!(run1.len(), run2.len());
        for (a, b) in run1.iter().zip(run2.iter()) {
            assert_eq!(a.suggested_id, b.suggested_id);
        }
    }

    #[test]
    fn backfill_preview_shows_suggestions() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
",
        );
        let preview = backfill_preview(&meta);
        assert!(preview.contains("ent_customer"));
        assert!(preview.contains("attr_customer_id"));
    }

    #[test]
    fn backfill_preview_empty_when_all_present() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
",
        );
        let preview = backfill_preview(&meta);
        assert_eq!(preview, "No missing IDs found.\n");
    }

    #[test]
    fn check_does_not_mutate_metadata() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
",
        );
        let json_before = serde_json::to_string(&meta).unwrap();
        let _result = check_ids(&meta);
        let json_after = serde_json::to_string(&meta).unwrap();
        assert_eq!(json_before, json_after);
    }

    #[test]
    fn empty_metadata_no_issues() {
        let meta = parse_yaml_metadata("{}");
        let (result, diags) = check_ids(&meta);
        assert!(result.missing.is_empty());
        assert!(result.invalid.is_empty());
        assert!(result.duplicate.is_empty());
        assert!(diags.is_empty());
    }

    #[test]
    fn normalize_name_cases() {
        assert_eq!(normalize_name("customer"), "customer");
        assert_eq!(normalize_name("Customer Order"), "customer_order");
        assert_eq!(normalize_name("UPPER_CASE"), "upper_case");
        assert_eq!(normalize_name("a--b"), "a_b");
        assert_eq!(normalize_name("__leading__"), "leading");
    }

    #[test]
    fn check_result_serializes() {
        let result = IdCheckResult {
            missing: vec![MissingId {
                object_type: "entity".into(),
                name: "customer".into(),
                yaml_path: "entities[0]".into(),
                suggested_id: "ent_customer".into(),
            }],
            invalid: vec![],
            duplicate: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: IdCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.missing.len(), 1);
    }

    #[test]
    fn all_diagnostics_have_yaml_path() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
relationships:
  - from_entity: order
    to_entity: customer
",
        );
        let (_result, diags) = check_ids(&meta);
        for d in &diags {
            assert!(
                d.yaml_path.is_some(),
                "diagnostic {} should have yaml_path",
                d.code
            );
        }
    }

    #[test]
    fn quality_rule_missing_id() {
        let meta = parse_yaml_metadata(
            r"
entities:
  - id: ent_customer
    name: customer
    attributes: []
rules:
  - name: email_format
    entity: customer
",
        );
        let (result, _diags) = check_ids(&meta);
        assert!(result
            .missing
            .iter()
            .any(|m| m.object_type == "quality_rule"));
    }
}
