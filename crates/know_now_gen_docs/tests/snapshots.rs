use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAssumption, ContractAttribute, ContractEntity, ContractGovernance,
    ContractOpenQuestion, ContractProject, ContractRelationship, ContractSourceColumn,
    ContractSourceSystem, ContractSourceTable, ContractTrace, GeneratorContract,
};
use know_now_gen_docs::DocsGenerator;

/// Build a rich contract with entities that have descriptions, relationships,
/// source mappings, governance, questions and assumptions.
fn rich_contract() -> GeneratorContract {
    GeneratorContract {
        contract_version: "1.0".into(),
        project: Some(ContractProject {
            name: "Ecommerce".into(),
            description: Some("Online retail data platform.".into()),
            owner: Some("data-team".into()),
            tags: vec!["retail".into()],
        }),
        target_database: None,
        entities: vec![
            ContractEntity {
                id: "ent_customer".into(),
                name: "customer".into(),
                display_name: Some("Customer".into()),
                domain: Some("sales".into()),
                module: None,
                owner: None,
                steward: None,
                classification: None,
                retention_policy: None,
                description: Some("Core customer entity.".into()),
                entity_type: Some("dimension".into()),
                tags: vec!["core".into(), "pii".into()],
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
                        description: Some("Primary surrogate key.".into()),
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_email".into(),
                        name: "email".into(),
                        logical_type: Some("string".into()),
                        semantic_type: Some("email".into()),
                        sensitivity: None,
                        pii: Some(true),
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: Some("Customer email address.".into()),
                        attr_type: None,
                    },
                ],
            },
            ContractEntity {
                id: "ent_order".into(),
                name: "order_line".into(),
                display_name: Some("Order Line".into()),
                domain: Some("sales".into()),
                module: None,
                owner: None,
                steward: None,
                classification: None,
                retention_policy: None,
                description: Some("Individual line items in orders.".into()),
                entity_type: Some("fact".into()),
                tags: vec![],
                business_key: vec!["id".into()],
                attributes: vec![
                    ContractAttribute {
                        id: "attr_order_id".into(),
                        name: "id".into(),
                        logical_type: Some("integer".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: Some(true),
                        constraints: vec![],
                        description: Some("Order line identifier.".into()),
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_order_amount".into(),
                        name: "amount".into(),
                        logical_type: Some("decimal".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: Some("Line item amount.".into()),
                        attr_type: None,
                    },
                ],
            },
        ],
        relationships: vec![ContractRelationship {
            id: "rel_order_customer".into(),
            from_entity: "order_line".into(),
            to_entity: "customer".into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("customer_id".into()),
            to_key: Some("id".into()),
            description: None,
        }],
        source_systems: vec![ContractSourceSystem {
            name: "crm_db".into(),
            kind: Some("database".into()),
            description: None,
            tables: vec![ContractSourceTable {
                name: "customers".into(),
                entity: Some("customer".into()),
                schema: Some("public".into()),
                columns: vec![
                    ContractSourceColumn {
                        source: "cust_email".into(),
                        target: "email".into(),
                        transform: Some("LOWER(cust_email)".into()),
                    },
                    ContractSourceColumn {
                        source: "cust_id".into(),
                        target: "id".into(),
                        transform: None,
                    },
                ],
            }],
        }],
        quality_rules: vec![],
        governance: Some(ContractGovernance {
            data_owner: Some("analytics-team".into()),
            data_steward: Some("jane@example.com".into()),
            classification_default: Some("internal".into()),
            retention_default: Some("3 years".into()),
        }),
        open_questions: vec![ContractOpenQuestion {
            id: "q_email_validation".into(),
            question: "Should email addresses be validated against DNS?".into(),
            context: Some("Currently only format validation is applied.".into()),
            entity: Some("customer".into()),
            priority: Some("medium".into()),
        }],
        assumptions: vec![ContractAssumption {
            id: "a_email_unique".into(),
            statement: "Email addresses are unique per customer.".into(),
            rationale: Some("Business rule confirmed by product team.".into()),
            entity: Some("customer".into()),
            risk: Some("low".into()),
        }],
        trace: ContractTrace::default(),
    }
}

/// Redact the generator version from ownership comments so snapshots stay stable.
fn redact_version(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.contains("Version:") && line.contains("Generated by know-now") {
                // Replace the semver version with a placeholder.
                let mut redacted = String::new();
                let mut rest = line;
                if let Some(pos) = rest.find("Version: ") {
                    redacted.push_str(&rest[..pos]);
                    redacted.push_str("Version: [redacted]");
                    rest = &rest[pos + "Version: ".len()..];
                    // Skip over the version number (digits and dots).
                    let end = rest
                        .find(|c: char| !c.is_ascii_digit() && c != '.')
                        .unwrap_or(rest.len());
                    redacted.push_str(&rest[end..]);
                    return redacted;
                }
                line.to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn docs_overview_snapshot() {
    let gen = DocsGenerator::new();
    let artifacts = gen.generate(&rich_contract()).unwrap();
    let overview = artifacts
        .iter()
        .find(|a| a.path == "docs/README.md")
        .expect("overview artifact should exist");

    let content = redact_version(&overview.content);
    insta::assert_snapshot!(content);
}

#[test]
fn docs_customer_entity_snapshot() {
    let gen = DocsGenerator::new();
    let artifacts = gen.generate(&rich_contract()).unwrap();
    let entity = artifacts
        .iter()
        .find(|a| a.path == "docs/entities/customer.md")
        .expect("customer entity artifact should exist");

    let content = redact_version(&entity.content);
    insta::assert_snapshot!(content);
}

#[test]
fn docs_order_entity_snapshot() {
    let gen = DocsGenerator::new();
    let artifacts = gen.generate(&rich_contract()).unwrap();
    let entity = artifacts
        .iter()
        .find(|a| a.path == "docs/entities/order_line.md")
        .expect("order_line entity artifact should exist");

    let content = redact_version(&entity.content);
    insta::assert_snapshot!(content);
}
