use crate::diagnostic::{Diagnostic, DiagnosticRenderer};

pub struct JsonRenderer;

impl JsonRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticRenderer for JsonRenderer {
    fn render(&self, diagnostic: &Diagnostic) -> String {
        serde_json::to_string(diagnostic)
            .unwrap_or_else(|e| format!("{{\"error\":\"failed to serialize diagnostic: {e}\"}}"))
    }

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
    use std::path::PathBuf;

    use super::*;
    use crate::diagnostic::Severity;

    #[test]
    fn renders_diagnostic_as_json() {
        let d = Diagnostic::new(
            Severity::Error,
            "META-REL-001",
            "relationship references unknown entity `customers`",
        )
        .with_location(PathBuf::from("metadata/relationships/orders.yml"), 12, 18)
        .with_help("did you mean `customer`?");

        let renderer = JsonRenderer::new();
        let json_str = renderer.render(&d);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["severity"], "error");
        assert_eq!(parsed["code"], "META-REL-001");
        assert_eq!(
            parsed["message"],
            "relationship references unknown entity `customers`"
        );
        assert_eq!(
            parsed["location"]["file"],
            "metadata/relationships/orders.yml"
        );
        assert_eq!(parsed["location"]["line"], 12);
        assert_eq!(parsed["location"]["column"], 18);
        assert_eq!(parsed["help"], "did you mean `customer`?");
    }

    #[test]
    fn omits_none_fields() {
        let d = Diagnostic::new(Severity::Warning, "WARN-001", "test");

        let renderer = JsonRenderer::new();
        let json_str = renderer.render(&d);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(parsed.get("location").is_none());
        assert!(parsed.get("yaml_path").is_none());
        assert!(parsed.get("help").is_none());
        assert!(parsed.get("source_text").is_none());
    }

    #[test]
    fn json_roundtrip() {
        let d = Diagnostic::new(Severity::Error, "META-001", "test")
            .with_location(PathBuf::from("test.yml"), 1, 1)
            .with_yaml_path("entities.customer.name")
            .with_metadata_object_id("ent_customer")
            .with_help("fix it");

        let renderer = JsonRenderer::new();
        let json_str = renderer.render(&d);
        let parsed: Diagnostic = serde_json::from_str(&json_str).unwrap();

        assert_eq!(d, parsed);
    }

    #[test]
    fn no_timestamp_or_machine_fields() {
        let d = Diagnostic::new(Severity::Error, "META-001", "test").with_location(
            PathBuf::from("test.yml"),
            1,
            1,
        );

        let renderer = JsonRenderer::new();
        let json_str = renderer.render(&d);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let obj = parsed.as_object().unwrap();
        for key in obj.keys() {
            assert!(
                !["timestamp", "hostname", "username", "machine_id"].contains(&key.as_str()),
                "diagnostic must not contain machine-specific field: {key}"
            );
        }
    }
}
