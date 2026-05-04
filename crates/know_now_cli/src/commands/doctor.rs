use std::path::Path;

use serde::Serialize;

use crate::commands::lock::resolve_current_versions;
use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct DoctorArgs {
    /// Check for updates (requires network)
    #[arg(long)]
    pub check_updates: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    doctor_schema_version: String,
    engine: EngineInfo,
    project: ProjectInfo,
    lockfile: LockfileInfo,
    contracts: ContractInfo,
    policy: PolicyInfo,
    target_database: Option<TargetDatabaseInfo>,
    dbt_validation: DbtValidationInfo,
    generators: Vec<GeneratorInfo>,
    issues: IssuesInfo,
    last_generation: Option<LastGenerationInfo>,
    security_warnings: Vec<SecurityWarning>,
    findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
struct EngineInfo {
    version: String,
    os: String,
    arch: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectInfo {
    root: String,
    config_present: bool,
    metadata_files_count: usize,
    objects_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct LockfileInfo {
    present: bool,
    valid: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ContractInfo {
    metadata_schema_version: String,
    generator_contract_version: String,
}

#[derive(Debug, Clone, Serialize)]
struct PolicyInfo {
    pack: Option<String>,
    version: Option<String>,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
struct TargetDatabaseInfo {
    kind: String,
    version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DbtValidationInfo {
    mode: String,
    available: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GeneratorInfo {
    name: String,
    version: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
struct IssuesInfo {
    unresolved_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct LastGenerationInfo {
    manifest_present: bool,
    artifact_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SecurityWarning {
    code: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    severity: String,
    code: String,
    message: String,
}

pub fn collect_report(ctx: &CommandContext, _args: &DoctorArgs, findings: &mut Vec<Finding>) -> DoctorReport {
    let engine = EngineInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
    };

    let project = check_project(ctx, findings);
    let lockfile = check_lockfile(ctx, findings);
    let policy = check_policy(ctx, findings);
    let target_database = check_target_database(ctx);
    let dbt_validation = check_dbt_validation(ctx);
    let generators = check_generators();
    let issues = check_issues(ctx);
    let last_generation = check_last_generation(ctx);
    let security_warnings = check_security(ctx);

    DoctorReport {
        doctor_schema_version: "1.0".into(),
        engine,
        project,
        lockfile,
        contracts: ContractInfo {
            metadata_schema_version: "1.0".into(),
            generator_contract_version: "1.0".into(),
        },
        policy,
        target_database,
        dbt_validation,
        generators,
        issues,
        last_generation,
        security_warnings,
        findings: findings.clone(),
    }
}

pub fn run(ctx: &CommandContext, args: &DoctorArgs) -> anyhow::Result<()> {
    let mut findings = Vec::new();
    let report = collect_report(ctx, args, &mut findings);

    let error_count = findings.iter().filter(|f| f.severity == "error").count();
    let warning_count = findings
        .iter()
        .filter(|f| f.severity == "warning")
        .count();

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into())
            );
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            print_text_report(&report, &findings, error_count, warning_count);
        }
    }

    if error_count > 0 {
        std::process::exit(2);
    } else if warning_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn print_text_report(
    report: &DoctorReport,
    findings: &[Finding],
    error_count: usize,
    warning_count: usize,
) {
    println!("know-now doctor");
    println!("  engine: v{} ({}/{})", report.engine.version, report.engine.os, report.engine.arch);
    println!("  project: {}", report.project.root);
    println!("    config: {}", if report.project.config_present { "found" } else { "MISSING" });
    println!(
        "    metadata: {} files, {} objects",
        report.project.metadata_files_count, report.project.objects_count
    );
    println!(
        "  lockfile: {}",
        if report.lockfile.present {
            if report.lockfile.valid { "valid" } else { "INVALID" }
        } else {
            "not present"
        }
    );
    if let Some(ref db) = report.target_database {
        println!("  target database: {} {}", db.kind, db.version.as_deref().unwrap_or(""));
    }
    println!(
        "  dbt validation: mode={}, available={}",
        report.dbt_validation.mode, report.dbt_validation.available
    );
    println!(
        "  policy: {}",
        report.policy.pack.as_deref().unwrap_or("(none)")
    );
    println!("  generators: {} registered", report.generators.len());
    if let Some(ref gen) = report.last_generation {
        println!("  last generation: {} artifacts", gen.artifact_count);
    }
    if report.issues.unresolved_count > 0 {
        println!("  issues: {} unresolved", report.issues.unresolved_count);
    }
    println!();
    if findings.is_empty() {
        println!("All checks passed.");
    } else {
        for f in findings {
            println!("  [{}] {}: {}", f.severity, f.code, f.message);
        }
        println!();
        println!("{error_count} error(s), {warning_count} warning(s)");
    }
}

fn check_project(ctx: &CommandContext, findings: &mut Vec<Finding>) -> ProjectInfo {
    let config_present = ctx.project_root.join("know-now.yml").exists()
        || ctx.project_root.join("know-now.yaml").exists();

    if !config_present {
        findings.push(Finding {
            severity: "warning".into(),
            code: "DOC-CFG-001".into(),
            message: "no know-now.yml config file found".into(),
        });
    }

    let metadata_dir = ctx.project_root.join("metadata");
    let (file_count, object_count) = if metadata_dir.is_dir() {
        count_metadata(&metadata_dir)
    } else {
        findings.push(Finding {
            severity: "error".into(),
            code: "DOC-META-001".into(),
            message: "no metadata/ directory found".into(),
        });
        (0, 0)
    };

    ProjectInfo {
        root: ctx.project_root.display().to_string(),
        config_present,
        metadata_files_count: file_count,
        objects_count: object_count,
    }
}

fn count_metadata(metadata_dir: &Path) -> (usize, usize) {
    let mut file_count = 0;
    let mut object_count = 0;
    if let Ok(entries) = std::fs::read_dir(metadata_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
            {
                file_count += 1;
                if let Ok(content) = std::fs::read_to_string(&path) {
                    object_count += content.matches("- name:").count();
                    object_count += content.matches("name:").count().min(1);
                }
            }
        }
    }
    (file_count, object_count)
}

fn check_lockfile(ctx: &CommandContext, findings: &mut Vec<Finding>) -> LockfileInfo {
    let lock_path = ctx.project_root.join("know-now.lock");
    if !lock_path.exists() {
        return LockfileInfo {
            present: false,
            valid: false,
        };
    }

    let valid = if let Ok(lf) = know_now_lock::lockfile::Lockfile::read_from(&lock_path) {
        let resolved = resolve_current_versions();
        let result = know_now_lock::check::check_lockfile(&lf, &resolved);
        if result.is_ok() {
            true
        } else {
            findings.push(Finding {
                severity: "warning".into(),
                code: "DOC-LOCK-001".into(),
                message: "lockfile is stale — run 'know-now lock update'".into(),
            });
            false
        }
    } else {
        findings.push(Finding {
            severity: "error".into(),
            code: "DOC-LOCK-002".into(),
            message: "lockfile is corrupt".into(),
        });
        false
    };

    LockfileInfo {
        present: true,
        valid,
    }
}

fn check_policy(ctx: &CommandContext, findings: &mut Vec<Finding>) -> PolicyInfo {
    let config_path = ctx.project_root.join("know-now.yml");
    if !config_path.exists() {
        return PolicyInfo {
            pack: None,
            version: None,
            status: "unconfigured".into(),
        };
    }

    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if content.contains("pack:") {
            let pack = content
                .lines()
                .find(|l| l.contains("pack:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_owned());
            let version = content
                .lines()
                .find(|l| l.trim().starts_with("version:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().trim_matches('"').to_owned());
            return PolicyInfo {
                pack,
                version,
                status: "configured".into(),
            };
        }
    }

    findings.push(Finding {
        severity: "info".into(),
        code: "DOC-POL-001".into(),
        message: "no policy pack configured".into(),
    });

    PolicyInfo {
        pack: None,
        version: None,
        status: "none".into(),
    }
}

fn check_target_database(ctx: &CommandContext) -> Option<TargetDatabaseInfo> {
    let config_path = ctx.project_root.join("know-now.yml");
    let content = std::fs::read_to_string(config_path).ok()?;
    if content.contains("target_database:") || content.contains("kind:") {
        let kind = content
            .lines()
            .find(|l| l.trim().starts_with("kind:"))
            .and_then(|l| l.split(':').nth(1))
            .map_or_else(|| "unknown".into(), |s| s.trim().to_owned());
        let version = content
            .lines()
            .find(|l| l.trim().starts_with("version:") && !l.contains("contract"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().trim_matches('"').to_owned());
        Some(TargetDatabaseInfo { kind, version })
    } else {
        None
    }
}

fn check_dbt_validation(ctx: &CommandContext) -> DbtValidationInfo {
    let config_path = ctx.project_root.join("know-now.yml");
    let mode = std::fs::read_to_string(config_path)
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.contains("dbt_validation:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_owned())
        })
        .unwrap_or_else(|| "none".into());

    let available = mode != "none"
        && std::process::Command::new("dbt")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());

    DbtValidationInfo { mode, available }
}

fn check_generators() -> Vec<GeneratorInfo> {
    let versions = resolve_current_versions();
    versions
        .generators
        .iter()
        .map(|(name, version)| GeneratorInfo {
            name: name.clone(),
            version: version.clone(),
            status: "healthy".into(),
        })
        .collect()
}

fn check_issues(ctx: &CommandContext) -> IssuesInfo {
    let issues_path = ctx.project_root.join(".knownow/issues.json");
    let count = std::fs::read_to_string(issues_path)
        .ok()
        .and_then(|content| {
            let v: serde_json::Value = serde_json::from_str(&content).ok()?;
            v.as_array().map(Vec::len)
        })
        .unwrap_or(0);
    IssuesInfo {
        unresolved_count: count,
    }
}

fn check_last_generation(ctx: &CommandContext) -> Option<LastGenerationInfo> {
    let manifest_path = ctx.project_root.join("generated/manifest.json");
    if !manifest_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let artifact_count = json["artifacts"]
        .as_array()
        .map_or(0, std::vec::Vec::len);
    Some(LastGenerationInfo {
        manifest_present: true,
        artifact_count,
    })
}

fn check_security(ctx: &CommandContext) -> Vec<SecurityWarning> {
    let mut warnings = Vec::new();
    let env_path = ctx.project_root.join(".env");
    if env_path.exists() {
        let gitignore = ctx.project_root.join(".gitignore");
        let env_ignored = std::fs::read_to_string(gitignore)
            .ok()
            .is_some_and(|content| content.lines().any(|l| l.trim() == ".env"));
        if !env_ignored {
            warnings.push(SecurityWarning {
                code: "SEC-ENV-001".into(),
                message: ".env file exists but is not in .gitignore".into(),
            });
        }
    }
    warnings
}
