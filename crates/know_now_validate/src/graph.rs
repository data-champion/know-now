use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNode {
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: NodeId,
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
    pub attributes: Vec<AttributeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeNode {
    pub id: NodeId,
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
pub struct RelationshipNode {
    pub id: NodeId,
    pub from_entity: String,
    pub to_entity: String,
    pub cardinality: Option<String>,
    pub from_key: Option<String>,
    pub to_key: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSystemNode {
    pub name: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub entities: Vec<String>,
    pub tables: Vec<SourceTableNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTableNode {
    pub name: String,
    pub entity: Option<String>,
    pub schema: Option<String>,
    pub columns: Vec<SourceColumnNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceColumnNode {
    pub source: String,
    pub target: String,
    pub transform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleNode {
    pub id: NodeId,
    pub name: String,
    pub entity: Option<String>,
    pub attribute: Option<String>,
    pub rule_type: Option<String>,
    pub expression: Option<String>,
    pub severity: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNode {
    pub data_owner: Option<String>,
    pub data_steward: Option<String>,
    pub classification_default: Option<String>,
    pub retention_default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestionNode {
    pub id: NodeId,
    pub question: String,
    pub context: Option<String>,
    pub entity: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionNode {
    pub id: NodeId,
    pub statement: String,
    pub rationale: Option<String>,
    pub entity: Option<String>,
    pub risk: Option<String>,
}

pub(crate) struct ProjectGraphParts {
    pub project: Option<ProjectNode>,
    pub domains: Vec<DomainNode>,
    pub modules: Vec<ModuleNode>,
    pub entities: Vec<EntityNode>,
    pub relationships: Vec<RelationshipNode>,
    pub sources: Vec<SourceSystemNode>,
    pub rules: Vec<QualityRuleNode>,
    pub governance: Option<GovernanceNode>,
    pub open_questions: Vec<OpenQuestionNode>,
    pub assumptions: Vec<AssumptionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGraph {
    project: Option<ProjectNode>,
    domains: Vec<DomainNode>,
    modules: Vec<ModuleNode>,
    entities: Vec<EntityNode>,
    relationships: Vec<RelationshipNode>,
    sources: Vec<SourceSystemNode>,
    rules: Vec<QualityRuleNode>,
    governance: Option<GovernanceNode>,
    open_questions: Vec<OpenQuestionNode>,
    assumptions: Vec<AssumptionNode>,

    entity_index_by_name: BTreeMap<String, usize>,
    entity_index_by_domain: BTreeMap<String, Vec<usize>>,
    relationships_by_entity: BTreeMap<String, Vec<usize>>,
    all_node_ids: BTreeSet<NodeId>,
}

impl ProjectGraph {
    pub(crate) fn from_parts(parts: ProjectGraphParts) -> Self {
        let mut entity_index_by_name = BTreeMap::new();
        let mut entity_index_by_domain: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut relationships_by_entity: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut all_node_ids = BTreeSet::new();

        for (idx, entity) in parts.entities.iter().enumerate() {
            entity_index_by_name.insert(entity.name.clone(), idx);
            if let Some(ref domain) = entity.domain {
                entity_index_by_domain
                    .entry(domain.clone())
                    .or_default()
                    .push(idx);
            }
            all_node_ids.insert(entity.id.clone());
            for attr in &entity.attributes {
                all_node_ids.insert(attr.id.clone());
            }
        }

        for (idx, rel) in parts.relationships.iter().enumerate() {
            relationships_by_entity
                .entry(rel.from_entity.clone())
                .or_default()
                .push(idx);
            relationships_by_entity
                .entry(rel.to_entity.clone())
                .or_default()
                .push(idx);
            all_node_ids.insert(rel.id.clone());
        }

        for rule in &parts.rules {
            all_node_ids.insert(rule.id.clone());
        }
        for q in &parts.open_questions {
            all_node_ids.insert(q.id.clone());
        }
        for a in &parts.assumptions {
            all_node_ids.insert(a.id.clone());
        }

        Self {
            project: parts.project,
            domains: parts.domains,
            modules: parts.modules,
            entities: parts.entities,
            relationships: parts.relationships,
            sources: parts.sources,
            rules: parts.rules,
            governance: parts.governance,
            open_questions: parts.open_questions,
            assumptions: parts.assumptions,
            entity_index_by_name,
            entity_index_by_domain,
            relationships_by_entity,
            all_node_ids,
        }
    }

    #[must_use]
    pub fn project(&self) -> Option<&ProjectNode> {
        self.project.as_ref()
    }

    #[must_use]
    pub fn domains(&self) -> &[DomainNode] {
        &self.domains
    }

    #[must_use]
    pub fn modules(&self) -> &[ModuleNode] {
        &self.modules
    }

    #[must_use]
    pub fn entities(&self) -> &[EntityNode] {
        &self.entities
    }

    #[must_use]
    pub fn relationships(&self) -> &[RelationshipNode] {
        &self.relationships
    }

    #[must_use]
    pub fn sources(&self) -> &[SourceSystemNode] {
        &self.sources
    }

    #[must_use]
    pub fn rules(&self) -> &[QualityRuleNode] {
        &self.rules
    }

    #[must_use]
    pub fn governance(&self) -> Option<&GovernanceNode> {
        self.governance.as_ref()
    }

    #[must_use]
    pub fn open_questions(&self) -> &[OpenQuestionNode] {
        &self.open_questions
    }

    #[must_use]
    pub fn assumptions(&self) -> &[AssumptionNode] {
        &self.assumptions
    }

    #[must_use]
    pub fn entity_by_name(&self, name: &str) -> Option<&EntityNode> {
        self.entity_index_by_name
            .get(name)
            .map(|&idx| &self.entities[idx])
    }

    #[must_use]
    pub fn entities_by_domain(&self, domain: &str) -> Vec<&EntityNode> {
        self.entity_index_by_domain
            .get(domain)
            .map(|indices| indices.iter().map(|&idx| &self.entities[idx]).collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn attributes_by_entity(&self, entity_name: &str) -> Vec<&AttributeNode> {
        self.entity_by_name(entity_name)
            .map(|e| e.attributes.iter().collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn relationships_touching(&self, entity_name: &str) -> Vec<&RelationshipNode> {
        self.relationships_by_entity
            .get(entity_name)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&idx| &self.relationships[idx])
                    .collect()
            })
            .unwrap_or_default()
    }

    #[must_use]
    pub fn node_id_exists(&self, id: &NodeId) -> bool {
        self.all_node_ids.contains(id)
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    #[must_use]
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> ProjectGraph {
        let entities = vec![
            EntityNode {
                id: NodeId("ent_customer".into()),
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
                business_key: vec!["email".into()],
                attributes: vec![
                    AttributeNode {
                        id: NodeId("attr_customer_id".into()),
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
                    AttributeNode {
                        id: NodeId("attr_customer_email".into()),
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
            },
            EntityNode {
                id: NodeId("ent_order".into()),
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
                business_key: vec![],
                attributes: vec![],
            },
        ];

        let relationships = vec![RelationshipNode {
            id: NodeId("rel_order_customer".into()),
            from_entity: "order".into(),
            to_entity: "customer".into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("customer_id".into()),
            to_key: Some("id".into()),
            description: None,
        }];

        ProjectGraph::from_parts(ProjectGraphParts {
            project: None,
            domains: vec![DomainNode {
                id: "sales".into(),
                name: "Sales".into(),
                description: None,
                owner: None,
            }],
            modules: vec![],
            entities,
            relationships,
            sources: vec![],
            rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
        })
    }

    #[test]
    fn entity_by_name_found() {
        let g = sample_graph();
        let e = g.entity_by_name("customer").unwrap();
        assert_eq!(e.name, "customer");
    }

    #[test]
    fn entity_by_name_not_found() {
        let g = sample_graph();
        assert!(g.entity_by_name("missing").is_none());
    }

    #[test]
    fn entities_by_domain() {
        let g = sample_graph();
        let sales = g.entities_by_domain("sales");
        assert_eq!(sales.len(), 2);
    }

    #[test]
    fn entities_by_domain_empty() {
        let g = sample_graph();
        assert!(g.entities_by_domain("missing").is_empty());
    }

    #[test]
    fn attributes_by_entity() {
        let g = sample_graph();
        let attrs = g.attributes_by_entity("customer");
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn relationships_touching_entity() {
        let g = sample_graph();
        let rels = g.relationships_touching("customer");
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].from_entity, "order");
    }

    #[test]
    fn relationships_touching_both_sides() {
        let g = sample_graph();
        let from_rels = g.relationships_touching("order");
        let to_rels = g.relationships_touching("customer");
        assert_eq!(from_rels.len(), 1);
        assert_eq!(to_rels.len(), 1);
        assert_eq!(from_rels[0].id, to_rels[0].id);
    }

    #[test]
    fn node_id_exists() {
        let g = sample_graph();
        assert!(g.node_id_exists(&NodeId("ent_customer".into())));
        assert!(g.node_id_exists(&NodeId("attr_customer_email".into())));
        assert!(g.node_id_exists(&NodeId("rel_order_customer".into())));
        assert!(!g.node_id_exists(&NodeId("nonexistent".into())));
    }

    #[test]
    fn graph_counts() {
        let g = sample_graph();
        assert_eq!(g.entity_count(), 2);
        assert_eq!(g.relationship_count(), 1);
    }

    #[test]
    fn graph_is_not_mutable() {
        let g = sample_graph();
        let _entities: &[EntityNode] = g.entities();
        let _rels: &[RelationshipNode] = g.relationships();
        assert_eq!(g.domains().len(), 1);
    }

    #[test]
    fn graph_json_roundtrip() {
        let g = sample_graph();
        let json = serde_json::to_string(&g).unwrap();
        let parsed: ProjectGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_count(), g.entity_count());
        assert_eq!(parsed.relationship_count(), g.relationship_count());
    }
}
