use serde::Serialize;

use crate::semver::{classify_version_diff, matches_any, VersionDiff};
use crate::{Catalog, ProjectState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftClass {
    None,
    Patch,
    Minor,
    Major,
    Unknown,
    Unapproved,
}

impl std::fmt::Display for DriftClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Patch => write!(f, "patch"),
            Self::Minor => write!(f, "minor"),
            Self::Major => write!(f, "major"),
            Self::Unknown => write!(f, "unknown"),
            Self::Unapproved => write!(f, "unapproved"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DriftEntry {
    pub component: String,
    pub name: String,
    pub installed: String,
    pub drift: DriftClass,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriftReport {
    pub overall: DriftClass,
    pub entries: Vec<DriftEntry>,
}

pub fn classify_drift(catalog: &Catalog, state: &ProjectState) -> DriftReport {
    let mut entries = Vec::new();

    classify_core_versions(catalog, state, &mut entries);
    classify_named_map("policy", &state.policies, &catalog.approved.policies, &mut entries);
    classify_named_map("template", &state.templates, &catalog.approved.templates, &mut entries);
    classify_named_map("template_renderer", &state.template_renderers, &catalog.approved.template_renderers, &mut entries);
    classify_targets(catalog, state, &mut entries);

    let overall = entries
        .iter()
        .map(|e| e.drift)
        .max_by_key(|c| severity_rank(*c))
        .unwrap_or(DriftClass::None);

    DriftReport { overall, entries }
}

fn classify_core_versions(catalog: &Catalog, state: &ProjectState, entries: &mut Vec<DriftEntry>) {
    if let Some(engine_ver) = &state.engine_version {
        if let Some(ranges) = catalog.approved.engines.get("know-now") {
            entries.push(classify_component("engine", "know-now", engine_ver, ranges));
        }
    }

    if let Some(schema_ver) = &state.metadata_schema_version {
        entries.push(classify_component(
            "metadata_schema",
            "metadata_schema",
            schema_ver,
            &catalog.approved.metadata_schema_versions,
        ));
    }

    if let Some(contract_ver) = &state.generator_contract_version {
        entries.push(classify_component(
            "generator_contract",
            "generator_contract",
            contract_ver,
            &catalog.approved.generator_contract_versions,
        ));
    }
}

fn classify_named_map(
    component: &str,
    installed_map: &std::collections::HashMap<String, String>,
    approved_map: &std::collections::HashMap<String, Vec<String>>,
    entries: &mut Vec<DriftEntry>,
) {
    for (name, installed) in installed_map {
        let ranges = approved_map.get(name).cloned().unwrap_or_default();
        if ranges.is_empty() {
            entries.push(DriftEntry {
                component: component.into(),
                name: name.clone(),
                installed: installed.clone(),
                drift: DriftClass::Unapproved,
                reason: format!("{component} '{name}' is not in the approved catalog"),
            });
        } else {
            entries.push(classify_component(component, name, installed, &ranges));
        }
    }
}

fn classify_targets(catalog: &Catalog, state: &ProjectState, entries: &mut Vec<DriftEntry>) {
    for (name, installed) in &state.targets {
        let Some(spec) = catalog.approved.targets.get(name) else {
            entries.push(DriftEntry {
                component: "target".into(),
                name: name.clone(),
                installed: installed.clone(),
                drift: DriftClass::Unapproved,
                reason: format!("target '{name}' is not in the approved catalog"),
            });
            continue;
        };

        if !spec.allowed.is_empty() {
            let drift = if spec.allowed.contains(installed) { DriftClass::None } else { DriftClass::Major };
            let reason = if drift == DriftClass::None {
                String::new()
            } else {
                format!("target '{name}' version '{installed}' is not in allowed list")
            };
            entries.push(DriftEntry { component: "target".into(), name: name.clone(), installed: installed.clone(), drift, reason });
        } else if let Some(floor) = &spec.floor {
            let drift = if installed >= floor { DriftClass::None } else { DriftClass::Major };
            let reason = if drift == DriftClass::None {
                String::new()
            } else {
                format!("target '{name}' version '{installed}' is below floor '{floor}'")
            };
            entries.push(DriftEntry { component: "target".into(), name: name.clone(), installed: installed.clone(), drift, reason });
        }
    }
}

fn classify_component(component: &str, name: &str, installed: &str, ranges: &[String]) -> DriftEntry {
    if matches_any(installed, ranges) {
        return DriftEntry {
            component: component.into(),
            name: name.into(),
            installed: installed.into(),
            drift: DriftClass::None,
            reason: String::new(),
        };
    }

    let diff = classify_version_diff(installed, ranges);
    let drift = match diff {
        VersionDiff::None => DriftClass::None,
        VersionDiff::Patch => DriftClass::Patch,
        VersionDiff::Minor => DriftClass::Minor,
        VersionDiff::Major => DriftClass::Major,
        VersionDiff::Unknown => DriftClass::Unknown,
    };

    let reason = if drift == DriftClass::None {
        String::new()
    } else {
        format!(
            "{component} '{name}' version '{installed}' does not match approved ranges: {}",
            ranges.join(", ")
        )
    };

    DriftEntry {
        component: component.into(),
        name: name.into(),
        installed: installed.into(),
        drift,
        reason,
    }
}

fn severity_rank(class: DriftClass) -> u8 {
    match class {
        DriftClass::None => 0,
        DriftClass::Patch => 1,
        DriftClass::Minor => 2,
        DriftClass::Major => 3,
        DriftClass::Unknown => 4,
        DriftClass::Unapproved => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApprovedVersions, TargetSpec};

    fn sample_catalog() -> Catalog {
        Catalog {
            approved: ApprovedVersions {
                engines: [("know-now".into(), vec!["1.0.x".into()])].into(),
                metadata_schema_versions: vec!["1.0".into()],
                generator_contract_versions: vec!["1.0".into()],
                policies: [("dc_standard".into(), vec!["1.0.x".into()])].into(),
                templates: [("internal_api_docs".into(), vec!["1.0.x".into()])].into(),
                template_renderers: [("know-now-minijinja-v1".into(), vec!["1".into()])].into(),
                targets: [(
                    "postgres".into(),
                    TargetSpec {
                        floor: Some("16".into()),
                        allowed: vec!["16".into(), "17".into(), "18".into()],
                    },
                )]
                .into(),
            },
        }
    }

    #[test]
    fn no_drift_when_all_match() {
        let catalog = sample_catalog();
        let state = ProjectState {
            engine_version: Some("1.0.5".into()),
            metadata_schema_version: Some("1.0".into()),
            generator_contract_version: Some("1.0".into()),
            policies: [("dc_standard".into(), "1.0.2".into())].into(),
            templates: [("internal_api_docs".into(), "1.0.0".into())].into(),
            template_renderers: [("know-now-minijinja-v1".into(), "1".into())].into(),
            targets: [("postgres".into(), "16".into())].into(),
        };
        let report = classify_drift(&catalog, &state);
        assert_eq!(report.overall, DriftClass::None);
        assert!(
            report.entries.iter().all(|e| e.drift == DriftClass::None),
            "all entries should be None drift: {report:?}"
        );
    }

    #[test]
    fn minor_drift_on_engine() {
        let catalog = sample_catalog();
        let state = ProjectState {
            engine_version: Some("1.2.0".into()),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let engine_entry = report.entries.iter().find(|e| e.name == "know-now").unwrap();
        assert_eq!(engine_entry.drift, DriftClass::Minor);
    }

    #[test]
    fn major_drift_on_engine() {
        let catalog = sample_catalog();
        let state = ProjectState {
            engine_version: Some("2.0.0".into()),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let engine_entry = report.entries.iter().find(|e| e.name == "know-now").unwrap();
        assert_eq!(engine_entry.drift, DriftClass::Major);
    }

    #[test]
    fn unapproved_policy() {
        let catalog = sample_catalog();
        let state = ProjectState {
            policies: [("custom_corp".into(), "1.0.0".into())].into(),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let policy_entry = report.entries.iter().find(|e| e.name == "custom_corp").unwrap();
        assert_eq!(policy_entry.drift, DriftClass::Unapproved);
    }

    #[test]
    fn target_not_in_allowed_list() {
        let catalog = sample_catalog();
        let state = ProjectState {
            targets: [("postgres".into(), "15".into())].into(),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let target_entry = report.entries.iter().find(|e| e.name == "postgres").unwrap();
        assert_eq!(target_entry.drift, DriftClass::Major);
    }

    #[test]
    fn target_in_allowed_list() {
        let catalog = sample_catalog();
        let state = ProjectState {
            targets: [("postgres".into(), "17".into())].into(),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let target_entry = report.entries.iter().find(|e| e.name == "postgres").unwrap();
        assert_eq!(target_entry.drift, DriftClass::None);
    }

    #[test]
    fn unapproved_target() {
        let catalog = sample_catalog();
        let state = ProjectState {
            targets: [("mysql".into(), "8.0".into())].into(),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let target_entry = report.entries.iter().find(|e| e.name == "mysql").unwrap();
        assert_eq!(target_entry.drift, DriftClass::Unapproved);
    }

    #[test]
    fn overall_is_worst_case() {
        let catalog = sample_catalog();
        let state = ProjectState {
            engine_version: Some("1.0.5".into()),
            policies: [("rogue_policy".into(), "0.1.0".into())].into(),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        assert_eq!(report.overall, DriftClass::Unapproved);
    }

    #[test]
    fn empty_state_produces_empty_report() {
        let catalog = sample_catalog();
        let state = ProjectState::default();
        let report = classify_drift(&catalog, &state);
        assert!(report.entries.is_empty());
        assert_eq!(report.overall, DriftClass::None);
    }

    #[test]
    fn report_serializes_to_json() {
        let catalog = sample_catalog();
        let state = ProjectState {
            engine_version: Some("1.0.5".into()),
            ..Default::default()
        };
        let report = classify_drift(&catalog, &state);
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"drift\":\"none\""));
    }
}
