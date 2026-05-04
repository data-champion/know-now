use std::fmt::Write;

use crate::diagnostic::{Diagnostic, DiagnosticRenderer};

pub struct TextRenderer {
    pub color: bool,
}

impl TextRenderer {
    #[must_use]
    pub fn new(color: bool) -> Self {
        Self { color }
    }
}

impl DiagnosticRenderer for TextRenderer {
    fn render(&self, d: &Diagnostic) -> String {
        let mut out = String::new();

        let severity = d.severity.to_string();
        let _ = write!(out, "{severity}[{}]: {}", d.code, d.message);

        if let Some(loc) = &d.location {
            let _ = write!(
                out,
                "\n  --> {}:{}:{}",
                loc.file.display(),
                loc.line,
                loc.column
            );
        }

        if let Some(snippet) = &d.source_text {
            let line_num = snippet.line_number.to_string();
            let gutter_width = line_num.len();
            let indent = " ".repeat(snippet.highlight_start as usize);
            let highlight = "^".repeat(snippet.highlight_len as usize);

            let _ = write!(out, "\n{:>gutter_width$} |", "");
            let _ = write!(out, "\n{line_num} |     {}", snippet.text);
            let _ = write!(
                out,
                "\n{:>gutter_width$} |{indent}{highlight} {}",
                "", snippet.label,
            );
        }

        if let Some(help) = &d.help {
            if d.source_text.is_some() {
                let gutter_width = d
                    .source_text
                    .as_ref()
                    .map_or(1, |s| s.line_number.to_string().len());
                let _ = write!(out, "\n{:>gutter_width$} |", "");
            }
            let _ = write!(out, "\nhelp: {help}");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::diagnostic::Severity;

    #[test]
    fn renders_prd_12_4_example_byte_identical() {
        let d = Diagnostic::new(
            Severity::Error,
            "META-REL-001",
            "relationship references unknown entity `customers`",
        )
        .with_location(PathBuf::from("metadata/relationships/orders.yml"), 12, 18)
        .with_source_snippet(12, "to_entity: customers", 16, 9, "unknown entity")
        .with_help("did you mean `customer`?");

        let renderer = TextRenderer::new(false);
        let output = renderer.render(&d);

        let expected = "\
error[META-REL-001]: relationship references unknown entity `customers`
  --> metadata/relationships/orders.yml:12:18
   |
12 |     to_entity: customers
   |                ^^^^^^^^^ unknown entity
   |
help: did you mean `customer`?";

        assert_eq!(output, expected);
    }

    #[test]
    fn renders_simple_diagnostic_without_location() {
        let d = Diagnostic::new(Severity::Warning, "WARN-001", "something is off");

        let renderer = TextRenderer::new(false);
        let output = renderer.render(&d);

        assert_eq!(output, "warning[WARN-001]: something is off");
    }

    #[test]
    fn renders_diagnostic_with_help_only() {
        let d = Diagnostic::new(Severity::Info, "INFO-001", "did you know?").with_help("try this");

        let renderer = TextRenderer::new(false);
        let output = renderer.render(&d);

        assert_eq!(output, "info[INFO-001]: did you know?\nhelp: try this");
    }
}
