use serde::{Deserialize, Serialize};

use crate::artifact::ArtifactKind;

/// Capability declaration for a built-in generator (PRD §8.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorCapability {
    pub name: String,
    pub version: String,
    pub contract_versions: Vec<String>,
    pub artifact_kinds: Vec<ArtifactKind>,
    pub supported_dialects: Vec<DialectSupport>,
    pub supported_logical_types: Vec<String>,
    pub supported_semantic_types: Vec<String>,
    pub validation_gates: Vec<String>,
    pub unsupported_constructs: Vec<String>,
    pub experimental_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialectSupport {
    pub dialect: String,
    pub versions: Vec<String>,
}

/// In-memory capability registry.
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    generators: Vec<GeneratorCapability>,
}

impl CapabilityRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, capability: GeneratorCapability) {
        self.generators.push(capability);
    }

    #[must_use]
    pub fn generators(&self) -> &[GeneratorCapability] {
        &self.generators
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&GeneratorCapability> {
        self.generators.iter().find(|g| g.name == name)
    }

    #[must_use]
    pub fn generators_for_artifact(&self, kind: &ArtifactKind) -> Vec<&GeneratorCapability> {
        self.generators
            .iter()
            .filter(|g| g.artifact_kinds.contains(kind))
            .collect()
    }

    #[must_use]
    pub fn supports_contract_version(&self, name: &str, version: &str) -> bool {
        self.find_by_name(name)
            .is_some_and(|g| g.contract_versions.iter().any(|v| v == version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_capability() -> GeneratorCapability {
        GeneratorCapability {
            name: "know_now_gen_postgres".into(),
            version: "0.1.0".into(),
            contract_versions: vec!["1.0".into()],
            artifact_kinds: vec![ArtifactKind::PostgresDdl],
            supported_dialects: vec![DialectSupport {
                dialect: "postgres".into(),
                versions: vec!["16".into(), "17".into(), "18".into()],
            }],
            supported_logical_types: vec![
                "integer".into(),
                "string".into(),
                "boolean".into(),
                "timestamp".into(),
            ],
            supported_semantic_types: vec!["email".into(), "url".into()],
            validation_gates: vec!["parse_validation".into()],
            unsupported_constructs: vec![],
            experimental_features: vec![],
        }
    }

    #[test]
    fn register_and_find() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_capability());
        assert_eq!(reg.generators().len(), 1);
        assert!(reg.find_by_name("know_now_gen_postgres").is_some());
        assert!(reg.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn generators_for_artifact() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_capability());
        let pg_gens = reg.generators_for_artifact(&ArtifactKind::PostgresDdl);
        assert_eq!(pg_gens.len(), 1);
        let dbt_gens = reg.generators_for_artifact(&ArtifactKind::DbtModel);
        assert!(dbt_gens.is_empty());
    }

    #[test]
    fn supports_contract_version() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_capability());
        assert!(reg.supports_contract_version("know_now_gen_postgres", "1.0"));
        assert!(!reg.supports_contract_version("know_now_gen_postgres", "2.0"));
        assert!(!reg.supports_contract_version("nonexistent", "1.0"));
    }

    #[test]
    fn capability_json_roundtrip() {
        let cap = sample_capability();
        let json = serde_json::to_string(&cap).unwrap();
        let parsed: GeneratorCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, cap.name);
        assert_eq!(parsed.contract_versions, cap.contract_versions);
    }

    #[test]
    fn empty_registry() {
        let reg = CapabilityRegistry::new();
        assert!(reg.generators().is_empty());
        assert!(reg.find_by_name("anything").is_none());
    }

    #[test]
    fn multiple_generators() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_capability());
        reg.register(GeneratorCapability {
            name: "know_now_gen_dbt".into(),
            version: "0.1.0".into(),
            contract_versions: vec!["1.0".into()],
            artifact_kinds: vec![ArtifactKind::DbtModel, ArtifactKind::DbtSchema],
            supported_dialects: vec![],
            supported_logical_types: vec![],
            supported_semantic_types: vec![],
            validation_gates: vec![],
            unsupported_constructs: vec![],
            experimental_features: vec![],
        });
        assert_eq!(reg.generators().len(), 2);
        let dbt_gens = reg.generators_for_artifact(&ArtifactKind::DbtModel);
        assert_eq!(dbt_gens.len(), 1);
        assert_eq!(dbt_gens[0].name, "know_now_gen_dbt");
    }
}
