use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FixtureEntry {
    pub filename: &'static str,
    pub expected_error: Option<&'static str>,
    pub description: &'static str,
}

pub const PARSER_FIXTURES: &[FixtureEntry] = &[
    FixtureEntry {
        filename: "valid_minimal.yml",
        expected_error: None,
        description: "Minimal valid metadata",
    },
    FixtureEntry {
        filename: "valid_multifile_project.yml",
        expected_error: None,
        description: "Multi-entity valid metadata with sources",
    },
    FixtureEntry {
        filename: "duplicate_key.yml",
        expected_error: Some("META-PAR-DUP"),
        description: "Duplicate mapping key",
    },
    FixtureEntry {
        filename: "anchor.yml",
        expected_error: Some("META-PAR-ANCHOR"),
        description: "YAML anchor (banned)",
    },
    FixtureEntry {
        filename: "alias.yml",
        expected_error: Some("META-PAR-ALIAS"),
        description: "YAML alias (banned)",
    },
    FixtureEntry {
        filename: "merge_key.yml",
        expected_error: Some("META-PAR-MERGE"),
        description: "YAML merge key (banned)",
    },
    FixtureEntry {
        filename: "custom_tag.yml",
        expected_error: Some("META-PAR-TAG"),
        description: "YAML custom tag (banned)",
    },
    FixtureEntry {
        filename: "include_directive.yml",
        expected_error: Some("META-PAR-INCLUDE"),
        description: "Include directive (banned)",
    },
    FixtureEntry {
        filename: "multi_document.yml",
        expected_error: Some("META-PAR-MULTIDOC"),
        description: "Multi-document YAML (banned)",
    },
    FixtureEntry {
        filename: "deep_nesting.yml",
        expected_error: Some("META-PAR-NESTING"),
        description: "Exceeds nesting depth budget",
    },
    FixtureEntry {
        filename: "large_file.yml",
        expected_error: Some("META-PAR-SIZE"),
        description: "Exceeds file size budget",
    },
    FixtureEntry {
        filename: "bad_scalar_type.yml",
        expected_error: Some("META-PAR-SCALAR"),
        description: "Non-string scalar value where string expected",
    },
    FixtureEntry {
        filename: "unknown_field.yml",
        expected_error: Some("META-PAR-UNKNOWN"),
        description: "Unrecognized top-level field",
    },
];

#[must_use]
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("parser")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fixture_files_exist() {
        let dir = fixtures_dir();
        for entry in PARSER_FIXTURES {
            let path = dir.join(entry.filename);
            assert!(
                path.exists(),
                "missing parser fixture: {} ({})",
                entry.filename,
                entry.description
            );
        }
    }

    #[test]
    fn exactly_13_fixtures() {
        assert_eq!(PARSER_FIXTURES.len(), 13);
    }

    #[test]
    fn two_valid_fixtures() {
        let valid_count = PARSER_FIXTURES
            .iter()
            .filter(|f| f.expected_error.is_none())
            .count();
        assert_eq!(valid_count, 2);
    }

    #[test]
    fn eleven_invalid_fixtures() {
        let invalid_count = PARSER_FIXTURES
            .iter()
            .filter(|f| f.expected_error.is_some())
            .count();
        assert_eq!(invalid_count, 11);
    }

    #[test]
    fn error_codes_are_unique() {
        let mut codes: Vec<&str> = PARSER_FIXTURES
            .iter()
            .filter_map(|f| f.expected_error)
            .collect();
        let before = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(
            codes.len(),
            before,
            "duplicate error codes in fixture catalog"
        );
    }

    #[test]
    fn invalid_fixtures_have_do_not_fix_header() {
        let dir = fixtures_dir();
        for entry in PARSER_FIXTURES {
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
        let dir = fixtures_dir();
        for entry in PARSER_FIXTURES {
            if let Some(code) = entry.expected_error {
                let content = std::fs::read_to_string(dir.join(entry.filename)).unwrap();
                assert!(
                    content.contains(code),
                    "{} should document expected error code {} in its header",
                    entry.filename,
                    code
                );
            }
        }
    }

    #[test]
    fn all_error_codes_follow_naming_convention() {
        for entry in PARSER_FIXTURES {
            if let Some(code) = entry.expected_error {
                assert!(
                    code.starts_with("META-PAR-"),
                    "error code {code} does not follow META-PAR-* convention"
                );
            }
        }
    }
}
