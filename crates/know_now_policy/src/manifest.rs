use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub severity_profile: SeverityProfile,
    #[serde(default)]
    pub rules: Vec<RuleDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SeverityProfile {
    #[default]
    Standard,
    Strict,
    Relaxed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    #[serde(default = "default_severity")]
    pub severity: RuleSeverity,
    pub applies_to: AppliesTo,
    pub expression: RuleExpression,
    #[serde(default)]
    pub rationale: String,
    #[serde(default)]
    pub remediation: String,
}

fn default_severity() -> RuleSeverity {
    RuleSeverity::Warning
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppliesTo {
    Entity,
    Attribute,
    Relationship,
    Module,
    Domain,
    SourceSystem,
    QualityRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuleExpression {
    AttributePresence {
        attribute: String,
    },
    NamingConvention {
        pattern: String,
    },
    EnumMembership {
        field: String,
        allowed: Vec<String>,
    },
    CardinalityCheck {
        allowed: Vec<String>,
    },
    TagPresence {
        tag: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_deserializes_from_json() {
        let json = r#"{
            "name": "my_corp",
            "version": "1.0.0",
            "description": "Corporate policy pack",
            "severity_profile": "strict",
            "rules": [
                {
                    "id": "CORP-001",
                    "severity": "error",
                    "applies_to": "entity",
                    "expression": {
                        "kind": "attribute_presence",
                        "attribute": "description"
                    },
                    "rationale": "All entities must have a description",
                    "remediation": "Add a description field"
                },
                {
                    "id": "CORP-002",
                    "severity": "warning",
                    "applies_to": "entity",
                    "expression": {
                        "kind": "naming_convention",
                        "pattern": "^[a-z][a-z0-9_]*$"
                    },
                    "rationale": "Entity names must be snake_case",
                    "remediation": "Rename to snake_case"
                },
                {
                    "id": "CORP-003",
                    "severity": "warning",
                    "applies_to": "attribute",
                    "expression": {
                        "kind": "enum_membership",
                        "field": "logical_type",
                        "allowed": ["string", "integer", "boolean", "timestamp", "date", "decimal"]
                    },
                    "rationale": "Only standard types allowed",
                    "remediation": "Use a standard logical type"
                },
                {
                    "id": "CORP-004",
                    "severity": "info",
                    "applies_to": "relationship",
                    "expression": {
                        "kind": "cardinality_check",
                        "allowed": ["one-to-many", "many-to-one"]
                    },
                    "rationale": "Many-to-many requires a junction table",
                    "remediation": "Consider adding a junction entity"
                }
            ]
        }"#;

        let manifest: PackManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "my_corp");
        assert_eq!(manifest.rules.len(), 4);
        assert!(matches!(manifest.severity_profile, SeverityProfile::Strict));

        let rule = &manifest.rules[0];
        assert_eq!(rule.id, "CORP-001");
        assert_eq!(rule.severity, RuleSeverity::Error);
        assert_eq!(rule.applies_to, AppliesTo::Entity);
        assert!(matches!(&rule.expression, RuleExpression::AttributePresence { attribute } if attribute == "description"));
    }

    #[test]
    fn manifest_with_defaults() {
        let json = r#"{
            "name": "minimal",
            "version": "0.1.0",
            "rules": [
                {
                    "id": "MIN-001",
                    "applies_to": "entity",
                    "expression": {
                        "kind": "tag_presence",
                        "tag": "owner"
                    }
                }
            ]
        }"#;

        let manifest: PackManifest = serde_json::from_str(json).unwrap();
        assert!(matches!(manifest.severity_profile, SeverityProfile::Standard));
        assert_eq!(manifest.rules[0].severity, RuleSeverity::Warning);
    }

    #[test]
    fn manifest_roundtrips() {
        let manifest = PackManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "test pack".into(),
            severity_profile: SeverityProfile::Standard,
            rules: vec![RuleDefinition {
                id: "T-001".into(),
                severity: RuleSeverity::Warning,
                applies_to: AppliesTo::Entity,
                expression: RuleExpression::NamingConvention {
                    pattern: "^[a-z_]+$".into(),
                },
                rationale: "names must be lowercase".into(),
                remediation: "rename".into(),
            }],
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: PackManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.rules.len(), 1);
    }
}
