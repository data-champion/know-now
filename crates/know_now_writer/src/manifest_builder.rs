use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::manifest::{
    ArtifactEntry, ArtifactSpan, ArtifactTrace, ManifestV1, PolicyRef, TargetDatabase,
    TemplateRendererRef, MANIFEST_SCHEMA_VERSION,
};

pub struct ManifestBuilder {
    engine_version: String,
    generator_contract_version: String,
    project_id: String,
    input_hash: String,
    lockfile_hash: String,
    target_database: TargetDatabase,
    policy: PolicyRef,
    template_renderers: Vec<TemplateRendererRef>,
    artifacts: Vec<ArtifactEntry>,
    warnings: Vec<String>,
}

pub struct ArtifactInput {
    pub path: String,
    pub kind: String,
    pub artifact_id: String,
    pub generator: String,
    pub generator_version: String,
    pub content: Vec<u8>,
    pub metadata_object_ids: Vec<String>,
    pub trace: Vec<TraceInput>,
}

pub struct TraceInput {
    pub line_start: u32,
    pub line_end: u32,
    pub metadata_object_ids: Vec<String>,
    pub policy_rule_ids: Vec<String>,
}

impl ManifestBuilder {
    #[must_use]
    pub fn new(engine_version: &str, generator_contract_version: &str) -> Self {
        Self {
            engine_version: engine_version.to_owned(),
            generator_contract_version: generator_contract_version.to_owned(),
            project_id: String::new(),
            input_hash: String::new(),
            lockfile_hash: String::new(),
            target_database: TargetDatabase {
                kind: String::new(),
                version: String::new(),
                compatibility_floor: String::new(),
            },
            policy: PolicyRef {
                pack: String::new(),
                version: String::new(),
                hash: String::new(),
            },
            template_renderers: Vec::new(),
            artifacts: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn project_id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.project_id);
        self
    }

    #[must_use]
    pub fn input_hash_from_bytes(mut self, data: &[u8]) -> Self {
        self.input_hash = sha256_hex(data);
        self
    }

    #[must_use]
    pub fn lockfile_hash(mut self, hash: &str) -> Self {
        hash.clone_into(&mut self.lockfile_hash);
        self
    }

    #[must_use]
    pub fn target_database(mut self, db: TargetDatabase) -> Self {
        self.target_database = db;
        self
    }

    #[must_use]
    pub fn policy(mut self, policy: PolicyRef) -> Self {
        self.policy = policy;
        self
    }

    #[must_use]
    pub fn warning(mut self, msg: String) -> Self {
        self.warnings.push(msg);
        self
    }

    pub fn add_artifact(&mut self, input: ArtifactInput) {
        let hash = sha256_hex(&input.content);
        let mut trace: Vec<ArtifactTrace> = input
            .trace
            .into_iter()
            .map(|t| ArtifactTrace {
                artifact_span: ArtifactSpan {
                    line_start: t.line_start,
                    line_end: t.line_end,
                },
                metadata_object_ids: t.metadata_object_ids,
                policy_rule_ids: t.policy_rule_ids,
            })
            .collect();
        trace.sort_by_key(|t| t.artifact_span.line_start);

        self.artifacts.push(ArtifactEntry {
            path: PathBuf::from(input.path),
            kind: input.kind,
            artifact_id: input.artifact_id,
            generator: input.generator,
            generator_version: input.generator_version,
            hash,
            metadata_object_ids: input.metadata_object_ids,
            trace,
        });
    }

    #[must_use]
    pub fn build(mut self) -> ManifestV1 {
        self.artifacts.sort_by(|a, b| a.path.cmp(&b.path));

        ManifestV1 {
            engine_version: self.engine_version,
            metadata_schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
            generator_contract_version: self.generator_contract_version,
            project_id: self.project_id,
            input_hash: self.input_hash,
            lockfile_hash: self.lockfile_hash,
            target_database: self.target_database,
            policy: self.policy,
            template_renderers: self.template_renderers,
            artifacts: self.artifacts,
            warnings: self.warnings,
        }
    }
}

#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("sha256:{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_deterministic() {
        let a = sha256_hex(b"hello world");
        let b = sha256_hex(b"hello world");
        assert_eq!(a, b);
        assert!(a.starts_with("sha256:"));
    }

    #[test]
    fn sha256_hex_known_value() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn builder_produces_sorted_artifacts() {
        let mut builder = ManifestBuilder::new("0.1.0", "1.0");
        builder.add_artifact(ArtifactInput {
            path: "docs/entities/order.md".into(),
            kind: "markdown_doc".into(),
            artifact_id: "art_docs_order".into(),
            generator: "gen_docs".into(),
            generator_version: "0.1.0".into(),
            content: b"# Order".to_vec(),
            metadata_object_ids: vec![],
            trace: vec![],
        });
        builder.add_artifact(ArtifactInput {
            path: "ddl/postgres/schema.sql".into(),
            kind: "postgres_ddl".into(),
            artifact_id: "art_pg".into(),
            generator: "gen_postgres".into(),
            generator_version: "0.1.0".into(),
            content: b"CREATE TABLE t();".to_vec(),
            metadata_object_ids: vec![],
            trace: vec![],
        });

        let manifest = builder.build();
        assert_eq!(
            manifest.artifacts[0].path,
            PathBuf::from("ddl/postgres/schema.sql")
        );
        assert_eq!(
            manifest.artifacts[1].path,
            PathBuf::from("docs/entities/order.md")
        );
    }

    #[test]
    fn builder_computes_artifact_hashes() {
        let mut builder = ManifestBuilder::new("0.1.0", "1.0");
        builder.add_artifact(ArtifactInput {
            path: "test.sql".into(),
            kind: "postgres_ddl".into(),
            artifact_id: "art_1".into(),
            generator: "gen".into(),
            generator_version: "0.1.0".into(),
            content: b"SELECT 1;".to_vec(),
            metadata_object_ids: vec![],
            trace: vec![],
        });

        let manifest = builder.build();
        assert!(manifest.artifacts[0].hash.starts_with("sha256:"));
        assert_eq!(manifest.artifacts[0].hash, sha256_hex(b"SELECT 1;"));
    }

    #[test]
    fn builder_computes_input_hash() {
        let builder =
            ManifestBuilder::new("0.1.0", "1.0").input_hash_from_bytes(b"metadata content");
        let manifest = builder.build();
        assert!(manifest.input_hash.starts_with("sha256:"));
        assert_eq!(manifest.input_hash, sha256_hex(b"metadata content"));
    }

    #[test]
    fn trace_sorted_by_line_start() {
        let mut builder = ManifestBuilder::new("0.1.0", "1.0");
        builder.add_artifact(ArtifactInput {
            path: "test.sql".into(),
            kind: "ddl".into(),
            artifact_id: "art_1".into(),
            generator: "gen".into(),
            generator_version: "0.1.0".into(),
            content: b"content".to_vec(),
            metadata_object_ids: vec![],
            trace: vec![
                TraceInput {
                    line_start: 20,
                    line_end: 25,
                    metadata_object_ids: vec!["b".into()],
                    policy_rule_ids: vec![],
                },
                TraceInput {
                    line_start: 5,
                    line_end: 10,
                    metadata_object_ids: vec!["a".into()],
                    policy_rule_ids: vec![],
                },
            ],
        });

        let manifest = builder.build();
        let trace = &manifest.artifacts[0].trace;
        assert_eq!(trace[0].artifact_span.line_start, 5);
        assert_eq!(trace[1].artifact_span.line_start, 20);
    }

    #[test]
    fn manifest_schema_version_set() {
        let builder = ManifestBuilder::new("0.1.0", "1.0");
        let manifest = builder.build();
        assert_eq!(manifest.metadata_schema_version, MANIFEST_SCHEMA_VERSION);
    }

    #[test]
    fn builder_no_volatile_fields() {
        let builder = ManifestBuilder::new("0.1.0", "1.0").project_id("test");
        let manifest = builder.build();
        let json = manifest.to_json_pretty();
        for field in ["timestamp", "hostname", "username", "run_id", "started_at"] {
            assert!(!json.contains(field), "manifest must not contain {field}");
        }
    }
}
