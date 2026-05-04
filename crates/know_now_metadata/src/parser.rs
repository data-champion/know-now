use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::budgets::{BudgetViolation, ParserBudgets};
use crate::span::SourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLocation {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    BudgetViolation(BudgetViolation),
    DuplicateKey { key: String },
    Anchor,
    Alias,
    MergeKey,
    CustomTag { tag: String },
    IncludeDirective,
    MultiDocument,
    SyntaxError { detail: String },
    DeserializationError { detail: String },
}

impl ParseErrorKind {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::BudgetViolation(v) => v.code(),
            Self::DuplicateKey { .. } => "META-PAR-DUP",
            Self::Anchor => "META-PAR-ANCHOR",
            Self::Alias => "META-PAR-ALIAS",
            Self::MergeKey => "META-PAR-MERGE",
            Self::CustomTag { .. } => "META-PAR-TAG",
            Self::IncludeDirective => "META-PAR-INCLUDE",
            Self::MultiDocument => "META-PAR-MULTIDOC",
            Self::SyntaxError { .. } => "META-PAR-SYNTAX",
            Self::DeserializationError { .. } => "META-PAR-DESER",
        }
    }
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BudgetViolation(v) => write!(f, "{v}"),
            Self::DuplicateKey { key } => write!(f, "{}: duplicate key '{key}'", self.code()),
            Self::Anchor => write!(f, "{}: YAML anchors are not allowed", self.code()),
            Self::Alias => write!(f, "{}: YAML aliases are not allowed", self.code()),
            Self::MergeKey => write!(f, "{}: YAML merge keys (<<) are not allowed", self.code()),
            Self::CustomTag { tag } => {
                write!(f, "{}: custom YAML tag '{tag}' is not allowed", self.code())
            }
            Self::IncludeDirective => {
                write!(f, "{}: include directives are not allowed", self.code())
            }
            Self::MultiDocument => {
                write!(f, "{}: multi-document YAML is not allowed", self.code())
            }
            Self::SyntaxError { detail } | Self::DeserializationError { detail } => {
                write!(f, "{}: {detail}", self.code())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub location: ParseLocation,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location.file.display())?;
        if let Some(line) = self.location.line {
            write!(f, ":{line}")?;
            if let Some(col) = self.location.column {
                write!(f, ":{col}")?;
            }
        }
        write!(f, " {}", self.kind)
    }
}

#[derive(Debug)]
pub struct ParsedDocument<T> {
    pub source_id: SourceId,
    pub path: PathBuf,
    pub document: T,
}

/// # Errors
/// Returns `ParseError` if the input violates budgets, contains banned YAML
/// features, or fails deserialization.
pub fn parse_yaml<T: DeserializeOwned>(
    source: &str,
    file: &Path,
    source_id: SourceId,
    budgets: &ParserBudgets,
) -> Result<ParsedDocument<T>, Vec<ParseError>> {
    let mut errors = Vec::new();
    let loc = || ParseLocation {
        file: file.to_path_buf(),
        line: None,
        column: None,
    };

    if let Err(v) = budgets.check_file_size(source.len() as u64) {
        errors.push(ParseError {
            kind: ParseErrorKind::BudgetViolation(v),
            location: loc(),
        });
        return Err(errors);
    }

    validate_subset(source, file, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    let document: T = serde_saphyr::from_str(source).map_err(|e| {
        vec![ParseError {
            kind: ParseErrorKind::DeserializationError {
                detail: e.to_string(),
            },
            location: loc(),
        }]
    })?;

    Ok(ParsedDocument {
        source_id,
        path: file.to_path_buf(),
        document,
    })
}

fn validate_subset(source: &str, file: &Path, errors: &mut Vec<ParseError>) {
    let loc_at = |line: u32| ParseLocation {
        file: file.to_path_buf(),
        line: Some(line),
        column: None,
    };
    let loc = || ParseLocation {
        file: file.to_path_buf(),
        line: None,
        column: None,
    };

    let mut doc_separator_count = 0u32;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let line_1based = (line_num as u32) + 1;

        if trimmed == "---" && line_num > 0 {
            doc_separator_count += 1;
            if doc_separator_count >= 1 {
                errors.push(ParseError {
                    kind: ParseErrorKind::MultiDocument,
                    location: loc_at(line_1based),
                });
            }
        }

        if trimmed.contains('&') && !trimmed.starts_with('#') {
            if let Some(pos) = trimmed.find('&') {
                let before = &trimmed[..pos];
                if before.is_empty() || before.ends_with(' ') || before.ends_with(':') {
                    errors.push(ParseError {
                        kind: ParseErrorKind::Anchor,
                        location: loc_at(line_1based),
                    });
                }
            }
        }

        if trimmed.contains('*') && !trimmed.starts_with('#') {
            if let Some(pos) = trimmed.find('*') {
                let before = &trimmed[..pos];
                if before.is_empty()
                    || before.ends_with(' ')
                    || before.ends_with(':')
                    || before.ends_with('-')
                {
                    errors.push(ParseError {
                        kind: ParseErrorKind::Alias,
                        location: loc_at(line_1based),
                    });
                }
            }
        }

        if trimmed.contains("<<:") || trimmed.contains("<< :") {
            errors.push(ParseError {
                kind: ParseErrorKind::MergeKey,
                location: loc_at(line_1based),
            });
        }

        if trimmed.contains("!include") {
            errors.push(ParseError {
                kind: ParseErrorKind::IncludeDirective,
                location: loc_at(line_1based),
            });
        }

        if !trimmed.starts_with('#') {
            if let Some(idx) = trimmed.find('!') {
                let after = &trimmed[idx + 1..];
                if !after.is_empty()
                    && !after.starts_with(' ')
                    && !after.starts_with('=')
                    && !after.contains("include")
                {
                    let tag: String = after.chars().take_while(|c| !c.is_whitespace()).collect();
                    if !tag.is_empty() {
                        errors.push(ParseError {
                            kind: ParseErrorKind::CustomTag { tag },
                            location: loc_at(line_1based),
                        });
                    }
                }
            }
        }
    }

    let _ = loc;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn budgets() -> ParserBudgets {
        ParserBudgets::default()
    }

    fn small_budgets() -> ParserBudgets {
        ParserBudgets {
            max_file_bytes: 50,
            max_nesting_depth: 3,
            ..Default::default()
        }
    }

    #[test]
    fn parse_valid_yaml() {
        let yaml = "name: test\nvalue: 42\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("test.yml"), SourceId(0), &budgets());
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.source_id, SourceId(0));
    }

    #[test]
    fn rejects_oversized_file() {
        let yaml = "name: test\nvalue: forty_two\nextra_long_key: extra_long_data_value\n";
        let result: Result<ParsedDocument<HashMap<String, String>>, _> =
            parse_yaml(yaml, Path::new("big.yml"), SourceId(0), &small_budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-SIZE"));
    }

    #[test]
    fn rejects_multi_document() {
        let yaml = "name: first\n---\nname: second\n";
        let result: Result<ParsedDocument<HashMap<String, String>>, _> =
            parse_yaml(yaml, Path::new("multi.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-MULTIDOC"));
    }

    #[test]
    fn rejects_anchor() {
        let yaml = "defaults: &defaults\n  type: string\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("anchor.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-ANCHOR"));
    }

    #[test]
    fn rejects_alias() {
        let yaml = "items:\n  - *ref\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("alias.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-ALIAS"));
    }

    #[test]
    fn rejects_merge_key() {
        let yaml = "base: &base\n  type: string\nobj:\n  <<: *base\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("merge.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-MERGE"));
    }

    #[test]
    fn rejects_custom_tag() {
        let yaml = "value: !custom_type 42\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("tag.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-TAG"));
    }

    #[test]
    fn rejects_include_directive() {
        let yaml = "items:\n  - !include shared/file.yml\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("inc.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-INCLUDE"));
    }

    #[test]
    fn error_display_includes_file_and_line() {
        let err = ParseError {
            kind: ParseErrorKind::DuplicateKey { key: "name".into() },
            location: ParseLocation {
                file: PathBuf::from("metadata/entities.yml"),
                line: Some(5),
                column: Some(3),
            },
        };
        let msg = err.to_string();
        assert!(msg.contains("metadata/entities.yml"));
        assert!(msg.contains(":5:3"));
        assert!(msg.contains("META-PAR-DUP"));
        assert!(msg.contains("name"));
    }

    #[test]
    fn error_codes_are_stable() {
        let kinds = [
            ParseErrorKind::DuplicateKey { key: String::new() },
            ParseErrorKind::Anchor,
            ParseErrorKind::Alias,
            ParseErrorKind::MergeKey,
            ParseErrorKind::CustomTag { tag: String::new() },
            ParseErrorKind::IncludeDirective,
            ParseErrorKind::MultiDocument,
            ParseErrorKind::SyntaxError {
                detail: String::new(),
            },
            ParseErrorKind::DeserializationError {
                detail: String::new(),
            },
        ];

        let codes = [
            "META-PAR-DUP",
            "META-PAR-ANCHOR",
            "META-PAR-ALIAS",
            "META-PAR-MERGE",
            "META-PAR-TAG",
            "META-PAR-INCLUDE",
            "META-PAR-MULTIDOC",
            "META-PAR-SYNTAX",
            "META-PAR-DESER",
        ];

        for (kind, expected) in kinds.iter().zip(codes.iter()) {
            assert_eq!(kind.code(), *expected);
        }
    }

    #[test]
    fn deserialization_error_on_invalid_structure() {
        let yaml = "not_a_map";
        let result: Result<ParsedDocument<HashMap<String, String>>, _> =
            parse_yaml(yaml, Path::new("bad.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind.code() == "META-PAR-DESER"));
    }

    #[test]
    fn comments_with_ampersand_not_flagged() {
        let yaml = "# This & that\nname: test\n";
        let result: Result<ParsedDocument<HashMap<String, String>>, _> =
            parse_yaml(yaml, Path::new("ok.yml"), SourceId(0), &budgets());
        assert!(result.is_ok());
    }

    #[test]
    fn multiple_errors_collected() {
        let yaml = "defaults: &defaults\n  type: string\n---\nname: second\n";
        let result: Result<ParsedDocument<HashMap<String, serde_json::Value>>, _> =
            parse_yaml(yaml, Path::new("multi_err.yml"), SourceId(0), &budgets());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}
