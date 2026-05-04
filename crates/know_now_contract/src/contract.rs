use serde::{Deserialize, Serialize};

/// Versioned generator contract (PRD §8.3 contract layer 3).
///
/// This is the stable, versioned structure that built-in generators,
/// declarative templates, and future external generators all consume.
/// It carries a projected view of the `ProjectGraph` — no raw YAML
/// nodes, no serde-saphyr types, no internal graph types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorContract {
    pub contract_version: String,
    pub project: Option<ContractProject>,
    pub target_database: Option<ContractTargetDatabase>,
    pub entities: Vec<ContractEntity>,
    pub relationships: Vec<ContractRelationship>,
    pub source_systems: Vec<ContractSourceSystem>,
    pub quality_rules: Vec<ContractQualityRule>,
    pub governance: Option<ContractGovernance>,
    pub open_questions: Vec<ContractOpenQuestion>,
    pub assumptions: Vec<ContractAssumption>,
    pub trace: ContractTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractProject {
    pub name: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTargetDatabase {
    pub kind: String,
    pub version: Option<String>,
    pub compatibility_floor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEntity {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub domain: Option<String>,
    pub module: Option<String>,
    pub owner: Option<String>,
    pub steward: Option<String>,
    pub classification: Option<String>,
    pub retention_policy: Option<String>,
    pub description: Option<String>,
    pub entity_type: Option<String>,
    pub tags: Vec<String>,
    pub business_key: Vec<String>,
    pub attributes: Vec<ContractAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAttribute {
    pub id: String,
    pub name: String,
    pub logical_type: Option<String>,
    pub semantic_type: Option<String>,
    pub sensitivity: Option<String>,
    pub pii: Option<bool>,
    pub required: Option<bool>,
    pub is_unique: Option<bool>,
    pub constraints: Vec<String>,
    pub description: Option<String>,
    pub attr_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRelationship {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub cardinality: Option<String>,
    pub from_key: Option<String>,
    pub to_key: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSourceSystem {
    pub name: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tables: Vec<ContractSourceTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSourceTable {
    pub name: String,
    pub entity: Option<String>,
    pub schema: Option<String>,
    pub columns: Vec<ContractSourceColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSourceColumn {
    pub source: String,
    pub target: String,
    pub transform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractQualityRule {
    pub id: String,
    pub name: String,
    pub entity: Option<String>,
    pub attribute: Option<String>,
    pub rule_type: Option<String>,
    pub expression: Option<String>,
    pub severity: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGovernance {
    pub data_owner: Option<String>,
    pub data_steward: Option<String>,
    pub classification_default: Option<String>,
    pub retention_default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractOpenQuestion {
    pub id: String,
    pub question: String,
    pub context: Option<String>,
    pub entity: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAssumption {
    pub id: String,
    pub statement: String,
    pub rationale: Option<String>,
    pub entity: Option<String>,
    pub risk: Option<String>,
}

/// Graph-to-contract trace map (PRD §8.7, §8.11).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractTrace {
    pub entity_ids: Vec<String>,
    pub attribute_ids: Vec<String>,
    pub relationship_ids: Vec<String>,
    pub rule_ids: Vec<String>,
    pub question_ids: Vec<String>,
    pub assumption_ids: Vec<String>,
}

impl GeneratorContract {
    /// Contract is Send + Sync + Clone by derive — verify at compile time.
    const _ASSERT_SEND: fn() = || {
        fn assert_send<T: Send + Sync + Clone>() {}
        assert_send::<Self>();
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contract() -> GeneratorContract {
        GeneratorContract {
            contract_version: "1.0".into(),
            project: Some(ContractProject {
                name: "demo".into(),
                description: Some("Demo project".into()),
                owner: Some("team".into()),
                tags: vec!["demo".into()],
            }),
            target_database: Some(ContractTargetDatabase {
                kind: "postgres".into(),
                version: Some("18".into()),
                compatibility_floor: Some("16".into()),
            }),
            entities: vec![ContractEntity {
                id: "ent_customer".into(),
                name: "customer".into(),
                display_name: None,
                domain: Some("sales".into()),
                module: None,
                owner: None,
                steward: None,
                classification: None,
                retention_policy: None,
                description: None,
                entity_type: Some("dimension".into()),
                tags: vec![],
                business_key: vec!["email".into()],
                attributes: vec![
                    ContractAttribute {
                        id: "attr_customer_id".into(),
                        name: "id".into(),
                        logical_type: Some("integer".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: Some(true),
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_email".into(),
                        name: "email".into(),
                        logical_type: Some("string".into()),
                        semantic_type: Some("email".into()),
                        sensitivity: None,
                        pii: Some(true),
                        required: None,
                        is_unique: None,
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                ],
            }],
            relationships: vec![ContractRelationship {
                id: "rel_order_customer".into(),
                from_entity: "order".into(),
                to_entity: "customer".into(),
                cardinality: Some("many_to_one".into()),
                from_key: Some("customer_id".into()),
                to_key: Some("id".into()),
                description: None,
            }],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace {
                entity_ids: vec!["ent_customer".into()],
                attribute_ids: vec!["attr_customer_id".into(), "attr_customer_email".into()],
                relationship_ids: vec!["rel_order_customer".into()],
                rule_ids: vec![],
                question_ids: vec![],
                assumption_ids: vec![],
            },
        }
    }

    #[test]
    fn contract_json_roundtrip() {
        let c = sample_contract();
        let json = serde_json::to_string_pretty(&c).unwrap();
        let parsed: GeneratorContract = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.contract_version, "1.0");
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.relationships.len(), 1);
    }

    #[test]
    fn contract_version_present() {
        let c = sample_contract();
        assert_eq!(c.contract_version, "1.0");
    }

    #[test]
    fn trace_captures_ids() {
        let c = sample_contract();
        assert_eq!(c.trace.entity_ids.len(), 1);
        assert_eq!(c.trace.attribute_ids.len(), 2);
        assert_eq!(c.trace.relationship_ids.len(), 1);
    }

    #[test]
    fn contract_is_send_sync_clone() {
        fn assert_send_sync_clone<T: Send + Sync + Clone>() {}
        assert_send_sync_clone::<GeneratorContract>();
    }

    #[test]
    fn empty_contract() {
        let c = GeneratorContract {
            contract_version: "1.0".into(),
            project: None,
            target_database: None,
            entities: vec![],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace::default(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: GeneratorContract = serde_json::from_str(&json).unwrap();
        assert!(parsed.entities.is_empty());
    }

    #[test]
    fn schema_version_constant() {
        assert_eq!(crate::CONTRACT_SCHEMA_VERSION, "1.0");
    }
}
