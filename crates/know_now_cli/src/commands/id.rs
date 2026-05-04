use crate::context::CommandContext;

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

pub fn run(_ctx: &CommandContext, cmd: &IdCommand) -> anyhow::Result<()> {
    match cmd {
        IdCommand::Check(_) => {
            anyhow::bail!("id check not yet implemented (Phase 2A: know-now-63g.5)")
        }
        IdCommand::Suggest(_) => {
            anyhow::bail!("id suggest not yet implemented (Phase 2A: know-now-63g.5)")
        }
        IdCommand::Backfill(args) => {
            if !args.dry_run {
                anyhow::bail!("id backfill --apply is not available until Phase 3");
            }
            anyhow::bail!("id backfill --dry-run not yet implemented (Phase 2A: know-now-63g.5)")
        }
    }
}
