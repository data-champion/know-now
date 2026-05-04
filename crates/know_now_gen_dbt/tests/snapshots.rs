use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractProject, ContractRelationship,
    ContractSourceColumn, ContractSourceSystem, ContractSourceTable, ContractTrace,
    GeneratorContract,
};
use know_now_gen_dbt::DbtGenerator;

fn redact_version(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.contains("Generator: know_now_gen_dbt. Version:") {
                let idx = line.find("Version:").unwrap();
                format!("{}Version: [REDACTED].", &line[..idx])
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn demo_contract() -> GeneratorContract {
    GeneratorContract {
        contract_version: "1.0".into(),
        project: Some(ContractProject {
            name: "demo".into(),
            description: None,
            owner: None,
            tags: vec![],
        }),
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
                description: Some("Core customer entity.".into()),
                entity_type: None,
                tags: vec![],
                business_key: vec!["id".into()],
                attributes: vec![
                    ContractAttribute {
                        id: "attr_id".into(),
                        name: "id".into(),
                        logical_type: Some("integer".into()),
                        semantic_type: None,
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: Some(true),
                        constraints: vec![],
                        description: Some("Primary key.".into()),
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_email".into(),
                        name: "email".into(),
                        logical_type: Some("string".into()),
                        semantic_type: Some("email".into()),
                        sensitivity: None,
                        pii: Some(true),
                        required: Some(true),
                        is_unique: None,
                        constraints: vec!["max_length:320".into()],
                        description: Some("Contact email.".into()),
                        attr_type: None,
                    },
                ],
            },
            ContractEntity {
                id: "ent_order".into(),
                name: "order".into(),
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
                        id: "attr_order_total".into(),
                        name: "total".into(),
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
                ],
            },
        ],
        relationships: vec![ContractRelationship {
            id: "rel_order_customer".into(),
            from_entity: "order".into(),
            to_entity: "customer".into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("customer_id".into()),
            to_key: Some("id".into()),
            description: None,
        }],
        source_systems: vec![ContractSourceSystem {
            name: "crm_db".into(),
            kind: Some("database".into()),
            description: Some("CRM database.".into()),
            tables: vec![ContractSourceTable {
                name: "customers".into(),
                entity: Some("customer".into()),
                schema: Some("public".into()),
                columns: vec![
                    ContractSourceColumn {
                        source: "cust_id".into(),
                        target: "id".into(),
                        transform: None,
                    },
                    ContractSourceColumn {
                        source: "cust_email".into(),
                        target: "email".into(),
                        transform: Some("LOWER(cust_email)".into()),
                    },
                ],
            }],
        }],
        quality_rules: vec![],
        governance: None,
        open_questions: vec![],
        assumptions: vec![],
        trace: ContractTrace::default(),
    }
}

#[test]
fn dbt_project_yml_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let project = artifacts
        .iter()
        .find(|a| a.path == "dbt/dbt_project.yml")
        .unwrap();
    insta::assert_snapshot!(redact_version(&project.content));
}

#[test]
fn dbt_sources_yml_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let sources = artifacts
        .iter()
        .find(|a| a.path == "dbt/models/sources.yml")
        .unwrap();
    insta::assert_snapshot!(redact_version(&sources.content));
}

#[test]
fn dbt_staging_model_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let stg = artifacts
        .iter()
        .find(|a| a.path.contains("stg_crm_db_customers"))
        .unwrap();
    insta::assert_snapshot!(redact_version(&stg.content));
}

#[test]
fn dbt_mart_model_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let mart = artifacts
        .iter()
        .find(|a| a.path == "dbt/models/marts/order.sql")
        .unwrap();
    insta::assert_snapshot!(redact_version(&mart.content));
}

#[test]
fn dbt_marts_schema_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let schema = artifacts
        .iter()
        .find(|a| a.path == "dbt/models/marts/schema.yml")
        .unwrap();
    insta::assert_snapshot!(redact_version(&schema.content));
}

#[test]
fn dbt_generic_test_snapshot() {
    let gen = DbtGenerator::new();
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let test = artifacts
        .iter()
        .find(|a| a.path == "dbt/tests/generic/is_valid_email.sql")
        .unwrap();
    insta::assert_snapshot!(redact_version(&test.content));
}
