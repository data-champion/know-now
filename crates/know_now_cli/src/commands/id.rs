use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Subcommand)]
pub enum IdCommand {
    /// List missing or required IDs
    Check(IdCheckArgs),
    /// Propose deterministic IDs to stdout
    Suggest(IdSuggestArgs),
    /// Preview or apply stable ID backfill
    Backfill(IdBackfillArgs),
}

#[derive(Debug, clap::Args)]
pub struct IdCheckArgs;

#[derive(Debug, clap::Args)]
pub struct IdSuggestArgs;

#[derive(Debug, clap::Args)]
pub struct IdBackfillArgs {
    /// Preview changes without writing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(ctx: &CommandContext, cmd: &IdCommand) -> anyhow::Result<()> {
    let metadata = crate::commands::load_project_metadata(ctx)?;

    match cmd {
        IdCommand::Check(_) => {
            let (result, _diagnostics) = know_now_identity::check_ids(&metadata);
            match ctx.format {
                OutputFormat::Json => {
                    let envelope = crate::output::JsonEnvelope::success("id check", &result);
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                }
                OutputFormat::Quiet => {}
                _ => {
                    if result.missing.is_empty()
                        && result.invalid.is_empty()
                        && result.duplicate.is_empty()
                    {
                        println!("All objects have valid stable IDs.");
                    } else {
                        for m in &result.missing {
                            println!(
                                "  missing: {} '{}' at {} (suggested: {})",
                                m.object_type, m.name, m.yaml_path, m.suggested_id
                            );
                        }
                        for i in &result.invalid {
                            println!(
                                "  invalid: {} '{}' at {}: {}",
                                i.object_type, i.id, i.yaml_path, i.reason
                            );
                        }
                        for d in &result.duplicate {
                            println!("  duplicate: '{}' at {:?}", d.id, d.locations);
                        }
                    }
                }
            }
            if !result.duplicate.is_empty() {
                std::process::exit(crate::exit_code::VALIDATION_ERROR);
            }
            Ok(())
        }
        IdCommand::Suggest(_) => {
            let suggestions = know_now_identity::suggest_all_ids(&metadata);
            match ctx.format {
                OutputFormat::Json => {
                    let envelope = crate::output::JsonEnvelope::success("id suggest", &suggestions);
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                }
                OutputFormat::Quiet => {}
                _ => {
                    if suggestions.is_empty() {
                        println!("All objects already have stable IDs.");
                    } else {
                        for s in &suggestions {
                            println!("  {} '{}': id: {}", s.object_type, s.name, s.suggested_id);
                        }
                    }
                }
            }
            Ok(())
        }
        IdCommand::Backfill(args) => {
            if !args.dry_run {
                anyhow::bail!("id backfill --apply is not available until Phase 3");
            }
            let preview = know_now_identity::backfill_preview(&metadata);
            match ctx.format {
                OutputFormat::Quiet => {}
                _ => print!("{preview}"),
            }
            Ok(())
        }
    }
}
