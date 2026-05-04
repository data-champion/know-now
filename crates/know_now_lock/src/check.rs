use crate::lockfile::Lockfile;
use crate::resolved::ResolvedVersions;
use crate::SchemaVersion;

pub const LOCK_SCHEMA_001: &str = "LOCK-SCHEMA-001";
pub const LOCK_CONTRACT_002: &str = "LOCK-CONTRACT-002";
pub const LOCK_STALE_003: &str = "LOCK-STALE-003";
pub const LOCK_MISSING_004: &str = "LOCK-MISSING-004";
pub const LOCK_CORRUPT_005: &str = "LOCK-CORRUPT-005";
pub const LOCK_UNKNOWN_006: &str = "LOCK-UNKNOWN-006";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DriftedField {
    pub field: String,
    pub locked_value: String,
    pub resolved_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockCheckError {
    SchemaMismatch {
        lockfile_version: String,
        engine_version: String,
    },
    ContractBreaking {
        locked: String,
        resolved: String,
    },
    Stale(Vec<DriftedField>),
    Missing,
    Corrupt(String),
}

#[derive(Debug)]
pub struct LockCheckResult {
    pub drifted_fields: Vec<DriftedField>,
    pub warnings: Vec<String>,
    pub error: Option<LockCheckError>,
}

impl LockCheckResult {
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.error.is_none() && self.drifted_fields.is_empty()
    }
}

#[must_use]
pub fn check_lockfile(lockfile: &Lockfile, resolved: &ResolvedVersions) -> LockCheckResult {
    let mut result = LockCheckResult {
        drifted_fields: Vec::new(),
        warnings: Vec::new(),
        error: None,
    };

    if SchemaVersion::from_version_str(&lockfile.lockfile_schema_version).is_none() {
        result.error = Some(LockCheckError::SchemaMismatch {
            lockfile_version: lockfile.lockfile_schema_version.clone(),
            engine_version: SchemaVersion::current().as_str().to_owned(),
        });
        return result;
    }

    let unknown = lockfile.unknown_field_names();
    if !unknown.is_empty() {
        for field in &unknown {
            result.warnings.push(format!(
                "{LOCK_UNKNOWN_006}: lockfile contains unrecognized field '{field}'"
            ));
        }
    }

    compare_field(
        &mut result.drifted_fields,
        "engine_version",
        &lockfile.engine_version,
        &resolved.engine_version,
    );
    compare_field(
        &mut result.drifted_fields,
        "metadata_schema_version",
        &lockfile.metadata_schema_version,
        &resolved.metadata_schema_version,
    );
    compare_field(
        &mut result.drifted_fields,
        "generator_contract_version",
        &lockfile.generator_contract_version,
        &resolved.generator_contract_version,
    );
    compare_field(
        &mut result.drifted_fields,
        "policy.pack",
        &lockfile.policy.pack,
        &resolved.policy_pack,
    );
    compare_field(
        &mut result.drifted_fields,
        "policy.version",
        &lockfile.policy.version,
        &resolved.policy_version,
    );
    compare_field(
        &mut result.drifted_fields,
        "policy.hash",
        &lockfile.policy.hash,
        &resolved.policy_hash,
    );

    for (name, locked_gen) in &lockfile.generators {
        if let Some(resolved_version) = resolved.generators.get(name) {
            compare_field(
                &mut result.drifted_fields,
                &format!("generators.{name}.version"),
                &locked_gen.version,
                resolved_version,
            );
        } else {
            result.drifted_fields.push(DriftedField {
                field: format!("generators.{name}"),
                locked_value: locked_gen.version.clone(),
                resolved_value: "(removed)".to_owned(),
            });
        }
    }
    for (name, version) in &resolved.generators {
        if !lockfile.generators.contains_key(name) {
            result.drifted_fields.push(DriftedField {
                field: format!("generators.{name}"),
                locked_value: "(absent)".to_owned(),
                resolved_value: version.clone(),
            });
        }
    }

    if !result.drifted_fields.is_empty() {
        result.error = Some(LockCheckError::Stale(result.drifted_fields.clone()));
    }

    result
}

#[must_use]
pub fn is_contract_breaking(locked_version: &str, resolved_version: &str) -> bool {
    let locked_major = locked_version.split('.').next().unwrap_or("0");
    let resolved_major = resolved_version.split('.').next().unwrap_or("0");
    locked_major != resolved_major
}

fn compare_field(drifted: &mut Vec<DriftedField>, field: &str, locked: &str, resolved: &str) {
    if locked != resolved {
        drifted.push(DriftedField {
            field: field.to_owned(),
            locked_value: locked.to_owned(),
            resolved_value: resolved.to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::{GeneratorLock, Lockfile, PolicyLock};
    use std::collections::BTreeMap;

    fn matching_pair() -> (Lockfile, ResolvedVersions) {
        let mut generators = BTreeMap::new();
        generators.insert(
            "gen_pg".to_owned(),
            GeneratorLock {
                version: "0.1.0".to_owned(),
            },
        );

        let lockfile = Lockfile {
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
        };

        let mut res_generators = BTreeMap::new();
        res_generators.insert("gen_pg".to_owned(), "0.1.0".to_owned());

        let resolved = ResolvedVersions {
            engine_version: "0.1.0".to_owned(),
            metadata_schema_version: "0.1.0".to_owned(),
            generator_contract_version: "1.0".to_owned(),
            generators: res_generators,
            policy_pack: "dc_standard".to_owned(),
            policy_version: "1.0".to_owned(),
            policy_hash: "sha256:embedded".to_owned(),
            target_profiles: vec![],
            semantic_type_mappings: BTreeMap::new(),
        };

        (lockfile, resolved)
    }

    #[test]
    fn no_drift_when_matching() {
        let (lockfile, resolved) = matching_pair();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(result.is_ok());
        assert!(result.drifted_fields.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn detects_engine_version_drift() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.engine_version = "0.2.0".to_owned();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(result
            .drifted_fields
            .iter()
            .any(|d| d.field == "engine_version"));
    }

    #[test]
    fn detects_policy_drift() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.policy.version = "2.0".to_owned();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(result
            .drifted_fields
            .iter()
            .any(|d| d.field == "policy.version"));
    }

    #[test]
    fn detects_generator_version_drift() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.generators.get_mut("gen_pg").unwrap().version = "0.2.0".to_owned();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(result
            .drifted_fields
            .iter()
            .any(|d| d.field == "generators.gen_pg.version"));
    }

    #[test]
    fn detects_removed_generator() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.generators.insert(
            "gen_removed".to_owned(),
            GeneratorLock {
                version: "1.0.0".to_owned(),
            },
        );
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(result
            .drifted_fields
            .iter()
            .any(|d| d.field == "generators.gen_removed" && d.resolved_value == "(removed)"));
    }

    #[test]
    fn detects_added_generator() {
        let (lockfile, mut resolved) = matching_pair();
        resolved
            .generators
            .insert("gen_new".to_owned(), "1.0.0".to_owned());
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(result
            .drifted_fields
            .iter()
            .any(|d| d.field == "generators.gen_new" && d.locked_value == "(absent)"));
    }

    #[test]
    fn schema_mismatch_is_error() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.lockfile_schema_version = "99.0".to_owned();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(!result.is_ok());
        assert!(matches!(
            result.error,
            Some(LockCheckError::SchemaMismatch { .. })
        ));
    }

    #[test]
    fn unknown_fields_produce_warnings() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile
            .unknown_fields
            .insert("from_future".to_owned(), serde_json::Value::Bool(true));
        let result = check_lockfile(&lockfile, &resolved);
        assert!(result.is_ok());
        assert!(result.warnings.iter().any(|w| w.contains(LOCK_UNKNOWN_006)));
    }

    #[test]
    fn contract_breaking_detects_major_change() {
        assert!(is_contract_breaking("1.0", "2.0"));
        assert!(!is_contract_breaking("1.0", "1.1"));
        assert!(!is_contract_breaking("1.0", "1.0"));
    }

    #[test]
    fn multiple_drifts_all_reported() {
        let (mut lockfile, resolved) = matching_pair();
        lockfile.engine_version = "0.2.0".to_owned();
        lockfile.policy.pack = "custom".to_owned();
        let result = check_lockfile(&lockfile, &resolved);
        assert!(result.drifted_fields.len() >= 2);
    }
}
