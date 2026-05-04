use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_lock::check::{self, LOCK_CORRUPT_005, LOCK_MISSING_004, LOCK_STALE_003};
use know_now_lock::lockfile::Lockfile;
use know_now_lock::LOCKFILE_NAME;

use crate::commands::lock::resolve_current_versions;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Require lockfile match (LIFE-008)
    #[arg(long)]
    pub locked: bool,
}

pub fn run(ctx: &CommandContext, args: &CheckArgs) -> anyhow::Result<()> {
    let metadata = crate::commands::load_project_metadata(ctx)?;
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();

    let (_id_result, id_diags) = know_now_identity::check_ids(&metadata);
    all_diagnostics.extend(id_diags);

    let build_result = know_now_validate::builder::build_project_graph(&metadata);
    all_diagnostics.extend(build_result.diagnostics);

    let policy = know_now_policy::dc_standard::DcStandard;
    let policy_diags = know_now_policy::engine::evaluate_policy(&policy, &metadata);
    all_diagnostics.extend(policy_diags);

    let (lock_warnings, lock_error) = if args.locked {
        run_lockfile_check(ctx)
    } else {
        (Vec::new(), None)
    };

    all_diagnostics.sort_by(|a, b| b.severity.cmp(&a.severity));

    let has_diag_errors = all_diagnostics.iter().any(Diagnostic::is_error);
    let has_errors = has_diag_errors || lock_error.is_some();

    emit_output(
        ctx,
        args,
        &all_diagnostics,
        &lock_warnings,
        lock_error.as_deref(),
        has_errors,
    )?;

    if has_errors {
        std::process::exit(crate::exit_code::VALIDATION_ERROR);
    }
    Ok(())
}

fn run_lockfile_check(ctx: &CommandContext) -> (Vec<String>, Option<String>) {
    let lock_path = ctx.project_root.join(LOCKFILE_NAME);
    let mut warnings = Vec::new();

    if !lock_path.exists() {
        return (
            warnings,
            Some(format!(
                "{LOCK_MISSING_004}: lockfile not found; run 'know-now lock update'"
            )),
        );
    }

    let lockfile = match Lockfile::read_from(&lock_path) {
        Ok(lf) => lf,
        Err(e) => return (warnings, Some(format!("{LOCK_CORRUPT_005}: {e}"))),
    };

    let resolved = resolve_current_versions();
    let result = check::check_lockfile(&lockfile, &resolved);
    let error = if result.is_ok() {
        None
    } else {
        let drift_summary: Vec<String> = result
            .drifted_fields
            .iter()
            .map(|d| {
                format!(
                    "{}: locked={} resolved={}",
                    d.field, d.locked_value, d.resolved_value
                )
            })
            .collect();
        Some(format!(
            "{LOCK_STALE_003}: lockfile drift detected: {}",
            drift_summary.join("; ")
        ))
    };
    warnings.extend(result.warnings);
    (warnings, error)
}

fn emit_output(
    ctx: &CommandContext,
    args: &CheckArgs,
    diagnostics: &[Diagnostic],
    lock_warnings: &[String],
    lock_error: Option<&str>,
    has_errors: bool,
) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let payload = CheckPayload {
                passed: !has_errors,
                diagnostics,
                lock_status: if args.locked {
                    Some(LockStatus {
                        passed: lock_error.is_none(),
                        error: lock_error,
                        warnings: lock_warnings,
                    })
                } else {
                    None
                },
            };
            let envelope = if has_errors {
                JsonEnvelope::error("check", &payload)
            } else {
                JsonEnvelope::success("check", &payload)
            };
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Sarif => {
            let sarif = crate::commands::validate::build_sarif(diagnostics);
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text => {
            emit_text(diagnostics, lock_warnings, lock_error, has_errors);
        }
    }
    Ok(())
}

fn emit_text(
    diagnostics: &[Diagnostic],
    lock_warnings: &[String],
    lock_error: Option<&str>,
    has_errors: bool,
) {
    for d in diagnostics {
        let path_suffix = d
            .yaml_path
            .as_ref()
            .map_or(String::new(), |p| format!(" at {p}"));
        println!("  {}: [{}] {}{path_suffix}", d.severity, d.code, d.message);
        if let Some(ref help) = d.help {
            println!("    help: {help}");
        }
    }

    for w in lock_warnings {
        println!("  warning: {w}");
    }
    if let Some(err) = lock_error {
        println!("  error: {err}");
    }

    let error_count = diagnostics.iter().filter(|d| d.is_error()).count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    if has_errors {
        println!();
        println!(
            "Check failed: {} error(s), {} warning(s){}",
            error_count,
            warning_count,
            if lock_error.is_some() {
                ", lockfile drift"
            } else {
                ""
            }
        );
    } else if diagnostics.is_empty() && lock_warnings.is_empty() {
        println!("Check passed. Project is ready to generate.");
    } else {
        println!();
        println!(
            "Check passed with {} warning(s). Project is ready to generate.",
            warning_count + lock_warnings.len()
        );
    }
}

#[derive(serde::Serialize)]
struct CheckPayload<'a> {
    passed: bool,
    diagnostics: &'a [Diagnostic],
    #[serde(skip_serializing_if = "Option::is_none")]
    lock_status: Option<LockStatus<'a>>,
}

#[derive(serde::Serialize)]
struct LockStatus<'a> {
    passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
    warnings: &'a [String],
}
