use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const MANIFEST_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestV1 {
    pub engine_version: String,
    pub metadata_schema_version: String,
    pub generator_contract_version: String,
    pub project_id: String,
    pub input_hash: String,
    pub lockfile_hash: String,
    pub target_database: TargetDatabase,
    pub policy: PolicyRef,
    pub template_renderers: Vec<TemplateRendererRef>,
    pub artifacts: Vec<ArtifactEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetDatabase {
    pub kind: String,
    pub version: String,
    pub compatibility_floor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRef {
    pub pack: String,
    pub version: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateRendererRef {
    pub profile: String,
    pub engine: String,
    pub profile_version: String,
    pub limits: RendererLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererLimits {
    pub max_fuel: u64,
    pub max_output_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub path: PathBuf,
    pub kind: String,
    pub artifact_id: String,
    pub generator: String,
    pub generator_version: String,
    pub hash: String,
    pub metadata_object_ids: Vec<String>,
    pub trace: Vec<ArtifactTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTrace {
    pub artifact_span: ArtifactSpan,
    pub metadata_object_ids: Vec<String>,
    pub policy_rule_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSpan {
    pub line_start: u32,
    pub line_end: u32,
}

impl ManifestV1 {
    /// # Panics
    /// Panics if serde serialization fails (should never happen for this type).
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("ManifestV1 is always serializable")
    }

    /// # Panics
    /// Panics if serde serialization fails (should never happen for this type).
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ManifestV1 is always serializable")
    }

    /// # Errors
    /// Returns a serde_json error if the input is not valid ManifestV1 JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> ManifestV1 {
        ManifestV1 {
            engine_version: "1.0.0".into(),
            metadata_schema_version: "1.0".into(),
            generator_contract_version: "1.0".into(),
            project_id: "ecommerce_demo".into(),
            input_hash: "sha256:abc123".into(),
            lockfile_hash: "sha256:def456".into(),
            target_database: TargetDatabase {
                kind: "postgres".into(),
                version: "18".into(),
                compatibility_floor: "16".into(),
            },
            policy: PolicyRef {
                pack: "dc_standard".into(),
                version: "1.0".into(),
                hash: "sha256:pol789".into(),
            },
            template_renderers: vec![TemplateRendererRef {
                profile: "know-now-minijinja-v1".into(),
                engine: "minijinja".into(),
                profile_version: "1".into(),
                limits: RendererLimits {
                    max_fuel: 50000,
                    max_output_bytes: 10_485_760,
                },
            }],
            artifacts: vec![ArtifactEntry {
                path: PathBuf::from("generated/ddl/postgres/schema.sql"),
                kind: "postgres_ddl".into(),
                artifact_id: "art_postgres_schema".into(),
                generator: "know_now_gen_postgres".into(),
                generator_version: "1.0.0".into(),
                hash: "sha256:art001".into(),
                metadata_object_ids: vec!["ent_customer".into(), "attr_customer_email".into()],
                trace: vec![ArtifactTrace {
                    artifact_span: ArtifactSpan {
                        line_start: 12,
                        line_end: 18,
                    },
                    metadata_object_ids: vec!["ent_customer".into(), "attr_customer_email".into()],
                    policy_rule_ids: vec!["pol_email_max_length".into()],
                }],
            }],
            warnings: vec![],
        }
    }

    #[test]
    fn pretty_print_roundtrip() {
        let manifest = sample_manifest();
        let json = manifest.to_json_pretty();
        let parsed = ManifestV1::from_json(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn compact_roundtrip() {
        let manifest = sample_manifest();
        let json = manifest.to_json();
        let parsed = ManifestV1::from_json(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn pretty_print_is_deterministic() {
        let manifest = sample_manifest();
        let json1 = manifest.to_json_pretty();
        let json2 = manifest.to_json_pretty();
        assert_eq!(json1, json2);
    }

    #[test]
    fn no_timestamp_or_machine_fields() {
        let manifest = sample_manifest();
        let json = manifest.to_json_pretty();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();

        for key in [
            "timestamp",
            "hostname",
            "username",
            "machine_id",
            "run_id",
            "started_at",
            "finished_at",
            "duration_ms",
        ] {
            assert!(
                !obj.contains_key(key),
                "manifest must not contain volatile field: {key}"
            );
        }
    }

    #[test]
    fn artifacts_sorted_by_path() {
        let mut manifest = sample_manifest();
        manifest.artifacts.push(ArtifactEntry {
            path: PathBuf::from("generated/ddl/postgres/a_first.sql"),
            kind: "postgres_ddl".into(),
            artifact_id: "art_first".into(),
            generator: "know_now_gen_postgres".into(),
            generator_version: "1.0.0".into(),
            hash: "sha256:art002".into(),
            metadata_object_ids: vec![],
            trace: vec![],
        });

        manifest.artifacts.sort_by(|a, b| a.path.cmp(&b.path));

        assert_eq!(
            manifest.artifacts[0].path,
            PathBuf::from("generated/ddl/postgres/a_first.sql")
        );
        assert_eq!(
            manifest.artifacts[1].path,
            PathBuf::from("generated/ddl/postgres/schema.sql")
        );
    }

    #[test]
    fn hash_prefix_convention() {
        let manifest = sample_manifest();
        assert!(manifest.input_hash.starts_with("sha256:"));
        assert!(manifest.lockfile_hash.starts_with("sha256:"));
        assert!(manifest.policy.hash.starts_with("sha256:"));
        for artifact in &manifest.artifacts {
            assert!(artifact.hash.starts_with("sha256:"));
        }
    }

    #[test]
    fn matches_prd_8_11_field_names() {
        let manifest = sample_manifest();
        let json = manifest.to_json_pretty();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();

        let required_fields = [
            "engine_version",
            "metadata_schema_version",
            "generator_contract_version",
            "project_id",
            "input_hash",
            "lockfile_hash",
            "target_database",
            "policy",
            "template_renderers",
            "artifacts",
            "warnings",
        ];

        for field in required_fields {
            assert!(
                obj.contains_key(field),
                "manifest missing required field: {field}"
            );
        }
    }

    #[test]
    fn manifest_json_uses_lf_line_endings() {
        let manifest = sample_manifest();
        let json = manifest.to_json_pretty();
        assert!(
            !json.contains('\r'),
            "manifest JSON must use LF line endings, not CRLF (NFR-PO3)"
        );
    }

    #[test]
    fn manifest_json_is_valid_utf8_without_bom() {
        let manifest = sample_manifest();
        let json = manifest.to_json_pretty();
        let bytes = json.as_bytes();
        assert!(
            !bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
            "manifest JSON must not contain a UTF-8 BOM (NFR-PO3)"
        );
    }

    #[test]
    fn artifact_paths_use_forward_slashes() {
        let manifest = sample_manifest();
        for artifact in &manifest.artifacts {
            let path_str = artifact.path.to_string_lossy();
            assert!(
                !path_str.contains('\\'),
                "artifact path must use forward slashes for cross-platform determinism: {path_str}"
            );
        }
    }

    #[test]
    fn non_ascii_identifiers_roundtrip() {
        let mut manifest = sample_manifest();
        manifest.project_id = "café_données".into();
        manifest.artifacts[0].metadata_object_ids = vec!["ent_données".into(), "attr_名前".into()];

        let json = manifest.to_json_pretty();
        let parsed = ManifestV1::from_json(&json).unwrap();
        assert_eq!(manifest, parsed);

        assert!(json.contains("café_données"));
        assert!(json.contains("ent_données"));
        assert!(json.contains("attr_名前"));
    }
}
