use std::collections::HashMap;
use std::path::{Path, PathBuf};

use know_now_catalog::{classify_drift, validate, Catalog, ProjectState};
use know_now_lock::lockfile::Lockfile;
use know_now_lock::LOCKFILE_NAME;
use serde::Serialize;

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum AdminCommand {
    /// Scan filesystem for know-now projects and aggregate governance state
    Scan(AdminScanArgs),
    /// Validate an approved-version catalog file
    CatalogCheck(AdminCatalogCheckArgs),
}

#[derive(Debug, clap::Args)]
pub struct AdminScanArgs {
    /// Root path to scan for know-now projects
    pub path: PathBuf,

    /// Path to approved-version catalog for drift classification
    #[arg(long)]
    pub catalog: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct AdminCatalogCheckArgs {
    /// Path to the catalog file to validate
    pub path: PathBuf,
}

pub fn run(ctx: &CommandContext, cmd: &AdminCommand) -> anyhow::Result<()> {
    match cmd {
        AdminCommand::Scan(args) => run_scan(ctx, args),
        AdminCommand::CatalogCheck(args) => run_catalog_check(ctx, args),
    }
}

#[derive(Debug, Serialize)]
struct ScanReport {
    scanned_root: String,
    projects: Vec<ProjectSummary>,
    catalog_used: bool,
}

#[derive(Debug, Serialize)]
struct ProjectSummary {
    path: String,
    project_id: Option<String>,
    engine_version: Option<String>,
    metadata_schema_version: Option<String>,
    lockfile_status: String,
    policy_pack: Option<String>,
    policy_version: Option<String>,
    last_generation: Option<String>,
    drift_class: Option<String>,
}

fn run_scan(ctx: &CommandContext, args: &AdminScanArgs) -> anyhow::Result<()> {
    if !args.path.is_dir() {
        anyhow::bail!("scan path is not a directory: {}", args.path.display());
    }

    let catalog = args.catalog.as_ref().map(load_catalog_file).transpose()?;

    let mut projects = Vec::new();
    discover_projects(&args.path, &mut projects);

    let mut summaries: Vec<ProjectSummary> = projects
        .iter()
        .map(|p| build_project_summary(p, catalog.as_ref()))
        .collect();
    summaries.sort_by(|a, b| a.path.cmp(&b.path));

    let report = ScanReport {
        scanned_root: args.path.display().to_string(),
        projects: summaries,
        catalog_used: catalog.is_some(),
    };

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("admin scan", &report);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!(
                "Scanned: {} ({} project(s) found)",
                report.scanned_root,
                report.projects.len()
            );
            println!();
            for proj in &report.projects {
                let drift = proj
                    .drift_class
                    .as_deref()
                    .map_or(String::new(), |d| format!(" [drift: {d}]"));
                println!(
                    "  {} ({}){drift}",
                    proj.path,
                    proj.lockfile_status
                );
                if let Some(ref engine) = proj.engine_version {
                    println!("    engine: v{engine}");
                }
                if let Some(ref policy) = proj.policy_pack {
                    let ver = proj.policy_version.as_deref().unwrap_or("?");
                    println!("    policy: {policy} v{ver}");
                }
            }
        }
    }
    Ok(())
}

fn discover_projects(root: &Path, projects: &mut Vec<PathBuf>) {
    if is_project_root(root) {
        projects.push(root.to_path_buf());
        return;
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            discover_projects(&path, projects);
        }
    }
}

fn is_project_root(dir: &Path) -> bool {
    dir.join("metadata").is_dir()
        && (dir.join("know-now.yml").exists()
            || dir.join("know-now.yaml").exists()
            || dir.join(LOCKFILE_NAME).exists())
}

fn build_project_summary(project_path: &Path, catalog: Option<&Catalog>) -> ProjectSummary {
    let lockfile = load_lockfile(project_path);

    let engine_version = lockfile.as_ref().map(|l| l.engine_version.clone());
    let metadata_schema_version = lockfile.as_ref().map(|l| l.metadata_schema_version.clone());
    let policy_pack = lockfile.as_ref().map(|l| l.policy.pack.clone());
    let policy_version = lockfile.as_ref().map(|l| l.policy.version.clone());

    let lockfile_status = if lockfile.is_some() {
        "locked"
    } else if project_path.join(LOCKFILE_NAME).exists() {
        "corrupt"
    } else {
        "missing"
    };

    let last_generation = load_last_generation_time(project_path);

    let project_id = lockfile.as_ref().and_then(|l| {
        l.unknown_fields
            .get("project_id")
            .and_then(|v| v.as_str().map(String::from))
    });

    let drift_class = catalog.map(|cat| {
        let state = ProjectState {
            engine_version: engine_version.clone(),
            metadata_schema_version: metadata_schema_version.clone(),
            generator_contract_version: lockfile
                .as_ref()
                .map(|l| l.generator_contract_version.clone()),
            policies: policy_pack
                .as_ref()
                .zip(policy_version.as_ref())
                .map(|(p, v)| [(p.clone(), v.clone())].into())
                .unwrap_or_default(),
            templates: HashMap::default(),
            template_renderers: HashMap::default(),
            targets: HashMap::default(),
        };
        let report = classify_drift(cat, &state);
        report.overall.to_string()
    });

    ProjectSummary {
        path: project_path.display().to_string(),
        project_id,
        engine_version,
        metadata_schema_version,
        lockfile_status: lockfile_status.into(),
        policy_pack,
        policy_version,
        last_generation,
        drift_class,
    }
}

fn load_lockfile(project_path: &Path) -> Option<Lockfile> {
    let lock_path = project_path.join(LOCKFILE_NAME);
    let content = std::fs::read_to_string(&lock_path).ok()?;
    Lockfile::from_json(&content).ok()
}

fn load_last_generation_time(project_path: &Path) -> Option<String> {
    let path = project_path
        .join(".knownow")
        .join("last_generation.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("finished_at")
        .and_then(|v| v.as_str().map(String::from))
}

#[derive(Debug, Serialize)]
struct CatalogCheckReport {
    path: String,
    valid: bool,
    errors: Vec<String>,
}

fn run_catalog_check(ctx: &CommandContext, args: &AdminCatalogCheckArgs) -> anyhow::Result<()> {
    if !args.path.exists() {
        anyhow::bail!("catalog file not found: {}", args.path.display());
    }

    let content = std::fs::read_to_string(&args.path)?;
    let catalog: Catalog = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("invalid catalog JSON: {e}"))?;

    let errors = validate(&catalog);
    let error_strings: Vec<String> = errors.iter().map(std::string::ToString::to_string).collect();

    let report = CatalogCheckReport {
        path: args.path.display().to_string(),
        valid: errors.is_empty(),
        errors: error_strings.clone(),
    };

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("admin catalog-check", &report);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            if errors.is_empty() {
                println!("Catalog valid: {}", args.path.display());
            } else {
                println!(
                    "Catalog invalid: {} ({} error(s))",
                    args.path.display(),
                    errors.len()
                );
                for err in &error_strings {
                    println!("  - {err}");
                }
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("catalog validation failed with {} error(s)", errors.len());
    }

    Ok(())
}

fn load_catalog_file(path: &PathBuf) -> anyhow::Result<Catalog> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read catalog file {}: {e}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("invalid catalog JSON in {}: {e}", path.display()))
}
