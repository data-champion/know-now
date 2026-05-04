use std::path::PathBuf;

use know_now_diagnostics::diagnostic::{Diagnostic, DiagnosticRenderer, Severity};
use know_now_diagnostics::render_json::JsonRenderer;
use know_now_diagnostics::render_text::TextRenderer;

/// Build a fully-populated diagnostic with all fields set.
fn full_diagnostic() -> Diagnostic {
    Diagnostic::new(
        Severity::Error,
        "META-REL-001",
        "relationship references unknown entity `customers`",
    )
    .with_location(PathBuf::from("metadata/relationships/orders.yml"), 12, 18)
    .with_yaml_path("relationships.orders.to_entity")
    .with_metadata_object_id("rel_order_customer")
    .with_source_snippet(12, "to_entity: customers", 16, 9, "unknown entity")
    .with_help("did you mean `customer`?")
}

/// Build a minimal diagnostic with only required fields.
fn minimal_diagnostic() -> Diagnostic {
    Diagnostic::new(Severity::Warning, "WARN-001", "something is off")
}

/// Build a diagnostic with help but no location or snippet.
fn help_only_diagnostic() -> Diagnostic {
    Diagnostic::new(Severity::Info, "INFO-042", "consider using a business key")
        .with_help("add a `business_key` field to the entity definition")
}

#[test]
fn text_full_diagnostic_snapshot() {
    let renderer = TextRenderer::new(false);
    let output = renderer.render(&full_diagnostic());
    insta::assert_snapshot!(output);
}

#[test]
fn text_minimal_diagnostic_snapshot() {
    let renderer = TextRenderer::new(false);
    let output = renderer.render(&minimal_diagnostic());
    insta::assert_snapshot!(output);
}

#[test]
fn text_help_only_diagnostic_snapshot() {
    let renderer = TextRenderer::new(false);
    let output = renderer.render(&help_only_diagnostic());
    insta::assert_snapshot!(output);
}

#[test]
fn json_full_diagnostic_snapshot() {
    let renderer = JsonRenderer::new();
    let output = renderer.render(&full_diagnostic());
    // Pretty-print the JSON for a readable snapshot.
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    insta::assert_snapshot!(pretty);
}

#[test]
fn json_minimal_diagnostic_snapshot() {
    let renderer = JsonRenderer::new();
    let output = renderer.render(&minimal_diagnostic());
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    insta::assert_snapshot!(pretty);
}

#[test]
fn json_multiple_diagnostics_snapshot() {
    let renderer = JsonRenderer::new();
    let diagnostics = vec![
        full_diagnostic(),
        minimal_diagnostic(),
        help_only_diagnostic(),
    ];
    let output = renderer.render_all(&diagnostics);
    // render_all produces newline-delimited JSON; pretty-print each line.
    let pretty: String = output
        .lines()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            serde_json::to_string_pretty(&value).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n---\n");
    insta::assert_snapshot!(pretty);
}
