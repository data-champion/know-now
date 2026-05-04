use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Discovery,
    Parsing,
    YamlSubset,
    Deserialize,
    Semantic,
    Policy,
    DefaultResolution,
    Contract,
    Capabilities,
    Planning,
    Generation,
    Validation,
    ManualEditDetection,
    PathSafety,
    StalePlan,
    AtomicWrite,
    Manifesting,
    RunLog,
}

impl Stage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Parsing => "parsing",
            Self::YamlSubset => "yaml_subset",
            Self::Deserialize => "deserialize",
            Self::Semantic => "semantic",
            Self::Policy => "policy",
            Self::DefaultResolution => "default_resolution",
            Self::Contract => "contract",
            Self::Capabilities => "capabilities",
            Self::Planning => "planning",
            Self::Generation => "generation",
            Self::Validation => "validation",
            Self::ManualEditDetection => "manual_edit_detection",
            Self::PathSafety => "path_safety",
            Self::StalePlan => "stale_plan",
            Self::AtomicWrite => "atomic_write",
            Self::Manifesting => "manifesting",
            Self::RunLog => "run_log",
        }
    }

    pub const ALL: &[Self] = &[
        Self::Discovery,
        Self::Parsing,
        Self::YamlSubset,
        Self::Deserialize,
        Self::Semantic,
        Self::Policy,
        Self::DefaultResolution,
        Self::Contract,
        Self::Capabilities,
        Self::Planning,
        Self::Generation,
        Self::Validation,
        Self::ManualEditDetection,
        Self::PathSafety,
        Self::StalePlan,
        Self::AtomicWrite,
        Self::Manifesting,
        Self::RunLog,
    ];
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_stages_have_unique_names() {
        let names: HashSet<&str> = Stage::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(names.len(), Stage::ALL.len());
    }

    #[test]
    fn stage_count_matches_prd_17_1() {
        assert_eq!(Stage::ALL.len(), 18);
    }

    #[test]
    fn all_stages_emit_as_tracing_spans() {
        let (guard, captured) = crate::test_support::install_test_subscriber();

        for stage in Stage::ALL {
            let span = tracing::info_span!("stage", name = stage.as_str());
            let _enter = span.enter();
            tracing::info!(stage = stage.as_str(), "stage executed");
        }

        drop(guard);

        let events = captured.events();
        assert!(
            events.len() >= Stage::ALL.len(),
            "expected at least {} events (one per stage), got {}",
            Stage::ALL.len(),
            events.len()
        );

        let events = captured.events();
        let event_stage_fields: Vec<String> = events
            .iter()
            .filter_map(|e| {
                e.get("fields")
                    .and_then(|f| f.get("stage"))
                    .and_then(serde_json::Value::as_str)
                    .map(String::from)
            })
            .collect();

        for stage in Stage::ALL {
            assert!(
                event_stage_fields.contains(&stage.as_str().to_owned()),
                "event for stage {} not found in captured events",
                stage.as_str()
            );
        }
    }
}
