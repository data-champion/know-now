use know_now_codegen::artifact::{ArtifactDescriptor, ArtifactKind};
use know_now_codegen::generator::{GenerationError, Generator};
use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractRelationship, GeneratorContract,
};

const GENERATOR_NAME: &str = "know_now_gen_quality";

pub struct QualityContractGenerator {
    version: String,
}

impl QualityContractGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

impl Default for QualityContractGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for QualityContractGenerator {
    fn name(&self) -> &str {
        GENERATOR_NAME
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn generate(
        &self,
        contract: &GeneratorContract,
    ) -> Result<Vec<ArtifactDescriptor>, Vec<GenerationError>> {
        let mut artifacts = Vec::new();

        for entity in &contract.entities {
            artifacts.push(emit_quality_contract(
                entity,
                &contract.relationships,
                &self.version,
            ));
        }

        artifacts.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(artifacts)
    }
}

fn emit_quality_contract(
    entity: &ContractEntity,
    relationships: &[ContractRelationship],
    version: &str,
) -> ArtifactDescriptor {
    let mut lines = vec![
        ownership_comment(&format!("art_qc_{}", entity.name), version),
        String::new(),
        "version: \"1.0\"".to_owned(),
        format!("entity: {}", entity.id),
        "checks:".to_owned(),
    ];

    for attr in &entity.attributes {
        emit_attribute_checks(attr, entity, relationships, &mut lines);
    }

    for rel in relationships
        .iter()
        .filter(|r| r.from_entity == entity.name)
    {
        emit_relationship_check(rel, &mut lines);
    }

    lines.push(String::new());

    ArtifactDescriptor {
        path: format!("quality_contracts/{}.yml", entity.name),
        kind: ArtifactKind::QualityContract,
        artifact_id: format!("art_qc_{}", entity.name),
        generator: GENERATOR_NAME.into(),
        generator_version: version.to_owned(),
        content: lines.join("\n"),
        metadata_object_ids: vec![entity.id.clone()],
    }
}

fn emit_attribute_checks(
    attr: &ContractAttribute,
    entity: &ContractEntity,
    relationships: &[ContractRelationship],
    lines: &mut Vec<String>,
) {
    let attr_slug = format!("{}_{}", entity.name, attr.name);

    if attr.required == Some(true) {
        lines.push(format!("  - id: chk_{attr_slug}_not_null"));
        lines.push("    target:".to_owned());
        lines.push("      kind: column".to_owned());
        lines.push(format!("      attribute_id: {}", attr.id));
        lines.push("    kind: not_null".to_owned());
        lines.push("    severity: error".to_owned());
        lines.push("    origin: { source: dc_standard_v1, rule: POL-REQ-001 }".to_owned());
    }

    if attr.is_unique == Some(true) {
        lines.push(format!("  - id: chk_{attr_slug}_unique"));
        lines.push("    target:".to_owned());
        lines.push("      kind: column".to_owned());
        lines.push(format!("      attribute_id: {}", attr.id));
        lines.push("    kind: unique".to_owned());
        lines.push("    severity: error".to_owned());
        lines.push("    origin: { source: dc_standard_v1, rule: POL-UNQ-001 }".to_owned());
    }

    if let Some(rel) = relationships.iter().find(|r| {
        r.from_entity == entity.name && r.from_key.as_deref() == Some(&attr.name)
    }) {
        lines.push(format!("  - id: chk_{attr_slug}_fk"));
        lines.push("    target:".to_owned());
        lines.push("      kind: column".to_owned());
        lines.push(format!("      attribute_id: {}", attr.id));
        lines.push("    kind: referential_integrity".to_owned());
        lines.push(format!(
            "    reference: {{ entity: {}, attribute: {} }}",
            rel.to_entity,
            rel.to_key.as_deref().unwrap_or("id")
        ));
        lines.push("    severity: error".to_owned());
        lines.push("    origin: { source: dc_standard_v1, rule: POL-FK-001 }".to_owned());
    }

    match attr.semantic_type.as_deref() {
        Some("email") => {
            lines.push(format!("  - id: chk_{attr_slug}_email_format"));
            lines.push("    target:".to_owned());
            lines.push("      kind: column".to_owned());
            lines.push(format!("      attribute_id: {}", attr.id));
            lines.push("    kind: regex_match".to_owned());
            lines.push(
                "    pattern: \"^[^\\\\s@]+@[^\\\\s@]+\\\\.[^\\\\s@]+$\"".to_owned(),
            );
            lines.push("    severity: warning".to_owned());
            lines.push("    origin: { source: dc_standard_v1, rule: POL-SEM-EMAIL }".to_owned());
        }
        Some("country_code") => {
            lines.push(format!("  - id: chk_{attr_slug}_country_code"));
            lines.push("    target:".to_owned());
            lines.push("      kind: column".to_owned());
            lines.push(format!("      attribute_id: {}", attr.id));
            lines.push("    kind: accepted_values".to_owned());
            lines.push("    values: [US, GB, DE, FR, NL, JP, AU, CA, BR, IN, SE, NO, DK, FI, ES, IT, CH, AT, BE, PT]".to_owned());
            lines.push("    severity: warning".to_owned());
            lines.push(
                "    origin: { source: dc_standard_v1, rule: POL-SEM-COUNTRY }".to_owned(),
            );
        }
        Some("postal_code") => {
            lines.push(format!("  - id: chk_{attr_slug}_max_length"));
            lines.push("    target:".to_owned());
            lines.push("      kind: column".to_owned());
            lines.push(format!("      attribute_id: {}", attr.id));
            lines.push("    kind: max_length".to_owned());
            lines.push("    max_value: 20".to_owned());
            lines.push("    severity: warning".to_owned());
            lines.push(
                "    origin: { source: dc_standard_v1, rule: POL-SEM-POSTAL }".to_owned(),
            );
        }
        Some("currency_amount") => {
            if parse_constraint_value(&attr.constraints, "minimum").is_some_and(|v| v >= 0) {
                lines.push(format!("  - id: chk_{attr_slug}_not_negative"));
                lines.push("    target:".to_owned());
                lines.push("      kind: column".to_owned());
                lines.push(format!("      attribute_id: {}", attr.id));
                lines.push("    kind: not_negative".to_owned());
                lines.push("    severity: error".to_owned());
                lines.push(
                    "    origin: { source: dc_standard_v1, rule: POL-NUM-001 }".to_owned(),
                );
            }
        }
        _ => {}
    }

    if let Some(max_len) = parse_constraint_value(&attr.constraints, "max_length") {
        lines.push(format!("  - id: chk_{attr_slug}_max_length"));
        lines.push("    target:".to_owned());
        lines.push("      kind: column".to_owned());
        lines.push(format!("      attribute_id: {}", attr.id));
        lines.push("    kind: max_length".to_owned());
        lines.push(format!("    max_value: {max_len}"));
        lines.push("    severity: warning".to_owned());
        lines.push("    origin: { source: dc_standard_v1, rule: POL-LEN-001 }".to_owned());
    }
}

fn emit_relationship_check(rel: &ContractRelationship, lines: &mut Vec<String>) {
    let slug = format!("{}_{}_fk", rel.from_entity, rel.from_key.as_deref().unwrap_or("id"));
    lines.push(format!("  - id: chk_{slug}_relationship"));
    lines.push("    target:".to_owned());
    lines.push("      kind: relationship".to_owned());
    lines.push(format!("      relationship_id: {}", rel.id));
    lines.push("    kind: referential_integrity".to_owned());
    lines.push(format!(
        "    from: {{ entity: {}, key: {} }}",
        rel.from_entity,
        rel.from_key.as_deref().unwrap_or("id")
    ));
    lines.push(format!(
        "    to: {{ entity: {}, key: {} }}",
        rel.to_entity,
        rel.to_key.as_deref().unwrap_or("id")
    ));
    lines.push("    severity: error".to_owned());
    lines.push("    origin: { source: dc_standard_v1, rule: POL-REL-001 }".to_owned());
}

fn ownership_comment(artifact_id: &str, version: &str) -> String {
    format!("# Generated by know-now. Artifact ID: {artifact_id}. Generator: {GENERATOR_NAME}. Version: {version}. Do not edit directly unless you intend to fork this artifact.")
}

fn parse_constraint_value(constraints: &[String], key: &str) -> Option<i64> {
    let prefix = format!("{key}:");
    constraints
        .iter()
        .find(|c| c.starts_with(&prefix))
        .and_then(|c| c[prefix.len()..].trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::{
        ContractAttribute, ContractEntity, ContractProject, ContractRelationship, ContractTrace,
        GeneratorContract,
    };

    fn customer_entity() -> ContractEntity {
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
            ],
        }
    }

    fn order_entity() -> ContractEntity {
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
        }
    }

    fn minimal_contract() -> GeneratorContract {
        GeneratorContract {
            contract_version: "1.0".into(),
            project: Some(ContractProject {
                name: "demo".into(),
                description: None,
                owner: None,
                tags: vec![],
            }),
            target_database: None,
            entities: vec![customer_entity()],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace::default(),
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
    fn empty_contract_produces_no_artifacts() {
        let gen = QualityContractGenerator::new();
        let contract = GeneratorContract {
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
        assert!(gen.generate(&contract).unwrap().is_empty());
    }

    #[test]
    fn produces_one_contract_per_entity() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert!(artifacts.iter().any(|a| a.path == "quality_contracts/customer.yml"));
        assert!(artifacts.iter().any(|a| a.path == "quality_contracts/order.yml"));
    }

    #[test]
    fn all_artifacts_are_quality_contract_kind() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        for a in &artifacts {
            assert_eq!(a.kind, ArtifactKind::QualityContract);
        }
    }

    #[test]
    fn contract_has_version_and_entity() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let qc = &artifacts[0];
        assert!(qc.content.contains("version: \"1.0\""));
        assert!(qc.content.contains("entity: ent_customer"));
    }

    #[test]
    fn required_attribute_generates_not_null_check() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let qc = &artifacts[0];
        assert!(qc.content.contains("chk_customer_id_not_null"));
        assert!(qc.content.contains("kind: not_null"));
        assert!(qc.content.contains("attribute_id: attr_customer_id"));
    }

    #[test]
    fn unique_attribute_generates_unique_check() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let qc = &artifacts[0];
        assert!(qc.content.contains("chk_customer_id_unique"));
        assert!(qc.content.contains("kind: unique"));
    }

    #[test]
    fn email_semantic_generates_regex_check() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let qc = &artifacts[0];
        assert!(qc.content.contains("chk_customer_email_email_format"));
        assert!(qc.content.contains("kind: regex_match"));
        assert!(qc.content.contains("severity: warning"));
    }

    #[test]
    fn max_length_constraint_generates_check() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let qc = &artifacts[0];
        assert!(qc.content.contains("chk_customer_email_max_length"));
        assert!(qc.content.contains("max_value: 320"));
    }

    #[test]
    fn fk_generates_referential_integrity_check() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let order_qc = artifacts.iter().find(|a| a.path.contains("order")).unwrap();
        assert!(order_qc.content.contains("chk_order_customer_id_fk"));
        assert!(order_qc.content.contains("kind: referential_integrity"));
        assert!(order_qc.content.contains("entity: customer"));
    }

    #[test]
    fn relationship_check_at_entity_level() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let order_qc = artifacts.iter().find(|a| a.path.contains("order")).unwrap();
        assert!(order_qc.content.contains("chk_order_customer_id_fk_relationship"));
        assert!(order_qc.content.contains("relationship_id: rel_order_customer"));
    }

    #[test]
    fn currency_amount_with_minimum_zero_generates_not_negative() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let order_qc = artifacts.iter().find(|a| a.path.contains("order")).unwrap();
        assert!(order_qc.content.contains("chk_order_total_not_negative"));
        assert!(order_qc.content.contains("kind: not_negative"));
    }

    #[test]
    fn all_checks_have_severity() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        for a in &artifacts {
            for line in a.content.lines() {
                if line.trim_start().starts_with("- id: chk_") {
                    break;
                }
            }
            let check_blocks: Vec<&str> = a
                .content
                .split("\n  - id: chk_")
                .skip(1)
                .collect();
            for block in check_blocks {
                assert!(
                    block.contains("severity:"),
                    "check missing severity in {}: {}",
                    a.path,
                    block.lines().next().unwrap_or("")
                );
            }
        }
    }

    #[test]
    fn all_checks_have_origin() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        for a in &artifacts {
            let check_blocks: Vec<&str> = a
                .content
                .split("\n  - id: chk_")
                .skip(1)
                .collect();
            for block in check_blocks {
                assert!(
                    block.contains("origin:"),
                    "check missing origin in {}: {}",
                    a.path,
                    block.lines().next().unwrap_or("")
                );
            }
        }
    }

    #[test]
    fn ownership_comments_present() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        for a in &artifacts {
            assert!(
                a.content.starts_with("# Generated by know-now."),
                "missing ownership comment in {}",
                a.path
            );
        }
    }

    #[test]
    fn deterministic_output() {
        let gen = QualityContractGenerator::new();
        let contract = multi_entity_contract();
        let a = gen.generate(&contract).unwrap();
        let b = gen.generate(&contract).unwrap();
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.content, y.content, "artifact {} differs", x.path);
        }
    }

    #[test]
    fn no_crlf_in_output() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        for a in &artifacts {
            assert!(
                !a.content.contains('\r'),
                "NFR-PO3: LF only in {}",
                a.path
            );
        }
    }

    #[test]
    fn artifacts_sorted_by_path() {
        let gen = QualityContractGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.path.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<QualityContractGenerator>();
    }
}
