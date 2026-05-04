use crate::context::CommandContext;

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Project template to use
    #[arg(long)]
    pub template: Option<String>,

    /// Run interactive guided setup
    #[arg(long)]
    pub guided: bool,
}

pub fn run(_ctx: &CommandContext, _args: &InitArgs) -> anyhow::Result<()> {
    anyhow::bail!("init command not yet implemented (Phase 2A: know-now-63g.2)")
}
