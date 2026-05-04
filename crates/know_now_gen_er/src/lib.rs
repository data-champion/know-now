//! Mermaid ER diagram generator crate for know-now.
//!
//! Produces `.mmd` (standalone Mermaid) and `.md` (GitHub-renderable
//! Markdown wrapper) artifacts from the generator contract.

use std::collections::BTreeMap;

use know_now_codegen::artifact::{ArtifactDescriptor, ArtifactKind};
use know_now_codegen::generator::{GenerationError, Generator};
use know_now_contract::contract::{ContractEntity, ContractRelationship, GeneratorContract};

const GENERATOR_NAME: &str = "know_now_gen_er";

pub struct ErDiagramGenerator {
    version: String,
}

impl ErDiagramGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

impl Default for ErDiagramGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for ErDiagramGenerator {
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
        if contract.entities.is_empty() {
            return Ok(vec![]);
        }

        let mut artifacts = Vec::new();

        let all_mmd = render_er_diagram(&contract.entities, &contract.relationships);
        let input_hash = compute_contract_hash(contract);

        artifacts.push(ArtifactDescriptor {
            path: "diagrams/er/all.mmd".into(),
            kind: ArtifactKind::MermaidDiagram,
            artifact_id: "art_er_all_mmd".into(),
            generator: GENERATOR_NAME.into(),
            generator_version: self.version.clone(),
            content: all_mmd.clone(),
            metadata_object_ids: all_metadata_ids(contract),
        });

        let md_content = render_markdown_wrapper(
            &all_mmd,
            &input_hash,
            &self.version,
            contract.project.as_ref().map(|p| p.name.as_str()),
        );
        artifacts.push(ArtifactDescriptor {
            path: "diagrams/er/all.md".into(),
            kind: ArtifactKind::MermaidDiagram,
            artifact_id: "art_er_all_md".into(),
            generator: GENERATOR_NAME.into(),
            generator_version: self.version.clone(),
            content: md_content,
            metadata_object_ids: all_metadata_ids(contract),
        });

        let domains = group_by_domain(&contract.entities);
        if domains.len() > 1 {
            for (domain, domain_entities) in &domains {
                let domain_entity_names: Vec<&str> =
                    domain_entities.iter().map(|e| e.name.as_str()).collect();
                let domain_rels: Vec<&ContractRelationship> = contract
                    .relationships
                    .iter()
                    .filter(|r| {
                        domain_entity_names.contains(&r.from_entity.as_str())
                            || domain_entity_names.contains(&r.to_entity.as_str())
                    })
                    .collect();

                let mmd_for_domain =
                    render_er_diagram(domain_entities, &refs_to_owned(&domain_rels));

                let safe_domain = sanitize_domain_name(domain);

                artifacts.push(ArtifactDescriptor {
                    path: format!("diagrams/er/{safe_domain}.mmd"),
                    kind: ArtifactKind::MermaidDiagram,
                    artifact_id: format!("art_er_{safe_domain}_mmd"),
                    generator: GENERATOR_NAME.into(),
                    generator_version: self.version.clone(),
                    content: mmd_for_domain.clone(),
                    metadata_object_ids: domain_entities.iter().map(|e| e.id.clone()).collect(),
                });

                let wrapper = render_markdown_wrapper(
                    &mmd_for_domain,
                    &input_hash,
                    &self.version,
                    Some(domain),
                );
                artifacts.push(ArtifactDescriptor {
                    path: format!("diagrams/er/{safe_domain}.md"),
                    kind: ArtifactKind::MermaidDiagram,
                    artifact_id: format!("art_er_{safe_domain}_md"),
                    generator: GENERATOR_NAME.into(),
                    generator_version: self.version.clone(),
                    content: wrapper,
                    metadata_object_ids: domain_entities.iter().map(|e| e.id.clone()).collect(),
                });
            }
        }

        artifacts.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(artifacts)
    }
}

fn render_er_diagram(
    entities: &[ContractEntity],
    relationships: &[ContractRelationship],
) -> String {
    let mut lines = Vec::new();
    lines.push("erDiagram".to_owned());

    let mut sorted_entities: Vec<&ContractEntity> = entities.iter().collect();
    sorted_entities.sort_by_key(|e| &e.name);

    for entity in &sorted_entities {
        lines.push(format!("    {} {{", entity.name));
        for attr in &entity.attributes {
            let logical_type = attr.logical_type.as_deref().unwrap_or("string");
            let mut markers = Vec::new();
            if entity.business_key.contains(&attr.name)
                || attr.is_unique == Some(true) && attr.required == Some(true)
            {
                markers.push("PK");
            }
            if is_fk_attribute(&attr.name, relationships, &entity.name) {
                markers.push("FK");
            }
            let marker_str = if markers.is_empty() {
                String::new()
            } else {
                format!(" {}", markers.join(","))
            };
            let comment = attr
                .description
                .as_ref()
                .map_or(String::new(), |d| format!(" \"{}\"", truncate_comment(d)));
            lines.push(format!(
                "        {logical_type} {name}{marker_str}{comment}",
                name = attr.name,
            ));
        }
        lines.push("    }".to_owned());
    }

    let mut sorted_rels: Vec<&ContractRelationship> = relationships.iter().collect();
    sorted_rels.sort_by(|a, b| (&a.from_entity, &a.to_entity).cmp(&(&b.from_entity, &b.to_entity)));

    for rel in &sorted_rels {
        let arrow = cardinality_arrow(rel.cardinality.as_deref());
        let label = rel
            .description
            .as_deref()
            .unwrap_or_else(|| rel_default_label(rel));
        lines.push(format!(
            "    {} {} {} : \"{}\"",
            rel.from_entity, arrow, rel.to_entity, label
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn rel_default_label(rel: &ContractRelationship) -> &str {
    if rel.from_entity == rel.to_entity {
        "self-references"
    } else {
        "relates to"
    }
}

fn is_fk_attribute(
    attr_name: &str,
    relationships: &[ContractRelationship],
    entity_name: &str,
) -> bool {
    relationships
        .iter()
        .any(|r| r.from_entity == entity_name && r.from_key.as_deref() == Some(attr_name))
}

#[allow(clippy::match_same_arms)]
fn cardinality_arrow(cardinality: Option<&str>) -> &'static str {
    match cardinality {
        Some("one_to_one") => "||--||",
        Some("one_to_many") => "||--o{",
        Some("many_to_one") => "}o--||",
        Some("many_to_many") => "}o--o{",
        _ => "||--o{",
    }
}

fn truncate_comment(s: &str) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() > 60 {
        format!("{}...", &first_line[..57])
    } else {
        first_line.to_owned()
    }
}

fn render_markdown_wrapper(
    mmd_content: &str,
    input_hash: &str,
    generator_version: &str,
    scope_name: Option<&str>,
) -> String {
    let title = scope_name.map_or_else(|| "ER Diagram".to_owned(), |n| format!("{n} — ER Diagram"));
    let mut lines = Vec::new();

    lines.push(format!(
        "<!-- Generated by know-now. Generator: {GENERATOR_NAME}. Version: {generator_version}. Input hash: {input_hash}. Do not edit directly unless you intend to fork this artifact. -->"
    ));
    lines.push(String::new());
    lines.push(format!("# {title}"));
    lines.push(String::new());
    lines.push("```mermaid".to_owned());
    lines.push(mmd_content.trim_end().to_owned());
    lines.push("```".to_owned());
    lines.push(String::new());

    lines.join("\n")
}

fn group_by_domain(entities: &[ContractEntity]) -> BTreeMap<String, Vec<ContractEntity>> {
    let mut domains: BTreeMap<String, Vec<ContractEntity>> = BTreeMap::new();
    for entity in entities {
        if let Some(ref domain) = entity.domain {
            domains
                .entry(domain.clone())
                .or_default()
                .push(entity.clone());
        }
    }
    domains
}

fn refs_to_owned(rels: &[&ContractRelationship]) -> Vec<ContractRelationship> {
    rels.iter().map(|&r| r.clone()).collect()
}

fn sanitize_domain_name(domain: &str) -> String {
    domain
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

fn all_metadata_ids(contract: &GeneratorContract) -> Vec<String> {
    let mut ids: Vec<String> = contract
        .entities
        .iter()
        .flat_map(|e| {
            let mut entity_ids = vec![e.id.clone()];
            entity_ids.extend(e.attributes.iter().map(|a| a.id.clone()));
            entity_ids
        })
        .collect();
    ids.extend(contract.relationships.iter().map(|r| r.id.clone()));
    ids
}

fn compute_contract_hash(contract: &GeneratorContract) -> String {
    let json = serde_json::to_string(contract).unwrap_or_default();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in json.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("fnv1a:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::{
        ContractAttribute, ContractEntity, ContractProject, ContractRelationship, ContractTrace,
        GeneratorContract,
    };

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
                    required: None,
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
    fn empty_contract_produces_no_artifacts() {
        let gen = ErDiagramGenerator::new();
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
        let artifacts = gen.generate(&contract).unwrap();
        assert!(artifacts.is_empty());
    }

    #[test]
    fn single_entity_produces_mmd_and_md() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        assert!(artifacts.iter().any(|a| a.path == "diagrams/er/all.mmd"));
        assert!(artifacts.iter().any(|a| a.path == "diagrams/er/all.md"));
    }

    #[test]
    fn mmd_contains_entity_block() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let mmd = artifacts.iter().find(|a| a.path.ends_with(".mmd")).unwrap();
        assert!(mmd.content.contains("erDiagram"));
        assert!(mmd.content.contains("customer {"));
        assert!(mmd.content.contains("integer id PK"));
        assert!(mmd.content.contains("string email"));
    }

    #[test]
    fn mmd_shows_relationships() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap()
            .content;
        assert!(mmd.contains("order }o--|| customer"));
        assert!(mmd.contains("\"places\""));
    }

    #[test]
    fn mmd_marks_fk_attributes() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap()
            .content;
        assert!(mmd.contains("integer customer_id FK"));
    }

    #[test]
    fn md_wraps_mmd_in_fenced_block() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let md = artifacts.iter().find(|a| a.path.ends_with(".md")).unwrap();
        assert!(md.content.contains("```mermaid"));
        assert!(md.content.contains("erDiagram"));
        assert!(md.content.contains("```\n"));
    }

    #[test]
    fn md_has_ownership_comment() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let md = artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.md")
            .unwrap();
        assert!(md.content.starts_with("<!-- Generated by know-now."));
        assert!(md.content.contains("know_now_gen_er"));
    }

    #[test]
    fn deterministic_output() {
        let gen = ErDiagramGenerator::new();
        let contract = multi_entity_contract();
        let a = gen.generate(&contract).unwrap();
        let b = gen.generate(&contract).unwrap();
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.content, y.content, "artifact {} differs", x.path);
        }
    }

    #[test]
    fn no_crlf_in_output() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        for artifact in &artifacts {
            assert!(
                !artifact.content.contains('\r'),
                "NFR-PO3: LF only in {}",
                artifact.path
            );
        }
    }

    #[test]
    fn entities_sorted_alphabetically() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap()
            .content;
        let customer_pos = mmd.find("customer {").unwrap();
        let order_pos = mmd.find("order {").unwrap();
        assert!(customer_pos < order_pos, "entities must be sorted");
    }

    #[test]
    fn all_cardinality_types() {
        assert_eq!(cardinality_arrow(Some("one_to_one")), "||--||");
        assert_eq!(cardinality_arrow(Some("one_to_many")), "||--o{");
        assert_eq!(cardinality_arrow(Some("many_to_one")), "}o--||");
        assert_eq!(cardinality_arrow(Some("many_to_many")), "}o--o{");
        assert_eq!(cardinality_arrow(None), "||--o{");
    }

    #[test]
    fn self_referencing_relationship() {
        let gen = ErDiagramGenerator::new();
        let mut contract = minimal_contract();
        contract.relationships.push(ContractRelationship {
            id: "rel_self".into(),
            from_entity: "customer".into(),
            to_entity: "customer".into(),
            cardinality: Some("one_to_many".into()),
            from_key: Some("parent_id".into()),
            to_key: Some("id".into()),
            description: None,
        });
        let artifacts = gen.generate(&contract).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap()
            .content;
        assert!(mmd.contains("customer ||--o{ customer"));
        assert!(mmd.contains("\"self-references\""));
    }

    #[test]
    fn many_to_many_relationship() {
        let gen = ErDiagramGenerator::new();
        let mut contract = multi_entity_contract();
        contract.relationships.push(ContractRelationship {
            id: "rel_m2m".into(),
            from_entity: "customer".into(),
            to_entity: "order".into(),
            cardinality: Some("many_to_many".into()),
            from_key: None,
            to_key: None,
            description: Some("shared".into()),
        });
        let artifacts = gen.generate(&contract).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap()
            .content;
        assert!(mmd.contains("}o--o{"));
    }

    #[test]
    fn multi_domain_produces_per_domain_diagrams() {
        let gen = ErDiagramGenerator::new();
        let mut contract = multi_entity_contract();
        contract.entities[1].domain = Some("logistics".into());
        let artifacts = gen.generate(&contract).unwrap();
        assert!(artifacts.iter().any(|a| a.path == "diagrams/er/sales.mmd"));
        assert!(artifacts
            .iter()
            .any(|a| a.path == "diagrams/er/logistics.mmd"));
        assert!(artifacts.iter().any(|a| a.path == "diagrams/er/sales.md"));
        assert!(artifacts
            .iter()
            .any(|a| a.path == "diagrams/er/logistics.md"));
    }

    #[test]
    fn single_domain_no_per_domain_diagrams() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(paths.len(), 2, "only all.mmd and all.md");
    }

    #[test]
    fn metadata_ids_include_entities_and_relationships() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&multi_entity_contract()).unwrap();
        let mmd = artifacts
            .iter()
            .find(|a| a.path == "diagrams/er/all.mmd")
            .unwrap();
        assert!(mmd.metadata_object_ids.contains(&"ent_customer".into()));
        assert!(mmd.metadata_object_ids.contains(&"ent_order".into()));
        assert!(mmd
            .metadata_object_ids
            .contains(&"rel_order_customer".into()));
    }

    #[test]
    fn artifact_kind_is_mermaid_diagram() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        for a in &artifacts {
            assert_eq!(a.kind, ArtifactKind::MermaidDiagram);
        }
    }

    #[test]
    fn generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ErDiagramGenerator>();
    }

    #[test]
    fn generator_name_and_version() {
        let gen = ErDiagramGenerator::new();
        assert_eq!(gen.name(), GENERATOR_NAME);
        assert!(!gen.version().is_empty());
    }

    #[test]
    fn pk_description_appears_as_comment() {
        let gen = ErDiagramGenerator::new();
        let artifacts = gen.generate(&minimal_contract()).unwrap();
        let mmd = &artifacts
            .iter()
            .find(|a| a.path.ends_with(".mmd"))
            .unwrap()
            .content;
        assert!(mmd.contains("\"Primary key.\""));
    }

    #[test]
    fn long_description_truncated() {
        let truncated = truncate_comment(
            "This is a very long description that exceeds sixty characters and should be truncated by the generator",
        );
        assert!(truncated.len() <= 63);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn sanitize_domain_name_special_chars() {
        assert_eq!(
            sanitize_domain_name("Sales & Marketing"),
            "sales___marketing"
        );
        assert_eq!(sanitize_domain_name("data-ops"), "data-ops");
        assert_eq!(sanitize_domain_name("My Domain"), "my_domain");
    }

    #[test]
    fn artifacts_sorted_by_path() {
        let gen = ErDiagramGenerator::new();
        let mut contract = multi_entity_contract();
        contract.entities[1].domain = Some("logistics".into());
        let artifacts = gen.generate(&contract).unwrap();
        let paths: Vec<&str> = artifacts.iter().map(|a| a.path.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }
}
