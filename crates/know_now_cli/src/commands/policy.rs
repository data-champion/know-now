use std::collections::HashMap;

use serde::Serialize;

use know_now_catalog::{classify_drift, Catalog, DriftClass, DriftReport, ProjectState};
use know_now_policy::dc_standard::DcStandard;
use know_now_policy::discovery::{discover_packs, PackSource};
use know_now_policy::engine::{PolicyPack, PolicyPackInfo};
use know_now_policy::manifest::RuleDefinition;

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum PolicyCommand {
    /// Report policy version status and drift classification
    Status(PolicyStatusArgs),
    /// Explain a policy finding code
    Explain(PolicyExplainArgs),
}

#[derive(Debug, clap::Args)]
pub struct PolicyStatusArgs {
    /// Path to approved-version catalog file
    #[arg(long)]
    pub catalog: Option<std::path::PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct PolicyExplainArgs {
    /// Policy finding code to explain (e.g. POL-NAM-001)
    pub code: String,
}

pub fn run(ctx: &CommandContext, cmd: &PolicyCommand) -> anyhow::Result<()> {
    match cmd {
        PolicyCommand::Status(args) => run_status(ctx, args),
        PolicyCommand::Explain(args) => run_explain(ctx, args),
    }
}

#[derive(Debug, Serialize)]
struct PolicyStatusReport {
    packs: Vec<PackStatus>,
    drift: Option<DriftReport>,
}

#[derive(Debug, Serialize)]
struct PackStatus {
    name: String,
    configured_version: String,
    locked_version: Option<String>,
    source: String,
    drift: Option<DriftClass>,
}

fn run_status(ctx: &CommandContext, args: &PolicyStatusArgs) -> anyhow::Result<()> {
    let mut packs_status = Vec::new();

    let builtin = DcStandard;
    let info = builtin.info();
    packs_status.push(PackStatus {
        name: info.pack.clone(),
        configured_version: info.version.clone(),
        locked_version: read_locked_policy_version(ctx, &info.pack),
        source: "built_in".into(),
        drift: None,
    });

    let discovered = discover_packs(&ctx.project_root);
    for dp in &discovered {
        packs_status.push(PackStatus {
            name: dp.manifest.name.clone(),
            configured_version: dp.manifest.version.clone(),
            locked_version: read_locked_policy_version(ctx, &dp.manifest.name),
            source: match dp.source {
                PackSource::ProjectLocal => "project_local".into(),
                PackSource::Custom => "custom".into(),
                PackSource::BuiltIn => "built_in".into(),
            },
            drift: None,
        });
    }

    let drift_report = load_catalog(ctx, args)?.map(|catalog| {
        let state = build_project_state(ctx, &info, &discovered);
        let report = classify_drift(&catalog, &state);

        for pack in &mut packs_status {
            if let Some(entry) = report.entries.iter().find(|e| e.name == pack.name) {
                pack.drift = Some(entry.drift);
            }
        }

        report
    });

    let report = PolicyStatusReport {
        packs: packs_status,
        drift: drift_report,
    };

    emit_status(ctx, &report)
}

fn emit_status(ctx: &CommandContext, report: &PolicyStatusReport) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("policy status", report);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("Policy Status");
            println!("{}", "-".repeat(60));

            for pack in &report.packs {
                let locked = pack
                    .locked_version
                    .as_deref()
                    .unwrap_or("(not locked)");
                let drift_label = pack
                    .drift
                    .map(|d| format!(" [drift: {d}]"))
                    .unwrap_or_default();
                println!(
                    "  {} v{} (locked: {}, source: {}){drift_label}",
                    pack.name, pack.configured_version, locked, pack.source
                );
            }

            println!();
            if let Some(ref drift) = report.drift {
                println!("Overall drift: {}", drift.overall);
                for entry in &drift.entries {
                    if entry.drift != DriftClass::None {
                        println!(
                            "  {}/{}: {} ({})",
                            entry.component, entry.name, entry.drift, entry.reason
                        );
                    }
                }
            } else {
                println!("No approved-version catalog configured. Use --catalog to specify one.");
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct PolicyExplainReport {
    code: String,
    found: bool,
    rule: Option<RuleExplanation>,
}

#[derive(Debug, Serialize)]
struct RuleExplanation {
    code: String,
    name: String,
    severity: String,
    source: String,
    rationale: String,
    remediation: String,
}

fn run_explain(ctx: &CommandContext, args: &PolicyExplainArgs) -> anyhow::Result<()> {
    let code = &args.code;

    if let Some(explanation) = find_builtin_rule(code) {
        return emit_explain(ctx, code, Some(explanation));
    }

    if let Some(explanation) = find_declarative_rule(ctx, code) {
        return emit_explain(ctx, code, Some(explanation));
    }

    emit_explain(ctx, code, None)
}

fn find_builtin_rule(code: &str) -> Option<RuleExplanation> {
    let builtin = DcStandard;
    builtin.rules().iter().find(|r| r.code == code).map(|r| {
        RuleExplanation {
            code: r.code.to_owned(),
            name: r.name.to_owned(),
            severity: "warning".into(),
            source: "dc_standard (built-in)".into(),
            rationale: r.rationale.to_owned(),
            remediation: r.remediation.to_owned(),
        }
    })
}

fn find_declarative_rule(ctx: &CommandContext, code: &str) -> Option<RuleExplanation> {
    let discovered = discover_packs(&ctx.project_root);
    for dp in &discovered {
        if let Some(rule) = find_rule_in_manifest(&dp.manifest.rules, code) {
            let source = format!("{} ({})", dp.manifest.name, source_label(&dp.source));
            return Some(RuleExplanation {
                code: rule.id.clone(),
                name: rule.id.clone(),
                severity: format!("{:?}", rule.severity).to_lowercase(),
                source,
                rationale: rule.rationale.clone(),
                remediation: rule.remediation.clone(),
            });
        }
    }
    None
}

fn find_rule_in_manifest<'a>(rules: &'a [RuleDefinition], code: &str) -> Option<&'a RuleDefinition> {
    rules.iter().find(|r| r.id == code)
}

fn source_label(source: &PackSource) -> &'static str {
    match source {
        PackSource::BuiltIn => "built-in",
        PackSource::ProjectLocal => "project-local",
        PackSource::Custom => "custom",
    }
}

fn emit_explain(
    ctx: &CommandContext,
    code: &str,
    explanation: Option<RuleExplanation>,
) -> anyhow::Result<()> {
    let report = PolicyExplainReport {
        code: code.to_owned(),
        found: explanation.is_some(),
        rule: explanation,
    };

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("policy explain", &report);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            if let Some(ref rule) = report.rule {
                println!("Policy Rule: {}", rule.code);
                println!("{}", "-".repeat(60));
                println!("  Name:        {}", rule.name);
                println!("  Severity:    {}", rule.severity);
                println!("  Source:      {}", rule.source);
                println!("  Rationale:   {}", rule.rationale);
                println!("  Remediation: {}", rule.remediation);
            } else {
                println!("Unknown policy code: {code}");
                println!("Run 'know-now policy status' to see available packs and their rules.");
            }
        }
    }
    Ok(())
}

fn load_catalog(
    ctx: &CommandContext,
    args: &PolicyStatusArgs,
) -> anyhow::Result<Option<Catalog>> {
    let catalog_path = if let Some(ref path) = args.catalog {
        path.clone()
    } else {
        let default = ctx.project_root.join(".knownow").join("catalog.json");
        if !default.exists() {
            return Ok(None);
        }
        default
    };

    if !catalog_path.exists() {
        anyhow::bail!(
            "catalog file not found: {}",
            catalog_path.display()
        );
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog: Catalog = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("invalid catalog JSON: {e}"))?;
    Ok(Some(catalog))
}

fn build_project_state(
    _ctx: &CommandContext,
    builtin_info: &PolicyPackInfo,
    discovered: &[know_now_policy::discovery::DiscoveredPack],
) -> ProjectState {
    let mut policies = HashMap::new();
    policies.insert(builtin_info.pack.clone(), builtin_info.version.clone());
    for dp in discovered {
        policies.insert(dp.manifest.name.clone(), dp.manifest.version.clone());
    }

    ProjectState {
        engine_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        metadata_schema_version: Some("0.1.0".to_owned()),
        generator_contract_version: Some(know_now_contract::CONTRACT_SCHEMA_VERSION.to_owned()),
        policies,
        templates: HashMap::new(),
        template_renderers: HashMap::new(),
        targets: HashMap::new(),
    }
}

fn read_locked_policy_version(ctx: &CommandContext, pack_name: &str) -> Option<String> {
    let lock_path = ctx.project_root.join(know_now_lock::LOCKFILE_NAME);
    let content = std::fs::read_to_string(&lock_path).ok()?;
    let lockfile = know_now_lock::lockfile::Lockfile::from_json(&content).ok()?;
    if lockfile.policy.pack == pack_name {
        Some(lockfile.policy.version)
    } else {
        None
    }
}
