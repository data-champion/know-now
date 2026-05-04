use std::path::PathBuf;

use know_now_diff::{diff, format_json, format_text};

use crate::commands::load_project_metadata;
use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct DiffArgs {
    /// Baseline to compare against
    #[arg(long, default_value = "last-generation")]
    pub baseline: String,

    /// Require stable IDs; fail fast if missing
    #[arg(long)]
    pub migration_safe: bool,
}

pub fn run(ctx: &CommandContext, args: &DiffArgs) -> anyhow::Result<()> {
    let metadata = load_project_metadata(ctx)?;
    let build_result = know_now_validate::builder::build_project_graph(&metadata);
    let graph = build_result
        .graph
        .ok_or_else(|| anyhow::anyhow!("project graph unavailable — metadata has errors"))?;
    let right_contract = know_now_core::projection::project_graph_to_contract(&graph);

    let left_contract = load_baseline(ctx, &args.baseline)?;

    if args.migration_safe {
        check_stable_ids(&right_contract)?;
    }

    let result = diff(&left_contract, &right_contract);

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            println!("{}", format_json(&result));
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("{}", format_text(&result));
        }
    }

    if args.migration_safe && result.summary.has_breaking() {
        anyhow::bail!(
            "DIFF-BREAK-001: breaking or destructive changes detected in --migration-safe mode"
        );
    }

    Ok(())
}

fn load_baseline(
    ctx: &CommandContext,
    baseline: &str,
) -> anyhow::Result<know_now_contract::contract::GeneratorContract> {
    if baseline == "last-generation" {
        return load_manifest_baseline(ctx);
    }

    if let Some(path) = baseline.strip_prefix("manifest:") {
        return load_manifest_from_path(&PathBuf::from(path));
    }

    if baseline.starts_with("git:") {
        anyhow::bail!("DIFF-GIT-001: git:<ref> baseline is not yet implemented (Phase 3+)");
    }

    anyhow::bail!("unknown baseline format: {baseline}. Use 'last-generation', 'manifest:<path>', or 'git:<ref>'");
}

fn load_manifest_baseline(
    ctx: &CommandContext,
) -> anyhow::Result<know_now_contract::contract::GeneratorContract> {
    let manifest_path = ctx.project_root.join("generated/manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!(
            "DIFF-NO-BASELINE-002: no previous generation found at {}. Run 'know-now generate' first, or use --baseline manifest:<path>",
            manifest_path.display()
        );
    }
    load_manifest_from_path(&manifest_path)
}

fn load_manifest_from_path(
    path: &PathBuf,
) -> anyhow::Result<know_now_contract::contract::GeneratorContract> {
    let content = std::fs::read_to_string(path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;

    let contract = manifest
        .get("contract")
        .ok_or_else(|| anyhow::anyhow!("manifest missing 'contract' field"))?;

    let contract: know_now_contract::contract::GeneratorContract =
        serde_json::from_value(contract.clone())?;
    Ok(contract)
}

fn check_stable_ids(
    contract: &know_now_contract::contract::GeneratorContract,
) -> anyhow::Result<()> {
    let mut missing = Vec::new();
    for entity in &contract.entities {
        if entity.id.is_empty() {
            missing.push(format!("entity '{}'", entity.name));
        }
        for attr in &entity.attributes {
            if attr.id.is_empty() {
                missing.push(format!(
                    "attribute '{}' in entity '{}'",
                    attr.name, entity.name
                ));
            }
        }
    }
    for rel in &contract.relationships {
        if rel.id.is_empty() {
            missing.push(format!(
                "relationship '{}' -> '{}'",
                rel.from_entity, rel.to_entity
            ));
        }
    }

    if !missing.is_empty() {
        let list = missing.join(", ");
        anyhow::bail!(
            "DIFF-ID-003: --migration-safe requires stable IDs. Missing: {list}. Run 'know-now id suggest' to generate them."
        );
    }
    Ok(())
}
