//! Property-based tests for the validation → graph → contract projection
//! pipeline.
//!
//! Generates random valid metadata, runs it through `build_project_graph` and
//! `project_graph_to_contract`, and verifies structural invariants.

use know_now_core::projection::project_graph_to_contract;
use know_now_metadata::authoring::{Attribute, AuthoringMetadata, Entity, LogicalType};
use know_now_validate::builder::build_project_graph;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies (mirror the metadata crate strategies but kept self-contained)
// ---------------------------------------------------------------------------

/// Entity/attribute names: lowercase ASCII letter start, then lowercase +
/// digits + underscore, 1-20 chars total.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,19}")
        .expect("regex should be valid")
        .prop_filter("must not be empty", |s| !s.is_empty())
}

fn logical_type_strategy() -> impl Strategy<Value = LogicalType> {
    prop::sample::select(vec![
        LogicalType::Integer,
        LogicalType::String,
        LogicalType::Boolean,
        LogicalType::Decimal,
        LogicalType::Timestamp,
        LogicalType::Date,
        LogicalType::Uuid,
        LogicalType::Json,
        LogicalType::Text,
        LogicalType::Float,
    ])
}

fn attribute_strategy() -> impl Strategy<Value = Attribute> {
    (
        entity_name_strategy(),
        prop::option::of(logical_type_strategy()),
    )
        .prop_map(|(name, logical_type)| Attribute {
            id: None,
            name,
            logical_type,
            semantic_type: None,
            sensitivity: None,
            pii: None,
            required: None,
            is_unique: None,
            constraints: vec![],
            description: None,
            attr_type: None,
        })
}

fn entity_strategy() -> impl Strategy<Value = Entity> {
    (
        entity_name_strategy(),
        prop::collection::vec(attribute_strategy(), 1..=5),
    )
        .prop_map(|(name, attributes)| Entity {
            id: None,
            name,
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
            business_key: vec![],
            attributes,
        })
}

/// Generate metadata with 1-5 entities that have **unique** names.
/// Duplicate entity names within the same domain cause validation errors,
/// so we deduplicate before building the metadata.
fn metadata_strategy() -> impl Strategy<Value = AuthoringMetadata> {
    prop::collection::vec(entity_strategy(), 1..=5).prop_map(|mut entities| {
        // Deduplicate entity names to avoid META-ENT-001 validation errors.
        let mut seen = std::collections::HashSet::new();
        entities.retain(|e| seen.insert(e.name.clone()));

        // Also deduplicate attribute names within each entity.
        for entity in &mut entities {
            let mut attr_seen = std::collections::HashSet::new();
            entity
                .attributes
                .retain(|a| attr_seen.insert(a.name.clone()));
            // Ensure at least one attribute remains after dedup.
            if entity.attributes.is_empty() {
                entity.attributes.push(Attribute {
                    id: None,
                    name: "fallback_id".to_owned(),
                    logical_type: Some(LogicalType::Integer),
                    semantic_type: None,
                    sensitivity: None,
                    pii: None,
                    required: None,
                    is_unique: None,
                    constraints: vec![],
                    description: None,
                    attr_type: None,
                });
            }
        }

        AuthoringMetadata {
            version: None,
            project: None,
            target_database: None,
            policy: None,
            domains: vec![],
            modules: vec![],
            entities,
            relationships: vec![],
            sources: vec![],
            rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
        }
    })
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// The full pipeline (validate → graph → contract) never panics on valid
    /// metadata with unique entity names.
    #[test]
    fn pipeline_never_panics(meta in metadata_strategy()) {
        let build = build_project_graph(&meta);
        // With deduplicated names and no cross-references, the graph should
        // always build successfully.
        prop_assert!(
            build.graph.is_some(),
            "Expected graph to build, got diagnostics: {:?}",
            build.diagnostics
        );
        let graph = build.graph.unwrap();
        let _contract = project_graph_to_contract(&graph);
    }

    /// The contract entity count always matches the input metadata entity
    /// count.
    #[test]
    fn contract_entity_count_matches_input(meta in metadata_strategy()) {
        let build = build_project_graph(&meta);
        let graph = build.graph.expect("graph should build");
        let contract = project_graph_to_contract(&graph);
        prop_assert_eq!(
            meta.entities.len(),
            contract.entities.len(),
            "entity count mismatch"
        );
    }

    /// For every entity, the attribute count in the contract matches the
    /// input metadata.
    #[test]
    fn contract_attribute_counts_match_input(meta in metadata_strategy()) {
        let build = build_project_graph(&meta);
        let graph = build.graph.expect("graph should build");
        let contract = project_graph_to_contract(&graph);
        for (orig, proj) in meta.entities.iter().zip(contract.entities.iter()) {
            prop_assert_eq!(
                orig.attributes.len(),
                proj.attributes.len(),
                "attribute count mismatch for entity '{}'",
                orig.name
            );
        }
    }

    /// The graph entity count equals the metadata entity count.
    #[test]
    fn graph_entity_count_matches(meta in metadata_strategy()) {
        let build = build_project_graph(&meta);
        let graph = build.graph.expect("graph should build");
        prop_assert_eq!(meta.entities.len(), graph.entity_count());
    }

    /// Every entity name from the metadata is findable in the graph.
    #[test]
    fn all_entities_findable_by_name(meta in metadata_strategy()) {
        let build = build_project_graph(&meta);
        let graph = build.graph.expect("graph should build");
        for entity in &meta.entities {
            prop_assert!(
                graph.entity_by_name(&entity.name).is_some(),
                "entity '{}' not found in graph",
                entity.name
            );
        }
    }
}
