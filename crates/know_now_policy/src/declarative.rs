use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_metadata::authoring::AuthoringMetadata;

use crate::engine::{PolicyPack, PolicyPackInfo, PolicyRule};
use crate::manifest::{
    AppliesTo, PackManifest, RuleDefinition, RuleExpression, RuleSeverity,
};

pub struct DeclarativePack {
    manifest: PackManifest,
    rules_cache: Vec<PolicyRule>,
}

impl DeclarativePack {
    pub fn from_manifest(manifest: PackManifest) -> Self {
        let rules_cache = Vec::new();
        Self {
            manifest,
            rules_cache,
        }
    }
}

impl PolicyPack for DeclarativePack {
    fn info(&self) -> PolicyPackInfo {
        PolicyPackInfo {
            pack: self.manifest.name.clone(),
            version: self.manifest.version.clone(),
            hash: format!("sha256:manifest-{}", self.manifest.name),
        }
    }

    fn rules(&self) -> &[PolicyRule] {
        &self.rules_cache
    }

    fn evaluate(&self, metadata: &AuthoringMetadata) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.manifest.rules {
            evaluate_rule(rule, metadata, &mut diagnostics);
        }
        diagnostics
    }
}

fn to_severity(s: &RuleSeverity) -> Severity {
    match s {
        RuleSeverity::Info => Severity::Info,
        RuleSeverity::Warning => Severity::Warning,
        RuleSeverity::Error => Severity::Error,
    }
}

fn evaluate_rule(
    rule: &RuleDefinition,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match rule.applies_to {
        AppliesTo::Entity => evaluate_entity_rule(rule, metadata, diagnostics),
        AppliesTo::Attribute => evaluate_attribute_rule(rule, metadata, diagnostics),
        AppliesTo::Relationship => evaluate_relationship_rule(rule, metadata, diagnostics),
        AppliesTo::Module => evaluate_module_rule(rule, metadata, diagnostics),
        _ => {}
    }
}

fn evaluate_entity_rule(
    rule: &RuleDefinition,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (idx, entity) in metadata.entities.iter().enumerate() {
        let violation = match &rule.expression {
            RuleExpression::AttributePresence { attribute } => {
                let has_field = match attribute.as_str() {
                    "description" => entity.description.is_some(),
                    "id" => entity.id.is_some(),
                    _ => true,
                };
                if has_field { None } else {
                    Some(format!("entity '{}' is missing '{attribute}'", entity.name))
                }
            }
            RuleExpression::NamingConvention { pattern } => {
                match regex_matches(&entity.name, pattern) {
                    Ok(true) => None,
                    Ok(false) => Some(format!(
                        "entity name '{}' does not match pattern '{pattern}'",
                        entity.name
                    )),
                    Err(e) => Some(format!("invalid regex '{pattern}': {e}")),
                }
            }
            RuleExpression::TagPresence { tag } => {
                let has_tag = entity.tags.iter().any(|t| t == tag);
                if has_tag { None } else {
                    Some(format!("entity '{}' is missing tag '{tag}'", entity.name))
                }
            }
            _ => None,
        };

        if let Some(message) = violation {
            diagnostics.push(
                Diagnostic::new(to_severity(&rule.severity), &rule.id, message)
                    .with_yaml_path(format!("entities[{idx}]"))
                    .with_help(&rule.remediation),
            );
        }
    }
}

fn evaluate_attribute_rule(
    rule: &RuleDefinition,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (ent_idx, entity) in metadata.entities.iter().enumerate() {
        for (attr_idx, attr) in entity.attributes.iter().enumerate() {
            let violation = match &rule.expression {
                RuleExpression::NamingConvention { pattern } => {
                    match regex_matches(&attr.name, pattern) {
                        Ok(true) => None,
                        Ok(false) => Some(format!(
                            "attribute '{}'.'{}'  does not match pattern '{pattern}'",
                            entity.name, attr.name
                        )),
                        Err(e) => Some(format!("invalid regex '{pattern}': {e}")),
                    }
                }
                RuleExpression::EnumMembership { field, allowed } => {
                    let value = match field.as_str() {
                        "logical_type" => attr
                            .logical_type
                            .as_ref()
                            .and_then(|lt| serde_json::to_value(lt).ok())
                            .and_then(|v| v.as_str().map(String::from)),
                        _ => None,
                    };
                    value.and_then(|v| {
                        if allowed.iter().any(|a| a == &v) {
                            None
                        } else {
                            Some(format!(
                                "attribute '{}'.'{}'  has {field}='{v}' not in allowed set",
                                entity.name, attr.name
                            ))
                        }
                    })
                }
                RuleExpression::AttributePresence { attribute } => {
                    let has_field = match attribute.as_str() {
                        "description" => attr.description.is_some(),
                        _ => true,
                    };
                    if has_field { None } else {
                        Some(format!(
                            "attribute '{}'.'{}'  is missing '{attribute}'",
                            entity.name, attr.name
                        ))
                    }
                }
                _ => None,
            };

            if let Some(message) = violation {
                diagnostics.push(
                    Diagnostic::new(to_severity(&rule.severity), &rule.id, message)
                        .with_yaml_path(format!(
                            "entities[{ent_idx}].attributes[{attr_idx}]"
                        ))
                        .with_help(&rule.remediation),
                );
            }
        }
    }
}

fn evaluate_relationship_rule(
    rule: &RuleDefinition,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (idx, rel) in metadata.relationships.iter().enumerate() {
        let violation = match &rule.expression {
            RuleExpression::CardinalityCheck { allowed } => {
                let card = rel.cardinality.as_deref().unwrap_or("unknown");
                if allowed.iter().any(|a| a == card) {
                    None
                } else {
                    Some(format!(
                        "relationship '{}'->'{}' has cardinality '{card}' not in allowed set",
                        rel.from_entity, rel.to_entity
                    ))
                }
            }
            _ => None,
        };

        if let Some(message) = violation {
            diagnostics.push(
                Diagnostic::new(to_severity(&rule.severity), &rule.id, message)
                    .with_yaml_path(format!("relationships[{idx}]"))
                    .with_help(&rule.remediation),
            );
        }
    }
}

fn evaluate_module_rule(
    rule: &RuleDefinition,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (idx, module) in metadata.modules.iter().enumerate() {
        let violation = match &rule.expression {
            RuleExpression::NamingConvention { pattern } => {
                match regex_matches(&module.id, pattern) {
                    Ok(true) => None,
                    Ok(false) => Some(format!(
                        "module id '{}' does not match pattern '{pattern}'",
                        module.id
                    )),
                    Err(e) => Some(format!("invalid regex '{pattern}': {e}")),
                }
            }
            _ => None,
        };

        if let Some(message) = violation {
            diagnostics.push(
                Diagnostic::new(to_severity(&rule.severity), &rule.id, message)
                    .with_yaml_path(format!("modules[{idx}]"))
                    .with_help(&rule.remediation),
            );
        }
    }
}

fn regex_matches(value: &str, pattern: &str) -> Result<bool, String> {
    let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
    Ok(re.is_match(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{PackManifest, SeverityProfile};

    fn test_metadata() -> AuthoringMetadata {
        know_now_metadata::test_support::parse_yaml_metadata(
            r#"
version: "1.0"
entities:
  - name: customer
    description: A customer
    tags:
      - "owner:team-a"
    attributes:
      - name: id
        logical_type: integer
        description: Primary key
  - name: order
    attributes:
      - name: total
        logical_type: decimal
relationships:
  - from_entity: order
    to_entity: customer
    cardinality: many-to-one
"#,
        )
    }

    fn make_pack(rules: Vec<RuleDefinition>) -> DeclarativePack {
        DeclarativePack::from_manifest(PackManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: String::new(),
            severity_profile: SeverityProfile::Standard,
            rules,
        })
    }

    #[test]
    fn attribute_presence_on_entity() {
        let pack = make_pack(vec![RuleDefinition {
            id: "T-001".into(),
            severity: RuleSeverity::Warning,
            applies_to: AppliesTo::Entity,
            expression: RuleExpression::AttributePresence {
                attribute: "description".into(),
            },
            rationale: String::new(),
            remediation: "Add description".into(),
        }]);

        let diags = pack.evaluate(&test_metadata());
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("order"));
    }

    #[test]
    fn naming_convention_on_entity() {
        let pack = make_pack(vec![RuleDefinition {
            id: "T-002".into(),
            severity: RuleSeverity::Error,
            applies_to: AppliesTo::Entity,
            expression: RuleExpression::NamingConvention {
                pattern: "^[a-z][a-z0-9_]*$".into(),
            },
            rationale: String::new(),
            remediation: String::new(),
        }]);

        let diags = pack.evaluate(&test_metadata());
        assert!(diags.is_empty(), "both names match snake_case pattern");
    }

    #[test]
    fn enum_membership_on_attribute() {
        let pack = make_pack(vec![RuleDefinition {
            id: "T-003".into(),
            severity: RuleSeverity::Warning,
            applies_to: AppliesTo::Attribute,
            expression: RuleExpression::EnumMembership {
                field: "logical_type".into(),
                allowed: vec!["string".into(), "integer".into(), "boolean".into()],
            },
            rationale: String::new(),
            remediation: String::new(),
        }]);

        let diags = pack.evaluate(&test_metadata());
        assert_eq!(diags.len(), 1, "decimal is not in allowed set");
        assert!(diags[0].message.contains("decimal"));
    }

    #[test]
    fn cardinality_check_on_relationship() {
        let pack = make_pack(vec![RuleDefinition {
            id: "T-004".into(),
            severity: RuleSeverity::Info,
            applies_to: AppliesTo::Relationship,
            expression: RuleExpression::CardinalityCheck {
                allowed: vec!["one-to-many".into()],
            },
            rationale: String::new(),
            remediation: String::new(),
        }]);

        let diags = pack.evaluate(&test_metadata());
        assert_eq!(diags.len(), 1, "many-to-one is not in allowed set");
    }

    #[test]
    fn tag_presence_on_entity() {
        let pack = make_pack(vec![RuleDefinition {
            id: "T-005".into(),
            severity: RuleSeverity::Warning,
            applies_to: AppliesTo::Entity,
            expression: RuleExpression::TagPresence {
                tag: "owner:team-a".into(),
            },
            rationale: String::new(),
            remediation: "Add owner tag".into(),
        }]);

        let diags = pack.evaluate(&test_metadata());
        assert_eq!(diags.len(), 1, "order entity missing the tag");
        assert!(diags[0].message.contains("order"));
    }

    #[test]
    fn pack_info_reflects_manifest() {
        let pack = make_pack(vec![]);
        let info = pack.info();
        assert_eq!(info.pack, "test");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn empty_rules_produces_no_diagnostics() {
        let pack = make_pack(vec![]);
        let diags = pack.evaluate(&test_metadata());
        assert!(diags.is_empty());
    }
}
