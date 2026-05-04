use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractRelationship, ContractTrace, GeneratorContract,
};
use know_now_gen_postgres::PostgresGenerator;

/// Build a contract with a single `customer` entity covering
/// integer, string, boolean types plus a business key.
fn customer_contract() -> GeneratorContract {
    GeneratorContract {
        contract_version: "1.0".into(),
        project: None,
        target_database: None,
        entities: vec![ContractEntity {
            id: "ent_customer".into(),
            name: "customer".into(),
            display_name: None,
            domain: None,
            module: None,
            owner: None,
            steward: None,
            classification: None,
            retention_policy: None,
            description: None,
            entity_type: None,
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
                    required: Some(true),
                    is_unique: None,
                    constraints: vec![],
                    description: None,
                    attr_type: None,
                },
                ContractAttribute {
                    id: "attr_customer_active".into(),
                    name: "active".into(),
                    logical_type: Some("boolean".into()),
                    semantic_type: None,
                    sensitivity: None,
                    pii: None,
                    required: Some(true),
                    is_unique: None,
                    constraints: vec![],
                    description: None,
                    attr_type: None,
                },
            ],
        }],
        relationships: vec![],
        source_systems: vec![],
        quality_rules: vec![],
        governance: None,
        open_questions: vec![],
        assumptions: vec![],
        trace: ContractTrace::default(),
    }
}

/// Build a multi-entity contract: customer + order with a relationship,
/// covering integer, string, boolean, decimal, and timestamp types.
fn multi_entity_contract() -> GeneratorContract {
    GeneratorContract {
        contract_version: "1.0".into(),
        project: None,
        target_database: None,
        entities: vec![
            ContractEntity {
                id: "ent_customer".into(),
                name: "customer".into(),
                display_name: None,
                domain: None,
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
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_active".into(),
                        name: "active".into(),
                        logical_type: Some("boolean".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: None,
                        attr_type: None,
                    },
                ],
            },
            ContractEntity {
                id: "ent_order".into(),
                name: "order_line".into(),
                display_name: None,
                domain: None,
                module: None,
                owner: None,
                steward: None,
                classification: None,
                retention_policy: None,
                description: None,
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
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_order_customer_id".into(),
                        name: "customer_id".into(),
                        logical_type: Some("integer".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: None,
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
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_order_created_at".into(),
                        name: "created_at".into(),
                        logical_type: Some("timestamp".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec![],
                        description: None,
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
        source_systems: vec![],
        quality_rules: vec![],
        governance: None,
        open_questions: vec![],
        assumptions: vec![],
        trace: ContractTrace::default(),
    }
}

#[test]
fn postgres_ddl_snapshot() {
    let gen = PostgresGenerator::new();
    let artifacts = gen.generate(&customer_contract()).unwrap();
    assert_eq!(artifacts.len(), 1);

    // Redact the generator version line so the snapshot is stable across releases.
    let sql = redact_version(&artifacts[0].content);
    insta::assert_snapshot!(sql);
}

#[test]
fn postgres_ddl_multi_entity_snapshot() {
    let gen = PostgresGenerator::new();
    let artifacts = gen.generate(&multi_entity_contract()).unwrap();
    assert_eq!(artifacts.len(), 1);

    let sql = redact_version(&artifacts[0].content);
    insta::assert_snapshot!(sql);
}

/// Replace the volatile generator-version and input-hash header lines with
/// stable placeholders so snapshots don't break on version bumps.
fn redact_version(sql: &str) -> String {
    sql.lines()
        .map(|line| {
            if line.starts_with("-- Generator version:") {
                "-- Generator version: [redacted]".to_owned()
            } else if line.starts_with("-- Input hash:") {
                "-- Input hash: [redacted]".to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
