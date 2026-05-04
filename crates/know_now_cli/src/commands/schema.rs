use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct SchemaArgs;

pub fn run(_ctx: &CommandContext, _args: &SchemaArgs) -> anyhow::Result<()> {
    anyhow::bail!("schema command not yet implemented (Phase 2A: know-now-63g.6)")
}
