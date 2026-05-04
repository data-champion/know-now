use know_now_contract::contract::{
    ContractAssumption, ContractAttribute, ContractEntity, ContractGovernance,
    ContractOpenQuestion, ContractQualityRule, ContractRelationship, ContractSourceColumn,
    ContractSourceSystem, ContractSourceTable, ContractTrace, GeneratorContract,
};
use know_now_contract::CONTRACT_SCHEMA_VERSION;
use know_now_validate::graph::{
    AssumptionNode, EntityNode, GovernanceNode, OpenQuestionNode, ProjectGraph, QualityRuleNode,
    RelationshipNode, SourceSystemNode,
};

/// Project a validated `ProjectGraph` onto a versioned `GeneratorContract`.
#[must_use]
pub fn project_graph_to_contract(graph: &ProjectGraph) -> GeneratorContract {
    let project = graph
        .project()
        .map(|p| know_now_contract::contract::ContractProject {
            name: p.name.clone().unwrap_or_default(),
            description: p.description.clone(),
            owner: p.owner.clone(),
            tags: p.tags.clone(),
        });

    let target_database = None;

    let entities: Vec<ContractEntity> = graph.entities().iter().map(project_entity).collect();

    let relationships: Vec<ContractRelationship> = graph
        .relationships()
        .iter()
        .map(project_relationship)
        .collect();

    let source_systems: Vec<ContractSourceSystem> =
        graph.sources().iter().map(project_source).collect();

    let quality_rules: Vec<ContractQualityRule> = graph.rules().iter().map(project_rule).collect();

    let governance = graph.governance().map(project_governance);

    let open_questions: Vec<ContractOpenQuestion> = graph
        .open_questions()
        .iter()
        .map(project_question)
        .collect();

    let assumptions: Vec<ContractAssumption> =
        graph.assumptions().iter().map(project_assumption).collect();

    let trace = build_trace(graph);

    GeneratorContract {
        contract_version: CONTRACT_SCHEMA_VERSION.to_owned(),
        project,
        target_database,
        entities,
        relationships,
        source_systems,
        quality_rules,
        governance,
        open_questions,
        assumptions,
        trace,
    }
}

fn project_entity(e: &EntityNode) -> ContractEntity {
    ContractEntity {
        id: e.id.0.clone(),
        name: e.name.clone(),
        display_name: e.display_name.clone(),
        domain: e.domain.clone(),
        module: e.module.clone(),
        owner: e.owner.clone(),
        steward: e.steward.clone(),
        classification: e.classification.clone(),
        retention_policy: e.retention_policy.clone(),
        description: e.description.clone(),
        entity_type: e.entity_type.clone(),
        tags: e.tags.clone(),
        business_key: e.business_key.clone(),
        attributes: e
            .attributes
            .iter()
            .map(|a| ContractAttribute {
                id: a.id.0.clone(),
                name: a.name.clone(),
                logical_type: a.logical_type.clone(),
                semantic_type: a.semantic_type.clone(),
                sensitivity: a.sensitivity.clone(),
                pii: a.pii,
                required: a.required,
                is_unique: a.is_unique,
                constraints: a.constraints.clone(),
                description: a.description.clone(),
                attr_type: a.attr_type.clone(),
            })
            .collect(),
    }
}

fn project_relationship(r: &RelationshipNode) -> ContractRelationship {
    ContractRelationship {
        id: r.id.0.clone(),
        from_entity: r.from_entity.clone(),
        to_entity: r.to_entity.clone(),
        cardinality: r.cardinality.clone(),
        from_key: r.from_key.clone(),
        to_key: r.to_key.clone(),
        description: r.description.clone(),
    }
}

fn project_source(s: &SourceSystemNode) -> ContractSourceSystem {
    ContractSourceSystem {
        name: s.name.clone(),
        kind: s.kind.clone(),
        description: s.description.clone(),
        tables: s
            .tables
            .iter()
            .map(|t| ContractSourceTable {
                name: t.name.clone(),
                entity: t.entity.clone(),
                schema: t.schema.clone(),
                columns: t
                    .columns
                    .iter()
                    .map(|c| ContractSourceColumn {
                        source: c.source.clone(),
                        target: c.target.clone(),
                        transform: c.transform.clone(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn project_rule(r: &QualityRuleNode) -> ContractQualityRule {
    ContractQualityRule {
        id: r.id.0.clone(),
        name: r.name.clone(),
        entity: r.entity.clone(),
        attribute: r.attribute.clone(),
        rule_type: r.rule_type.clone(),
        expression: r.expression.clone(),
        severity: r.severity.clone(),
        description: r.description.clone(),
    }
}

fn project_governance(g: &GovernanceNode) -> ContractGovernance {
    ContractGovernance {
        data_owner: g.data_owner.clone(),
        data_steward: g.data_steward.clone(),
        classification_default: g.classification_default.clone(),
        retention_default: g.retention_default.clone(),
    }
}

fn project_question(q: &OpenQuestionNode) -> ContractOpenQuestion {
    ContractOpenQuestion {
        id: q.id.0.clone(),
        question: q.question.clone(),
        context: q.context.clone(),
        entity: q.entity.clone(),
        priority: q.priority.clone(),
    }
}

fn project_assumption(a: &AssumptionNode) -> ContractAssumption {
    ContractAssumption {
        id: a.id.0.clone(),
        statement: a.statement.clone(),
        rationale: a.rationale.clone(),
        entity: a.entity.clone(),
        risk: a.risk.clone(),
    }
}

fn build_trace(graph: &ProjectGraph) -> ContractTrace {
    let mut entity_ids = Vec::new();
    let mut attribute_ids = Vec::new();
    let mut relationship_ids = Vec::new();
    let mut rule_ids = Vec::new();
    let mut question_ids = Vec::new();
    let mut assumption_ids = Vec::new();

    for e in graph.entities() {
        entity_ids.push(e.id.0.clone());
        for a in &e.attributes {
            attribute_ids.push(a.id.0.clone());
        }
    }
    for r in graph.relationships() {
        relationship_ids.push(r.id.0.clone());
    }
    for r in graph.rules() {
        rule_ids.push(r.id.0.clone());
    }
    for q in graph.open_questions() {
        question_ids.push(q.id.0.clone());
    }
    for a in graph.assumptions() {
        assumption_ids.push(a.id.0.clone());
    }

    ContractTrace {
        entity_ids,
        attribute_ids,
        relationship_ids,
        rule_ids,
        question_ids,
        assumption_ids,
    }
}
