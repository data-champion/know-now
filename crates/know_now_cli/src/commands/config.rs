use know_now_lock::lockfile::Lockfile;
use know_now_lock::LOCKFILE_NAME;

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommand {
    /// Show effective configuration
    Inspect(ConfigInspectArgs),
}

#[derive(Debug, clap::Args)]
pub struct ConfigInspectArgs;

pub fn run(ctx: &CommandContext, cmd: &ConfigCommand) -> anyhow::Result<()> {
    match cmd {
        ConfigCommand::Inspect(_) => inspect(ctx),
    }
}

fn inspect(ctx: &CommandContext) -> anyhow::Result<()> {
    let info = gather_project_info(ctx);
    match ctx.format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::success("config inspect", &info);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        _ => emit_text(&info),
    }
    Ok(())
}

fn gather_project_info(ctx: &CommandContext) -> ProjectInfo {
    let config_exists = ctx.project_root.join("know-now.yml").exists();
    let metadata_dir = ctx.project_root.join("metadata");
    let metadata_exists = metadata_dir.is_dir();

    let metadata_stats = if metadata_exists {
        gather_metadata_stats(ctx)
    } else {
        None
    };

    let lock_path = ctx.project_root.join(LOCKFILE_NAME);
    let lockfile_state = if lock_path.exists() {
        match Lockfile::read_from(&lock_path) {
            Ok(lf) => Some(LockfileState {
                present: true,
                valid: true,
                schema_version: Some(lf.lockfile_schema_version),
                engine_version: Some(lf.engine_version),
                generator_count: Some(lf.generators.len()),
                policy_pack: Some(lf.policy.pack),
            }),
            Err(_) => Some(LockfileState {
                present: true,
                valid: false,
                schema_version: None,
                engine_version: None,
                generator_count: None,
                policy_pack: None,
            }),
        }
    } else {
        Some(LockfileState {
            present: false,
            valid: false,
            schema_version: None,
            engine_version: None,
            generator_count: None,
            policy_pack: None,
        })
    };

    let registry = crate::commands::version::build_registry();
    let generators: Vec<GeneratorSummary> = registry
        .generators()
        .iter()
        .map(|g| GeneratorSummary {
            name: g.name.clone(),
            version: g.version.clone(),
        })
        .collect();

    ProjectInfo {
        project_root: ctx.project_root.display().to_string(),
        config_file: config_exists,
        metadata_dir: metadata_exists,
        metadata: metadata_stats,
        lockfile: lockfile_state,
        engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        generators,
        policy_pack: "dc_standard".to_owned(),
        policy_version: "1.0".to_owned(),
    }
}

fn gather_metadata_stats(ctx: &CommandContext) -> Option<MetadataStats> {
    let metadata = crate::commands::load_project_metadata(ctx).ok()?;
    Some(MetadataStats {
        project_name: metadata.project.as_ref().map(|p| p.name.clone()),
        entity_count: metadata.entities.len(),
        relationship_count: metadata.relationships.len(),
        domain_count: metadata.domains.len(),
        module_count: metadata.modules.len(),
        source_count: metadata.sources.len(),
        rule_count: metadata.rules.len(),
    })
}

fn emit_text(info: &ProjectInfo) {
    println!("Project root: {}", info.project_root);
    println!(
        "Config file:  {}",
        if info.config_file {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "Metadata dir: {}",
        if info.metadata_dir {
            "found"
        } else {
            "not found"
        }
    );
    println!();

    if let Some(ref meta) = info.metadata {
        if let Some(ref name) = meta.project_name {
            println!("Project:       {name}");
        }
        println!("Entities:      {}", meta.entity_count);
        println!("Relationships: {}", meta.relationship_count);
        println!("Domains:       {}", meta.domain_count);
        println!("Modules:       {}", meta.module_count);
        println!("Sources:       {}", meta.source_count);
        println!("Rules:         {}", meta.rule_count);
        println!();
    }

    println!("Engine:        {}", info.engine_version);
    println!(
        "Policy:        {} v{}",
        info.policy_pack, info.policy_version
    );
    println!("Generators:");
    for g in &info.generators {
        println!("  {:<30} {}", g.name, g.version);
    }
    println!();

    if let Some(ref lock) = info.lockfile {
        if lock.present && lock.valid {
            println!(
                "Lockfile:      valid (schema {}, engine {})",
                lock.schema_version.as_deref().unwrap_or("?"),
                lock.engine_version.as_deref().unwrap_or("?")
            );
        } else if lock.present {
            println!("Lockfile:      corrupt or unreadable");
        } else {
            println!("Lockfile:      not found");
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct ProjectInfo {
    project_root: String,
    config_file: bool,
    metadata_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<MetadataStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lockfile: Option<LockfileState>,
    engine_version: String,
    generators: Vec<GeneratorSummary>,
    policy_pack: String,
    policy_version: String,
}

#[derive(Debug, serde::Serialize)]
struct MetadataStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    project_name: Option<String>,
    entity_count: usize,
    relationship_count: usize,
    domain_count: usize,
    module_count: usize,
    source_count: usize,
    rule_count: usize,
}

#[derive(Debug, serde::Serialize)]
struct LockfileState {
    present: bool,
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engine_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generator_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_pack: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct GeneratorSummary {
    name: String,
    version: String,
}
