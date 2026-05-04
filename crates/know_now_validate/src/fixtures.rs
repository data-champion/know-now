use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ValidationFixtureEntry {
    pub filename: &'static str,
    pub expected_error: Option<&'static str>,
    pub description: &'static str,
}

pub const VALIDATION_FIXTURES: &[ValidationFixtureEntry] = &[
    ValidationFixtureEntry {
        filename: "valid_full.yml",
        expected_error: None,
        description: "Full valid metadata with all cross-references",
    },
    ValidationFixtureEntry {
        filename: "unknown_entity_in_relationship.yml",
        expected_error: Some("META-REL-001"),
        description: "Relationship references unknown entity",
    },
    ValidationFixtureEntry {
        filename: "unknown_attribute_in_relationship.yml",
        expected_error: Some("META-REL-002"),
        description: "Relationship key references unknown attribute",
    },
    ValidationFixtureEntry {
        filename: "duplicate_entity_name.yml",
        expected_error: Some("META-ENT-001"),
        description: "Duplicate entity name within a domain",
    },
    ValidationFixtureEntry {
        filename: "duplicate_object_id.yml",
        expected_error: Some("META-ID-001"),
        description: "Duplicate stable object ID",
    },
    ValidationFixtureEntry {
        filename: "invalid_business_key.yml",
        expected_error: Some("META-ENT-002"),
        description: "Business key not in entity attributes",
    },
    ValidationFixtureEntry {
        filename: "source_unknown_entity.yml",
        expected_error: Some("META-SRC-001"),
        description: "Source table maps to unknown entity",
    },
    ValidationFixtureEntry {
        filename: "unknown_domain_reference.yml",
        expected_error: Some("META-DOM-001"),
        description: "Entity references unknown domain",
    },
    ValidationFixtureEntry {
        filename: "unknown_module_reference.yml",
        expected_error: Some("META-MOD-001"),
        description: "Entity references unknown module",
    },
    ValidationFixtureEntry {
        filename: "open_question_unknown_entity.yml",
        expected_error: Some("META-Q-001"),
        description: "Open question references unknown entity",
    },
    ValidationFixtureEntry {
        filename: "assumption_unknown_entity.yml",
        expected_error: Some("META-ASM-001"),
        description: "Assumption references unknown entity",
    },
];

#[must_use]
pub fn validation_fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("validation")
}

#[cfg(test)]
mod tests {
    use know_now_metadata::test_support::parse_yaml_metadata;

    use super::*;
    use crate::builder::build_project_graph;

    #[test]
    fn all_fixture_files_exist() {
        let dir = validation_fixtures_dir();
        for entry in VALIDATION_FIXTURES {
            let path = dir.join(entry.filename);
            assert!(
                path.exists(),
                "missing validation fixture: {} ({})",
                entry.filename,
                entry.description
            );
        }
    }

    #[test]
    fn exactly_11_fixtures() {
        assert_eq!(VALIDATION_FIXTURES.len(), 11);
    }

    #[test]
    fn one_valid_fixture() {
        let valid_count = VALIDATION_FIXTURES
            .iter()
            .filter(|f| f.expected_error.is_none())
            .count();
        assert_eq!(valid_count, 1);
    }

    #[test]
    fn ten_invalid_fixtures() {
        let invalid_count = VALIDATION_FIXTURES
            .iter()
            .filter(|f| f.expected_error.is_some())
            .count();
        assert_eq!(invalid_count, 10);
    }

    #[test]
    fn error_codes_are_unique() {
        let mut codes: Vec<&str> = VALIDATION_FIXTURES
            .iter()
            .filter_map(|f| f.expected_error)
            .collect();
        let before = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(
            codes.len(),
            before,
            "duplicate error codes in validation fixture catalog"
        );
    }

    #[test]
    fn invalid_fixtures_have_do_not_fix_header() {
        let dir = validation_fixtures_dir();
        for entry in VALIDATION_FIXTURES {
            if entry.expected_error.is_some() {
                let content = std::fs::read_to_string(dir.join(entry.filename)).unwrap();
                assert!(
                    content.contains("DO NOT FIX"),
                    "{} is invalid but missing DO NOT FIX header",
                    entry.filename
                );
            }
        }
    }

    #[test]
    fn invalid_fixtures_document_expected_error() {
        let dir = validation_fixtures_dir();
        for entry in VALIDATION_FIXTURES {
            if let Some(code) = entry.expected_error {
                let content = std::fs::read_to_string(dir.join(entry.filename)).unwrap();
                assert!(
                    content.contains(code),
                    "{} should document expected error code {} in header",
                    entry.filename,
                    code
                );
            }
        }
    }

    #[test]
    fn all_error_codes_follow_naming_convention() {
        for entry in VALIDATION_FIXTURES {
            if let Some(code) = entry.expected_error {
                assert!(
                    code.starts_with("META-"),
                    "error code {code} does not follow META-* convention"
                );
            }
        }
    }

    #[test]
    fn valid_fixtures_build_without_errors() {
        let dir = validation_fixtures_dir();
        for entry in VALIDATION_FIXTURES {
            if entry.expected_error.is_none() {
                let content = std::fs::read_to_string(dir.join(entry.filename)).unwrap();
                let meta = parse_yaml_metadata(&content);
                let result = build_project_graph(&meta);
                assert!(
                    result.diagnostics.is_empty(),
                    "{} should produce no errors but got: {:?}",
                    entry.filename,
                    result.diagnostics
                );
                assert!(
                    result.graph.is_some(),
                    "{} should produce a graph",
                    entry.filename
                );
            }
        }
    }

    #[test]
    fn invalid_fixtures_produce_expected_error() {
        let dir = validation_fixtures_dir();
        for entry in VALIDATION_FIXTURES {
            if let Some(expected_code) = entry.expected_error {
                let content = std::fs::read_to_string(dir.join(entry.filename)).unwrap();
                let meta = parse_yaml_metadata(&content);
                let result = build_project_graph(&meta);
                assert!(
                    result.diagnostics.iter().any(|d| d.code == expected_code),
                    "{} should produce error {} but got: {:?}",
                    entry.filename,
                    expected_code,
                    result.diagnostics
                );
            }
        }
    }
}
