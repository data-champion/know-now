use serde::Serialize;

use know_now_writer::manifest::{ArtifactEntry, ManifestV1};

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct ExplainArgs {
    /// Artifact path to explain (e.g. generated/ddl/postgres/schema.sql)
    #[arg(long)]
    pub artifact: Option<String>,

    /// Metadata object ID to trace (e.g. ent_customer)
    #[arg(long)]
    pub object_id: Option<String>,

    /// List all artifacts in the manifest
    #[arg(long)]
    pub list: bool,
}

#[derive(Debug, Serialize)]
struct ExplainReport {
    manifest_path: String,
    engine_version: String,
    project_id: String,
    artifact_count: usize,
    query: ExplainQuery,
    results: Vec<ArtifactExplanation>,
}

#[derive(Debug, Serialize)]
struct ExplainQuery {
    kind: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct ArtifactExplanation {
    path: String,
    kind: String,
    artifact_id: String,
    generator: String,
    generator_version: String,
    hash: String,
    metadata_object_ids: Vec<String>,
    trace_count: usize,
}

pub fn run(ctx: &CommandContext, args: &ExplainArgs) -> anyhow::Result<()> {
    let manifest_path = ctx.project_root.join("generated/manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!(
            "EXPLAIN-NO-MANIFEST-001: no manifest found at {}. Run 'know-now generate' first.",
            manifest_path.display()
        );
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest = ManifestV1::from_json(&content)
        .map_err(|e| anyhow::anyhow!("EXPLAIN-PARSE-002: manifest is not valid JSON: {e}"))?;

    if args.list {
        return emit_list(ctx, &manifest, &manifest_path);
    }

    if let Some(ref path) = args.artifact {
        let results = find_by_path(&manifest, path);
        return emit_results(ctx, &manifest, &manifest_path, "artifact", path, &results);
    }

    if let Some(ref oid) = args.object_id {
        let results = find_by_object_id(&manifest, oid);
        return emit_results(ctx, &manifest, &manifest_path, "object_id", oid, &results);
    }

    anyhow::bail!("usage: provide --artifact <path>, --object-id <id>, or --list");
}

fn find_by_path<'a>(manifest: &'a ManifestV1, path: &str) -> Vec<&'a ArtifactEntry> {
    manifest
        .artifacts
        .iter()
        .filter(|a| {
            let artifact_path = a.path.to_string_lossy();
            artifact_path == path || artifact_path.ends_with(path)
        })
        .collect()
}

fn find_by_object_id<'a>(manifest: &'a ManifestV1, oid: &str) -> Vec<&'a ArtifactEntry> {
    manifest
        .artifacts
        .iter()
        .filter(|a| a.metadata_object_ids.iter().any(|id| id == oid))
        .collect()
}

fn to_explanation(entry: &ArtifactEntry) -> ArtifactExplanation {
    ArtifactExplanation {
        path: entry.path.display().to_string(),
        kind: entry.kind.clone(),
        artifact_id: entry.artifact_id.clone(),
        generator: entry.generator.clone(),
        generator_version: entry.generator_version.clone(),
        hash: entry.hash.clone(),
        metadata_object_ids: entry.metadata_object_ids.clone(),
        trace_count: entry.trace.len(),
    }
}

fn emit_list(
    ctx: &CommandContext,
    manifest: &ManifestV1,
    manifest_path: &std::path::Path,
) -> anyhow::Result<()> {
    let explanations: Vec<_> = manifest.artifacts.iter().map(to_explanation).collect();

    let report = ExplainReport {
        manifest_path: manifest_path.display().to_string(),
        engine_version: manifest.engine_version.clone(),
        project_id: manifest.project_id.clone(),
        artifact_count: manifest.artifacts.len(),
        query: ExplainQuery {
            kind: "list".into(),
            value: "*".into(),
        },
        results: explanations,
    };

    emit_report(ctx, &report)
}

fn emit_results(
    ctx: &CommandContext,
    manifest: &ManifestV1,
    manifest_path: &std::path::Path,
    query_kind: &str,
    query_value: &str,
    results: &[&ArtifactEntry],
) -> anyhow::Result<()> {
    let explanations: Vec<_> = results.iter().map(|e| to_explanation(e)).collect();

    let report = ExplainReport {
        manifest_path: manifest_path.display().to_string(),
        engine_version: manifest.engine_version.clone(),
        project_id: manifest.project_id.clone(),
        artifact_count: manifest.artifacts.len(),
        query: ExplainQuery {
            kind: query_kind.into(),
            value: query_value.into(),
        },
        results: explanations,
    };

    emit_report(ctx, &report)
}

fn emit_report(ctx: &CommandContext, report: &ExplainReport) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("explain", report);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!(
                "know-now explain (manifest: {}, engine: v{})",
                report.manifest_path, report.engine_version
            );
            println!(
                "  project: {}, {} artifact(s) in manifest",
                report.project_id, report.artifact_count
            );
            println!(
                "  query: {} = {}",
                report.query.kind, report.query.value
            );
            println!();

            if report.results.is_empty() {
                println!("  No matching artifacts found.");
            } else {
                for (i, art) in report.results.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    println!("  {}:", art.path);
                    println!("    kind: {}", art.kind);
                    println!("    generator: {} v{}", art.generator, art.generator_version);
                    println!("    artifact_id: {}", art.artifact_id);
                    println!("    hash: {}", art.hash);
                    if !art.metadata_object_ids.is_empty() {
                        println!("    metadata objects: {}", art.metadata_object_ids.join(", "));
                    }
                    if art.trace_count > 0 {
                        println!("    trace spans: {}", art.trace_count);
                    }
                }
            }
        }
    }

    Ok(())
}
