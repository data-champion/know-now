use std::collections::{BTreeMap, BTreeSet};

use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_metadata::authoring::AuthoringMetadata;

use crate::graph::{
    AssumptionNode, AttributeNode, DomainNode, EntityNode, GovernanceNode, ModuleNode, NodeId,
    OpenQuestionNode, ProjectGraph, ProjectGraphParts, ProjectNode, QualityRuleNode,
    RelationshipNode, SourceColumnNode, SourceSystemNode, SourceTableNode,
};

/// Build result: graph (when error-free) plus all diagnostics.
#[derive(Debug)]
pub struct BuildResult {
    pub graph: Option<ProjectGraph>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Build a validated `ProjectGraph` from authoring metadata.
///
/// # Errors
///
/// Returns diagnostics in `BuildResult.diagnostics`.
#[must_use]
pub fn build_project_graph(metadata: &AuthoringMetadata) -> BuildResult {
    let mut ctx = BuildContext::new(metadata);

    let domains = build_domains(metadata);
    let modules = build_modules(metadata);
    let entities = build_entities(metadata, &mut ctx);
    check_duplicate_entity_names(&ctx.entity_names, &mut ctx.diagnostics);
    let entity_names = ctx.entity_name_set.clone();
    let relationships = build_relationships(metadata, &entity_names, &mut ctx);
    let sources = build_sources(metadata, &entity_names, &mut ctx.diagnostics);
    let rules = build_rules(metadata, &entity_names, &mut ctx);
    let governance = build_governance(metadata);
    let open_questions = build_open_questions(metadata, &entity_names, &mut ctx);
    let assumptions = build_assumptions(metadata, &entity_names, &mut ctx);

    let has_errors = ctx.diagnostics.iter().any(Diagnostic::is_error);
    let graph = if has_errors {
        None
    } else {
        Some(ProjectGraph::from_parts(ProjectGraphParts {
            project: metadata.project.as_ref().map(|p| ProjectNode {
                name: Some(p.name.clone()),
                description: p.description.clone(),
                owner: p.owner.clone(),
                tags: p.tags.clone(),
            }),
            domains,
            modules,
            entities,
            relationships,
            sources,
            rules,
            governance,
            open_questions,
            assumptions,
        }))
    };

    BuildResult {
        graph,
        diagnostics: ctx.diagnostics,
    }
}

struct BuildContext {
    diagnostics: Vec<Diagnostic>,
    all_ids: BTreeSet<String>,
    domain_ids: BTreeSet<String>,
    module_ids: BTreeSet<String>,
    entity_name_set: BTreeSet<String>,
    entity_names: BTreeMap<String, Vec<String>>,
}

impl BuildContext {
    fn new(metadata: &AuthoringMetadata) -> Self {
        Self {
            diagnostics: Vec::new(),
            all_ids: BTreeSet::new(),
            domain_ids: metadata.domains.iter().map(|d| d.id.clone()).collect(),
            module_ids: metadata.modules.iter().map(|m| m.id.clone()).collect(),
            entity_name_set: metadata.entities.iter().map(|e| e.name.clone()).collect(),
            entity_names: BTreeMap::new(),
        }
    }
}

fn check_duplicate_id(
    id: &str,
    all_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    context_name: &str,
) {
    if id.starts_with("__auto_") {
        return;
    }
    if !all_ids.insert(id.to_owned()) {
        diagnostics.push(
            Diagnostic::new(
                Severity::Error,
                "META-ID-001",
                format!("duplicate object ID '{id}' (on object '{context_name}')"),
            )
            .with_metadata_object_id(id),
        );
    }
}

fn build_domains(metadata: &AuthoringMetadata) -> Vec<DomainNode> {
    metadata
        .domains
        .iter()
        .map(|d| DomainNode {
            id: d.id.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
            owner: d.owner.clone(),
        })
        .collect()
}

fn build_modules(metadata: &AuthoringMetadata) -> Vec<ModuleNode> {
    metadata
        .modules
        .iter()
        .map(|m| ModuleNode {
            id: m.id.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
        })
        .collect()
}

fn build_entities(metadata: &AuthoringMetadata, ctx: &mut BuildContext) -> Vec<EntityNode> {
    let mut entities = Vec::new();

    for (ent_idx, ent) in metadata.entities.iter().enumerate() {
        let ent_id = ent
            .id
            .clone()
            .unwrap_or_else(|| format!("__auto_ent_{ent_idx}"));

        check_duplicate_id(&ent_id, &mut ctx.all_ids, &mut ctx.diagnostics, &ent.name);

        let domain_key = ent.domain.clone().unwrap_or_default();
        ctx.entity_names
            .entry(domain_key)
            .or_default()
            .push(ent.name.clone());

        validate_entity_refs(
            ent_idx,
            ent,
            &ctx.domain_ids,
            &ctx.module_ids,
            &mut ctx.diagnostics,
        );

        let attr_nodes = build_attributes(ent_idx, ent, ctx);

        entities.push(EntityNode {
            id: NodeId(ent_id),
            name: ent.name.clone(),
            display_name: ent.display_name.clone(),
            domain: ent.domain.clone(),
            module: ent.module.clone(),
            owner: ent.owner.clone(),
            steward: ent.steward.clone(),
            classification: ent.classification.clone(),
            retention_policy: ent.retention_policy.clone(),
            description: ent.description.clone(),
            entity_type: ent.entity_type.clone(),
            tags: ent.tags.clone(),
            business_key: ent.business_key.clone(),
            attributes: attr_nodes,
        });
    }

    entities
}

fn validate_entity_refs(
    ent_idx: usize,
    ent: &know_now_metadata::authoring::Entity,
    domain_ids: &BTreeSet<String>,
    module_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(ref dom) = ent.domain {
        if !domain_ids.contains(dom) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "META-DOM-001",
                    format!("entity '{}' references unknown domain '{dom}'", ent.name),
                )
                .with_yaml_path(format!("entities[{ent_idx}].domain")),
            );
        }
    }

    if let Some(ref modref) = ent.module {
        if !module_ids.contains(modref) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "META-MOD-001",
                    format!("entity '{}' references unknown module '{modref}'", ent.name),
                )
                .with_yaml_path(format!("entities[{ent_idx}].module")),
            );
        }
    }

    let attr_names: BTreeSet<String> = ent.attributes.iter().map(|a| a.name.clone()).collect();
    for bk in &ent.business_key {
        if !attr_names.contains(bk) {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "META-ENT-002",
                    format!(
                        "entity '{}' business_key '{bk}' is not an attribute",
                        ent.name
                    ),
                )
                .with_yaml_path(format!("entities[{ent_idx}].business_key")),
            );
        }
    }
}

fn build_attributes(
    ent_idx: usize,
    ent: &know_now_metadata::authoring::Entity,
    ctx: &mut BuildContext,
) -> Vec<AttributeNode> {
    let mut attr_nodes = Vec::new();
    for (attr_idx, attr) in ent.attributes.iter().enumerate() {
        let attr_id = attr
            .id
            .clone()
            .unwrap_or_else(|| format!("__auto_attr_{ent_idx}_{attr_idx}"));

        check_duplicate_id(&attr_id, &mut ctx.all_ids, &mut ctx.diagnostics, &attr.name);

        let logical_type_str = attr.logical_type.as_ref().map(|lt| {
            let s = serde_json::to_string(lt).unwrap_or_default();
            s.trim_matches('"').to_owned()
        });

        let semantic_type_str = attr.semantic_type.as_ref().map(|st| {
            let s = serde_json::to_string(st).unwrap_or_default();
            s.trim_matches('"').to_owned()
        });

        attr_nodes.push(AttributeNode {
            id: NodeId(attr_id),
            name: attr.name.clone(),
            logical_type: logical_type_str,
            semantic_type: semantic_type_str,
            sensitivity: attr.sensitivity.clone(),
            pii: attr.pii,
            required: attr.required,
            is_unique: attr.is_unique,
            constraints: attr.constraints.clone(),
            description: attr.description.clone(),
            attr_type: attr.attr_type.clone(),
        });
    }
    attr_nodes
}

fn check_duplicate_entity_names(
    entity_names: &BTreeMap<String, Vec<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (domain, names) in entity_names {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for name in names {
            if !seen.insert(name.as_str()) {
                let scope = if domain.is_empty() {
                    "global scope".to_owned()
                } else {
                    format!("domain '{domain}'")
                };
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-ENT-001",
                        format!("duplicate entity name '{name}' within {scope}"),
                    )
                    .with_yaml_path("entities"),
                );
            }
        }
    }
}

fn build_relationships(
    metadata: &AuthoringMetadata,
    entity_name_set: &BTreeSet<String>,
    ctx: &mut BuildContext,
) -> Vec<RelationshipNode> {
    let mut relationships = Vec::new();
    for (rel_idx, rel) in metadata.relationships.iter().enumerate() {
        let rel_id = rel
            .id
            .clone()
            .unwrap_or_else(|| format!("__auto_rel_{rel_idx}"));

        check_duplicate_id(
            &rel_id,
            &mut ctx.all_ids,
            &mut ctx.diagnostics,
            &rel.from_entity,
        );
        validate_relationship_refs(
            rel_idx,
            rel,
            entity_name_set,
            metadata,
            &mut ctx.diagnostics,
        );

        relationships.push(RelationshipNode {
            id: NodeId(rel_id),
            from_entity: rel.from_entity.clone(),
            to_entity: rel.to_entity.clone(),
            cardinality: rel.cardinality.clone(),
            from_key: rel.from_key.clone(),
            to_key: rel.to_key.clone(),
            description: rel.description.clone(),
        });
    }
    relationships
}

fn validate_relationship_refs(
    rel_idx: usize,
    rel: &know_now_metadata::authoring::Relationship,
    entity_name_set: &BTreeSet<String>,
    metadata: &AuthoringMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !entity_name_set.contains(&rel.from_entity) {
        diagnostics.push(
            Diagnostic::new(
                Severity::Error,
                "META-REL-001",
                format!(
                    "relationship references unknown from_entity '{}'",
                    rel.from_entity
                ),
            )
            .with_yaml_path(format!("relationships[{rel_idx}].from_entity")),
        );
    }

    if !entity_name_set.contains(&rel.to_entity) {
        diagnostics.push(
            Diagnostic::new(
                Severity::Error,
                "META-REL-001",
                format!(
                    "relationship references unknown to_entity '{}'",
                    rel.to_entity
                ),
            )
            .with_yaml_path(format!("relationships[{rel_idx}].to_entity")),
        );
    }

    if let Some(ref from_key) = rel.from_key {
        if let Some(ent) = metadata.entities.iter().find(|e| e.name == rel.from_entity) {
            if !ent.attributes.iter().any(|a| &a.name == from_key) {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-REL-002",
                        format!(
                            "relationship from_key '{from_key}' not found in entity '{}'",
                            rel.from_entity
                        ),
                    )
                    .with_yaml_path(format!("relationships[{rel_idx}].from_key")),
                );
            }
        }
    }

    if let Some(ref to_key) = rel.to_key {
        if let Some(ent) = metadata.entities.iter().find(|e| e.name == rel.to_entity) {
            if !ent.attributes.iter().any(|a| &a.name == to_key) {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-REL-002",
                        format!(
                            "relationship to_key '{to_key}' not found in entity '{}'",
                            rel.to_entity
                        ),
                    )
                    .with_yaml_path(format!("relationships[{rel_idx}].to_key")),
                );
            }
        }
    }
}

fn build_sources(
    metadata: &AuthoringMetadata,
    entity_name_set: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<SourceSystemNode> {
    let sources: Vec<SourceSystemNode> = metadata
        .sources
        .iter()
        .map(|s| SourceSystemNode {
            name: s.name.clone(),
            kind: s.kind.clone(),
            description: s.description.clone(),
            entities: s.entities.clone(),
            tables: s
                .tables
                .iter()
                .map(|t| SourceTableNode {
                    name: t.name.clone(),
                    entity: t.entity.clone(),
                    schema: t.schema.clone(),
                    columns: t
                        .columns
                        .iter()
                        .map(|c| SourceColumnNode {
                            source: c.source.clone(),
                            target: c.target.clone(),
                            transform: c.transform.clone(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();

    for (src_idx, src) in metadata.sources.iter().enumerate() {
        for (tbl_idx, tbl) in src.tables.iter().enumerate() {
            if let Some(ref tent) = tbl.entity {
                if !entity_name_set.contains(tent) {
                    diagnostics.push(
                        Diagnostic::new(
                            Severity::Error,
                            "META-SRC-001",
                            format!(
                                "source '{}' table '{}' maps to unknown entity '{tent}'",
                                src.name, tbl.name
                            ),
                        )
                        .with_yaml_path(format!("sources[{src_idx}].tables[{tbl_idx}].entity")),
                    );
                }
            }
        }
    }

    sources
}

fn build_rules(
    metadata: &AuthoringMetadata,
    entity_name_set: &BTreeSet<String>,
    ctx: &mut BuildContext,
) -> Vec<QualityRuleNode> {
    let mut rules = Vec::new();
    for (rule_idx, rule) in metadata.rules.iter().enumerate() {
        let rule_id = rule
            .id
            .clone()
            .unwrap_or_else(|| format!("__auto_rule_{rule_idx}"));

        check_duplicate_id(&rule_id, &mut ctx.all_ids, &mut ctx.diagnostics, &rule.name);

        if let Some(ref ent_name) = rule.entity {
            if !entity_name_set.contains(ent_name) {
                ctx.diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-SRC-001",
                        format!(
                            "quality rule '{}' references unknown entity '{ent_name}'",
                            rule.name
                        ),
                    )
                    .with_yaml_path(format!("rules[{rule_idx}].entity")),
                );
            }
        }

        rules.push(QualityRuleNode {
            id: NodeId(rule_id),
            name: rule.name.clone(),
            entity: rule.entity.clone(),
            attribute: rule.attribute.clone(),
            rule_type: rule.rule_type.clone(),
            expression: rule.expression.clone(),
            severity: rule.severity.clone(),
            description: rule.description.clone(),
        });
    }
    rules
}

fn build_governance(metadata: &AuthoringMetadata) -> Option<GovernanceNode> {
    metadata.governance.as_ref().map(|g| GovernanceNode {
        data_owner: g.data_owner.clone(),
        data_steward: g.data_steward.clone(),
        classification_default: g.classification_default.clone(),
        retention_default: g.retention_default.clone(),
    })
}

fn build_open_questions(
    metadata: &AuthoringMetadata,
    entity_name_set: &BTreeSet<String>,
    ctx: &mut BuildContext,
) -> Vec<OpenQuestionNode> {
    let mut open_questions = Vec::new();
    for (q_idx, q) in metadata.open_questions.iter().enumerate() {
        let q_id = q.id.clone().unwrap_or_else(|| format!("__auto_q_{q_idx}"));

        check_duplicate_id(&q_id, &mut ctx.all_ids, &mut ctx.diagnostics, &q.question);

        if let Some(ref ent_name) = q.entity {
            if !entity_name_set.contains(ent_name) {
                ctx.diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-Q-001",
                        format!("open question '{q_id}' references unknown entity '{ent_name}'"),
                    )
                    .with_yaml_path(format!("open_questions[{q_idx}].entity")),
                );
            }
        }

        open_questions.push(OpenQuestionNode {
            id: NodeId(q_id),
            question: q.question.clone(),
            context: q.context.clone(),
            entity: q.entity.clone(),
            priority: q.priority.clone(),
        });
    }
    open_questions
}

fn build_assumptions(
    metadata: &AuthoringMetadata,
    entity_name_set: &BTreeSet<String>,
    ctx: &mut BuildContext,
) -> Vec<AssumptionNode> {
    let mut assumptions = Vec::new();
    for (a_idx, a) in metadata.assumptions.iter().enumerate() {
        let a_id =
            a.id.clone()
                .unwrap_or_else(|| format!("__auto_asm_{a_idx}"));

        check_duplicate_id(&a_id, &mut ctx.all_ids, &mut ctx.diagnostics, &a.statement);

        if let Some(ref ent_name) = a.entity {
            if !entity_name_set.contains(ent_name) {
                ctx.diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "META-ASM-001",
                        format!("assumption '{a_id}' references unknown entity '{ent_name}'"),
                    )
                    .with_yaml_path(format!("assumptions[{a_idx}].entity")),
                );
            }
        }

        assumptions.push(AssumptionNode {
            id: NodeId(a_id),
            statement: a.statement.clone(),
            rationale: a.rationale.clone(),
            entity: a.entity.clone(),
            risk: a.risk.clone(),
        });
    }
    assumptions
}

#[cfg(test)]
mod tests {
    use know_now_metadata::test_support::parse_yaml_metadata;

    use super::*;

    fn minimal_metadata() -> AuthoringMetadata {
        parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
      - name: email
        logical_type: string
        semantic_type: email
",
        )
    }

    fn full_metadata() -> AuthoringMetadata {
        parse_yaml_metadata(
            r#"
version: "1.0"
project:
  name: test_project
  description: Test
  owner: team
  tags: [demo]
domains:
  - id: sales
    name: Sales
modules:
  - id: core
    name: Core
entities:
  - id: ent_customer
    name: customer
    domain: sales
    module: core
    business_key: [email]
    attributes:
      - id: attr_customer_id
        name: id
        logical_type: integer
        required: true
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
  - id: ent_order
    name: order
    domain: sales
    attributes:
      - id: attr_order_id
        name: id
        logical_type: integer
      - id: attr_order_customer_id
        name: customer_id
        logical_type: integer
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_key: customer_id
    to_key: id
sources:
  - name: crm
    kind: postgres
    tables:
      - name: customers
        entity: customer
        columns:
          - source: cust_id
            target: id
rules:
  - id: rule_email_format
    name: email_format
    entity: customer
    attribute: email
governance:
  data_owner: team
  data_steward: jane
open_questions:
  - id: q_gdpr
    question: GDPR handling?
    entity: customer
assumptions:
  - id: asm_email
    statement: All customers have emails
    entity: customer
"#,
        )
    }

    #[test]
    fn minimal_builds_successfully() {
        let result = build_project_graph(&minimal_metadata());
        assert!(result.graph.is_some());
        assert!(result.diagnostics.is_empty());
        let g = result.graph.unwrap();
        assert_eq!(g.entity_count(), 1);
    }

    #[test]
    fn full_builds_successfully() {
        let result = build_project_graph(&full_metadata());
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
        let g = result.graph.unwrap();
        assert_eq!(g.entity_count(), 2);
        assert_eq!(g.relationship_count(), 1);
        assert_eq!(g.domains().len(), 1);
        assert_eq!(g.modules().len(), 1);
        assert_eq!(g.sources().len(), 1);
        assert_eq!(g.rules().len(), 1);
        assert!(g.governance().is_some());
        assert_eq!(g.open_questions().len(), 1);
        assert_eq!(g.assumptions().len(), 1);
    }

    #[test]
    fn unknown_entity_in_relationship() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes: []
relationships:
  - from_entity: nonexistent
    to_entity: customer
",
        );
        let result = build_project_graph(&meta);
        assert!(result.graph.is_none());
        assert!(result.diagnostics.iter().any(|d| d.code == "META-REL-001"));
    }

    #[test]
    fn unknown_attribute_in_relationship() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
  - name: order
    attributes:
      - name: id
        logical_type: integer
relationships:
  - from_entity: order
    to_entity: customer
    from_key: nonexistent_key
    to_key: id
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-REL-002"));
    }

    #[test]
    fn duplicate_entity_name() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
domains:
  - id: sales
    name: Sales
entities:
  - name: customer
    domain: sales
    attributes: []
  - name: customer
    domain: sales
    attributes: []
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-ENT-001"));
    }

    #[test]
    fn duplicate_object_id() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - id: same_id
    name: customer
    attributes: []
  - id: same_id
    name: order
    attributes: []
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-ID-001"));
    }

    #[test]
    fn invalid_business_key() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    business_key: [nonexistent]
    attributes:
      - name: id
        logical_type: integer
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-ENT-002"));
    }

    #[test]
    fn source_mapping_unknown_entity() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    attributes: []
sources:
  - name: crm
    tables:
      - name: users
        entity: nonexistent
        columns: []
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-SRC-001"));
    }

    #[test]
    fn unknown_domain_reference() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    domain: nonexistent_domain
    attributes: []
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-DOM-001"));
    }

    #[test]
    fn unknown_module_reference() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    module: nonexistent_module
    attributes: []
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-MOD-001"));
    }

    #[test]
    fn open_question_references_unknown_entity() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities: []
open_questions:
  - id: q_1
    question: test?
    entity: nonexistent
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-Q-001"));
    }

    #[test]
    fn assumption_references_unknown_entity() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities: []
assumptions:
  - id: a_1
    statement: test
    entity: nonexistent
",
        );
        let result = build_project_graph(&meta);
        assert!(result.diagnostics.iter().any(|d| d.code == "META-ASM-001"));
    }

    #[test]
    fn multiple_errors_collected() {
        let meta: AuthoringMetadata = parse_yaml_metadata(
            r"
entities:
  - name: customer
    domain: bad_domain
    module: bad_module
    business_key: [missing]
    attributes:
      - name: id
        logical_type: integer
relationships:
  - from_entity: nonexistent
    to_entity: also_nonexistent
",
        );
        let result = build_project_graph(&meta);
        assert!(result.graph.is_none());
        assert!(result.diagnostics.len() >= 4);
    }

    #[test]
    fn empty_metadata_builds() {
        let meta: AuthoringMetadata = parse_yaml_metadata("{}");
        let result = build_project_graph(&meta);
        assert!(result.graph.is_some());
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.graph.unwrap().entity_count(), 0);
    }

    #[test]
    fn traversal_helpers_work_on_full_graph() {
        let result = build_project_graph(&full_metadata());
        let g = result.graph.unwrap();

        assert_eq!(g.entities_by_domain("sales").len(), 2);
        assert_eq!(g.attributes_by_entity("customer").len(), 2);
        assert_eq!(g.relationships_touching("order").len(), 1);
        assert_eq!(g.relationships_touching("customer").len(), 1);
        assert!(g.entity_by_name("customer").is_some());
        assert!(g.entity_by_name("nonexistent").is_none());
    }
}
