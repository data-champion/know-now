use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractProject, ContractRelationship, ContractTrace,
    GeneratorContract,
};
use know_now_gen_quality::QualityContractGenerator;

fn redact_version(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.contains("Generator: know_now_gen_quality. Version:") {
                let idx = line.find("Version:").unwrap();
                format!("{}Version: [REDACTED].", &line[..idx])
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn full_contract() -> GeneratorContract {
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
                        constraints: vec!["max_length:320".into()],
                        description: None,
                        attr_type: None,
                    },
                    ContractAttribute {
                        id: "attr_customer_country".into(),
                        name: "country".into(),
                        logical_type: Some("string".into()),
                        semantic_type: Some("country_code".into()),
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
                        semantic_type: Some("currency_amount".into()),
                        sensitivity: None,
                        pii: None,
                        required: Some(true),
                        is_unique: None,
                        constraints: vec!["minimum:0".into()],
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
fn quality_contract_customer_snapshot() {
    let gen = QualityContractGenerator::new();
    let artifacts = gen.generate(&full_contract()).unwrap();
    let qc = artifacts
        .iter()
        .find(|a| a.path == "quality_contracts/customer.yml")
        .unwrap();
    insta::assert_snapshot!(redact_version(&qc.content));
}

#[test]
fn quality_contract_order_snapshot() {
    let gen = QualityContractGenerator::new();
    let artifacts = gen.generate(&full_contract()).unwrap();
    let qc = artifacts
        .iter()
        .find(|a| a.path == "quality_contracts/order.yml")
        .unwrap();
    insta::assert_snapshot!(redact_version(&qc.content));
}
