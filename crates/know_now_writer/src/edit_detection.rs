use std::path::Path;

use crate::manifest::ManifestV1;
use crate::manifest_builder::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditDetectionResult {
    pub edited: Vec<EditedArtifact>,
    pub missing: Vec<MissingArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditedArtifact {
    pub path: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingArtifact {
    pub path: String,
}

pub fn detect_edits(previous_manifest: &ManifestV1, target_dir: &Path) -> EditDetectionResult {
    let mut edited = Vec::new();
    let mut missing = Vec::new();

    for artifact in &previous_manifest.artifacts {
        let file_path = target_dir.join(&artifact.path);
        if !file_path.exists() {
            missing.push(MissingArtifact {
                path: artifact.path.display().to_string(),
            });
            continue;
        }

        let Ok(content) = std::fs::read(&file_path) else {
            missing.push(MissingArtifact {
                path: artifact.path.display().to_string(),
            });
            continue;
        };

        let actual_hash = hash_content(&content);
        if actual_hash != artifact.hash {
            edited.push(EditedArtifact {
                path: artifact.path.display().to_string(),
                expected_hash: artifact.hash.clone(),
                actual_hash,
            });
        }
    }

    EditDetectionResult { edited, missing }
}

pub fn has_blocking_edits(result: &EditDetectionResult) -> bool {
    !result.edited.is_empty()
}

fn hash_content(content: &[u8]) -> String {
    sha256_hex(content)
}

fn load_previous_manifest(target_dir: &Path) -> Option<ManifestV1> {
    let manifest_path = target_dir.join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    ManifestV1::from_json(&content).ok()
}

/// # Errors
/// Returns `Err` with the list of edited artifacts if manual edits are detected
/// and `accept_overwrite` is false.
pub fn check_for_manual_edits(
    target_dir: &Path,
    accept_overwrite: bool,
) -> Result<Vec<String>, Vec<EditedArtifact>> {
    let mut warnings = Vec::new();

    let Some(previous) = load_previous_manifest(target_dir) else {
        return Ok(warnings);
    };

    let result = detect_edits(&previous, target_dir);

    for m in &result.missing {
        warnings.push(format!(
            "WRITER-MISSING-PRIOR: previously generated file '{}' was deleted; it will be recreated",
            m.path
        ));
    }

    if result.edited.is_empty() {
        return Ok(warnings);
    }

    if accept_overwrite {
        for e in &result.edited {
            warnings.push(format!(
                "WRITER-EDIT-ACCEPTED: manual edit in '{}' will be overwritten (--accept-generated-overwrite)",
                e.path
            ));
        }
        Ok(warnings)
    } else {
        Err(result.edited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        ArtifactEntry, ArtifactSpan, ArtifactTrace, ManifestV1, PolicyRef, TargetDatabase,
    };
    use std::fs;
    use std::path::PathBuf;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("know_now_edit_det_{name}_{}", std::process::id()))
    }

    fn make_manifest(artifacts: Vec<ArtifactEntry>) -> ManifestV1 {
        ManifestV1 {
            engine_version: "0.1.0".into(),
            metadata_schema_version: "0.1.0".into(),
            generator_contract_version: "1.0".into(),
            project_id: "test".into(),
            input_hash: "sha256:input".into(),
            lockfile_hash: "sha256:lock".into(),
            target_database: TargetDatabase {
                kind: "postgres".into(),
                version: "16".into(),
                compatibility_floor: "16".into(),
            },
            policy: PolicyRef {
                pack: "dc_standard".into(),
                version: "1.0".into(),
                hash: "sha256:pol".into(),
            },
            template_renderers: vec![],
            artifacts,
            warnings: vec![],
        }
    }

    fn make_artifact(path: &str, content: &[u8]) -> ArtifactEntry {
        ArtifactEntry {
            path: PathBuf::from(path),
            kind: "test".into(),
            artifact_id: format!("art_{path}"),
            generator: "test_gen".into(),
            generator_version: "1.0".into(),
            hash: hash_content(content),
            metadata_object_ids: vec![],
            trace: vec![ArtifactTrace {
                artifact_span: ArtifactSpan {
                    line_start: 1,
                    line_end: 1,
                },
                metadata_object_ids: vec![],
                policy_rule_ids: vec![],
            }],
        }
    }

    #[test]
    fn no_edits_detected_when_content_matches() {
        let dir = test_dir("no_edit");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let content = b"CREATE TABLE test;";
        fs::write(dir.join("schema.sql"), content).unwrap();

        let manifest = make_manifest(vec![make_artifact("schema.sql", content)]);
        let result = detect_edits(&manifest, &dir);

        assert!(result.edited.is_empty());
        assert!(result.missing.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn edit_detected_when_content_differs() {
        let dir = test_dir("edit");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let original = b"CREATE TABLE test;";
        let modified = b"CREATE TABLE test; -- user added this";
        fs::write(dir.join("schema.sql"), modified).unwrap();

        let manifest = make_manifest(vec![make_artifact("schema.sql", original)]);
        let result = detect_edits(&manifest, &dir);

        assert_eq!(result.edited.len(), 1);
        assert_eq!(result.edited[0].path, "schema.sql");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_detected() {
        let dir = test_dir("missing");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let manifest = make_manifest(vec![make_artifact("gone.sql", b"content")]);
        let result = detect_edits(&manifest, &dir);

        assert!(result.edited.is_empty());
        assert_eq!(result.missing.len(), 1);
        assert_eq!(result.missing[0].path, "gone.sql");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn accept_overwrite_converts_edits_to_warnings() {
        let dir = test_dir("accept");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let original = b"ORIGINAL";
        let modified = b"MODIFIED";
        fs::write(dir.join("schema.sql"), modified).unwrap();

        let manifest = make_manifest(vec![make_artifact("schema.sql", original)]);
        fs::write(dir.join("manifest.json"), manifest.to_json_pretty()).unwrap();

        let result = check_for_manual_edits(&dir, true);
        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(warnings.iter().any(|w| w.contains("WRITER-EDIT-ACCEPTED")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reject_overwrite_returns_edited_artifacts() {
        let dir = test_dir("reject");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let original = b"ORIGINAL";
        let modified = b"MODIFIED";
        fs::write(dir.join("schema.sql"), modified).unwrap();

        let manifest = make_manifest(vec![make_artifact("schema.sql", original)]);
        fs::write(dir.join("manifest.json"), manifest.to_json_pretty()).unwrap();

        let result = check_for_manual_edits(&dir, false);
        assert!(result.is_err());
        let edited = result.unwrap_err();
        assert_eq!(edited.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_previous_manifest_means_no_edits() {
        let dir = test_dir("no_manifest");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = check_for_manual_edits(&dir, false);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_prior_file_produces_warning() {
        let dir = test_dir("missing_warn");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let manifest = make_manifest(vec![make_artifact("deleted.sql", b"content")]);
        fs::write(dir.join("manifest.json"), manifest.to_json_pretty()).unwrap();

        let result = check_for_manual_edits(&dir, false);
        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(warnings.iter().any(|w| w.contains("WRITER-MISSING-PRIOR")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_is_deterministic() {
        let content = b"test content";
        let h1 = hash_content(content);
        let h2 = hash_content(content);
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }
}
