use serde::{Deserialize, Serialize};

/// Descriptor for a generated artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub path: String,
    pub kind: ArtifactKind,
    pub artifact_id: String,
    pub generator: String,
    pub generator_version: String,
    pub content: String,
    pub metadata_object_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    PostgresDdl,
    DbtModel,
    DbtSchema,
    DbtTest,
    QualityContract,
    MarkdownDoc,
    MermaidDiagram,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_kind_serde_roundtrip() {
        let kinds = [
            ArtifactKind::PostgresDdl,
            ArtifactKind::DbtModel,
            ArtifactKind::DbtSchema,
            ArtifactKind::DbtTest,
            ArtifactKind::QualityContract,
            ArtifactKind::MarkdownDoc,
            ArtifactKind::MermaidDiagram,
        ];
        for kind in &kinds {
            let json = serde_json::to_string(kind).unwrap();
            let parsed: ArtifactKind = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, kind);
        }
    }

    #[test]
    fn artifact_descriptor_json_roundtrip() {
        let desc = ArtifactDescriptor {
            path: "generated/ddl/postgres/schema.sql".into(),
            kind: ArtifactKind::PostgresDdl,
            artifact_id: "art_pg_schema".into(),
            generator: "know_now_gen_postgres".into(),
            generator_version: "0.1.0".into(),
            content: "CREATE TABLE customer();".into(),
            metadata_object_ids: vec!["ent_customer".into()],
        };
        let json = serde_json::to_string(&desc).unwrap();
        let parsed: ArtifactDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, desc.path);
        assert_eq!(parsed.kind, desc.kind);
    }
}
