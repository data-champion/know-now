use crate::semver::VersionRange;
use crate::Catalog;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("CATALOG-SCHEMA-001: missing 'approved' section")]
    MissingApproved,

    #[error("CATALOG-RANGE-002: invalid version range '{range}' in {context}")]
    InvalidRange { range: String, context: String },

    #[error("CATALOG-TARGET-003: target '{name}' has no floor and no allowed versions")]
    EmptyTarget { name: String },

    #[error("CATALOG-TARGET-004: target '{name}' floor '{floor}' is not in allowed list")]
    FloorNotInAllowed { name: String, floor: String },
}

pub fn validate(catalog: &Catalog) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    validate_version_ranges(&catalog.approved.engines, "engines", &mut errors);
    validate_plain_versions(&catalog.approved.metadata_schema_versions, "metadata_schema_versions", &mut errors);
    validate_plain_versions(&catalog.approved.generator_contract_versions, "generator_contract_versions", &mut errors);
    validate_version_ranges(&catalog.approved.policies, "policies", &mut errors);
    validate_version_ranges(&catalog.approved.templates, "templates", &mut errors);
    validate_version_ranges(&catalog.approved.template_renderers, "template_renderers", &mut errors);

    for (name, spec) in &catalog.approved.targets {
        if spec.floor.is_none() && spec.allowed.is_empty() {
            errors.push(ValidationError::EmptyTarget { name: name.clone() });
        }
        if let Some(floor) = &spec.floor {
            if !spec.allowed.is_empty() && !spec.allowed.contains(floor) {
                errors.push(ValidationError::FloorNotInAllowed {
                    name: name.clone(),
                    floor: floor.clone(),
                });
            }
        }
    }

    errors
}

fn validate_version_ranges(
    map: &std::collections::HashMap<String, Vec<String>>,
    section: &str,
    errors: &mut Vec<ValidationError>,
) {
    for (name, ranges) in map {
        for range in ranges {
            if VersionRange::parse(range).is_none() {
                errors.push(ValidationError::InvalidRange {
                    range: range.clone(),
                    context: format!("{section}.{name}"),
                });
            }
        }
    }
}

fn validate_plain_versions(
    versions: &[String],
    section: &str,
    errors: &mut Vec<ValidationError>,
) {
    for v in versions {
        if VersionRange::parse(v).is_none() {
            errors.push(ValidationError::InvalidRange {
                range: v.clone(),
                context: section.into(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApprovedVersions, TargetSpec};

    fn empty_catalog() -> Catalog {
        Catalog {
            approved: ApprovedVersions::default(),
        }
    }

    #[test]
    fn valid_catalog_has_no_errors() {
        let catalog = Catalog {
            approved: ApprovedVersions {
                engines: [("know-now".into(), vec!["1.0.x".into()])].into(),
                metadata_schema_versions: vec!["1.0".into()],
                targets: [(
                    "postgres".into(),
                    TargetSpec {
                        floor: Some("16".into()),
                        allowed: vec!["16".into(), "17".into()],
                    },
                )]
                .into(),
                ..Default::default()
            },
        };
        let errors = validate(&catalog);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn empty_catalog_is_valid() {
        let errors = validate(&empty_catalog());
        assert!(errors.is_empty());
    }

    #[test]
    fn empty_target_spec_is_error() {
        let catalog = Catalog {
            approved: ApprovedVersions {
                targets: [(
                    "postgres".into(),
                    TargetSpec {
                        floor: None,
                        allowed: vec![],
                    },
                )]
                .into(),
                ..Default::default()
            },
        };
        let errors = validate(&catalog);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("CATALOG-TARGET-003"));
    }

    #[test]
    fn floor_not_in_allowed_is_error() {
        let catalog = Catalog {
            approved: ApprovedVersions {
                targets: [(
                    "postgres".into(),
                    TargetSpec {
                        floor: Some("15".into()),
                        allowed: vec!["16".into(), "17".into()],
                    },
                )]
                .into(),
                ..Default::default()
            },
        };
        let errors = validate(&catalog);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("CATALOG-TARGET-004"));
    }

    #[test]
    fn floor_without_allowed_is_valid() {
        let catalog = Catalog {
            approved: ApprovedVersions {
                targets: [(
                    "postgres".into(),
                    TargetSpec {
                        floor: Some("16".into()),
                        allowed: vec![],
                    },
                )]
                .into(),
                ..Default::default()
            },
        };
        let errors = validate(&catalog);
        assert!(errors.is_empty());
    }
}
