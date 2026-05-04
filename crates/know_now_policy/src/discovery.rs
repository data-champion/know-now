use std::path::Path;

use serde::Serialize;

use crate::manifest::PackManifest;

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredPack {
    pub source: PackSource,
    pub manifest: PackManifest,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackSource {
    BuiltIn,
    ProjectLocal,
    Custom,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackTriple {
    pub name: String,
    pub configured_version: Option<String>,
    pub locked_version: Option<String>,
    pub available_version: Option<String>,
    pub source: Option<PackSource>,
}

pub fn discover_packs(project_root: &Path) -> Vec<DiscoveredPack> {
    let mut packs = Vec::new();

    discover_directory_packs(&project_root.join("policy"), &PackSource::ProjectLocal, &mut packs);
    discover_directory_packs(
        &project_root.join("custom").join("policy"),
        &PackSource::Custom,
        &mut packs,
    );

    packs
}

fn discover_directory_packs(dir: &Path, source: &PackSource, packs: &mut Vec<DiscoveredPack>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && is_pack_manifest(&path) {
            if let Some(pack) = load_pack_manifest(&path, source) {
                packs.push(pack);
            }
            continue;
        }

        if path.is_dir() {
            let manifest_path = path.join("pack.json");
            if manifest_path.exists() {
                if let Some(pack) = load_pack_manifest(&manifest_path, source) {
                    packs.push(pack);
                }
            }
        }
    }
}

fn is_pack_manifest(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "json")
        && path.file_stem().is_some_and(|stem| {
            let s = stem.to_string_lossy();
            s.ends_with(".pack") || s == "pack"
        })
}

fn load_pack_manifest(path: &Path, source: &PackSource) -> Option<DiscoveredPack> {
    let content = std::fs::read_to_string(path).ok()?;
    let manifest: PackManifest = serde_json::from_str(&content).ok()?;
    Some(DiscoveredPack {
        source: source.clone(),
        manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_project_local_packs() {
        let tmp = tempfile::tempdir().unwrap();
        let policy_dir = tmp.path().join("policy");
        std::fs::create_dir(&policy_dir).unwrap();

        let manifest = r#"{
            "name": "local_pack",
            "version": "1.0.0",
            "rules": []
        }"#;
        std::fs::write(policy_dir.join("local_pack.pack.json"), manifest).unwrap();

        let packs = discover_packs(tmp.path());
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].manifest.name, "local_pack");
        assert_eq!(packs[0].source, PackSource::ProjectLocal);
    }

    #[test]
    fn discover_finds_custom_packs() {
        let tmp = tempfile::tempdir().unwrap();
        let custom_policy = tmp.path().join("custom").join("policy");
        std::fs::create_dir_all(&custom_policy).unwrap();

        let pack_dir = custom_policy.join("corp_rules");
        std::fs::create_dir(&pack_dir).unwrap();
        let manifest = r#"{
            "name": "corp_rules",
            "version": "2.0.0",
            "rules": []
        }"#;
        std::fs::write(pack_dir.join("pack.json"), manifest).unwrap();

        let packs = discover_packs(tmp.path());
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].manifest.name, "corp_rules");
        assert_eq!(packs[0].source, PackSource::Custom);
    }

    #[test]
    fn discover_returns_empty_when_no_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let packs = discover_packs(tmp.path());
        assert!(packs.is_empty());
    }

    #[test]
    fn discover_ignores_invalid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let policy_dir = tmp.path().join("policy");
        std::fs::create_dir(&policy_dir).unwrap();
        std::fs::write(policy_dir.join("bad.pack.json"), "not json").unwrap();

        let packs = discover_packs(tmp.path());
        assert!(packs.is_empty());
    }

    #[test]
    fn pack_triple_serializes() {
        let triple = PackTriple {
            name: "dc_standard".into(),
            configured_version: Some("1.0".into()),
            locked_version: Some("1.0".into()),
            available_version: Some("1.0".into()),
            source: Some(PackSource::BuiltIn),
        };
        let json = serde_json::to_string(&triple).unwrap();
        assert!(json.contains("dc_standard"));
        assert!(json.contains("built_in"));
    }
}
