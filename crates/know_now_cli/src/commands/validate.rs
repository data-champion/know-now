use know_now_diagnostics::diagnostic::{Diagnostic, Severity};

use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct ValidateArgs;

pub fn run(ctx: &CommandContext, _args: &ValidateArgs) -> anyhow::Result<()> {
    let metadata = crate::commands::load_project_metadata(ctx)?;
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();

    let (_id_result, id_diags) = know_now_identity::check_ids(&metadata);
    all_diagnostics.extend(id_diags);

    let build_result = know_now_validate::builder::build_project_graph(&metadata);
    all_diagnostics.extend(build_result.diagnostics);

    let policy = know_now_policy::dc_standard::DcStandard;
    let policy_diags = know_now_policy::engine::evaluate_policy(&policy, &metadata);
    all_diagnostics.extend(policy_diags);

    all_diagnostics.sort_by(|a, b| b.severity.cmp(&a.severity));

    let has_errors = all_diagnostics.iter().any(Diagnostic::is_error);

    match ctx.format {
        OutputFormat::Json => {
            let payload = ValidationPayload {
                valid: !has_errors,
                diagnostics: &all_diagnostics,
            };
            if has_errors {
                let envelope = crate::output::JsonEnvelope::error("validate", &payload);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                let envelope = crate::output::JsonEnvelope::success("validate", &payload);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
        }
        OutputFormat::Sarif => {
            let sarif = build_sarif(&all_diagnostics);
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text => {
            if all_diagnostics.is_empty() {
                println!("Validation passed. No issues found.");
            } else {
                for d in &all_diagnostics {
                    let path_suffix = d
                        .yaml_path
                        .as_ref()
                        .map_or(String::new(), |p| format!(" at {p}"));
                    println!("  {}: [{}] {}{path_suffix}", d.severity, d.code, d.message);
                    if let Some(ref help) = d.help {
                        println!("    help: {help}");
                    }
                }
                let error_count = all_diagnostics.iter().filter(|d| d.is_error()).count();
                let warning_count = all_diagnostics
                    .iter()
                    .filter(|d| d.severity == Severity::Warning)
                    .count();
                let info_count = all_diagnostics
                    .iter()
                    .filter(|d| d.severity == Severity::Info)
                    .count();
                println!();
                println!(
                    "Validation {}: {} error(s), {} warning(s), {} info",
                    if has_errors { "failed" } else { "passed" },
                    error_count,
                    warning_count,
                    info_count,
                );
            }
        }
    }

    if has_errors {
        std::process::exit(crate::exit_code::VALIDATION_ERROR);
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct ValidationPayload<'a> {
    valid: bool,
    diagnostics: &'a [Diagnostic],
}

fn severity_to_sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "note",
        Severity::Warning => "warning",
        Severity::Error | Severity::Blocking => "error",
    }
}

pub(crate) fn build_sarif(diagnostics: &[Diagnostic]) -> serde_json::Value {
    let results: Vec<serde_json::Value> = diagnostics
        .iter()
        .map(|d| {
            let mut result = serde_json::json!({
                "ruleId": d.code,
                "level": severity_to_sarif_level(d.severity),
                "message": { "text": d.message },
            });
            if let Some(ref loc) = d.location {
                result["locations"] = serde_json::json!([{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": loc.file.display().to_string(),
                        },
                        "region": {
                            "startLine": loc.line,
                            "startColumn": loc.column,
                        }
                    }
                }]);
            }
            result
        })
        .collect();

    let rule_ids: std::collections::BTreeSet<&str> =
        diagnostics.iter().map(|d| d.code.as_str()).collect();
    let rules: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "know-now",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}
