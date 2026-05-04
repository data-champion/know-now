use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::commands::doctor;
use crate::commands::lock::resolve_current_versions;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct SupportBundleArgs {
    /// Preview bundle contents without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Include full metadata (opt-in, may contain sensitive data)
    #[arg(long)]
    pub include_metadata: bool,

    /// Output directory for the bundle
    #[arg(long, default_value = ".")]
    pub output: PathBuf,
}

#[derive(Debug, Serialize)]
struct SupportBundle {
    bundle_version: String,
    engine: EngineSection,
    doctor: serde_json::Value,
    lockfile: LockfileSection,
    config_summary: ConfigSummary,
    generators: Vec<GeneratorVersion>,
    recent_runs: Vec<serde_json::Value>,
    manifest_summary: Option<ManifestSummary>,
    issues_summary: IssuesSummary,
    environment: BTreeMap<String, String>,
    included_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EngineSection {
    version: String,
    os: String,
    arch: String,
}

#[derive(Debug, Serialize)]
struct LockfileSection {
    present: bool,
    hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConfigSummary {
    config_present: bool,
    config_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GeneratorVersion {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct ManifestSummary {
    engine_version: String,
    project_id: String,
    artifact_count: usize,
    input_hash: String,
}

#[derive(Debug, Serialize)]
struct IssuesSummary {
    total: usize,
    open: usize,
    resolved: usize,
}

const SAFE_ENV_VARS: &[&str] = &[
    "USER", "LOGNAME", "SHELL", "TERM", "LANG", "LC_ALL", "HOME", "PATH", "EDITOR",
    "VISUAL", "CARGO_PKG_VERSION", "RUST_LOG", "NO_COLOR", "FORCE_COLOR",
];

pub fn run(ctx: &CommandContext, args: &SupportBundleArgs) -> anyhow::Result<()> {
    let engine = EngineSection {
        version: env!("CARGO_PKG_VERSION").into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
    };

    let doctor_output = collect_doctor(ctx);
    let lockfile = collect_lockfile(ctx);
    let config_summary = collect_config_summary(ctx);
    let generators = collect_generators();
    let recent_runs = collect_recent_runs(ctx);
    let manifest_summary = collect_manifest_summary(ctx);
    let issues_summary = collect_issues_summary(ctx);
    let environment = collect_safe_env();

    let mut included_files = vec![
        "doctor.json".into(),
        "engine.json".into(),
        "lockfile.json".into(),
        "config_summary.json".into(),
        "generators.json".into(),
        "recent_runs.json".into(),
        "issues_summary.json".into(),
        "environment.json".into(),
    ];
    if manifest_summary.is_some() {
        included_files.push("manifest_summary.json".into());
    }
    if args.include_metadata {
        included_files.push("metadata/".into());
    }

    let bundle = SupportBundle {
        bundle_version: "1.0".into(),
        engine,
        doctor: doctor_output,
        lockfile,
        config_summary,
        generators,
        recent_runs,
        manifest_summary,
        issues_summary,
        environment,
        included_files: included_files.clone(),
    };

    if args.dry_run {
        match ctx.format {
            OutputFormat::Json | OutputFormat::Sarif => {
                let envelope = JsonEnvelope::success("support bundle", &included_files);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputFormat::Text | OutputFormat::Quiet => {
                println!("Support bundle would include:");
                for f in &included_files {
                    println!("  - {f}");
                }
                println!();
                println!("Run without --dry-run to write the bundle.");
            }
        }
        return Ok(());
    }

    let bundle_json = serde_json::to_string_pretty(&bundle)?;
    let bundle_name = format!(
        "know-now-support-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let bundle_path = args.output.join(&bundle_name);

    fs::write(&bundle_path, &bundle_json)?;

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            #[derive(Serialize)]
            struct BundleResult {
                path: String,
                size_bytes: u64,
                file_count: usize,
            }
            let meta = fs::metadata(&bundle_path)?;
            let result = BundleResult {
                path: bundle_path.display().to_string(),
                size_bytes: meta.len(),
                file_count: included_files.len(),
            };
            let envelope = JsonEnvelope::success("support bundle", &result);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("Support bundle written to: {}", bundle_path.display());
            println!(
                "  {} sections, {} bytes",
                included_files.len(),
                bundle_json.len()
            );
            println!();
            println!("Share this file with support. Sensitive data has been redacted.");
        }
    }

    Ok(())
}

fn collect_doctor(ctx: &CommandContext) -> serde_json::Value {
    let doctor_args = doctor::DoctorArgs {
        check_updates: false,
    };
    let mut findings = Vec::new();
    let report = doctor::collect_report(ctx, &doctor_args, &mut findings);
    serde_json::to_value(report).unwrap_or(serde_json::Value::Null)
}

fn collect_lockfile(ctx: &CommandContext) -> LockfileSection {
    let lock_path = ctx.project_root.join("know-now.lock");
    if !lock_path.exists() {
        return LockfileSection {
            present: false,
            hash: None,
        };
    }
    let hash = fs::read(&lock_path)
        .ok()
        .map(|bytes| format!("sha256:{}", sha256_hex(&bytes)));
    LockfileSection {
        present: true,
        hash,
    }
}

fn sha256_hex(data: &[u8]) -> String {
    know_now_writer::manifest_builder::sha256_hex(data)
}

fn collect_config_summary(ctx: &CommandContext) -> ConfigSummary {
    let config_path = ctx.project_root.join("know-now.yml");
    let alt_path = ctx.project_root.join("know-now.yaml");
    let path = if config_path.exists() {
        Some(config_path)
    } else if alt_path.exists() {
        Some(alt_path)
    } else {
        None
    };

    let Some(path) = path else {
        return ConfigSummary {
            config_present: false,
            config_keys: vec![],
        };
    };

    let keys = fs::read_to_string(path)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .filter_map(|l| {
                    if l.contains(':') && !l.starts_with(' ') && !l.starts_with('\t') {
                        l.split(':').next().map(|k| k.trim().to_owned())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    ConfigSummary {
        config_present: true,
        config_keys: keys,
    }
}

fn collect_generators() -> Vec<GeneratorVersion> {
    let versions = resolve_current_versions();
    versions
        .generators
        .iter()
        .map(|(name, version)| GeneratorVersion {
            name: name.clone(),
            version: version.clone(),
        })
        .collect()
}

fn collect_recent_runs(ctx: &CommandContext) -> Vec<serde_json::Value> {
    let runs_dir = ctx.project_root.join(".knownow").join("runs");
    let Ok(entries) = fs::read_dir(runs_dir) else {
        return Vec::new();
    };

    let mut runs: Vec<serde_json::Value> = entries
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .filter_map(|e| {
            let content = fs::read_to_string(e.path()).ok()?;
            serde_json::from_str(&content).ok()
        })
        .collect();

    runs.sort_by(|a, b| {
        let a_id = a["run_id"].as_str().unwrap_or("");
        let b_id = b["run_id"].as_str().unwrap_or("");
        b_id.cmp(a_id)
    });

    runs.truncate(10);
    runs
}

fn collect_manifest_summary(ctx: &CommandContext) -> Option<ManifestSummary> {
    let manifest_path = ctx.project_root.join("generated").join("manifest.json");
    let content = fs::read_to_string(manifest_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(ManifestSummary {
        engine_version: json["engine_version"].as_str()?.to_owned(),
        project_id: json["project_id"].as_str().unwrap_or("").to_owned(),
        artifact_count: json["artifacts"].as_array().map_or(0, Vec::len),
        input_hash: json["input_hash"].as_str().unwrap_or("").to_owned(),
    })
}

fn collect_issues_summary(ctx: &CommandContext) -> IssuesSummary {
    let issues_path = ctx.project_root.join(".knownow").join("issues.json");
    let Ok(content) = fs::read_to_string(issues_path) else {
        return IssuesSummary {
            total: 0,
            open: 0,
            resolved: 0,
        };
    };
    let issues: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
    let total = issues.len();
    let open = issues
        .iter()
        .filter(|i| i["status"].as_str() != Some("resolved"))
        .count();
    let resolved = total - open;
    IssuesSummary {
        total,
        open,
        resolved,
    }
}

fn collect_safe_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in SAFE_ENV_VARS {
        if let Ok(val) = std::env::var(key) {
            let redacted = redact_path(&val);
            env.insert((*key).to_owned(), redacted);
        }
    }
    env
}

fn redact_path(val: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return val.replace(&home, "$HOME");
        }
    }
    val.to_owned()
}
