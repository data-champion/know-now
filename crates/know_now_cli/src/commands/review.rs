use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use know_now_contract::contract::GeneratorContract;
use serde::{Deserialize, Serialize};

use crate::commands::load_project_metadata;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum ReviewCommand {
    /// Export a review pack for stakeholder review
    Export(ReviewExportArgs),
}

#[derive(Debug, clap::Args)]
pub struct ReviewExportArgs {
    /// Output directory for the review pack
    #[arg(long, default_value = "docs/exported")]
    pub output: PathBuf,

    /// Preview export contents without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
struct ReviewPack {
    output_dir: String,
    files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewState {
    items: Vec<ReviewItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewItem {
    object_id: String,
    status: String,
    note: Option<String>,
}

pub fn run(ctx: &CommandContext, cmd: &ReviewCommand) -> anyhow::Result<()> {
    match cmd {
        ReviewCommand::Export(args) => run_export(ctx, args),
    }
}

fn run_export(ctx: &CommandContext, args: &ReviewExportArgs) -> anyhow::Result<()> {
    let metadata = load_project_metadata(ctx)?;
    let build_result = know_now_validate::builder::build_project_graph(&metadata);
    let graph = build_result
        .graph
        .ok_or_else(|| anyhow::anyhow!("project graph unavailable — metadata has errors"))?;
    let contract = know_now_core::projection::project_graph_to_contract(&graph);

    let review_state = load_review_state(ctx);
    let manifest_summary = load_manifest_summary(ctx);

    let date = today_date();
    let review_dir = args.output.join(format!("review_{date}"));

    let mut files = Vec::new();

    let summary_md = generate_summary(&contract, &review_state, manifest_summary.as_ref());
    files.push(("summary.md".to_owned(), summary_md));

    for entity in &contract.entities {
        let entity_md = generate_entity_md(entity, &review_state);
        files.push((format!("entities/{}.md", entity.name), entity_md));
    }

    let manifest_json = generate_manifest_summary_json(manifest_summary.as_ref());
    files.push(("manifest_summary.json".to_owned(), manifest_json));

    if args.dry_run {
        let file_names: Vec<String> = files.iter().map(|(name, _)| name.clone()).collect();
        match ctx.format {
            OutputFormat::Json | OutputFormat::Sarif => {
                let pack = ReviewPack {
                    output_dir: review_dir.display().to_string(),
                    files: file_names,
                };
                let envelope = JsonEnvelope::success("review export", &pack);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputFormat::Text | OutputFormat::Quiet => {
                println!("Review pack would be written to: {}", review_dir.display());
                for f in &file_names {
                    println!("  - {f}");
                }
            }
        }
        return Ok(());
    }

    let abs_review_dir = ctx.project_root.join(&review_dir);
    for (name, content) in &files {
        let file_path = abs_review_dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, content)?;
    }

    let file_names: Vec<String> = files.iter().map(|(name, _)| name.clone()).collect();
    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let pack = ReviewPack {
                output_dir: abs_review_dir.display().to_string(),
                files: file_names,
            };
            let envelope = JsonEnvelope::success("review export", &pack);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("Review pack exported to: {}", abs_review_dir.display());
            println!("  {} files written", files.len());
        }
    }

    Ok(())
}

fn load_review_state(ctx: &CommandContext) -> ReviewState {
    let path = ctx.project_root.join(".knownow").join("review_state.json");
    let Ok(content) = fs::read_to_string(path) else {
        return ReviewState { items: vec![] };
    };
    serde_json::from_str(&content).unwrap_or(ReviewState { items: vec![] })
}

#[derive(Debug, Clone, Serialize)]
struct ManifestSummaryInfo {
    manifest_hash: String,
    metadata_hash: String,
    generation_status: String,
    artifact_count: usize,
}

fn load_manifest_summary(ctx: &CommandContext) -> Option<ManifestSummaryInfo> {
    let manifest_path = ctx.project_root.join("generated").join("manifest.json");
    let content = fs::read_to_string(manifest_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(ManifestSummaryInfo {
        manifest_hash: format!(
            "sha256:{}",
            know_now_writer::manifest_builder::sha256_hex(content.as_bytes())
        ),
        metadata_hash: json["input_hash"].as_str().unwrap_or("").to_owned(),
        generation_status: "complete".to_owned(),
        artifact_count: json["artifacts"].as_array().map_or(0, Vec::len),
    })
}

fn generate_summary(
    contract: &GeneratorContract,
    review_state: &ReviewState,
    manifest_summary: Option<&ManifestSummaryInfo>,
) -> String {
    let mut md = String::new();
    md.push_str("# Review Summary\n\n");

    md.push_str("## Entities\n\n");
    md.push_str("| Entity | Attributes | Status |\n");
    md.push_str("|--------|-----------|--------|\n");
    for entity in &contract.entities {
        let status = review_status_for(&review_state.items, &entity.id);
        let _ = writeln!(md, "| {} | {} | {status} |", entity.name, entity.attributes.len());
    }
    md.push('\n');

    if !contract.relationships.is_empty() {
        md.push_str("## Relationships\n\n");
        md.push_str("| From | To | Type | Status |\n");
        md.push_str("|------|-----|------|--------|\n");
        for rel in &contract.relationships {
            let status = review_status_for(&review_state.items, &rel.id);
            let cardinality = rel.cardinality.as_deref().unwrap_or("\u{2014}");
            let _ = writeln!(md, "| {} | {} | {cardinality} | {status} |", rel.from_entity, rel.to_entity);
        }
        md.push('\n');
    }

    md.push_str("## Warnings\n\n");
    if contract.entities.iter().all(|e| !e.attributes.is_empty()) {
        md.push_str("No warnings.\n\n");
    } else {
        for entity in &contract.entities {
            if entity.attributes.is_empty() {
                let _ = writeln!(md, "- Entity `{}` has no attributes defined", entity.name);
            }
        }
        md.push('\n');
    }

    md.push_str("## Open Questions\n\n");
    let draft_items: Vec<_> = review_state
        .items
        .iter()
        .filter(|i| i.status == "draft" || i.status == "needs-confirmation")
        .collect();
    if draft_items.is_empty() {
        md.push_str("No open questions.\n\n");
    } else {
        for item in &draft_items {
            let note_suffix = item.note.as_deref().map_or(String::new(), |n| format!(" ({n})"));
            let _ = writeln!(md, "- `{}` \u{2014} status: {}{note_suffix}", item.object_id, item.status);
        }
        md.push('\n');
    }

    if let Some(ms) = manifest_summary {
        md.push_str("## Generation Status\n\n");
        let _ = writeln!(md, "- Manifest hash: `{}`", ms.manifest_hash);
        let _ = writeln!(md, "- Metadata hash: `{}`", ms.metadata_hash);
        let _ = writeln!(md, "- Status: {}", ms.generation_status);
        let _ = writeln!(md, "- Artifacts: {}", ms.artifact_count);
        md.push('\n');
    }

    md
}

fn generate_entity_md(
    entity: &know_now_contract::contract::ContractEntity,
    review_state: &ReviewState,
) -> String {
    let mut md = String::new();
    let status = review_status_for(&review_state.items, &entity.id);

    let _ = writeln!(md, "# {}\n", entity.name);
    let _ = writeln!(md, "**Status:** {status}\n");

    if let Some(ref desc) = entity.description {
        let _ = writeln!(md, "{desc}\n");
    }

    if !entity.attributes.is_empty() {
        md.push_str("## Attributes\n\n");
        md.push_str("| Name | Type | Required | Description |\n");
        md.push_str("|------|------|----------|-------------|\n");
        for attr in &entity.attributes {
            let required = attr.required.map_or("\u{2014}", |r| if r { "yes" } else { "no" });
            let logical = attr.logical_type.as_deref().unwrap_or("\u{2014}");
            let desc = attr.description.as_deref().unwrap_or("");
            let _ = writeln!(md, "| {} | {logical} | {required} | {desc} |", attr.name);
        }
        md.push('\n');
    }

    md
}

fn generate_manifest_summary_json(summary: Option<&ManifestSummaryInfo>) -> String {
    summary.map_or_else(
        || r#"{"generation_status": "not yet generated"}"#.into(),
        |s| serde_json::to_string_pretty(s).unwrap_or_else(|_| "{}".into()),
    )
}

fn review_status_for(items: &[ReviewItem], object_id: &str) -> &'static str {
    items
        .iter()
        .find(|i| i.object_id == object_id)
        .map_or("draft", |i| match i.status.as_str() {
            "confirmed" => "confirmed",
            "rejected" => "rejected",
            "deferred" => "deferred",
            "needs-confirmation" => "needs-confirmation",
            _ => "draft",
        })
}

fn today_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86_400;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let z = days_since_epoch + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
