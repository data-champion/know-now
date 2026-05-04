use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractProject, ContractRelationship, ContractTrace,
    GeneratorContract,
};
use know_now_gen_fixtures::FixtureGenerator;

fn redact_version(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.contains("Generator: know_now_gen_fixtures. Version:")
                || line.contains("know_now_gen_fixtures` v")
            {
                let idx = line.find("Version: ").or_else(|| line.find("` v"));
                match idx {
                    Some(i) if line.contains("Version: ") => {
                        format!("{}Version: [REDACTED].", &line[..i])
                    }
                    Some(i) if line.contains("` v") => {
                        format!("{}` v[REDACTED]", &line[..i])
                    }
                    _ => line.to_owned(),
                }
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
                description: None,
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
                        description: None,
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
                        constraints: vec![],
                        description: None,
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
        source_systems: vec![],
        quality_rules: vec![],
        governance: None,
        open_questions: vec![],
        assumptions: vec![],
        trace: ContractTrace::default(),
    }
}

#[test]
fn fixture_customer_csv_snapshot() {
    let gen = FixtureGenerator::new().with_row_count(5);
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let csv = artifacts
        .iter()
        .find(|a| a.path == "fixtures/customer.csv")
        .unwrap();
    insta::assert_snapshot!(csv.content);
}

#[test]
fn fixture_order_csv_snapshot() {
    let gen = FixtureGenerator::new().with_row_count(5);
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let csv = artifacts
        .iter()
        .find(|a| a.path == "fixtures/order.csv")
        .unwrap();
    insta::assert_snapshot!(csv.content);
}

#[test]
fn fixture_readme_snapshot() {
    let gen = FixtureGenerator::new().with_row_count(5);
    let artifacts = gen.generate(&demo_contract()).unwrap();
    let readme = artifacts
        .iter()
        .find(|a| a.path == "fixtures/README.md")
        .unwrap();
    insta::assert_snapshot!(redact_version(&readme.content));
}
