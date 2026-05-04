use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lockfile {
    pub lockfile_schema_version: String,
    pub engine_version: String,
    pub metadata_schema_version: String,
    pub generator_contract_version: String,
    pub generators: BTreeMap<String, GeneratorLock>,
    pub policy: PolicyLock,
    pub target_compatibility: Vec<TargetProfile>,
    pub semantic_type_mappings: BTreeMap<String, String>,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratorLock {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyLock {
    pub pack: String,
    pub version: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetProfile {
    pub kind: String,
    pub version: String,
    pub compatibility_floor: String,
}

#[derive(Debug)]
pub enum LockfileError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for LockfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "lockfile I/O error: {e}"),
            Self::Parse(e) => write!(f, "lockfile parse error: {e}"),
        }
    }
}

impl std::error::Error for LockfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
        }
    }
}

impl Lockfile {
    /// # Panics
    /// Panics if the lockfile cannot be serialized (should never happen for valid data).
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("lockfile should serialize")
    }

    /// # Errors
    /// Returns `LockfileError::Parse` if the JSON is malformed or missing required fields.
    pub fn from_json(json: &str) -> Result<Self, LockfileError> {
        serde_json::from_str(json).map_err(LockfileError::Parse)
    }

    /// # Errors
    /// Returns `LockfileError::Io` if the file cannot be read, or `LockfileError::Parse`
    /// if its content is not valid lockfile JSON.
    pub fn read_from(path: &Path) -> Result<Self, LockfileError> {
        let content = std::fs::read_to_string(path).map_err(LockfileError::Io)?;
        Self::from_json(&content)
    }

    /// # Errors
    /// Returns `LockfileError::Io` if the file cannot be written.
    pub fn write_to(&self, path: &Path) -> Result<(), LockfileError> {
        let mut content = self.to_json_pretty();
        if !content.ends_with('\n') {
            content.push('\n');
        }
        std::fs::write(path, content).map_err(LockfileError::Io)
    }

    #[must_use]
    pub fn unknown_field_names(&self) -> Vec<String> {
        self.unknown_fields.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lockfile() -> Lockfile {
        let mut generators = BTreeMap::new();
        generators.insert(
            "know_now_gen_postgres".to_owned(),
            GeneratorLock {
                version: "0.1.0".to_owned(),
            },
        );
        generators.insert(
            "know_now_gen_docs".to_owned(),
            GeneratorLock {
                version: "0.1.0".to_owned(),
            },
        );

        Lockfile {
            lockfile_schema_version: "1.0".to_owned(),
            engine_version: "0.1.0".to_owned(),
            metadata_schema_version: "0.1.0".to_owned(),
            generator_contract_version: "1.0".to_owned(),
            generators,
            policy: PolicyLock {
                pack: "dc_standard".to_owned(),
                version: "1.0".to_owned(),
                hash: "sha256:embedded".to_owned(),
            },
            target_compatibility: vec![],
            semantic_type_mappings: BTreeMap::new(),
            unknown_fields: BTreeMap::new(),
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let lockfile = sample_lockfile();
        let json = lockfile.to_json_pretty();
        let parsed = Lockfile::from_json(&json).unwrap();
        assert_eq!(lockfile, parsed);
    }

    #[test]
    fn serialization_is_deterministic() {
        let lockfile = sample_lockfile();
        let json1 = lockfile.to_json_pretty();
        let json2 = lockfile.to_json_pretty();
        assert_eq!(json1, json2);
    }

    #[test]
    fn unknown_fields_are_preserved() {
        let mut lockfile = sample_lockfile();
        lockfile.unknown_fields.insert(
            "future_field".to_owned(),
            serde_json::Value::String("future_value".to_owned()),
        );
        let json = lockfile.to_json_pretty();
        let parsed = Lockfile::from_json(&json).unwrap();
        assert_eq!(parsed.unknown_field_names(), vec!["future_field"]);
    }

    #[test]
    fn unknown_fields_roundtrip_from_raw_json() {
        let json = r#"{
            "lockfile_schema_version": "1.0",
            "engine_version": "0.1.0",
            "metadata_schema_version": "0.1.0",
            "generator_contract_version": "1.0",
            "generators": {},
            "policy": { "pack": "dc_standard", "version": "1.0", "hash": "sha256:embedded" },
            "target_compatibility": [],
            "semantic_type_mappings": {},
            "from_the_future": true
        }"#;
        let parsed = Lockfile::from_json(json).unwrap();
        assert_eq!(parsed.unknown_field_names(), vec!["from_the_future"]);
    }

    #[test]
    fn corrupt_json_returns_parse_error() {
        let result = Lockfile::from_json("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn missing_required_field_returns_parse_error() {
        let json = r#"{"lockfile_schema_version": "1.0"}"#;
        let result = Lockfile::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn write_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("kn_lock_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("know-now.lock");

        let lockfile = sample_lockfile();
        lockfile.write_to(&path).unwrap();
        let read_back = Lockfile::read_from(&path).unwrap();
        assert_eq!(lockfile, read_back);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generators_sorted_by_btreemap() {
        let lockfile = sample_lockfile();
        let json = lockfile.to_json_pretty();
        let docs_pos = json.find("know_now_gen_docs").unwrap();
        let pg_pos = json.find("know_now_gen_postgres").unwrap();
        assert!(
            docs_pos < pg_pos,
            "generators should be sorted alphabetically"
        );
    }

    #[test]
    fn written_file_ends_with_newline() {
        let dir = std::env::temp_dir().join(format!("kn_lock_nl_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("know-now.lock");

        let lockfile = sample_lockfile();
        lockfile.write_to(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.ends_with('\n'));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
