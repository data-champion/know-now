use serde::{Deserialize, Serialize};

use crate::span::SourceSpanIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringMetadata {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub project: Option<Project>,
    #[serde(default)]
    pub target_database: Option<TargetDatabase>,
    #[serde(default)]
    pub policy: Option<PolicyRef>,
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub modules: Vec<Module>,
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub sources: Vec<SourceSystem>,
    #[serde(default)]
    pub rules: Vec<QualityRule>,
    #[serde(default)]
    pub governance: Option<GovernanceMeta>,
    #[serde(default)]
    pub open_questions: Vec<OpenQuestion>,
    #[serde(default)]
    pub assumptions: Vec<Assumption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetDatabase {
    pub kind: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub compatibility_floor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRef {
    pub pack: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub steward: Option<String>,
    #[serde(default)]
    pub classification: Option<String>,
    #[serde(default)]
    pub retention_policy: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub business_key: Vec<String>,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub logical_type: Option<LogicalType>,
    #[serde(default)]
    pub semantic_type: Option<SemanticType>,
    #[serde(default)]
    pub sensitivity: Option<String>,
    #[serde(default)]
    pub pii: Option<bool>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default, rename = "unique")]
    pub is_unique: Option<bool>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub attr_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalType {
    Integer,
    Bigint,
    Smallint,
    Decimal,
    Float,
    Double,
    Boolean,
    String,
    Text,
    Date,
    Time,
    Timestamp,
    TimestampTz,
    Uuid,
    Json,
    Jsonb,
    Binary,
    Interval,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticType {
    Email,
    Phone,
    Url,
    Currency,
    Percentage,
    IpAddress,
    MacAddress,
    Ssn,
    CreditCard,
    PostalCode,
    Country,
    Language,
    Latitude,
    Longitude,
    GeoPoint,
    FilePath,
    MimeType,
    Markdown,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    #[serde(default)]
    pub id: Option<String>,
    pub from_entity: String,
    pub to_entity: String,
    #[serde(default)]
    pub cardinality: Option<String>,
    #[serde(default)]
    pub from_key: Option<String>,
    #[serde(default)]
    pub to_key: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSystem {
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub entities: Vec<String>,
    #[serde(default)]
    pub tables: Vec<SourceTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceTable {
    pub name: String,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub columns: Vec<SourceColumnMap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceColumnMap {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub transform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityRule {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub attribute: Option<String>,
    #[serde(default)]
    pub rule_type: Option<String>,
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernanceMeta {
    #[serde(default)]
    pub data_owner: Option<String>,
    #[serde(default)]
    pub data_steward: Option<String>,
    #[serde(default)]
    pub classification_default: Option<String>,
    #[serde(default)]
    pub retention_default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenQuestion {
    #[serde(default)]
    pub id: Option<String>,
    pub question: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assumption {
    #[serde(default)]
    pub id: Option<String>,
    pub statement: String,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedMetadataDocument {
    pub metadata: AuthoringMetadata,
    pub spans: SourceSpanIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_metadata_deserializes() {
        let yaml = r#"
entities:
  - name: customer
    attributes:
      - name: id
        type: integer
"#;
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(meta.entities.len(), 1);
        assert_eq!(meta.entities[0].name, "customer");
        assert_eq!(meta.entities[0].attributes.len(), 1);
    }

    #[test]
    fn full_metadata_deserializes() {
        let yaml = r#"
version: "1.0"
project:
  name: ecommerce
  description: E-commerce demo
  owner: data-team
  tags:
    - demo
    - ecommerce
target_database:
  kind: postgres
  version: "18"
  compatibility_floor: "16"
policy:
  pack: dc_standard
  version: "1.0"
domains:
  - id: sales
    name: Sales
    description: Sales domain
    owner: sales-team
modules:
  - id: core
    name: Core
    description: Core module
entities:
  - name: customer
    display_name: Customer
    domain: sales
    module: core
    owner: data-team
    steward: jane
    classification: internal
    retention_policy: 7y
    description: Customer entity
    type: dimension
    tags:
      - pii
    business_key:
      - email
    attributes:
      - name: id
        logical_type: integer
        required: true
        unique: true
        description: Primary key
      - name: email
        logical_type: string
        semantic_type: email
        pii: true
        sensitivity: high
relationships:
  - from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_key: customer_id
    to_key: id
sources:
  - name: crm
    kind: postgres
    entities:
      - customer
governance:
  data_owner: data-team
  data_steward: jane
  classification_default: internal
  retention_default: 5y
open_questions:
  - question: Should we track deleted customers?
    context: GDPR implications
    entity: customer
    priority: high
assumptions:
  - statement: All customers have emails
    rationale: Required by business process
    entity: customer
    risk: medium
"#;
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(meta.version, Some("1.0".into()));
        assert_eq!(meta.entities.len(), 1);
        assert_eq!(meta.entities[0].attributes.len(), 2);
        assert_eq!(meta.domains.len(), 1);
        assert_eq!(meta.relationships.len(), 1);
        assert_eq!(meta.sources.len(), 1);
        assert_eq!(meta.open_questions.len(), 1);
        assert_eq!(meta.assumptions.len(), 1);
        assert!(meta.governance.is_some());
    }

    #[test]
    fn unknown_field_rejected() {
        let yaml = r#"
entities:
  - name: customer
    foo_unknown: bar
    attributes: []
"#;
        let result: Result<AuthoringMetadata, _> = serde_saphyr::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn logical_type_snake_case() {
        let yaml = r#"
entities:
  - name: t
    attributes:
      - name: a
        logical_type: timestamp_tz
"#;
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(
            meta.entities[0].attributes[0].logical_type,
            Some(LogicalType::TimestampTz)
        );
    }

    #[test]
    fn semantic_type_snake_case() {
        let yaml = r#"
entities:
  - name: t
    attributes:
      - name: a
        semantic_type: ip_address
"#;
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(
            meta.entities[0].attributes[0].semantic_type,
            Some(SemanticType::IpAddress)
        );
    }

    #[test]
    fn empty_metadata_deserializes() {
        let yaml = "{}";
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        assert!(meta.entities.is_empty());
        assert!(meta.version.is_none());
    }

    #[test]
    fn parsed_metadata_document_wraps_spans() {
        let meta: AuthoringMetadata = serde_saphyr::from_str("{}").unwrap();
        let doc = ParsedMetadataDocument {
            metadata: meta,
            spans: SourceSpanIndex::new(),
        };
        assert!(doc.metadata.entities.is_empty());
        assert_eq!(doc.spans.object_id_count(), 0);
    }

    #[test]
    fn metadata_json_roundtrip() {
        let yaml = r#"
entities:
  - name: customer
    attributes:
      - name: id
        logical_type: integer
"#;
        let meta: AuthoringMetadata = serde_saphyr::from_str(yaml).unwrap();
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: AuthoringMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.entities[0].name, "customer");
    }

    #[test]
    fn all_logical_types_roundtrip() {
        let types = vec![
            LogicalType::Integer,
            LogicalType::Bigint,
            LogicalType::Smallint,
            LogicalType::Decimal,
            LogicalType::Float,
            LogicalType::Double,
            LogicalType::Boolean,
            LogicalType::String,
            LogicalType::Text,
            LogicalType::Date,
            LogicalType::Time,
            LogicalType::Timestamp,
            LogicalType::TimestampTz,
            LogicalType::Uuid,
            LogicalType::Json,
            LogicalType::Jsonb,
            LogicalType::Binary,
            LogicalType::Interval,
            LogicalType::Array,
        ];
        for lt in &types {
            let json = serde_json::to_string(lt).unwrap();
            let parsed: LogicalType = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, lt);
        }
    }

    #[test]
    fn all_semantic_types_roundtrip() {
        let types = vec![
            SemanticType::Email,
            SemanticType::Phone,
            SemanticType::Url,
            SemanticType::Currency,
            SemanticType::Percentage,
            SemanticType::IpAddress,
            SemanticType::MacAddress,
            SemanticType::Ssn,
            SemanticType::CreditCard,
            SemanticType::PostalCode,
            SemanticType::Country,
            SemanticType::Language,
            SemanticType::Latitude,
            SemanticType::Longitude,
            SemanticType::GeoPoint,
            SemanticType::FilePath,
            SemanticType::MimeType,
            SemanticType::Markdown,
            SemanticType::Html,
        ];
        for st in &types {
            let json = serde_json::to_string(st).unwrap();
            let parsed: SemanticType = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, st);
        }
    }
}
