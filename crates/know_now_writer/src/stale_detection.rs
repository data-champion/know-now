use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::manifest::ManifestV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleArtifact {
    pub path: PathBuf,
    pub generator: String,
    pub artifact_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntrackedFile {
    pub path: PathBuf,
}

pub fn detect_stale(
    previous_manifest: &ManifestV1,
    new_manifest: &ManifestV1,
) -> Vec<StaleArtifact> {
    let new_paths: BTreeSet<&Path> = new_manifest
        .artifacts
        .iter()
        .map(|a| a.path.as_path())
        .collect();

    previous_manifest
        .artifacts
        .iter()
        .filter(|a| !new_paths.contains(a.path.as_path()))
        .map(|a| StaleArtifact {
            path: a.path.clone(),
            generator: a.generator.clone(),
            artifact_id: a.artifact_id.clone(),
        })
        .collect()
}

pub fn detect_untracked(manifest: &ManifestV1, target_dir: &Path) -> Vec<UntrackedFile> {
    let manifest_paths: BTreeSet<PathBuf> =
        manifest.artifacts.iter().map(|a| a.path.clone()).collect();

    let mut untracked = Vec::new();
    collect_files_recursive(target_dir, target_dir, &manifest_paths, &mut untracked);
    untracked.sort_by(|a, b| a.path.cmp(&b.path));
    untracked
}

fn collect_files_recursive(
    base: &Path,
    dir: &Path,
    manifest_paths: &BTreeSet<PathBuf>,
    out: &mut Vec<UntrackedFile>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, manifest_paths, out);
        } else {
            let Ok(relative) = path.strip_prefix(base) else {
                continue;
            };
            if relative.as_os_str() == "manifest.json" {
                continue;
            }
            if let Some(name) = relative.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }
            if !manifest_paths.contains(relative) {
                out.push(UntrackedFile {
                    path: relative.to_path_buf(),
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneResult {
    pub deleted: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

pub fn prune_stale(stale: &[StaleArtifact], target_dir: &Path) -> PruneResult {
    let mut deleted = Vec::new();
    let mut warnings = Vec::new();

    for artifact in stale {
        let file_path = target_dir.join(&artifact.path);
        if !file_path.exists() {
            warnings.push(format!(
                "WRITER-STALE-MISSING: stale artifact '{}' already absent from disk",
                artifact.path.display()
            ));
            continue;
        }
        match std::fs::remove_file(&file_path) {
            Ok(()) => deleted.push(artifact.path.clone()),
            Err(e) => warnings.push(format!(
                "WRITER-STALE-DELETE-FAIL: could not delete '{}': {e}",
                artifact.path.display()
            )),
        }
    }

    deleted.sort();
    PruneResult { deleted, warnings }
}

pub fn stale_warnings(stale: &[StaleArtifact]) -> Vec<String> {
    stale
        .iter()
        .map(|a| {
            format!(
                "WRITER-STALE: '{}' (generator: {}, id: {}) is no longer in the generation plan",
                a.path.display(),
                a.generator,
                a.artifact_id,
            )
        })
        .collect()
}

pub fn untracked_warnings(untracked: &[UntrackedFile]) -> Vec<String> {
    untracked
        .iter()
        .map(|u| {
            format!(
                "WRITER-UNTRACKED: '{}' is not tracked by the manifest and will not be deleted",
                u.path.display()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        ArtifactEntry, ArtifactSpan, ArtifactTrace, ManifestV1, PolicyRef, TargetDatabase,
    };
    use crate::manifest_builder::sha256_hex;
    use std::fs;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("know_now_stale_{name}_{}", std::process::id()))
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
            artifact_id: format!("art_{}", path.replace('/', "_")),
            generator: "test_gen".into(),
            generator_version: "1.0".into(),
            hash: sha256_hex(content),
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

    // ── detect_stale ──────────────────────────────────────────

    #[test]
    fn no_stale_when_manifests_match() {
        let old = make_manifest(vec![make_artifact("a.sql", b"A")]);
        let new = make_manifest(vec![make_artifact("a.sql", b"A2")]);
        assert!(detect_stale(&old, &new).is_empty());
    }

    #[test]
    fn detects_stale_artifact() {
        let old = make_manifest(vec![
            make_artifact("customer.sql", b"C"),
            make_artifact("order.sql", b"O"),
        ]);
        let new = make_manifest(vec![make_artifact("customer.sql", b"C2")]);
        let stale = detect_stale(&old, &new);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].path, PathBuf::from("order.sql"));
    }

    #[test]
    fn entity_rename_produces_stale() {
        let old = make_manifest(vec![
            make_artifact("docs/customer.md", b"# Customer"),
            make_artifact("ddl/customer.sql", b"CREATE TABLE customer"),
        ]);
        let new = make_manifest(vec![
            make_artifact("docs/client.md", b"# Client"),
            make_artifact("ddl/client.sql", b"CREATE TABLE client"),
        ]);
        let stale = detect_stale(&old, &new);
        assert_eq!(stale.len(), 2);
        let paths: BTreeSet<_> = stale.iter().map(|s| s.path.clone()).collect();
        assert!(paths.contains(&PathBuf::from("docs/customer.md")));
        assert!(paths.contains(&PathBuf::from("ddl/customer.sql")));
    }

    #[test]
    fn all_removed_are_stale() {
        let old = make_manifest(vec![
            make_artifact("a.sql", b"A"),
            make_artifact("b.sql", b"B"),
        ]);
        let new = make_manifest(vec![]);
        let stale = detect_stale(&old, &new);
        assert_eq!(stale.len(), 2);
    }

    #[test]
    fn new_artifacts_not_stale() {
        let old = make_manifest(vec![]);
        let new = make_manifest(vec![make_artifact("new.sql", b"NEW")]);
        assert!(detect_stale(&old, &new).is_empty());
    }

    // ── detect_untracked ──────────────────────────────────────

    #[test]
    fn no_untracked_when_all_manifest_tracked() {
        let dir = test_dir("no_untracked");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.sql"), "A").unwrap();

        let manifest = make_manifest(vec![make_artifact("a.sql", b"A")]);
        let untracked = detect_untracked(&manifest, &dir);
        assert!(untracked.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_untracked_file() {
        let dir = test_dir("untracked");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tracked.sql"), "T").unwrap();
        fs::write(dir.join("rogue.txt"), "R").unwrap();

        let manifest = make_manifest(vec![make_artifact("tracked.sql", b"T")]);
        let untracked = detect_untracked(&manifest, &dir);
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, PathBuf::from("rogue.txt"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifest_json_is_not_untracked() {
        let dir = test_dir("manifest_skip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"), "{}").unwrap();

        let manifest = make_manifest(vec![]);
        let untracked = detect_untracked(&manifest, &dir);
        assert!(untracked.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dotfiles_are_not_untracked() {
        let dir = test_dir("dotfile_skip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".gitkeep"), "").unwrap();
        fs::write(dir.join(".gitignore"), "*.bak").unwrap();
        fs::write(dir.join("real.sql"), "SELECT 1").unwrap();

        let manifest = make_manifest(vec![]);
        let untracked = detect_untracked(&manifest, &dir);
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, PathBuf::from("real.sql"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn untracked_in_subdirectory() {
        let dir = test_dir("untracked_sub");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("ddl")).unwrap();
        fs::write(dir.join("ddl/tracked.sql"), "T").unwrap();
        fs::write(dir.join("ddl/extra.sql"), "E").unwrap();

        let manifest = make_manifest(vec![make_artifact("ddl/tracked.sql", b"T")]);
        let untracked = detect_untracked(&manifest, &dir);
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, PathBuf::from("ddl/extra.sql"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── prune_stale ───────────────────────────────────────────

    #[test]
    fn prune_deletes_stale_files() {
        let dir = test_dir("prune");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("keep.sql"), "KEEP").unwrap();
        fs::write(dir.join("stale.sql"), "STALE").unwrap();

        let stale = vec![StaleArtifact {
            path: PathBuf::from("stale.sql"),
            generator: "test_gen".into(),
            artifact_id: "art_stale".into(),
        }];
        let result = prune_stale(&stale, &dir);
        assert_eq!(result.deleted, vec![PathBuf::from("stale.sql")]);
        assert!(result.warnings.is_empty());
        assert!(!dir.join("stale.sql").exists());
        assert!(dir.join("keep.sql").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_does_not_delete_untracked_files() {
        let dir = test_dir("prune_safe");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tracked.sql"), "T").unwrap();
        fs::write(dir.join("untracked.txt"), "U").unwrap();

        let old = make_manifest(vec![
            make_artifact("tracked.sql", b"T"),
            make_artifact("removed.sql", b"R"),
        ]);
        let new = make_manifest(vec![make_artifact("tracked.sql", b"T2")]);
        let stale = detect_stale(&old, &new);

        let result = prune_stale(&stale, &dir);
        assert!(dir.join("untracked.txt").exists());
        assert!(dir.join("tracked.sql").exists());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("WRITER-STALE-MISSING")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_already_missing_produces_warning() {
        let dir = test_dir("prune_miss");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let stale = vec![StaleArtifact {
            path: PathBuf::from("gone.sql"),
            generator: "test_gen".into(),
            artifact_id: "art_gone".into(),
        }];
        let result = prune_stale(&stale, &dir);
        assert!(result.deleted.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("WRITER-STALE-MISSING")));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── warnings ──────────────────────────────────────────────

    #[test]
    fn stale_warnings_contain_path_and_generator() {
        let stale = vec![StaleArtifact {
            path: PathBuf::from("ddl/old_entity.sql"),
            generator: "know_now_gen_postgres".into(),
            artifact_id: "art_old".into(),
        }];
        let warnings = stale_warnings(&stale);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("WRITER-STALE:"));
        assert!(warnings[0].contains("ddl/old_entity.sql"));
        assert!(warnings[0].contains("know_now_gen_postgres"));
    }

    #[test]
    fn untracked_warnings_contain_path() {
        let untracked = vec![UntrackedFile {
            path: PathBuf::from("rogue.txt"),
        }];
        let warnings = untracked_warnings(&untracked);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("WRITER-UNTRACKED:"));
        assert!(warnings[0].contains("rogue.txt"));
        assert!(warnings[0].contains("will not be deleted"));
    }

    // ── integration: full stale + untracked flow ──────────────

    #[test]
    fn full_flow_entity_rename_with_prune() {
        let dir = test_dir("full_flow");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs")).unwrap();
        fs::create_dir_all(dir.join("ddl")).unwrap();
        fs::write(dir.join("docs/customer.md"), "# Customer").unwrap();
        fs::write(dir.join("ddl/customer.sql"), "CREATE TABLE customer").unwrap();
        fs::write(dir.join("notes.txt"), "user notes").unwrap();

        let old_manifest = make_manifest(vec![
            make_artifact("docs/customer.md", b"# Customer"),
            make_artifact("ddl/customer.sql", b"CREATE TABLE customer"),
        ]);
        let new_manifest = make_manifest(vec![
            make_artifact("docs/client.md", b"# Client"),
            make_artifact("ddl/client.sql", b"CREATE TABLE client"),
        ]);

        let stale = detect_stale(&old_manifest, &new_manifest);
        assert_eq!(stale.len(), 2);

        let untracked = detect_untracked(&old_manifest, &dir);
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, PathBuf::from("notes.txt"));

        let result = prune_stale(&stale, &dir);
        assert_eq!(result.deleted.len(), 2);
        assert!(result.warnings.is_empty());

        assert!(!dir.join("docs/customer.md").exists());
        assert!(!dir.join("ddl/customer.sql").exists());
        assert!(dir.join("notes.txt").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
