//! Property-based tests for metadata YAML/JSON round-trip fidelity.
//!
//! Generates structurally valid `AuthoringMetadata` values and verifies that
//! serialize → deserialize produces an identical document.

use know_now_metadata::authoring::{Attribute, AuthoringMetadata, Entity, LogicalType};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Entity/attribute names: lowercase ASCII letter start, then lowercase + digits
/// + underscore, 1-30 chars total.
fn entity_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,29}")
        .expect("regex should be valid")
        .prop_filter("must not be empty", |s| !s.is_empty())
}

/// Pick one of the LogicalType variants that the task enumerates.
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

/// Generate a valid Attribute with a name and optional logical type.
fn attribute_strategy() -> impl Strategy<Value = Attribute> {
    (
        entity_name_strategy(),
        prop::option::of(logical_type_strategy()),
        any::<Option<bool>>(),
        any::<Option<bool>>(),
    )
        .prop_map(|(name, logical_type, required, pii)| Attribute {
            id: None,
            name,
            logical_type,
            semantic_type: None,
            sensitivity: None,
            pii,
            required,
            is_unique: None,
            constraints: vec![],
            description: None,
            attr_type: None,
        })
}

/// Generate a valid Entity with 1-5 attributes.
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

/// Generate a valid AuthoringMetadata with 1-5 entities.
fn metadata_strategy() -> impl Strategy<Value = AuthoringMetadata> {
    prop::collection::vec(entity_strategy(), 1..=5).prop_map(|entities| AuthoringMetadata {
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
    })
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Serialize to JSON, deserialize back, re-serialize, and verify the two
    /// JSON representations are byte-identical.
    #[test]
    fn json_roundtrip_is_lossless(meta in metadata_strategy()) {
        let json1 = serde_json::to_string(&meta).expect("serialize to JSON");
        let parsed: AuthoringMetadata =
            serde_json::from_str(&json1).expect("deserialize from JSON");
        let json2 = serde_json::to_string(&parsed).expect("re-serialize to JSON");
        prop_assert_eq!(json1, json2);
    }

    /// Serialize to YAML via serde_saphyr, deserialize back, re-serialize to
    /// JSON, and verify the result equals a direct JSON serialization of the
    /// original.
    #[test]
    fn yaml_roundtrip_matches_json(meta in metadata_strategy()) {
        let yaml = serde_saphyr::to_string(&meta).expect("serialize to YAML");
        let parsed: AuthoringMetadata =
            serde_saphyr::from_str(&yaml).expect("deserialize from YAML");
        let json_original = serde_json::to_string(&meta).expect("original to JSON");
        let json_parsed = serde_json::to_string(&parsed).expect("parsed to JSON");
        prop_assert_eq!(json_original, json_parsed);
    }

    /// Every entity survives the round-trip with the correct number of
    /// attributes.
    #[test]
    fn entity_and_attribute_counts_preserved(meta in metadata_strategy()) {
        let json = serde_json::to_string(&meta).expect("serialize");
        let parsed: AuthoringMetadata = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(meta.entities.len(), parsed.entities.len());
        for (orig, rt) in meta.entities.iter().zip(parsed.entities.iter()) {
            prop_assert_eq!(orig.name.as_str(), rt.name.as_str());
            prop_assert_eq!(orig.attributes.len(), rt.attributes.len());
        }
    }
}
