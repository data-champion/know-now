use std::collections::BTreeMap;

use know_now_lock::check::{
    self, is_contract_breaking, LockCheckError, LOCK_CONTRACT_002, LOCK_CORRUPT_005,
    LOCK_MISSING_004, LOCK_SCHEMA_001, LOCK_STALE_003,
};
use know_now_lock::lockfile::Lockfile;
use know_now_lock::resolved::ResolvedVersions;
use know_now_lock::LOCKFILE_NAME;
use know_now_policy::engine::PolicyPack;

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum LockCommand {
    /// Update the lockfile from current metadata
    Update(LockUpdateArgs),
    /// Check that the lockfile matches current metadata
    Check(LockCheckArgs),
}

#[derive(Debug, clap::Args)]
pub struct LockUpdateArgs {
    /// Acknowledge a breaking generator contract version upgrade
    #[arg(long)]
    pub accept_contract_upgrade: bool,
}

#[derive(Debug, clap::Args)]
pub struct LockCheckArgs;

pub fn run(ctx: &CommandContext, cmd: &LockCommand) -> anyhow::Result<()> {
    match cmd {
        LockCommand::Update(args) => run_update(ctx, args),
        LockCommand::Check(_args) => run_check(ctx),
    }
}

pub(crate) fn resolve_current_versions() -> ResolvedVersions {
    let policy_info = know_now_policy::dc_standard::DcStandard.info();

    let mut generators = BTreeMap::new();
    generators.insert("know_now_gen_postgres".to_owned(), "0.1.0".to_owned());
    generators.insert("know_now_gen_docs".to_owned(), "0.1.0".to_owned());

    ResolvedVersions {
        engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        metadata_schema_version: "0.1.0".to_owned(),
        generator_contract_version: know_now_contract::CONTRACT_SCHEMA_VERSION.to_owned(),
        generators,
        policy_pack: policy_info.pack,
        policy_version: policy_info.version,
        policy_hash: policy_info.hash,
        target_profiles: vec![],
        semantic_type_mappings: BTreeMap::new(),
    }
}

fn run_update(ctx: &CommandContext, args: &LockUpdateArgs) -> anyhow::Result<()> {
    let lock_path = ctx.project_root.join(LOCKFILE_NAME);
    let resolved = resolve_current_versions();

    if lock_path.exists() {
        let existing = Lockfile::read_from(&lock_path).map_err(|e| anyhow::anyhow!("{e}"))?;

        if is_contract_breaking(
            &existing.generator_contract_version,
            &resolved.generator_contract_version,
        ) && !args.accept_contract_upgrade
        {
            anyhow::bail!(
                "{LOCK_CONTRACT_002}: generator contract version change is breaking \
                 ({} -> {}); pass --accept-contract-upgrade to proceed",
                existing.generator_contract_version,
                resolved.generator_contract_version
            );
        }
    }

    let lockfile = resolved.to_lockfile();
    lockfile
        .write_to(&lock_path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match ctx.format {
        OutputFormat::Json => {
            let payload = LockUpdatePayload {
                path: lock_path.display().to_string(),
                lockfile: &lockfile,
            };
            let envelope = JsonEnvelope::success("lock update", &payload);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            println!("Lockfile updated: {}", lock_path.display());
            println!("  engine_version: {}", lockfile.engine_version);
            println!(
                "  metadata_schema_version: {}",
                lockfile.metadata_schema_version
            );
            println!(
                "  generator_contract_version: {}",
                lockfile.generator_contract_version
            );
            let gen_list: Vec<String> = lockfile
                .generators
                .iter()
                .map(|(name, g)| format!("{name}@{}", g.version))
                .collect();
            println!("  generators: {}", gen_list.join(", "));
            println!(
                "  policy: {}@{}",
                lockfile.policy.pack, lockfile.policy.version
            );
        }
    }

    Ok(())
}

fn run_check(ctx: &CommandContext) -> anyhow::Result<()> {
    let lock_path = ctx.project_root.join(LOCKFILE_NAME);

    if !lock_path.exists() {
        return emit_check_error(
            ctx,
            LOCK_MISSING_004,
            "lockfile not found; run 'know-now lock update' to create it",
        );
    }

    let lockfile = match Lockfile::read_from(&lock_path) {
        Ok(lf) => lf,
        Err(e) => {
            return emit_check_error(ctx, LOCK_CORRUPT_005, &format!("lockfile is corrupt: {e}"));
        }
    };

    let resolved = resolve_current_versions();
    let result = check::check_lockfile(&lockfile, &resolved);

    match ctx.format {
        OutputFormat::Json => {
            let payload = LockCheckPayload {
                passed: result.is_ok(),
                drifted_fields: &result.drifted_fields,
                warnings: &result.warnings,
                error_code: result.error.as_ref().map(error_code),
            };
            if result.is_ok() {
                let envelope = JsonEnvelope::success("lock check", &payload);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                let envelope = JsonEnvelope::error("lock check", &payload);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            for warning in &result.warnings {
                println!("  warning: {warning}");
            }
            if result.is_ok() {
                println!("Lockfile check passed. All versions match.");
            } else if let Some(LockCheckError::SchemaMismatch {
                lockfile_version,
                engine_version,
            }) = &result.error
            {
                eprintln!(
                    "error: {LOCK_SCHEMA_001}: lockfile schema version '{lockfile_version}' \
                     is not recognized by engine (expected '{engine_version}')"
                );
            } else {
                println!("Lockfile check failed. Drifted fields:");
                for d in &result.drifted_fields {
                    println!(
                        "  {}: locked={} resolved={}",
                        d.field, d.locked_value, d.resolved_value
                    );
                }
            }
        }
    }

    if !result.is_ok() {
        std::process::exit(crate::exit_code::VALIDATION_ERROR);
    }

    Ok(())
}

fn emit_check_error(ctx: &CommandContext, code: &str, message: &str) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let payload = LockCheckPayload {
                passed: false,
                drifted_fields: &[],
                warnings: &[],
                error_code: Some(code.to_owned()),
            };
            let envelope = JsonEnvelope::error("lock check", &payload);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            eprintln!("error: {code}: {message}");
        }
    }
    std::process::exit(crate::exit_code::VALIDATION_ERROR);
}

fn error_code(err: &LockCheckError) -> String {
    match err {
        LockCheckError::SchemaMismatch { .. } => LOCK_SCHEMA_001.to_owned(),
        LockCheckError::ContractBreaking { .. } => LOCK_CONTRACT_002.to_owned(),
        LockCheckError::Stale(_) => LOCK_STALE_003.to_owned(),
        LockCheckError::Missing => LOCK_MISSING_004.to_owned(),
        LockCheckError::Corrupt(_) => LOCK_CORRUPT_005.to_owned(),
    }
}

#[derive(serde::Serialize)]
struct LockUpdatePayload<'a> {
    path: String,
    lockfile: &'a Lockfile,
}

#[derive(serde::Serialize)]
struct LockCheckPayload<'a> {
    passed: bool,
    drifted_fields: &'a [check::DriftedField],
    warnings: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
}
