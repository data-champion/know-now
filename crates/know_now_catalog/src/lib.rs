use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod drift;
mod semver;
mod validate;

pub use drift::{classify_drift, DriftClass, DriftReport, DriftEntry};
pub use validate::{validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub approved: ApprovedVersions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovedVersions {
    #[serde(default)]
    pub engines: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub metadata_schema_versions: Vec<String>,

    #[serde(default)]
    pub generator_contract_versions: Vec<String>,

    #[serde(default)]
    pub policies: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub templates: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub template_renderers: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub targets: HashMap<String, TargetSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSpec {
    #[serde(default)]
    pub floor: Option<String>,
    #[serde(default)]
    pub allowed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectState {
    #[serde(default)]
    pub engine_version: Option<String>,

    #[serde(default)]
    pub metadata_schema_version: Option<String>,

    #[serde(default)]
    pub generator_contract_version: Option<String>,

    #[serde(default)]
    pub policies: HashMap<String, String>,

    #[serde(default)]
    pub templates: HashMap<String, String>,

    #[serde(default)]
    pub template_renderers: HashMap<String, String>,

    #[serde(default)]
    pub targets: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_deserializes_from_json() {
        let json = r#"{
            "approved": {
                "engines": { "know-now": ["1.0.x"] },
                "metadata_schema_versions": ["1.0"],
                "generator_contract_versions": ["1.0"],
                "policies": { "dc_standard": ["1.0.x"] },
                "templates": { "internal_api_docs": ["1.0.x"] },
                "template_renderers": { "know-now-minijinja-v1": ["1"] },
                "targets": {
                    "postgres": { "floor": "16", "allowed": ["16","17","18"] }
                }
            }
        }"#;
        let catalog: Catalog = serde_json::from_str(json).unwrap();
        assert_eq!(catalog.approved.engines["know-now"], vec!["1.0.x"]);
        assert_eq!(catalog.approved.targets["postgres"].allowed, vec!["16", "17", "18"]);
    }

    #[test]
    fn project_state_deserializes_from_json() {
        let json = r#"{
            "engine_version": "1.0.3",
            "metadata_schema_version": "1.0",
            "policies": { "dc_standard": "1.0.2" },
            "targets": { "postgres": "16" }
        }"#;
        let state: ProjectState = serde_json::from_str(json).unwrap();
        assert_eq!(state.engine_version.as_deref(), Some("1.0.3"));
        assert_eq!(state.policies["dc_standard"], "1.0.2");
    }

    #[test]
    fn catalog_with_empty_approved_deserializes() {
        let json = r#"{ "approved": {} }"#;
        let catalog: Catalog = serde_json::from_str(json).unwrap();
        assert!(catalog.approved.engines.is_empty());
    }
}
