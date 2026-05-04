use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct ValidateArgs;

pub fn run(_ctx: &CommandContext, _args: &ValidateArgs) -> anyhow::Result<()> {
    anyhow::bail!("validate command not yet implemented (Phase 2A: know-now-63g.13)")
}
