use know_now_codegen::generator::Generator;
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractProject, ContractRelationship, ContractTrace,
    GeneratorContract,
};
use know_now_gen_er::ErDiagramGenerator;

fn customer_entity() -> ContractEntity {
    ContractEntity {
        id: "ent_customer".into(),
        name: "customer".into(),
        display_name: None,
        domain: Some("sales".into()),
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
                id: "attr_customer_id".into(),
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
                id: "attr_customer_email".into(),
                name: "email".into(),
                logical_type: Some("string".into()),
                semantic_type: Some("email".into()),
                sensitivity: None,
                pii: Some(true),
                required: Some(true),
                is_unique: None,
                constraints: vec![],
                description: Some("Contact email.".into()),
                attr_type: None,
            },
            ContractAttribute {
                id: "attr_customer_name".into(),
                name: "name".into(),
                logical_type: Some("string".into()),
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
    }
}

fn order_entity() -> ContractEntity {
    ContractEntity {
        id: "ent_order".into(),
        name: "order".into(),
        display_name: None,
        domain: Some("sales".into()),
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
    }
}

fn multi_entity_contract() -> GeneratorContract {
    GeneratorContract {
        contract_version: "1.0".into(),
        project: Some(ContractProject {
            name: "demo".into(),
            description: None,
            owner: None,
            tags: vec![],
        }),
        target_database: None,
        entities: vec![customer_entity(), order_entity()],
        relationships: vec![ContractRelationship {
            id: "rel_order_customer".into(),
            from_entity: "order".into(),
            to_entity: "customer".into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("customer_id".into()),
            to_key: Some("id".into()),
            description: Some("places".into()),
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
fn er_mmd_snapshot() {
    let gen = ErDiagramGenerator::new();
    let artifacts = gen.generate(&multi_entity_contract()).unwrap();
    let mmd = artifacts
        .iter()
        .find(|a| a.path == "diagrams/er/all.mmd")
        .unwrap();
    insta::assert_snapshot!("er_mmd", mmd.content);
}

#[test]
fn er_md_snapshot() {
    let gen = ErDiagramGenerator::new();
    let artifacts = gen.generate(&multi_entity_contract()).unwrap();
    let md = artifacts
        .iter()
        .find(|a| a.path == "diagrams/er/all.md")
        .unwrap();
    let stable_content = md
        .content
        .lines()
        .map(|line| {
            if line.starts_with("<!-- Generated") {
                "<!-- Generated by know-now. [redacted for snapshot stability] -->"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("er_md", stable_content);
}

#[test]
fn er_single_entity_snapshot() {
    let gen = ErDiagramGenerator::new();
    let contract = GeneratorContract {
        contract_version: "1.0".into(),
        project: None,
        target_database: None,
        entities: vec![customer_entity()],
        relationships: vec![],
        source_systems: vec![],
        quality_rules: vec![],
        governance: None,
        open_questions: vec![],
        assumptions: vec![],
        trace: ContractTrace::default(),
    };
    let artifacts = gen.generate(&contract).unwrap();
    let mmd = artifacts.iter().find(|a| a.path.ends_with(".mmd")).unwrap();
    insta::assert_snapshot!("er_single_entity", mmd.content);
}
