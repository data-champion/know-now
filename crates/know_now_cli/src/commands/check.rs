use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct CheckArgs;

pub fn run(_ctx: &CommandContext, _args: &CheckArgs) -> anyhow::Result<()> {
    anyhow::bail!("check command not yet implemented (Phase 2A: know-now-63g.14)")
}
