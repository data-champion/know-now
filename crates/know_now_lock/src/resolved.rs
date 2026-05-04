use std::collections::BTreeMap;

use crate::lockfile::{GeneratorLock, Lockfile, PolicyLock, TargetProfile};
use crate::CURRENT_SCHEMA_VERSION;

#[derive(Debug, Clone)]
pub struct ResolvedVersions {
    pub engine_version: String,
    pub metadata_schema_version: String,
    pub generator_contract_version: String,
    pub generators: BTreeMap<String, String>,
    pub policy_pack: String,
    pub policy_version: String,
    pub policy_hash: String,
    pub target_profiles: Vec<TargetProfile>,
    pub semantic_type_mappings: BTreeMap<String, String>,
}

impl ResolvedVersions {
    #[must_use]
    pub fn to_lockfile(&self) -> Lockfile {
        Lockfile {
            lockfile_schema_version: CURRENT_SCHEMA_VERSION.to_owned(),
            engine_version: self.engine_version.clone(),
            metadata_schema_version: self.metadata_schema_version.clone(),
            generator_contract_version: self.generator_contract_version.clone(),
            generators: self
                .generators
                .iter()
                .map(|(k, v)| (k.clone(), GeneratorLock { version: v.clone() }))
                .collect(),
            policy: PolicyLock {
                pack: self.policy_pack.clone(),
                version: self.policy_version.clone(),
                hash: self.policy_hash.clone(),
            },
            target_compatibility: self.target_profiles.clone(),
            semantic_type_mappings: self.semantic_type_mappings.clone(),
            unknown_fields: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resolved() -> ResolvedVersions {
        let mut generators = BTreeMap::new();
        generators.insert("gen_a".to_owned(), "1.0.0".to_owned());
        generators.insert("gen_b".to_owned(), "2.0.0".to_owned());

        ResolvedVersions {
            engine_version: "0.1.0".to_owned(),
            metadata_schema_version: "0.1.0".to_owned(),
            generator_contract_version: "1.0".to_owned(),
            generators,
            policy_pack: "dc_standard".to_owned(),
            policy_version: "1.0".to_owned(),
            policy_hash: "sha256:abc".to_owned(),
            target_profiles: vec![TargetProfile {
                kind: "postgres".to_owned(),
                version: "16".to_owned(),
                compatibility_floor: "14".to_owned(),
            }],
            semantic_type_mappings: BTreeMap::new(),
        }
    }

    #[test]
    fn to_lockfile_sets_schema_version() {
        let resolved = sample_resolved();
        let lockfile = resolved.to_lockfile();
        assert_eq!(lockfile.lockfile_schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn to_lockfile_maps_generators() {
        let resolved = sample_resolved();
        let lockfile = resolved.to_lockfile();
        assert_eq!(lockfile.generators.len(), 2);
        assert_eq!(lockfile.generators["gen_a"].version, "1.0.0");
        assert_eq!(lockfile.generators["gen_b"].version, "2.0.0");
    }

    #[test]
    fn to_lockfile_maps_policy() {
        let resolved = sample_resolved();
        let lockfile = resolved.to_lockfile();
        assert_eq!(lockfile.policy.pack, "dc_standard");
        assert_eq!(lockfile.policy.version, "1.0");
        assert_eq!(lockfile.policy.hash, "sha256:abc");
    }

    #[test]
    fn to_lockfile_maps_target_profiles() {
        let resolved = sample_resolved();
        let lockfile = resolved.to_lockfile();
        assert_eq!(lockfile.target_compatibility.len(), 1);
        assert_eq!(lockfile.target_compatibility[0].kind, "postgres");
    }

    #[test]
    fn to_lockfile_has_no_unknown_fields() {
        let resolved = sample_resolved();
        let lockfile = resolved.to_lockfile();
        assert!(lockfile.unknown_fields.is_empty());
    }

    #[test]
    fn to_lockfile_is_idempotent() {
        let resolved = sample_resolved();
        let l1 = resolved.to_lockfile();
        let l2 = resolved.to_lockfile();
        assert_eq!(l1, l2);
        assert_eq!(l1.to_json_pretty(), l2.to_json_pretty());
    }
}
