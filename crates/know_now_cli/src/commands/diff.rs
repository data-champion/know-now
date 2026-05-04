use std::fmt::Write as _;
use std::path::PathBuf;

use know_now_diff::{diff, format_json, format_text, DiffResult};
use serde::Serialize;

use crate::commands::load_project_metadata;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct DiffArgs {
    /// Baseline to compare against
    #[arg(long, default_value = "last-generation")]
    pub baseline: String,

    /// Require stable IDs; fail fast if missing
    #[arg(long)]
    pub migration_safe: bool,

    /// Show which generated artifacts are affected by changes
    #[arg(long)]
    pub impact: bool,

    /// Scan custom/ directory for references to changed objects
    #[arg(long)]
    pub scan_custom: bool,
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
            if args.impact || args.scan_custom {
                let extended = build_extended_output(ctx, &result, args);
                let envelope = JsonEnvelope::success("diff", &extended);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("{}", format_json(&result));
            }
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("{}", format_text(&result));
            if args.impact {
                print_impact_report(ctx, &result);
            }
            if args.scan_custom {
                print_custom_scan(ctx, &result);
            }
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

#[derive(Debug, Serialize)]
struct ExtendedDiffOutput {
    diff: serde_json::Value,
    impact: Option<Vec<ImpactEntry>>,
    custom_references: Option<Vec<CustomReference>>,
}

#[derive(Debug, Serialize)]
struct ImpactEntry {
    changed_object_id: String,
    changed_object_name: String,
    affected_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CustomReference {
    file: String,
    line: usize,
    reference_text: String,
    match_kind: String,
    matched_object: String,
}

fn build_extended_output(
    ctx: &CommandContext,
    result: &DiffResult,
    args: &DiffArgs,
) -> ExtendedDiffOutput {
    let diff_json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
    let impact = if args.impact {
        Some(compute_impact(ctx, result))
    } else {
        None
    };
    let custom_references = if args.scan_custom {
        Some(scan_custom_references(ctx, result))
    } else {
        None
    };
    ExtendedDiffOutput {
        diff: diff_json,
        impact,
        custom_references,
    }
}

fn compute_impact(ctx: &CommandContext, result: &DiffResult) -> Vec<ImpactEntry> {
    let manifest_path = ctx.project_root.join("generated").join("manifest.json");
    let Some(manifest) = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|c| know_now_writer::manifest::ManifestV1::from_json(&c).ok())
    else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for change in &result.changes {
        let affected: Vec<String> = manifest
            .artifacts
            .iter()
            .filter(|a| {
                a.metadata_object_ids.iter().any(|oid| {
                    oid == &change.id || oid == &change.name
                })
            })
            .map(|a| a.path.display().to_string())
            .collect();

        if !affected.is_empty() {
            entries.push(ImpactEntry {
                changed_object_id: change.id.clone(),
                changed_object_name: change.name.clone(),
                affected_artifacts: affected,
            });
        }
    }
    entries
}

fn scan_custom_references(ctx: &CommandContext, result: &DiffResult) -> Vec<CustomReference> {
    let custom_dir = ctx.project_root.join("custom");
    if !custom_dir.is_dir() {
        return Vec::new();
    }

    let changed_ids: Vec<(&str, &str)> = result
        .changes
        .iter()
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();

    let mut references = Vec::new();
    scan_directory(&custom_dir, &changed_ids, &mut references, &custom_dir);
    references
}

fn scan_directory(
    dir: &std::path::Path,
    changed: &[(&str, &str)],
    references: &mut Vec<CustomReference>,
    custom_root: &std::path::Path,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(&path, changed, references, custom_root);
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        let relative = path
            .strip_prefix(custom_root)
            .unwrap_or(&path)
            .display()
            .to_string();

        for (line_no, line) in content.lines().enumerate() {
            for &(id, name) in changed {
                if !id.is_empty() && line.contains(id) {
                    references.push(CustomReference {
                        file: relative.clone(),
                        line: line_no + 1,
                        reference_text: truncate_line(line, 120),
                        match_kind: "exact_id".into(),
                        matched_object: id.to_owned(),
                    });
                } else if !name.is_empty() && line.contains(name) {
                    let kind = if is_word_boundary(line, name) {
                        "exact_name"
                    } else {
                        "heuristic"
                    };
                    references.push(CustomReference {
                        file: relative.clone(),
                        line: line_no + 1,
                        reference_text: truncate_line(line, 120),
                        match_kind: kind.into(),
                        matched_object: name.to_owned(),
                    });
                }
            }
        }
    }
}

fn is_word_boundary(line: &str, word: &str) -> bool {
    let Some(pos) = line.find(word) else {
        return false;
    };
    let before_ok = pos == 0
        || !line.as_bytes()[pos - 1].is_ascii_alphanumeric()
            && line.as_bytes()[pos - 1] != b'_';
    let end = pos + word.len();
    let after_ok = end >= line.len()
        || !line.as_bytes()[end].is_ascii_alphanumeric()
            && line.as_bytes()[end] != b'_';
    before_ok && after_ok
}

fn truncate_line(line: &str, max: usize) -> String {
    if line.len() <= max {
        line.to_owned()
    } else {
        format!("{}...", &line[..max])
    }
}

fn print_impact_report(ctx: &CommandContext, result: &DiffResult) {
    let entries = compute_impact(ctx, result);
    if entries.is_empty() {
        println!("\nNo artifact impact detected.");
        return;
    }
    println!("\nImpact analysis:");
    for entry in &entries {
        let mut line = String::new();
        let _ = write!(line, "  {} ({})", entry.changed_object_name, entry.changed_object_id);
        println!("{line}");
        for artifact in &entry.affected_artifacts {
            println!("    -> {artifact}");
        }
    }
}

fn print_custom_scan(ctx: &CommandContext, result: &DiffResult) {
    let refs = scan_custom_references(ctx, result);
    if refs.is_empty() {
        println!("\nNo custom/ references to changed objects found.");
        return;
    }
    println!("\nCustom references to changed objects:");
    for r in &refs {
        println!(
            "  {}:{} [{}] {} — {}",
            r.file, r.line, r.match_kind, r.matched_object, r.reference_text
        );
    }
}
