use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Blocking,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => f.write_str("info"),
            Self::Warning => f.write_str("warning"),
            Self::Error => f.write_str("error"),
            Self::Blocking => f.write_str("blocking"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yaml_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<SourceSnippet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSnippet {
    pub line_number: u32,
    pub text: String,
    pub highlight_start: u32,
    pub highlight_len: u32,
    pub label: String,
}

impl Diagnostic {
    #[must_use]
    pub fn new(severity: Severity, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            location: None,
            yaml_path: None,
            metadata_object_id: None,
            help: None,
            source_text: None,
        }
    }

    #[must_use]
    pub fn with_location(mut self, file: PathBuf, line: u32, column: u32) -> Self {
        self.location = Some(SourceLocation { file, line, column });
        self
    }

    #[must_use]
    pub fn with_yaml_path(mut self, path: impl Into<String>) -> Self {
        self.yaml_path = Some(path.into());
        self
    }

    #[must_use]
    pub fn with_metadata_object_id(mut self, id: impl Into<String>) -> Self {
        self.metadata_object_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    #[must_use]
    pub fn with_source_snippet(
        mut self,
        line_number: u32,
        text: impl Into<String>,
        highlight_start: u32,
        highlight_len: u32,
        label: impl Into<String>,
    ) -> Self {
        self.source_text = Some(SourceSnippet {
            line_number,
            text: text.into(),
            highlight_start,
            highlight_len,
            label: label.into(),
        });
        self
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error | Severity::Blocking)
    }
}

pub trait DiagnosticRenderer {
    fn render(&self, diagnostic: &Diagnostic) -> String;
    fn render_all(&self, diagnostics: &[Diagnostic]) -> String {
        diagnostics
            .iter()
            .map(|d| self.render(d))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Blocking);
    }

    #[test]
    fn diagnostic_builder() {
        let d = Diagnostic::new(Severity::Error, "META-REL-001", "test message")
            .with_location(PathBuf::from("test.yml"), 12, 18)
            .with_help("try this instead");

        assert!(d.is_error());
        assert_eq!(d.code, "META-REL-001");
        assert!(d.location.is_some());
        assert!(d.help.is_some());
    }

    #[test]
    fn info_is_not_error() {
        let d = Diagnostic::new(Severity::Info, "INFO-001", "informational");
        assert!(!d.is_error());
    }

    #[test]
    fn blocking_is_error() {
        let d = Diagnostic::new(Severity::Blocking, "BLK-001", "blocking");
        assert!(d.is_error());
    }

    #[test]
    fn severity_serde_roundtrip() {
        let json = serde_json::to_string(&Severity::Error).unwrap();
        assert_eq!(json, "\"error\"");
        let parsed: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Severity::Error);
    }
}
