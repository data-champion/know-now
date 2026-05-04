use crate::context::CommandContext;

#[derive(Debug, clap::Subcommand)]
pub enum LockCommand {
    /// Update the lockfile from current metadata
    Update(LockUpdateArgs),
    /// Check that the lockfile matches current metadata
    Check(LockCheckArgs),
}

#[derive(Debug, clap::Args)]
pub struct LockUpdateArgs;

#[derive(Debug, clap::Args)]
pub struct LockCheckArgs;

pub fn run(_ctx: &CommandContext, cmd: &LockCommand) -> anyhow::Result<()> {
    match cmd {
        LockCommand::Update(_) => {
            anyhow::bail!("lock update not yet implemented (Phase 2A: know-now-63g.10)")
        }
        LockCommand::Check(_) => {
            anyhow::bail!("lock check not yet implemented (Phase 2A: know-now-63g.10)")
        }
    }
}
